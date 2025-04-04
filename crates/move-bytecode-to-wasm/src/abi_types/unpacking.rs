use alloy_sol_types::{SolType, sol_data};
use move_binary_format::file_format::{Signature, SignatureToken};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::utils::{add_swap_i32_bytes_function, add_swap_i64_bytes_function};

/// Builds the instructions to unpack the abi encoded values to WASM function parameters
///
/// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
/// and the pointer is pushed onto the stack in the parameter location.
pub fn build_unpack_instructions(
    function_builder: &mut InstrSeqBuilder,
    module: &mut Module,
    function_arguments_signature: &Signature,
    args_pointer: LocalId,
    args_len: LocalId,
    memory: MemoryId,
) {
    let length = module.locals.add(ValType::I32);
    function_builder.i32_const(0);
    function_builder.local_set(length);

    for signature_token in function_arguments_signature.0.iter() {
        add_unpack_instruction_for_signature_token(
            function_builder,
            signature_token,
            args_pointer,
            memory,
            length,
            module,
        );

        function_builder.local_get(length);
        function_builder.binop(BinaryOp::I32Add);
        function_builder.local_set(length);
    }

    // Validation block
    function_builder.block(None, |block| {
        let block_id = block.id();

        // Validate that current offset matches (args_pointer + args_len)
        // To ensure all bytes are consumed and no extra bytes are left
        block.local_get(args_len);
        block.local_get(length);
        block.binop(BinaryOp::I32Eq);
        block.br_if(block_id);
        block.unreachable(); // Throws an error
    });
}

/// Adds the instructions to unpack the abi type
///
/// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
/// and a pointer is pushed onto the stack in the parameter location.
///
/// Each block will leave the length of the unpacked value in the WASM stack.
fn add_unpack_instruction_for_signature_token(
    block: &mut InstrSeqBuilder,
    signature_token: &SignatureToken,
    args_pointer: LocalId,
    memory: MemoryId,
    prev_length: LocalId,
    module: &mut Module,
) {
    match signature_token {
        SignatureToken::Bool => {
            let encoded_size = sol_data::Bool::ENCODED_SIZE.expect("Bool should have a fixed size");

            // Load the value
            block.local_get(args_pointer);
            block.local_get(prev_length);
            block.binop(BinaryOp::I32Add);
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
        SignatureToken::U8 => {
            let encoded_size =
                sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");

            // Load the value
            block.local_get(args_pointer);
            block.local_get(prev_length);
            block.binop(BinaryOp::I32Add);
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
        SignatureToken::U16 => {
            let encoded_size =
                sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");

            // Load the value
            block.local_get(args_pointer);
            block.local_get(prev_length);
            block.binop(BinaryOp::I32Add);
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
        SignatureToken::U32 => {
            let encoded_size =
                sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");

            // Load the value
            block.local_get(args_pointer);
            block.local_get(prev_length);
            block.binop(BinaryOp::I32Add);
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
        SignatureToken::U64 => {
            let encoded_size =
                sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");

            // Load the value
            block.local_get(args_pointer);
            block.local_get(prev_length);
            block.binop(BinaryOp::I32Add);
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
        SignatureToken::U128 => {
            panic!("U128 is not supported yet"); // TODO
        }
        SignatureToken::U256 => {
            panic!("U256 is not supported yet"); // TODO
        }
        SignatureToken::Address => {
            panic!("Address is not supported yet"); // TODO
        }
        SignatureToken::Vector(_) => panic!("Vector is not supported"), // TODO
        SignatureToken::Signer => panic!("Signer is not supported"), // TODO: review how to handle this on public functions
        SignatureToken::Datatype(_) => panic!("Datatype is not supported yet"), // TODO
        SignatureToken::TypeParameter(_) => panic!("TypeParameter is not supported"), // TODO
        SignatureToken::DatatypeInstantiation(_) => {
            panic!("DatatypeInstantiation is not supported") // TODO
        }
        SignatureToken::Reference(_) => {
            panic!("Reference is not allowed as a public function argument")
        }
        SignatureToken::MutableReference(_) => {
            panic!("MutableReference is not allowed as a public function argument")
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, sol};
    use move_binary_format::file_format::Signature;
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
        let (mut raw_module, _, memory_id) = build_module();

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
            &Signature(vec![
                SignatureToken::Bool,
                SignatureToken::U16,
                SignatureToken::U64,
            ]),
            args_pointer,
            args_len,
            memory_id,
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
        let (mut raw_module, _, memory_id) = build_module();

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
            &Signature(vec![
                SignatureToken::U64,
                SignatureToken::U16,
                SignatureToken::Bool,
            ]),
            args_pointer,
            args_len,
            memory_id,
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
    #[should_panic(expected = "unreachable")]
    fn test_build_unpack_instructions_invalid_data_length() {
        let (mut raw_module, _, memory_id) = build_module();

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
            &Signature(vec![
                SignatureToken::U64,
                SignatureToken::U16,
                SignatureToken::Bool,
            ]),
            args_pointer,
            args_len,
            memory_id,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data = <sol!((uint64, uint16))>::abi_encode_params(&(123456789012345, 1234));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            data,
            "test_function",
            |_: u64, _: u32, _: u32| {},
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_offset_memory() {
        let (mut raw_module, _, memory_id) = build_module();

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
            &Signature(vec![
                SignatureToken::Bool,
                SignatureToken::U16,
                SignatureToken::U64,
            ]),
            args_pointer,
            args_len,
            memory_id,
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
