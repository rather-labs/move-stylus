use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{LoadKind, MemArg},
};

use crate::{
    translation::intermediate_types::{
        boolean::IBool,
        simple_integers::{IU8, IU16, IU32, IU64},
    },
    utils::{add_swap_i32_bytes_function, add_swap_i64_bytes_function},
};

use super::Unpackable;

impl Unpackable for IBool {
    fn add_unpack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        current_pointer: LocalId,
        memory: MemoryId,
        _allocator: FunctionId,
    ) {
        let encoded_size = sol_data::Bool::ENCODED_SIZE.expect("Bool should have a fixed size");
        unpack_i32_type_instructions(block, module, memory, current_pointer, encoded_size);
    }
}

impl Unpackable for IU8 {
    fn add_unpack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        current_pointer: LocalId,
        memory: MemoryId,
        _allocator: FunctionId,
    ) {
        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        unpack_i32_type_instructions(block, module, memory, current_pointer, encoded_size);
    }
}

impl Unpackable for IU16 {
    fn add_unpack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        current_pointer: LocalId,
        memory: MemoryId,
        _allocator: FunctionId,
    ) {
        let encoded_size =
            sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");
        unpack_i32_type_instructions(block, module, memory, current_pointer, encoded_size);
    }
}

impl Unpackable for IU32 {
    fn add_unpack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        current_pointer: LocalId,
        memory: MemoryId,
        _allocator: FunctionId,
    ) {
        let encoded_size =
            sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");
        unpack_i32_type_instructions(block, module, memory, current_pointer, encoded_size);
    }
}

impl Unpackable for IU64 {
    fn add_unpack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        current_pointer: LocalId,
        memory: MemoryId,
        _allocator: FunctionId,
    ) {
        let encoded_size =
            sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");
        unpack_i64_type_instructions(block, module, memory, current_pointer, encoded_size);
    }
}

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

    use alloy::{dyn_abi::SolType, sol};
    use walrus::{FunctionBuilder, FunctionId, ModuleConfig, ValType};
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
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Args data should already be stored in memory
        int_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            memory_id,
            allocator_func,
        );
        func_body.drop();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, mut store, entrypoint) =
            setup_wasmtime_module::<T>(&mut raw_module, data.to_vec(), "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_unpack_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IU8;

        let data = SolType::abi_encode_params(&(88,));
        test_uint(int_type, &data, 88, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type, &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type, &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(int_type, &data, (IntType::MAX - 1) as i32, ValType::I32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IU16;

        let data = SolType::abi_encode_params(&(1616,));
        test_uint(int_type, &data, 1616, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type, &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type, &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(int_type, &data, (IntType::MAX - 1) as i32, ValType::I32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u32() {
        let int_type = IU32;
        type IntType = u32;
        type SolType = sol!((uint32,));

        let data = SolType::abi_encode_params(&(323232,));
        test_uint(int_type, &data, 323232, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type, &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type, &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(int_type, &data, (IntType::MAX - 1) as i32, ValType::I32); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u64() {
        let int_type = IU64;
        type IntType = u64;
        type SolType = sol!((uint64,));

        let data = SolType::abi_encode_params(&(6464646464,));
        test_uint(int_type, &data, 6464646464i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type, &data, IntType::MAX as i64, ValType::I64); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type, &data, IntType::MIN as i64, ValType::I64); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(int_type, &data, (IntType::MAX - 1) as i64, ValType::I64); // max -1 (avoid symmetry)
    }
}
