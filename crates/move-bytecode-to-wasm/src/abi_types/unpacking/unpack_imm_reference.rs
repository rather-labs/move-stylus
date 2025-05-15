use super::Unpackable;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::intermediate_types::imm_reference::IRef;
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{MemArg, StoreKind},
};

impl IRef {
    pub fn add_unpack_instructions(
        inner: &IntermediateType,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        let ptr_local = module.locals.add(walrus::ValType::I32);

        // 1. Allocate memory
        builder.i32_const(inner.stack_data_size() as i32);
        builder.call(allocator);
        builder.local_tee(ptr_local);

        // 2. Unpack the inner value (which will push the value)
        inner.add_unpack_instructions(
            builder,
            module,
            reader_pointer,
            calldata_reader_pointer,
            memory,
            allocator,
        );

        // 3. Store the inner value at the allocated address
        builder.store(
            memory,
            match inner.stack_data_size() {
                4 => StoreKind::I32 { atomic: false },
                8 => StoreKind::I64 { atomic: false },
                _ => panic!("Unsupported stack_data_size for IRef"),
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        builder.local_get(ptr_local);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::setup_module_memory;
    use crate::translation::intermediate_types::IntermediateType;
    use alloy::primitives::U256;
    use alloy::{dyn_abi::SolType, sol};
    use walrus::{FunctionBuilder, ModuleConfig, ValType};
    use wasmtime::{
        Engine, Global, Instance, Linker, Module as WasmModule, Store, TypedFunc, WasmResults,
    };

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
    ) -> (Linker<()>, Instance, Store<()>, TypedFunc<(), R>, Global) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let linker = Linker::new(&engine);

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<(), R>(&mut store, function_name)
            .unwrap();

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        memory.write(&mut store, 0, &initial_memory_data).unwrap();

        (
            linker,
            instance,
            store,
            entrypoint,
            global_next_free_memory_pointer,
        )
    }

    fn test_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            calldata_reader_pointer,
            memory_id,
            allocator,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint, global_next_free_memory_pointer) =
            setup_wasmtime_module::<i32>(&mut raw_module, data.to_vec(), "test_function");

        let result_ptr = entrypoint.call(&mut store, ()).unwrap();

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

    #[test]
    fn test_unpack_ref_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88,));
        test_ref(&data, int_type.clone(), &88u8.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u32() {
        type IntType = u32;
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        test_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type IntType = u64;
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        test_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123456789u128,));

        let mut expected = Vec::new();
        expected.extend(&4u32.to_le_bytes());
        expected.extend(&123456789u128.to_le_bytes());

        test_ref(&data, int_type.clone(), &expected);
    }

    // Tests for heap types are failing because unpacked data comes with a 4 0 0 0 header at the beginning
    // Im not sure why
    #[test]
    fn test_unpack_ref_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let data = SolType::abi_encode_params(&(value,));
        test_ref(&data, int_type.clone(), &value.to_le_bytes::<32>());
    }

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let inner_type = IntermediateType::IU8;
        let vector_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            inner_type.clone(),
        ))));

        let vec_data = vec![1u8, 2u8, 3u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        // Expectation: the heap-allocated vector will have [length as u32 LE][1][2][3]
        let mut expected = Vec::new();
        expected.extend(&(vec_data.len() as u32).to_le_bytes());
        expected.extend(&vec_data);

        test_ref(&data, vector_type.clone(), &expected);
    }
}
