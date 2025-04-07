use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::utils::add_swap_i64_bytes_function;

pub fn unpack_u128_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    current_pointer: LocalId,
    allocator: FunctionId,
) {
    let encoded_size = sol_data::Uint::<128>::ENCODED_SIZE.expect("U128 should have a fixed size");

    // Big-endian to Little-endian
    let swap_i64_bytes_function = add_swap_i64_bytes_function(module);

    block.i32_const(encoded_size as i32);
    block.call(allocator);

    let unpacked_pointer = module.locals.add(ValType::I32);
    block.local_tee(unpacked_pointer);

    block.local_get(current_pointer);
    block.load(
        memory,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            // Loading first 8 bytes
            offset: 16,
        },
    );
    block.call(swap_i64_bytes_function);

    block.local_get(unpacked_pointer);
    block.local_get(current_pointer);
    block.load(
        memory,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            // Loading last 8 bytes
            offset: 24,
        },
    );
    block.call(swap_i64_bytes_function);

    block.store(
        memory,
        StoreKind::I64 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    block.store(
        memory,
        StoreKind::I64 { atomic: false },
        MemArg {
            align: 0,
            offset: 8,
        },
    );

    block.local_get(unpacked_pointer);
    block.i32_const(encoded_size as i32);
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, sol};
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

    fn test_uint_128(data: &[u8], expected_result: u128) {
        let (mut raw_module, allocator, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Mock args allocation
        func_body.i32_const(data.len() as i32);
        func_body.call(allocator);
        func_body.drop();

        // Args data should already be stored in memory
        unpack_u128_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            args_pointer,
            allocator,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, data.to_vec(), "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);

        let expected_result_bytes = expected_result.to_le_bytes();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_unpack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));

        let data = SolType::abi_encode_params(&(88,));
        test_uint_128(&data, 88);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint_128(&data, IntType::MAX); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint_128(&data, IntType::MIN); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint_128(&data, IntType::MAX - 1); // max -1 (avoid symmetry)
    }
}
