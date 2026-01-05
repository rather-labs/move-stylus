use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    abi_types::packing::Packable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

/// Generates a WASM function that packs a Move vector into Solidity ABI dynamic array format.
///
/// The function allocates memory at the end of calldata for the packed array, writes an offset
/// at the writer_pointer location, then writes the array length followed by the packed elements.
/// For dynamic inner types (vectors, structs, etc.), it writes offsets to the actual data.
///
/// # WASM Function Arguments:
/// * `vector_pointer` (i32) - pointer to the Move vector structure (contains length, capacity, and data)
/// * `writer_pointer` (i32) - pointer where the offset to the packed array should be written
/// * `calldata_reference_pointer` (i32) - reference point for calculating relative offsets
///
/// # WASM Function Returns:
/// * None - the result is written directly to memory, with an offset at writer_pointer and the array data at the end of calldata
pub fn pack_vector_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::PackVector.get_generic_function_name(compilation_ctx, &[inner])?;
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
    let vector_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    let data_pointer = module.locals.add(ValType::I32);
    let inner_data_reference = module.locals.add(ValType::I32);

    let len = IntermediateType::IU32.add_load_memory_to_local_instructions(
        module,
        &mut builder,
        vector_pointer,
        compilation_ctx.memory_id,
    )?;

    let inner_encoded_size = if inner.is_dynamic(compilation_ctx)? {
        32
    } else {
        inner.encoded_size(compilation_ctx)? as i32
    };

    // Allocate memory for the packed value, this will be allocate at the end of calldata
    // len * inner_encoded_size + 32 (length header)
    builder
        .local_get(len)
        .i32_const(inner_encoded_size)
        .binop(BinaryOp::I32Mul)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(data_pointer);

    // The value stored at this param position should be the distance from the start of this
    // calldata portion to the pointer
    let reference_value = module.locals.add(ValType::I32);

    builder
        .local_get(calldata_reference_pointer)
        .binop(BinaryOp::I32Sub)
        .local_set(reference_value);

    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;
    builder
        .local_get(reference_value)
        .local_get(writer_pointer)
        .call(pack_u32_function);

    // Set the vector pointer to point to the first element
    builder
        .skip_vec_header(vector_pointer)
        .local_set(vector_pointer);

    /*
     *  Store the values at allocated memory at the end of calldata
     */

    // Length
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;
    builder
        .local_get(len)
        .local_get(data_pointer)
        .call(pack_u32_function);

    // increment the data pointer
    builder
        .local_get(data_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_tee(data_pointer)
        .local_set(inner_data_reference); // This will be the reference for next allocated calldata

    // Outer block: if the vector length is 0, we skip to the end
    let mut inner_result: Result<(), AbiError> = Ok(());
    builder.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        // Loop through the vector values
        let i = module.locals.add(ValType::I32);
        outer_block.i32_const(0).local_set(i);
        outer_block.loop_(None, |loop_block| {
            inner_result = (|| {
                let loop_block_id = loop_block.id();

                let inner_local = inner.add_load_memory_to_local_instructions(
                    module,
                    loop_block,
                    vector_pointer,
                    compilation_ctx.memory_id,
                )?;

                if inner.is_dynamic(compilation_ctx)? {
                    inner.add_pack_instructions_dynamic(
                        loop_block,
                        module,
                        inner_local,
                        data_pointer,
                        inner_data_reference,
                        compilation_ctx,
                    )?;
                } else {
                    inner.add_pack_instructions(
                        loop_block,
                        module,
                        inner_local,
                        data_pointer,
                        inner_data_reference,
                        compilation_ctx,
                    )?;
                }

                // Increment the vector pointer to point to next value
                loop_block
                    .local_get(vector_pointer)
                    .i32_const(inner.wasm_memory_data_size()?)
                    .binop(BinaryOp::I32Add)
                    .local_set(vector_pointer);

                // increment data pointer
                loop_block
                    .local_get(data_pointer)
                    .i32_const(inner_encoded_size)
                    .binop(BinaryOp::I32Add)
                    .local_set(data_pointer);

                // increment i
                loop_block
                    .local_get(i)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_tee(i);

                loop_block
                    .local_get(len)
                    .binop(BinaryOp::I32LtU)
                    .br_if(loop_block_id);
                Ok(())
            })();
        });
    });

    Ok(function.finish(
        vec![vector_pointer, writer_pointer, calldata_reference_pointer],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use std::sync::Arc;

    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::SolValue;
    use rstest::rstest;
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    #[rstest]
    #[case::vector_u8(
        IntermediateType::IVector(Arc::new(IntermediateType::IU8)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice()].concat(),
        vec![1i8, 2i8, 3i8].abi_encode()
    )]
    #[case::vector_u16(
        IntermediateType::IVector(Arc::new(IntermediateType::IU16)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            0u16.to_le_bytes().as_slice(),
            0u16.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice()].concat(),
        vec![1u16, 2u16, 3u16].abi_encode()
    )]
    #[case::vector_u32(
        IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice()].concat(),
        vec![1u32, 2u32, 3u32].abi_encode()
    )]
    #[case::vector_u64(
        IntermediateType::IVector(Arc::new(IntermediateType::IU64)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            0u64.to_le_bytes().as_slice(),
            0u64.to_le_bytes().as_slice(),
            0u64.to_le_bytes().as_slice()].concat(),
        vec![1u64, 2u64, 3u64].abi_encode()
    )]
    #[case::vector_u128(
        IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            32u32.to_le_bytes().as_slice(),
            48u32.to_le_bytes().as_slice(),
            64u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice()].concat(),
        vec![1u128, 2u128, 3u128].abi_encode()
    )]
    #[case::vector_u256(
        IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
        [3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            32u32.to_le_bytes().as_slice(),
            64u32.to_le_bytes().as_slice(),
            96u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            U256::from(1).to_le_bytes::<32>().as_slice(),
            U256::from(2).to_le_bytes::<32>().as_slice(),
            U256::from(3).to_le_bytes::<32>().as_slice()].concat(),
        vec![U256::from(1), U256::from(2), U256::from(3)].abi_encode()
    )]
    #[case::vector_address(
        IntermediateType::IVector(Arc::new(IntermediateType::IAddress)),
        vec![
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            32u32.to_le_bytes().as_slice(),
            64u32.to_le_bytes().as_slice(),
            96u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
        ].concat(),
        vec![
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
        ].abi_encode()
    )]
    #[case::vector_vector_u32(
        IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        )))),
        vec![
            2u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            24u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
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
        vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32]].abi_encode()
    )]
    #[case::vector_vector_u128(
        IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        )))),
        vec![
            2u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            24u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
            72u32.to_le_bytes().as_slice(),
            88u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            136u32.to_le_bytes().as_slice(),
            152u32.to_le_bytes().as_slice(),
            168u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
        ].concat(),
        vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128]].abi_encode()
    )]
    fn test_vec_packing(
        #[case] int_type: IntermediateType,
        #[case] data: Vec<u8>,
        #[case] expected_result: Vec<u8>,
    ) {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(int_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(alloc_function);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Args data should already be stored in memory
        int_type
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
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        // the return is the pointer to the packed value
        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_vector_u8_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU8));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
                let capacity = len;
                let mut data: Vec<u8> = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());
                for value in &values {
                    data.push(value.to_le_bytes()[0]);
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<u8> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_u16_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU16));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
                let capacity = len;
                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<u16> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_u32_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU32));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u32>>()
            .for_each(|values: &Vec<u32>| {
                let len = values.len() as u32;
                let capacity = len;
                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<u32> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_u64_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU64));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
                let capacity = len;
                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());
                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<u64> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_u128_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU128));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
                let capacity = len;
                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());

                let base_offset = 8 + (len as usize * 4);
                for (i, _) in values.iter().enumerate() {
                    let value_offset = base_offset + (i * 16);
                    data.extend(&(value_offset as u32).to_le_bytes());
                }

                for value in values {
                    data.extend(&value.to_le_bytes());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<u128> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_u256_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU256));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<[u8; 32]>>()
            .for_each(|values: &Vec<[u8; 32]>| {
                let len = values.len() as u32;
                let capacity = len;
                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());

                let base_offset = 8 + (len as usize * 4);
                for (i, _) in values.iter().enumerate() {
                    let value_offset = base_offset + (i * 32);
                    data.extend(&(value_offset as u32).to_le_bytes());
                }

                for value in values {
                    data.extend(value);
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
                    .unwrap();

                let expected = values
                    .iter()
                    .map(|v| U256::from_le_bytes(*v))
                    .collect::<Vec<U256>>()
                    .abi_encode();

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
                    "Packed vec<u256> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_address_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IAddress));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<[u8; 20]>>()
            .for_each(|values: &Vec<[u8; 20]>| {
                let len = values.len() as u32;
                let capacity = len;

                let mut data = vec![];
                data.extend(&len.to_le_bytes());
                data.extend(&capacity.to_le_bytes());

                let base_offset = 8 + (len as usize * 4);
                for (i, _) in values.iter().enumerate() {
                    let value_offset = base_offset + (i * 32);
                    data.extend(&(value_offset as u32).to_le_bytes());
                }

                for value in values {
                    data.extend(&[0u8; 12]);
                    data.extend(value.as_slice());
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
                    .unwrap();

                let expected = values
                    .iter()
                    .map(Address::from)
                    .collect::<Vec<Address>>()
                    .abi_encode();

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
                    "Packed vec<address> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_vector_u32_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
            .for_each(|values: &Vec<Vec<u32>>| {
                let outer_len = values.len() as u32;
                let outer_capacity = outer_len;
                let mut data = vec![];
                data.extend(&outer_len.to_le_bytes());
                data.extend(&outer_capacity.to_le_bytes());

                let mut current_offset = 8 + (outer_len as usize * 4);
                for inner_vec in values {
                    data.extend(&(current_offset as u32).to_le_bytes());
                    current_offset += 8 + (inner_vec.len() * 4);
                }

                for inner_vec in values {
                    let inner_len = inner_vec.len() as u32;
                    data.extend(&inner_len.to_le_bytes());
                    data.extend(&inner_len.to_le_bytes());
                    for value in inner_vec {
                        data.extend(&value.to_le_bytes());
                    }
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<vec<u32>> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_vector_vector_u128_fuzz() {
        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(local);

        func_body.i32_const(vector_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        vector_type
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

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

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
            .for_each(|values: &Vec<Vec<u128>>| {
                let outer_len = values.len() as u32;
                let outer_capacity = outer_len;
                let mut data = vec![];
                data.extend(&outer_len.to_le_bytes());
                data.extend(&outer_capacity.to_le_bytes());

                let mut inner_vec_ptr_offset = 8 + (outer_len as usize * 4);
                for inner_vec in values {
                    data.extend(&(inner_vec_ptr_offset as u32).to_le_bytes());
                    inner_vec_ptr_offset += 8 + (inner_vec.len() * 4);
                }

                let mut u128_data_offset = inner_vec_ptr_offset;
                for inner_vec in values {
                    let inner_len = inner_vec.len() as u32;
                    data.extend(&inner_len.to_le_bytes());
                    data.extend(&inner_len.to_le_bytes());

                    for _ in 0..inner_vec.len() {
                        data.extend(&(u128_data_offset as u32).to_le_bytes());
                        u128_data_offset += 16;
                    }
                }

                for inner_vec in values {
                    for value in inner_vec {
                        data.extend(&value.to_le_bytes());
                    }
                }

                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), data.len() as i32)
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
                    "Packed vec<vec<u128>> did not match expected result",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}
