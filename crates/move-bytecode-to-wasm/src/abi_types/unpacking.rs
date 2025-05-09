use walrus::{FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType};

use crate::translation::intermediate_types::{
    IntermediateType,
    address::IAddress,
    boolean::IBool,
    heap_integers::{IU128, IU256},
    simple_integers::{IU8, IU16, IU32, IU64},
    vector::IVector,
};

mod unpack_heap_int;
mod unpack_native_int;
mod unpack_vector;

pub trait Unpackable {
    /// Adds the instructions to unpack the abi encoded type to WASM function parameters
    ///
    /// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
    /// and the pointer is pushed onto the stack in the parameter location.
    ///
    /// The reader pointer should be updated internally when a value is read from the args
    /// The calldata reader pointer should never be updated, it is considered static for each type value
    ///
    /// The stack at the end contains the value(or pointer to the value) as **i32/i64**
    fn add_unpack_instructions(
        &self,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        memory: MemoryId,
        allocator: FunctionId,
    );
}

/// Builds the instructions to unpack the abi encoded values to WASM function parameters
///
/// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
/// and the pointer is pushed onto the stack in the parameter location.
pub fn build_unpack_instructions<T: Unpackable>(
    function_builder: &mut InstrSeqBuilder,
    module: &mut Module,
    function_arguments_signature: &[T],
    args_pointer: LocalId,
    memory: MemoryId,
    allocator: FunctionId,
) {
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    function_builder.local_get(args_pointer);
    function_builder.local_tee(reader_pointer);
    function_builder.local_set(calldata_reader_pointer);

    // The ABI encoded params are always a tuple
    // Static types are stored in-place, but dynamic types are referenced to the call data
    for signature_token in function_arguments_signature.iter() {
        signature_token.add_unpack_instructions(
            function_builder,
            module,
            reader_pointer,
            calldata_reader_pointer,
            memory,
            allocator,
        );
    }
}

impl Unpackable for IntermediateType {
    fn add_unpack_instructions(
        &self,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        match self {
            IntermediateType::IBool => IBool::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU8 => IU8::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU16 => IU16::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU32 => IU32::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU64 => IU64::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU128 => IU128::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IU256 => IU256::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IAddress => IAddress::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
            IntermediateType::IVector(inner) => IVector::add_unpack_instructions(
                inner,
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                memory,
                allocator,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, sol};
    use walrus::{FunctionBuilder, FunctionId, MemoryId, ModuleConfig, ValType};
    use wasmtime::{Engine, IntoFunc, Linker, Module as WasmModule, Store, TypedFunc, WasmParams};

    use crate::{memory::setup_module_memory, utils::display_module};

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module<A: WasmParams, V>(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        function_name: &str,
        validator_func: impl IntoFunc<(), V, ()>,
    ) -> (Linker<()>, Store<()>, TypedFunc<A, ()>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        linker.func_wrap("", "validator", validator_func).unwrap();

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<A, ()>(&mut store, function_name)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        memory.write(&mut store, 0, &initial_memory_data).unwrap();
        // Print current memory
        let memory_data = memory.data(&mut store);
        println!(
            "Current memory: {:?}",
            memory_data.iter().take(64).collect::<Vec<_>>()
        );

        (linker, store, entrypoint)
    }

    #[test]
    fn test_build_unpack_instructions() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            memory_id,
            allocator_func,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            data,
            "test_function",
            |param: u32, param2: u32, param3: u64| {
                println!("validator: {}, {}, {}", param, param2, param3);

                assert_eq!(param, 1);
                assert_eq!(param2, 1234);
                assert_eq!(param3, 123456789012345);
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_reversed() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I64, ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IU64,
                IntermediateType::IU16,
                IntermediateType::IBool,
            ],
            args_pointer,
            memory_id,
            allocator_func,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((uint64, uint16, bool))>::abi_encode_params(&(123456789012345, 1234, true));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            data,
            "test_function",
            |param: u64, param2: u32, param3: u32| {
                println!("validator: {}, {}, {}", param, param2, param3);

                assert_eq!(param, 123456789012345);
                assert_eq!(param2, 1234);
                assert_eq!(param3, 1);
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_offset_memory() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            memory_id,
            allocator_func,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        // Offset data by 10 bytes
        data = [vec![0; 10], data].concat();
        println!("data: {:?}", data);
        let data_len = data.len() as i32;
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            data,
            "test_function",
            |param: u32, param2: u32, param3: u64| {
                println!("validator: {}, {}, {}", param, param2, param3);

                assert_eq!(param, 1);
                assert_eq!(param2, 1234);
                assert_eq!(param3, 123456789012345);
            },
        );

        entrypoint.call(&mut store, (10, data_len - 10)).unwrap();
    }
}
