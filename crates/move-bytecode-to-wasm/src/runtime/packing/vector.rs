use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    abi_types::packing::Packable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

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
    use std::rc::Rc;

    use alloy_primitives::{U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{ConstExpr, FunctionBuilder, ValType, ir::Value};

    use crate::{
        abi_types::packing::Packable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    fn test_vec_packing(int_type: impl Packable, data: &[u8], expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            false,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
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
    fn test_pack_vector_u8() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU8));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                1u8.to_le_bytes().as_slice(),
                2u8.to_le_bytes().as_slice(),
                3u8.to_le_bytes().as_slice(),
                0u8.to_le_bytes().as_slice(),
                0u8.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU16));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                1u16.to_le_bytes().as_slice(),
                2u16.to_le_bytes().as_slice(),
                3u16.to_le_bytes().as_slice(),
                0u16.to_le_bytes().as_slice(),
                0u16.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU32));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU64));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                1u64.to_le_bytes().as_slice(),
                2u64.to_le_bytes().as_slice(),
                3u64.to_le_bytes().as_slice(),
                0u64.to_le_bytes().as_slice(),
                0u64.to_le_bytes().as_slice(),
                0u64.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU128));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                32u32.to_le_bytes().as_slice(),
                48u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                1u128.to_le_bytes().as_slice(),
                2u128.to_le_bytes().as_slice(),
                3u128.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u256() {
        type SolType = sol!((uint256[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU256));

        let expected_result =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        test_vec_packing(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                32u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
                96u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                0u32.to_le_bytes().as_slice(),
                U256::from(1).to_le_bytes::<32>().as_slice(),
                U256::from(2).to_le_bytes::<32>().as_slice(),
                U256::from(3).to_le_bytes::<32>().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_address() {
        type SolType = sol!((address[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IAddress));

        let expected_result = SolType::abi_encode_params(&(vec![
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
        ],));
        test_vec_packing(
            int_type.clone(),
            &[
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
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU32,
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let data = [
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
        ]
        .concat();
        test_vec_packing(int_type.clone(), &data, &expected_result);
    }

    #[test]
    fn test_pack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU128,
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let data = [
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
        ]
        .concat();
        test_vec_packing(int_type.clone(), &data, &expected_result);
    }
}
