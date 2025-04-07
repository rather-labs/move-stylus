use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::utils::add_swap_i64_bytes_function;

pub fn pack_u128_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
    local: LocalId,
) {
    let encoded_size = sol_data::Uint::<128>::ENCODED_SIZE.expect("U64 should have a fixed size");

    // Little-endian to Big-endian
    let swap_i64_bytes_function = add_swap_i64_bytes_function(module);

    let pointer = module.locals.add(ValType::I32);

    // Allocate memory for the packed value
    block.i32_const(encoded_size as i32);
    block.call(alloc_function);
    block.local_set(pointer);

    for i in 0..2 {
        block.local_get(pointer);
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

    block.local_get(pointer);
    block.i32_const(encoded_size as i32);
}

pub fn pack_u256_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
    local: LocalId,
) {
    let encoded_size = sol_data::Uint::<128>::ENCODED_SIZE.expect("U64 should have a fixed size");

    // Little-endian to Big-endian
    let swap_i64_bytes_function = add_swap_i64_bytes_function(module);

    let pointer = module.locals.add(ValType::I32);

    // Allocate memory for the packed value
    block.i32_const(encoded_size as i32);
    block.call(alloc_function);
    block.local_set(pointer);

    for i in 0..4 {
        block.local_get(pointer);
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

    block.local_get(pointer);
    block.i32_const(encoded_size as i32);
}

/// Address is packed as a u256, but endianness is not relevant
pub fn pack_address_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
    local: LocalId,
) {
    let encoded_size = sol_data::Uint::<128>::ENCODED_SIZE.expect("U64 should have a fixed size");

    let pointer = module.locals.add(ValType::I32);

    // Allocate memory for the packed value
    block.i32_const(encoded_size as i32);
    block.call(alloc_function);
    block.local_set(pointer);

    for i in 0..4 {
        block.local_get(pointer);
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

    block.local_get(pointer);
    block.i32_const(encoded_size as i32);
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, primitives::U256, sol};
    use walrus::{FunctionBuilder, FunctionId, MemoryId, ModuleConfig, ValType};
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

    fn test_uint_128(literal: u128, expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(16);
        func_body.call(alloc_function);
        func_body.local_set(local);

        // Args data should already be stored in memory
        pack_u128_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            alloc_function,
            local,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<i32>(
            &mut raw_module,
            "test_function",
            literal.to_le_bytes().to_vec(),
        );

        // the return is the pointer to the packed value
        let result = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    fn test_uint_256(literal: U256, expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(16);
        func_body.call(alloc_function);
        func_body.local_set(local);

        // Args data should already be stored in memory
        pack_u256_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            alloc_function,
            local,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<i32>(
            &mut raw_module,
            "test_function",
            literal.to_le_bytes::<32>().to_vec(),
        );

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

        let expected_result = SolType::abi_encode_params(&(128128128128,));
        test_uint_128(128128128128, &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint_128(IntType::MAX, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint_128(IntType::MIN, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint_128(IntType::MAX - 1, &expected_result); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_pack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));

        let expected_result = SolType::abi_encode_params(&(U256::from(256256256256u128),));
        test_uint_256(U256::from(256256256256u128), &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint_256(IntType::MAX, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint_256(IntType::MIN, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        test_uint_256(IntType::MAX - U256::from(1), &expected_result); // max -1 (avoid symmetry)
    }
}
