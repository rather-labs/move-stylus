use alloy_sol_types::{SolType, sol_data};
use move_binary_format::file_format::{Signature, SignatureToken};
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

use crate::utils::{add_swap_i32_bytes_function, add_swap_i64_bytes_function};

/// Builds the instructions to pack the abi encoded values to WASM function return values
///
/// Each return value is encoded and loaded in memory. Complex data types are copied to
/// have a contiguous memory layout.
///
/// Variables should have been loaded in the WASM stack before calling this function.
pub fn build_pack_instructions(
    builder: &mut InstrSeqBuilder,
    function_return_signature: &Signature,
    module: &mut Module,
    memory: MemoryId,
    alloc_function: FunctionId,
) {
    if function_return_signature.0.is_empty() {
        builder.i32_const(0);
        builder.i32_const(0);
        return;
    }

    // We need to load all return types into locals in order to reverse the read order
    // Otherwise they would be popped in reverse order
    let mut locals = Vec::new();
    for signature_token in function_return_signature.0.iter().rev() {
        let local = load_return_type_to_local(module, builder, signature_token);
        locals.push(local);
    }
    locals.reverse();

    let pointer = module.locals.add(ValType::I32);
    let length = module.locals.add(ValType::I32);

    let mut first_token = false;
    for (local, signature_token) in locals.iter().zip(function_return_signature.0.iter()) {
        add_pack_instruction_for_signature_token(
            builder,
            module,
            *local,
            signature_token,
            memory,
            alloc_function,
        );

        // If the first one, we store the pointer and initialize the length
        if !first_token {
            first_token = true;
            builder.local_set(length);
            builder.local_set(pointer);
        } else {
            // increment the length
            builder.local_get(length);
            builder.binop(BinaryOp::I32Add);
            builder.local_set(length);
            // drop the pointer
            builder.drop();
        }
    }

    builder.local_get(pointer);
    builder.local_get(length);
}

/// Adds the instructions to pack the abi type and load it in memory
///
/// Each block will leave the memory pointer and the length of the packed value in the WASM stack.
/// [pointer, length]
fn add_pack_instruction_for_signature_token(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    local: LocalId,
    signature_token: &SignatureToken,
    memory: MemoryId,
    alloc_function: FunctionId,
) {
    match signature_token {
        SignatureToken::Bool => {
            let encoded_size = sol_data::Bool::ENCODED_SIZE.expect("Bool should have a fixed size");

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
        SignatureToken::U8 => {
            let encoded_size =
                sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");

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
        SignatureToken::U16 => {
            let encoded_size =
                sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");

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
        SignatureToken::U32 => {
            let encoded_size =
                sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");

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
        SignatureToken::U64 => {
            let encoded_size =
                sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");

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

fn load_return_type_to_local(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    signature_token: &SignatureToken,
) -> LocalId {
    match signature_token {
        SignatureToken::Bool | SignatureToken::U8 | SignatureToken::U16 | SignatureToken::U32 => {
            let local = module.locals.add(ValType::I32);
            builder.local_set(local);
            local
        }
        SignatureToken::U64 => {
            let local = module.locals.add(ValType::I64);
            builder.local_set(local);
            local
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
    use wasmtime::{
        Caller, Engine, Extern, IntoFunc, Linker, Module as WasmModule, Store, TypedFunc,
        WasmParams,
    };

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

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.i32_const(1);
        func_body.i32_const(1234);
        func_body.i64_const(123456789012345);

        build_pack_instructions(
            &mut func_body,
            &Signature(vec![
                SignatureToken::Bool,
                SignatureToken::U16,
                SignatureToken::U64,
            ]),
            &mut raw_module,
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
            vec![],
            "test_function",
            move |mut caller: Caller<()>, pointer: u32, length: u32| {
                println!("validator: {}, {}", pointer, length);

                assert_eq!(pointer, 0);
                assert_eq!(length, data_len as u32);

                let memory = caller.get_export("memory").unwrap();
                let memory = match memory {
                    Extern::Memory(memory) => memory,
                    _ => panic!("memory not found"),
                };

                let mut buffer = vec![0; length as usize];
                memory
                    .read(&mut caller, pointer as usize, &mut buffer)
                    .unwrap();
                assert_eq!(buffer, data);
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_memory_offset() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate some memory just to increase the offset
        func_body.i32_const(100);
        func_body.call(allocator_func);
        func_body.drop();

        // Load arguments to stack
        func_body.i32_const(1);
        func_body.i32_const(1234);
        func_body.i64_const(123456789012345);

        build_pack_instructions(
            &mut func_body,
            &Signature(vec![
                SignatureToken::Bool,
                SignatureToken::U16,
                SignatureToken::U64,
            ]),
            &mut raw_module,
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
            vec![],
            "test_function",
            move |mut caller: Caller<()>, pointer: u32, length: u32| {
                println!("validator: {}, {}", pointer, length);
                assert_eq!(pointer, 100);
                assert_eq!(length, data_len as u32);

                let memory = caller.get_export("memory").unwrap();
                let memory = match memory {
                    Extern::Memory(memory) => memory,
                    _ => panic!("memory not found"),
                };

                let mut buffer = vec![0; length as usize];
                memory
                    .read(&mut caller, pointer as usize, &mut buffer)
                    .unwrap();
                assert_eq!(buffer, data);
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }
}
