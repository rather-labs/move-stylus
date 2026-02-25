use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiOperationError},
    data::RuntimeErrorData,
    error::RuntimeError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
    wasm_builder_extensions::WasmBuilderExtension,
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

pub fn unpack_enum_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::UnpackEnum.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let enum_ = compilation_ctx.get_enum_by_intermediate_type(itype)?;
    if !enum_.is_simple {
        return Err(AbiError::Unpack(AbiOperationError::EnumIsNotSimple(
            enum_.identifier.to_owned(),
        ))
        .into());
    }
    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<8>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpack_u32_function =
        RuntimeFunction::UnpackU32.get(module, Some(compilation_ctx), None)?;

    // Save the variant to check it later
    let variant_number = module.locals.add(ValType::I32);
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .call(unpack_u32_function)
        .local_tee(variant_number);

    // Return error if the variant number is higher than the quantity of variants the enum contains
    builder
        .i32_const(enum_.variants.len() as i32 - 1)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                then.return_error(
                    module,
                    compilation_ctx,
                    Some(ValType::I32),
                    runtime_error_data,
                    RuntimeError::OutOfBounds,
                );
            },
            |_| {},
        );

    // The enum should occupy only 4 bytes since only the variant number is saved
    let enum_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(enum_ptr)
        .local_get(variant_number)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    builder.local_get(enum_ptr);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compilation_context::module_data::ModuleId,
        test_compilation_context,
        test_tools::{
            INITIAL_MEMORY_OFFSET, assert_runtime_error, build_module, setup_wasmtime_module,
        },
        translation::intermediate_types::enums::{IEnum, IEnumVariant},
    };
    use alloy_sol_types::{SolType, sol};
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::FunctionBuilder;

    // Helper function to create a test enum with a specific number of variants
    fn create_test_enum(
        variant_count: usize,
        module_data: &mut crate::ModuleData,
    ) -> IntermediateType {
        let mut variants = Vec::new();
        for i in 0..variant_count {
            variants.push(IEnumVariant::new(i as u16, 0, Vec::new()));
        }

        let enum_ = IEnum::new("TestEnum", 0, variants).unwrap();

        // Add enum to the provided module data
        module_data.enums.enums.push(enum_);

        IntermediateType::IEnum {
            module_id: ModuleId::new(
                crate::compilation_context::module_data::Address::from([0; 32]),
                "test",
            ),
            index: 0,
        }
    }

    #[rstest]
    #[case(0, 0u8)]
    #[case(1, 1u8)]
    #[case(2, 2u8)]
    fn test_unpack_enum_variant(#[case] variant_number: u8, #[case] expected_variant: u8) {
        type SolType = sol!((uint8,));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        // Create a test enum with 3 variants
        let mut module_data = Box::new(crate::ModuleData::default());
        let enum_type = create_test_enum(3, &mut module_data);

        // Update compilation context to use the new module data
        compilation_ctx.root_module_data = &*module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let reader_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Set reader pointer to start of memory
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(reader_pointer);

        // Call unpack_enum_function
        let unpack_enum_func = unpack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut runtime_error_data,
            &enum_type,
        )
        .unwrap();

        func_body.local_get(reader_pointer);
        func_body.call(unpack_enum_func);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // ABI encode the variant number as u8
        let abi_encoded = SolType::abi_encode_params(&(variant_number,));

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, abi_encoded, "test_function", None);

        let enum_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();

        // Read the variant number from the enum struct
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut variant_bytes = [0u8; 4];
        memory
            .read(&mut store, enum_ptr as usize, &mut variant_bytes)
            .unwrap();
        let variant = u32::from_le_bytes(variant_bytes);

        assert_eq!(
            variant, expected_variant as u32,
            "Unpacked enum variant did not match expected result"
        );
    }

    #[rstest]
    #[case(3)] // Out of bounds for 3-variant enum
    #[case(255)]
    fn test_unpack_enum_variant_out_of_bounds(
        #[case] variant_number: u8,
    ) -> Result<(), RuntimeFunctionError> {
        type SolType = sol!((uint8,));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        // Create a test enum with 3 variants (0, 1, 2)
        let mut module_data = Box::new(crate::ModuleData::default());
        let enum_type = create_test_enum(3, &mut module_data);

        // Update compilation context to use the new module data
        compilation_ctx.root_module_data = &*module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let reader_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Set reader pointer to start of memory
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(reader_pointer);

        // Call unpack_enum_function
        let unpack_enum_func = unpack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut runtime_error_data,
            &enum_type,
        )
        .unwrap();

        func_body.local_get(reader_pointer);
        func_body.call(unpack_enum_func);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // ABI encode the variant number as u8
        let abi_encoded = SolType::abi_encode_params(&(variant_number,));

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, abi_encoded, "test_function", None);

        let _: i32 = entrypoint.call(&mut store, ()).unwrap();

        assert_runtime_error(&mut store, &instance, RuntimeError::OutOfBounds);

        Ok(())
    }

    #[test]
    fn test_unpack_enum_variant_fuzz() {
        type SolType = sol!((uint8,));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        // Create a test enum with 10 variants (0-9 are valid)
        let mut module_data = Box::new(crate::ModuleData::default());
        let enum_type = create_test_enum(10, &mut module_data);

        // Update compilation context to use the new module data
        compilation_ctx.root_module_data = &*module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let reader_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Set reader pointer to start of memory
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(reader_pointer);

        // Call unpack_enum_function
        let unpack_enum_func = unpack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut runtime_error_data,
            &enum_type,
        )
        .unwrap();

        func_body.local_get(reader_pointer);
        func_body.call(unpack_enum_func);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u8>()
            .cloned()
            .for_each(|variant: u8| {
                // ABI encode the variant number
                let abi_encoded = SolType::abi_encode_params(&(variant,));

                // Write the encoded data to memory at INITIAL_MEMORY_OFFSET
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_encoded,
                    )
                    .unwrap();

                let mut store = store.0.borrow_mut();
                let result: Result<i32, _> = entrypoint.0.call(&mut *store, ());

                if variant < 10 {
                    // Should succeed for variants 0-9
                    match result {
                        Ok(enum_ptr) => {
                            // Read the variant number from the enum struct
                            let mut variant_bytes = [0u8; 4];
                            memory
                                .read(&mut *store, enum_ptr as usize, &mut variant_bytes)
                                .unwrap();
                            let result_variant = u32::from_le_bytes(variant_bytes);

                            assert_eq!(
                                result_variant, variant as u32,
                                "Unpacked enum variant did not match expected result for variant {variant}",
                            );
                        }
                        Err(_) => {
                            panic!("Expected success for variant {variant} but got error");
                        }
                    }
                } else {
                    // Should trap for out-of-bounds variants (>= 10)
                    assert_runtime_error(&mut store, &instance, RuntimeError::OutOfBounds);
                }

                reset_memory.0.call(&mut *store, ()).unwrap();
            });
    }
}
