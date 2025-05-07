use walrus::{
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
    ir::{MemArg, StoreKind},
};

use super::IntermediateType;

#[derive(Clone)]
pub struct IVector;

impl IVector {
    pub fn load_constant_instructions(
        inner: &IntermediateType,
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        // First byte is the length of the vector
        let vec_len = bytes.next().unwrap();

        let data_size: usize = inner.stack_data_size() as usize;

        // Vec len as i32 + data size * vec len
        let needed_bytes = 4 + data_size * (vec_len as usize);

        let pointer = module_locals.add(ValType::I32);

        builder.i32_const(needed_bytes as i32);
        builder.call(allocator);
        builder.local_tee(pointer);

        // Store length
        builder.i32_const(vec_len as i32);
        builder.store(
            memory,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        let mut store_offset: u32 = 4;

        builder.local_get(pointer);
        while (store_offset as usize) < needed_bytes {
            // Load the inner type
            inner.load_constant_instructions(module_locals, builder, bytes, allocator, memory);

            if data_size == 4 {
                // Store i32
                builder.store(
                    memory,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: store_offset,
                    },
                );

                store_offset += 4;
            } else if data_size == 8 {
                // Store i64
                builder.store(
                    memory,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: store_offset,
                    },
                );

                store_offset += 8;
            } else {
                panic!("Unsupported data size for vector: {}", data_size);
            }

            builder.local_get(pointer);
        }

        assert_eq!(
            needed_bytes, store_offset as usize,
            "Store offset is not aligned with the needed bytes"
        );
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::U256;
    use walrus::{FunctionBuilder, FunctionId, MemoryId, Module, ModuleConfig, ValType};
    use wasmtime::{Engine, Instance, Linker, Module as WasmModule, Store, TypedFunc, WasmResults};

    use crate::memory::setup_module_memory;

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module<R: WasmResults>(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        function_name: &str,
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

    fn test_vector(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        let data = data.to_vec();
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module.locals,
            &mut builder,
            &mut data.into_iter(),
            allocator,
            memory_id,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, vec![], "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_vector_bool() {
        let data = vec![4, 1, 0, 1, 0];
        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IBool, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u8() {
        let data = vec![4, 1, 2, 3, 4];
        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU8, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u16() {
        let data = [
            &[4u8],
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU16, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u32() {
        let data = [
            &[4u8],
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU32, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u64() {
        let data = [
            &[4u8],
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU64, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u128() {
        let data = [
            &[4u8],
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            36u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            68u32.to_le_bytes().as_slice(),
            // Referenced values
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU128, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u256() {
        let data = [
            &[4u8],
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
            U256::from(3u128).to_le_bytes::<32>().as_slice(),
            U256::from(4u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            116u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
            U256::from(3u128).to_le_bytes::<32>().as_slice(),
            U256::from(4u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU256, &expected_result_bytes);
    }

    #[test]
    fn test_vector_address() {
        let data = [
            &[4u8],
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            116u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IAddress, &expected_result_bytes);
    }

    #[test]
    fn test_vector_vector_u32() {
        let data = [
            &[2u8],
            [
                &[4u8],
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                &[4u8],
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(), // pointer to first vector
            32u32.to_le_bytes().as_slice(), // pointer to second vector
            [
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();
        test_vector(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vector_vector_u256() {
        let data = [
            &[2u8],
            [
                &[4u8],
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                &[4u8],
                U256::from(5u128).to_le_bytes::<32>().as_slice(),
                U256::from(6u128).to_le_bytes::<32>().as_slice(),
                U256::from(7u128).to_le_bytes::<32>().as_slice(),
                U256::from(8u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),  // pointer to first vector
            160u32.to_le_bytes().as_slice(), // pointer to second vector
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                32u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
                96u32.to_le_bytes().as_slice(),
                128u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                180u32.to_le_bytes().as_slice(),
                212u32.to_le_bytes().as_slice(),
                244u32.to_le_bytes().as_slice(),
                276u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(5u128).to_le_bytes::<32>().as_slice(),
                U256::from(6u128).to_le_bytes::<32>().as_slice(),
                U256::from(7u128).to_le_bytes::<32>().as_slice(),
                U256::from(8u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();
        test_vector(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_result_bytes,
        );
    }
}
