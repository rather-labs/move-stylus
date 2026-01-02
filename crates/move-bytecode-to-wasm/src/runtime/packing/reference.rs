use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiOperationError},
    abi_types::packing::Packable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg},
};

pub fn pack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::PackReference.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder: walrus::InstrSeqBuilder<'_> = function.name(name).func_body();

    // Arguments
    let reference_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    match inner {
        IntermediateType::ISigner
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            // Load the intermediate pointer and pack
            builder
                .local_get(reference_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(reference_pointer);

            inner.add_pack_instructions(
                &mut builder,
                module,
                reference_pointer,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            )?;
        }
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            // Load the intermediate pointer
            builder
                .local_get(reference_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_tee(reference_pointer);

            builder.load(
                compilation_ctx.memory_id,
                inner.load_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            let value_local = module.locals.add(ValType::try_from(inner)?);
            builder.local_set(value_local);

            inner.add_pack_instructions(
                &mut builder,
                module,
                value_local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            )?;
        }
        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            Err(AbiError::Pack(AbiOperationError::RefInsideRef))?;
        }
        IntermediateType::ITypeParameter(_) => {
            Err(AbiError::Pack(
                AbiOperationError::PackingGenericTypeParameter,
            ))?;
        }
    }

    Ok(function.finish(
        vec![
            reference_pointer,
            writer_pointer,
            calldata_reference_pointer,
        ],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::test_compilation_context;
    use crate::test_tools::build_module;
    use crate::test_tools::setup_wasmtime_module;
    use crate::translation::intermediate_types::IntermediateType;
    use alloy_primitives::Address;
    use alloy_primitives::U256;
    use alloy_primitives::address;
    use alloy_sol_types::SolValue;
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};

    // NOTE: The first value of data is always the pointer to the data
    // NOTE: u8 is represented as i8 because u8 does not implement abi_encode() and alloy
    // interprets them as bytes and not numbers
    #[rstest]
    // test_pack_ref_u8
    #[case::u8(
        IntermediateType::IRef(Arc::new(IntermediateType::IU8)),
        &[4u32.to_le_bytes().as_slice(), 88u8.to_le_bytes().as_slice()].concat(),
        &88_i8.abi_encode()
    )]
    // test_pack_ref_u32
    #[case::u32(
        IntermediateType::IRef(Arc::new(IntermediateType::IU32)),
        &[4u32.to_le_bytes().as_slice(), 88u32.to_le_bytes().as_slice()].concat(),
        &88_u32.abi_encode()
    )]
    // test_pack_ref_u64
    #[case::u64(
        IntermediateType::IRef(Arc::new(IntermediateType::IU64)),
        &[4u32.to_le_bytes().as_slice(), 88u64.to_le_bytes().as_slice()].concat(),
        &88_u64.abi_encode()
    )]
    // test_pack_ref_u128
    #[case::u128(
        IntermediateType::IRef(Arc::new(IntermediateType::IU128)),
        &[4u32.to_le_bytes().as_slice(), 88u128.to_le_bytes().as_slice()].concat(),
        &88_u128.abi_encode()
    )]
    // test_pack_ref_address
    #[case::address(
        IntermediateType::IRef(Arc::new(IntermediateType::IAddress)),
        &[
            4u32.to_le_bytes().as_slice(),
            (address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").abi_encode()).as_slice()
        ].concat(),
        &(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),).abi_encode()
    )]
    #[should_panic]
    #[case::signer(
        IntermediateType::IRef(Arc::new(IntermediateType::ISigner)),
        &[
            4u32.to_le_bytes().as_slice(),
            (address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").abi_encode()).as_slice()
        ].concat(),
        &(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),).abi_encode()
    )]
    // test_pack_ref_vec_u8
    #[case::vec_u8(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU8)))),
        &[
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ].concat(),
        &vec![1i8, 2i8, 3i8].abi_encode()
    )]
    // test_pack_ref_vec_u16
    #[case::vec_u16(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU16)))),
        &[
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
        ].concat(),
        &vec![1u16, 2u16, 3u16].abi_encode()
    )]
    // test_pack_ref_vec_u32
    #[case::vec_u32(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU32)))),
        &[
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ].concat(),
        &vec![1u32, 2u32, 3u32].abi_encode()
    )]
    // test_pack_ref_vec_u128
    #[case::vec_128(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU128)))),
        &[
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
        ].concat(),
        &vec![1u128, 2u128, 3u128].abi_encode()
    )]
    // test_pack_ref_vector_vector_u32
    #[case::vec_vec_u32(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU32)))))),
        &[
            4u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ].concat(),
        &vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32]].abi_encode()
    )]
    // test_pack_ref_vector_vector_u128
    #[case::vec_vec_u128(
        IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(IntermediateType::IU128)))))),
        &[
            4u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            88u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            40u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
            72u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            108u32.to_le_bytes().as_slice(),
            124u32.to_le_bytes().as_slice(),
            140u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
        ].concat(),
        &vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128]].abi_encode()
    )]
    fn test_pack_ref(
        #[case] ref_type: IntermediateType,
        #[case] data: &[u8],
        #[case] expected_calldata_bytes: &[u8],
    ) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate data (what to write)
        func_body.i32_const(data.len() as i32);
        func_body.call(allocator);
        func_body.local_set(local);

        // Allocate calldata (where to write)
        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        if ref_type.is_dynamic(&compilation_ctx).unwrap() {
            ref_type
                .add_pack_instructions_dynamic(
                    &mut func_body,
                    &mut raw_module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    &compilation_ctx,
                )
                .unwrap();
        } else {
            ref_type
                .add_pack_instructions(
                    &mut func_body,
                    &mut raw_module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    &compilation_ctx,
                )
                .unwrap();
        };

        // Return the writer pointer for reading the calldata back
        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_calldata_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_calldata_bytes,
            "Packed calldata did not match expected result"
        );
    }

    #[test]
    fn test_pack_ref_u8_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU8));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(5); // 4 bytes pointer + 1 byte u8
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<i8>()
            .cloned()
            .for_each(|value: i8| {
                let data = [
                    4u32.to_le_bytes().as_slice(),
                    value.to_le_bytes().as_slice(),
                ]
                .concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref u8 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_u32_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU32));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(8); // 4 bytes pointer + 4 bytes u32
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<u32>()
            .cloned()
            .for_each(|value: u32| {
                let data = [
                    4u32.to_le_bytes().as_slice(),
                    value.to_le_bytes().as_slice(),
                ]
                .concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref u32 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_u64_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU64));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(12); // 4 bytes pointer + 8 bytes u64
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<u64>()
            .cloned()
            .for_each(|value: u64| {
                let data = [
                    4u32.to_le_bytes().as_slice(),
                    value.to_le_bytes().as_slice(),
                ]
                .concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref u64 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_u128_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU128));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(20); // 4 bytes pointer + 16 bytes u128
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<u128>()
            .cloned()
            .for_each(|value: u128| {
                let data = [
                    4u32.to_le_bytes().as_slice(),
                    value.to_le_bytes().as_slice(),
                ]
                .concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref u128 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_u256_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU256));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(36); // 4 bytes pointer + 32 bytes u256
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<[u8; 32]>()
            .cloned()
            .for_each(|bytes: [u8; 32]| {
                let value = U256::from_le_bytes(bytes);
                let data = [4u32.to_le_bytes().as_slice(), bytes.as_slice()].concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref U256 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_address_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IAddress));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(36); // 4 bytes pointer + 32 bytes address (padded)
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<[u8; 20]>()
            .cloned()
            .for_each(|bytes: [u8; 20]| {
                let value = Address::from_slice(&bytes);
                let addr_data = value.abi_encode();
                let data = [4u32.to_le_bytes().as_slice(), addr_data.as_slice()].concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref address did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_u16_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU16));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(6); // 4 bytes pointer + 2 bytes u16
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

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
            .with_type::<u16>()
            .cloned()
            .for_each(|value: u16| {
                let data = [
                    4u32.to_le_bytes().as_slice(),
                    value.to_le_bytes().as_slice(),
                ]
                .concat();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref u16 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_u8_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU8,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space); // Allocate enough space for vector data
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space); // Allocate space for packed output
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u16>>()
            .for_each(|values: &Vec<u16>| {
                let values = values.iter().map(|v| v % 256).collect::<Vec<u16>>();

                let len = values.len() as u32;
                let mut data = vec![];
                data.extend(&4u32.to_le_bytes()); // Pointer to vector
                data.extend(&len.to_le_bytes()); // Length
                data.extend(&len.to_le_bytes()); // Capacity
                for value in &values {
                    data.push(value.to_le_bytes()[0]);
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = values.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<u8> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_u16_fuzz() {
        use alloy_sol_types::SolValue;
        use std::cell::RefCell;
        use std::panic::AssertUnwindSafe;
        use std::rc::Rc;

        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU16,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u16>>()
            .for_each(|values: &Vec<u16>| {
                let len = values.len() as u32;
                let mut data = vec![];
                data.extend(&4u32.to_le_bytes());
                data.extend(&len.to_le_bytes());
                data.extend(&len.to_le_bytes());
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = values.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<u16> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_u64_fuzz() {
        use alloy_sol_types::SolValue;
        use std::cell::RefCell;
        use std::panic::AssertUnwindSafe;
        use std::rc::Rc;

        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU64,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u64>>()
            .for_each(|values: &Vec<u64>| {
                let len = values.len() as u32;
                let mut data = vec![];
                data.extend(&4u32.to_le_bytes());
                data.extend(&len.to_le_bytes());
                data.extend(&len.to_le_bytes());
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = values.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<u64> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_u128_fuzz() {
        use alloy_sol_types::SolValue;
        use std::cell::RefCell;
        use std::panic::AssertUnwindSafe;
        use std::rc::Rc;

        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u128>>()
            .for_each(|values: &Vec<u128>| {
                let len = values.len() as u32;
                let mut data = vec![];
                data.extend(&4u32.to_le_bytes()); // Pointer to vector
                data.extend(&len.to_le_bytes()); // Length
                data.extend(&len.to_le_bytes()); // Capacity

                // For u128, we need pointers to heap-allocated values
                let base_ptr = 12u32 + values.len() as u32 * 4;
                for (i, _) in values.iter().enumerate() {
                    let ptr = base_ptr + (i as u32 * 16);
                    data.extend(&ptr.to_le_bytes());
                }

                // Add the actual u128 values
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = values.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<u128> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_vec_u32_fuzz() {
        use alloy_sol_types::SolValue;
        use std::cell::RefCell;
        use std::panic::AssertUnwindSafe;
        use std::rc::Rc;

        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<Vec<u32>>>()
            .for_each(|outer_vec: &Vec<Vec<u32>>| {
                let outer_len = outer_vec.len() as u32;
                // Maximum of 30 sub-vectors to avoid excessive test times
                if outer_len > 30 {
                    return;
                }

                let mut data = vec![];
                data.extend(&4u32.to_le_bytes()); // Pointer to outer vector
                data.extend(&outer_len.to_le_bytes()); // Outer length
                data.extend(&outer_len.to_le_bytes()); // Outer capacity

                let mut inner_vec_ptrs = vec![];

                let base_ptr = 12u32 + (outer_len * 4);
                let mut current_ptr = base_ptr; // After outer vector header and pointers

                // First pass: calculate pointers and add them
                for inner in outer_vec {
                    inner_vec_ptrs.push(current_ptr);
                    current_ptr += 4 * inner.len() as u32 + 8;
                }

                // Add pointers to inner vectors
                for ptr in &inner_vec_ptrs {
                    data.extend(&ptr.to_le_bytes());
                }

                // Second pass: add inner vector data
                for inner in outer_vec {
                    let inner_len = inner.len() as u32;
                    data.extend(&inner_len.to_le_bytes()); // Inner length
                    data.extend(&inner_len.to_le_bytes()); // Inner capacity
                    for value in inner {
                        data.extend(&value.to_le_bytes());
                    }
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = outer_vec.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<vec<u32>> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_ref_vec_vec_u128_fuzz() {
        use alloy_sol_types::SolValue;
        use std::cell::RefCell;
        use std::panic::AssertUnwindSafe;
        use std::rc::Rc;

        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let packed_output_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.local_get(packed_output_space);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        ref_type
            .add_pack_instructions_dynamic(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function =
            function_builder.finish(vec![data_space, packed_output_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), i32>(
            &mut raw_module,
            vec![],
            "test_function",
            None,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<Vec<u128>>>()
            .for_each(|outer_vec: &Vec<Vec<u128>>| {
                let outer_len = outer_vec.len() as u32;
                // Maximum of 30 sub-vectors to avoid excessive test times
                if outer_len > 30 {
                    return;
                }

                let mut data = vec![];
                data.extend(&4u32.to_le_bytes()); // Pointer to outer vector
                data.extend(&outer_len.to_le_bytes()); // Outer length
                data.extend(&outer_len.to_le_bytes()); // Outer capacity

                let mut inner_vec_ptrs = vec![];

                let base_ptr = 12u32 + (outer_len * 4);
                let mut current_ptr = base_ptr; // After outer vector header and pointers

                // First pass: calculate pointers to inner vectors
                for inner in outer_vec {
                    inner_vec_ptrs.push(current_ptr);
                    // Each inner vector: 8 bytes header + (len * 4 bytes) for u128 pointers
                    current_ptr += 8 + (inner.len() as u32 * 4);
                }

                // Track where u128 values will be stored
                let u128_start_ptr = current_ptr;

                // Add pointers to inner vectors
                for ptr in &inner_vec_ptrs {
                    data.extend(&ptr.to_le_bytes());
                }

                // Second pass: add inner vector headers and pointers to u128 values
                let mut u128_ptr = u128_start_ptr;
                for inner in outer_vec {
                    let inner_len = inner.len() as u32;
                    data.extend(&inner_len.to_le_bytes()); // Inner length
                    data.extend(&inner_len.to_le_bytes()); // Inner capacity

                    // Add pointers to u128 values for this inner vector
                    for _ in 0..inner.len() {
                        data.extend(&u128_ptr.to_le_bytes());
                        u128_ptr += 16; // Each u128 is 16 bytes
                    }
                }

                // Third pass: add actual u128 values
                for inner in outer_vec {
                    for value in inner {
                        data.extend(&value.to_le_bytes());
                    }
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(
                        &mut *store.0.borrow_mut(),
                        (
                            data.len() as i32,
                            ref_type.encoded_size(&compilation_ctx).unwrap() as i32,
                        ),
                    )
                    .unwrap();

                let expected = outer_vec.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed ref vec<vec<u128>> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}
