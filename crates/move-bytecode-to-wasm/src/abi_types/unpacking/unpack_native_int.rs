use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{LoadKind, MemArg},
};

use crate::utils::{add_swap_i32_bytes_function, add_swap_i64_bytes_function};

pub fn unpack_i32_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    current_pointer: LocalId,
    encoded_size: usize,
) {
    // Load the value
    block.local_get(current_pointer);
    block.load(
        memory,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 28,
        },
    );
    // Big-endian to Little-endian
    let swap_i32_bytes_function = add_swap_i32_bytes_function(module);
    block.call(swap_i32_bytes_function);

    block.i32_const(encoded_size as i32);
}

pub fn unpack_i64_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    current_pointer: LocalId,
    encoded_size: usize,
) {
    // Load the value
    block.local_get(current_pointer);
    block.load(
        memory,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 24,
        },
    );
    // Big-endian to Little-endian
    let swap_i64_bytes_function = add_swap_i64_bytes_function(module);
    block.call(swap_i64_bytes_function);

    block.i32_const(encoded_size as i32);
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use alloy::{dyn_abi::SolType, sol, sol_types::sol_data};
    use walrus::{FunctionBuilder, FunctionId, MemoryId, ModuleConfig, ValType};
    use wasmtime::{Engine, Linker, Module as WasmModule, Store, TypedFunc, WasmResults};

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
    ) -> (Linker<()>, Store<()>, TypedFunc<(), R>) {
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

        (linker, store, entrypoint)
    }

    fn test_uint<T: WasmResults + PartialEq + Debug>(
        encoded_size: usize,
        data: &[u8],
        expected_result: T,
    ) {
        let (mut raw_module, _, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Args data should already be stored in memory
        unpack_i32_type_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            args_pointer,
            encoded_size,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, mut store, entrypoint) =
            setup_wasmtime_module::<T>(&mut raw_module, data.to_vec(), "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    fn test_uint_64(encoded_size: usize, data: &[u8], expected_result: i64) {
        let (mut raw_module, _, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I64]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Args data should already be stored in memory
        unpack_i64_type_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            args_pointer,
            encoded_size,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, mut store, entrypoint) =
            setup_wasmtime_module::<i64>(&mut raw_module, data.to_vec(), "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_unpack_u8() {
        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        type IntType = u8;
        type SolType = sol!((uint8,));

        let data = SolType::abi_encode_params(&(88,));
        test_uint(encoded_size, &data, 88);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, &data, IntType::MAX as i32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, &data, IntType::MIN as i32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, &data, (IntType::MAX - 1) as i32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u16() {
        let encoded_size =
            sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");
        type IntType = u16;
        type SolType = sol!((uint16,));

        let data = SolType::abi_encode_params(&(1616,));
        test_uint(encoded_size, &data, 1616);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, &data, IntType::MAX as i32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, &data, IntType::MIN as i32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, &data, (IntType::MAX - 1) as i32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u32() {
        let encoded_size =
            sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");
        type IntType = u32;
        type SolType = sol!((uint32,));

        let data = SolType::abi_encode_params(&(323232,));
        test_uint(encoded_size, &data, 323232);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, &data, IntType::MAX as i32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, &data, IntType::MIN as i32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, &data, (IntType::MAX - 1) as i32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u64() {
        let encoded_size =
            sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");
        type IntType = u64;
        type SolType = sol!((uint64,));

        let data = SolType::abi_encode_params(&(6464646464,));
        test_uint_64(encoded_size, &data, 6464646464i64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint_64(encoded_size, &data, IntType::MAX as i64); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint_64(encoded_size, &data, IntType::MIN as i64); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint_64(encoded_size, &data, (IntType::MAX - 1) as i64); // max -1 (avoid symmetry)
    }
}
