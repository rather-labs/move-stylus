use walrus::{FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType, ir::BinaryOp};

use super::{Packable, pack_native_int::pack_i32_type_instructions};
use crate::translation::intermediate_types::{IntermediateType, vector::IVector};

impl IVector {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pack_instructions(
        inner: &IntermediateType,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        memory: MemoryId,
        alloc_function: FunctionId,
    ) {
        let data_pointer = module.locals.add(ValType::I32);
        let inner_data_reference = module.locals.add(ValType::I32);

        let length = IntermediateType::IU32.add_load_memory_to_local_instructions(
            &mut module.locals,
            block,
            local,
            memory,
        );

        // Allocate memory for the packed value, this will be allocate at the end of calldata
        block.local_get(length);
        block.i32_const(inner.encoded_size() as i32); // The size of each element
        block.binop(BinaryOp::I32Mul);
        block.i32_const(32); // The size of the length value itself
        block.binop(BinaryOp::I32Add);

        block.call(alloc_function);
        block.local_tee(data_pointer);

        // The value stored at this param position should be the distance from the start of this calldata portion to the pointer
        let reference_value = module.locals.add(ValType::I32);

        block.local_get(calldata_reference_pointer);
        block.binop(BinaryOp::I32Sub);
        block.local_set(reference_value);
        pack_i32_type_instructions(block, module, memory, reference_value, writer_pointer);

        // increment the local to point to first value
        block.local_get(local);
        block.i32_const(4);
        block.binop(BinaryOp::I32Add);
        block.local_set(local);

        /*
         *  Store the values at allocated memory at the end of calldata
         */

        // Length
        pack_i32_type_instructions(block, module, memory, length, data_pointer);

        // increment the data pointer
        block.local_get(data_pointer);
        block.i32_const(32);
        block.binop(BinaryOp::I32Add);
        block.local_tee(data_pointer);
        block.local_set(inner_data_reference); // This will be the reference for next allocated calldata

        // Loop through the vector values
        let i = module.locals.add(ValType::I32);
        block.i32_const(0);
        block.local_set(i);
        block.loop_(None, |loop_block| {
            let loop_block_id = loop_block.id();

            let inner_local = inner.add_load_memory_to_local_instructions(
                &mut module.locals,
                loop_block,
                local,
                memory,
            );

            inner.add_pack_instructions(
                loop_block,
                module,
                inner_local,
                data_pointer,
                inner_data_reference,
                memory,
                alloc_function,
            );

            // increment the local to point to next first value
            loop_block.local_get(local);
            loop_block.i32_const(inner.stack_data_size() as i32);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_set(local);

            // increment data pointer
            loop_block.local_get(data_pointer);
            loop_block.i32_const(inner.encoded_size() as i32);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_set(data_pointer);

            // increment i
            loop_block.local_get(i);
            loop_block.i32_const(1);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_tee(i);

            loop_block.local_get(length);
            loop_block.binop(BinaryOp::I32LtU);
            loop_block.br_if(loop_block_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        dyn_abi::SolType,
        hex::FromHex,
        primitives::{Address, U256},
        sol,
    };
    use walrus::{FunctionBuilder, FunctionId, MemoryId, ModuleConfig, ValType};
    use wasmtime::{Engine, Instance, Linker, Module as WasmModule, Store, TypedFunc, WasmResults};

    use crate::{
        abi_types::packing::Packable, memory::setup_module_memory,
        translation::intermediate_types::IntermediateType,
    };

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module<R: WasmResults>(
        module: &mut Module,
        function_name: &str,
        initial_memory_data: Vec<u8>,
    ) -> (Linker<()>, Instance, Store<()>, TypedFunc<(), R>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let linker = Linker::new(&engine);

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<(), R>(&mut store, function_name)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        memory.write(&mut store, 0, &initial_memory_data).unwrap();

        (linker, instance, store, entrypoint)
    }

    fn test_uint(int_type: impl Packable, data: &[u8], expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(int_type.encoded_size() as i32);
        func_body.call(alloc_function);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Args data should already be stored in memory
        int_type.add_pack_instructions(
            &mut func_body,
            &mut raw_module,
            local,
            writer_pointer,
            calldata_reference_pointer,
            memory_id,
            alloc_function,
        );

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, "test_function", data.to_vec());

        // the return is the pointer to the packed value
        let result = entrypoint.call(&mut store, ()).unwrap();

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
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU8));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU16));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU32));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU64));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                1u64.to_le_bytes().as_slice(),
                2u64.to_le_bytes().as_slice(),
                3u64.to_le_bytes().as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_pack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU128));

        let expected_result = SolType::abi_encode_params(&(vec![1, 2, 3],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                16u32.to_le_bytes().as_slice(),
                32u32.to_le_bytes().as_slice(),
                48u32.to_le_bytes().as_slice(),
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
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU256));

        let expected_result =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                16u32.to_le_bytes().as_slice(),
                48u32.to_le_bytes().as_slice(),
                80u32.to_le_bytes().as_slice(),
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
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IAddress));

        let expected_result = SolType::abi_encode_params(&(vec![
            Address::from_hex("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            Address::from_hex("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            Address::from_hex("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        ],));
        test_uint(
            int_type.clone(),
            &[
                3u32.to_le_bytes().as_slice(),
                16u32.to_le_bytes().as_slice(),
                48u32.to_le_bytes().as_slice(),
                80u32.to_le_bytes().as_slice(),
                &[0; 12],
                Address::from_hex("0x1234567890abcdef1234567890abcdef12345678")
                    .unwrap()
                    .as_slice(),
                &[0; 12],
                Address::from_hex("0x1234567890abcdef1234567890abcdef12345678")
                    .unwrap()
                    .as_slice(),
                &[0; 12],
                Address::from_hex("0x1234567890abcdef1234567890abcdef12345678")
                    .unwrap()
                    .as_slice(),
            ]
            .concat(),
            &expected_result,
        );
    }

    #[test]
    fn test_unpack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU32,
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let data = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_uint(int_type.clone(), &data, &expected_result);
    }

    #[test]
    fn test_unpack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU128,
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let data = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            76u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            92u32.to_le_bytes().as_slice(),
            108u32.to_le_bytes().as_slice(),
            124u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_uint(int_type.clone(), &data, &expected_result);
    }
}
