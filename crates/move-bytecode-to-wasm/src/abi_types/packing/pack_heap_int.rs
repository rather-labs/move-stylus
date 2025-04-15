use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{
    translation::intermediate_types::{
        address::IAddress,
        heap_integers::{IU128, IU256},
    },
    utils::add_swap_i64_bytes_function,
};

impl IU128 {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        // Little-endian to Big-endian
        let swap_i64_bytes_function = add_swap_i64_bytes_function(module);

        for i in 0..2 {
            block.local_get(writer_pointer);
            block.local_get(local);

            // Load from right to left
            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 8 - i * 8,
                },
            );
            block.call(swap_i64_bytes_function);

            // Store from left to right
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    // Abi is left-padded to 32 bytes
                    offset: 16 + i * 8,
                },
            );
        }
    }
}

impl IU256 {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        // Little-endian to Big-endian
        let swap_i64_bytes_function = add_swap_i64_bytes_function(module);

        for i in 0..4 {
            block.local_get(writer_pointer);
            block.local_get(local);

            // Load from right to left
            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 24 - i * 8,
                },
            );
            block.call(swap_i64_bytes_function);

            // Store from left to right
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }
    }
}

impl IAddress {
    /// Address is packed as a u160, but endianness is not relevant
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        _module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        for i in 0..4 {
            block.local_get(writer_pointer);
            block.local_get(local);

            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }
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

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(int_type.encoded_size() as i32);
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        // Args data should already be stored in memory
        int_type.add_pack_instructions(
            &mut func_body,
            &mut raw_module,
            local,
            writer_pointer,
            writer_pointer, // unused for this type
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
    fn test_pack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let expected_result = SolType::abi_encode_params(&(128128128128,));
        test_uint(
            int_type.clone(),
            &128128128128u128.to_le_bytes(),
            &expected_result,
        );

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            &IntType::MAX.to_le_bytes(),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            &IntType::MIN.to_le_bytes(),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &(IntType::MAX - 1).to_le_bytes(),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_pack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let expected_result = SolType::abi_encode_params(&(U256::from(256256256256u128),));
        test_uint(
            int_type.clone(),
            &U256::from(256256256256u128).to_le_bytes::<32>(),
            &expected_result,
        );

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            &IntType::MAX.to_le_bytes::<32>(),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            &IntType::MIN.to_le_bytes::<32>(),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        test_uint(
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_pack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let expected_result = SolType::abi_encode_params(&(Address::ZERO,));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result = SolType::abi_encode_params(&(Address::from_hex(
            "0x1234567890abcdef1234567890abcdef12345678",
        )
        .unwrap(),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result = SolType::abi_encode_params(&(Address::from_hex(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        )
        .unwrap(),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result = SolType::abi_encode_params(&(Address::from_hex(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE",
        )
        .unwrap(),));
        test_uint(int_type.clone(), &expected_result, &expected_result);
    }
}
