use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::utils::{add_swap_i32_bytes_function, add_swap_i64_bytes_function};

pub fn pack_i32_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
    local: LocalId,
    encoded_size: usize,
) {
    let pointer = module.locals.add(ValType::I32);

    // Allocate memory for the packed value
    block.i32_const(encoded_size as i32);
    block.call(alloc_function);
    block.local_tee(pointer);

    // Load the local value to the stack
    block.local_get(local);

    // Little-endian to Big-endian
    let swap_i32_bytes_function = add_swap_i32_bytes_function(module);
    block.call(swap_i32_bytes_function);

    block.store(
        memory,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 28,
        },
    );

    block.local_get(pointer);
    block.i32_const(encoded_size as i32);
}

pub fn pack_i64_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
    local: LocalId,
    encoded_size: usize,
) {
    let pointer = module.locals.add(ValType::I32);

    // Allocate memory for the packed value
    block.i32_const(encoded_size as i32);
    block.call(alloc_function);
    block.local_tee(pointer);

    // Load the local value to the stack
    block.local_get(local);

    // Little-endian to Big-endian
    let swap_i64_bytes_function = add_swap_i64_bytes_function(module);
    block.call(swap_i64_bytes_function);

    block.store(
        memory,
        StoreKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 24,
        },
    );

    block.local_get(pointer);
    block.i32_const(encoded_size as i32);
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, sol, sol_types::sol_data};
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
    ) -> (Linker<()>, Instance, Store<()>, TypedFunc<(), R>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let linker = Linker::new(&engine);

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<(), R>(&mut store, function_name)
            .unwrap();

        (linker, instance, store, entrypoint)
    }

    fn test_uint(encoded_size: usize, literal: i32, expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(literal);
        func_body.local_set(local);

        // Args data should already be stored in memory
        pack_i32_type_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            alloc_function,
            local,
            encoded_size,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, "test_function");

        // the return is the pointer to the packed value
        let result = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    fn test_uint_64(encoded_size: usize, literal: i64, expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let local = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();
        func_body.i64_const(literal);
        func_body.local_set(local);

        // Args data should already be stored in memory
        pack_i64_type_instructions(
            &mut func_body,
            &mut raw_module,
            memory_id,
            alloc_function,
            local,
            encoded_size,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_u8() {
        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        type IntType = u8;
        type SolType = sol!((uint8,));

        let expected_result = SolType::abi_encode_params(&(88,));
        test_uint(encoded_size, 88, &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, IntType::MAX as i32, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, IntType::MIN as i32, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, (IntType::MAX - 1) as i32, &expected_result); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u16() {
        let encoded_size =
            sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");
        type IntType = u16;
        type SolType = sol!((uint16,));

        let expected_result = SolType::abi_encode_params(&(1616,));
        test_uint(encoded_size, 1616, &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, IntType::MAX as i32, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, IntType::MIN as i32, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, (IntType::MAX - 1) as i32, &expected_result); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u32() {
        let encoded_size =
            sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");
        type IntType = u32;
        type SolType = sol!((uint32,));

        let expected_result = SolType::abi_encode_params(&(323232,));
        test_uint(encoded_size, 323232, &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(encoded_size, IntType::MAX as i32, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(encoded_size, IntType::MIN as i32, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(encoded_size, (IntType::MAX - 1) as i32, &expected_result); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u64() {
        let encoded_size =
            sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");
        type IntType = u64;
        type SolType = sol!((uint64,));

        let expected_result = SolType::abi_encode_params(&(6464646464,));
        test_uint_64(encoded_size, 6464646464i64, &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint_64(encoded_size, IntType::MAX as i64, &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint_64(encoded_size, IntType::MIN as i64, &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint_64(encoded_size, (IntType::MAX - 1) as i64, &expected_result); // max -1 (avoid symmetry)
    }
}
