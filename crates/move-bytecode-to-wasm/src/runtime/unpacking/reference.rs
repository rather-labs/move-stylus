use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiUnpackError},
    abi_types::unpacking::Unpackable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::MemArg};

pub fn unpack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    match itype {
        // If inner is a heap type, forward the pointer
        IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;
        }
        // For immediates, allocate and store
        IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IBool => {
            let ptr_local = module.locals.add(walrus::ValType::I32);

            let data_size = itype.wasm_memory_data_size()?;
            function_body
                .i32_const(data_size)
                .call(compilation_ctx.allocator)
                .local_tee(ptr_local);

            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;

            function_body.store(
                compilation_ctx.memory_id,
                itype.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            function_body.local_get(ptr_local);
        }

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::RefInsideRef,
            )));
        }
        IntermediateType::ITypeParameter(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::UnpackingGenericTypeParameter,
            )));
        }
    }

    function_builder.name(RuntimeFunction::UnpackReference.name().to_owned());
    Ok(function_builder.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{U256, address};
    use alloy_sol_types::{SolType, sol};
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    /// Test helper for unpacking reference types
    fn unpack_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(Some(data.len() as i32));
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_memory_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_memory_bytes,
            "Heap memory at returned pointer does not match expected content"
        );
    }

    // ============================================================================
    // Reference Types - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u8() {
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88u8,));
        let expected = 88u8.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u16() {
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(88u16,));
        let expected = 88u16.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u32() {
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        unpack_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        unpack_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    // ============================================================================
    // Reference Types - Heap-Allocated Element Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u128() {
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123u128,));
        let expected = 123u128.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u256() {
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let expected = value.to_le_bytes::<32>().to_vec();

        let data = SolType::abi_encode_params(&(value,));
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Rc::new(IntermediateType::IAddress));

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_ref(&data, ref_type.clone(), &data);
    }

    // ============================================================================
    // Reference Types - Vector Elements
    // ============================================================================

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let vector_type = IntermediateType::IRef(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU8,
        ))));

        let vec_data = vec![1u8, 2u8, 3u8, 4u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&4u32.to_le_bytes()); // length
        expected.extend(&4u32.to_le_bytes()); // capacity
        expected.extend(&1u8.to_le_bytes()); // first elem
        expected.extend(&2u8.to_le_bytes()); // second elem
        expected.extend(&3u8.to_le_bytes()); // third elem
        expected.extend(&4u8.to_le_bytes()); // fourth elem
        unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_u128() {
        type SolType = sol!((uint128[],));
        let vector_type = IntermediateType::IRef(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU128,
        ))));

        let vec_data = vec![1u128, 2u128, 3u128];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&3u32.to_le_bytes()); // length
        expected.extend(&3u32.to_le_bytes()); // capacity
        // pointers to heap elements
        expected.extend(&180u32.to_le_bytes());
        expected.extend(&196u32.to_le_bytes());
        expected.extend(&212u32.to_le_bytes());
        expected.extend(&1u128.to_le_bytes());
        expected.extend(&2u128.to_le_bytes());
        expected.extend(&3u128.to_le_bytes());

        unpack_ref(&data, vector_type.clone(), &expected);
    }
}
