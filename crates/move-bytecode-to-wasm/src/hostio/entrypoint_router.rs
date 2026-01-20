use walrus::{
    ConstExpr, FunctionBuilder, FunctionId, GlobalId, GlobalKind, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp, Value},
};

use crate::{
    CompilationContext,
    abi_types::{error_encoding::build_abort_error_message, public_function::PublicFunction},
    data::{DATA_ABORT_MESSAGE_PTR_OFFSET, DATA_CALLDATA_OFFSET, RuntimeErrorData},
    memory::MEMORY_PAGE_SIZE,
    runtime::RuntimeFunction,
    translation::intermediate_types::IntermediateType,
};

use super::{error::HostIOError, host_functions};

const ERROR_NO_FUNCTION_MATCH: i64 = 100;

/// Builds an entrypoint router for the list of public functions provided
/// and adds it to the module exporting it as `user_entrypoint`
///
/// Status is 0 for success and non-zero for failure.
pub fn build_entrypoint_router(
    module: &mut Module,
    functions: &[PublicFunction],
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    dynamic_fields_global_variables: &Vec<(GlobalId, IntermediateType)>,
) -> Result<(), HostIOError> {
    let (read_args_function, _) = host_functions::read_args(module);
    let (write_return_data_function, _) = host_functions::write_result(module);

    let mut router = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut router_builder = router.func_body();

    // Arguments
    let data_len = module.locals.add(ValType::I32); // Length of the calldata

    // Locals
    let data_pointer = module.locals.add(ValType::I32); // Pointer to the calldata
    let function_selector = module.locals.add(ValType::I32); // Selector of the function to call
    let status = module.locals.add(ValType::I32); // Status of the function call

    // Read the calldata from the host and store it in the memory
    router_builder
        .local_get(data_len)
        .call(compilation_ctx.allocator)
        .local_tee(data_pointer)
        .call(read_args_function);

    // Set status to 0 (success) as default
    router_builder.i32_const(0).local_set(status);

    // Save the calldata length and pointer into the data section
    router_builder
        .i32_const(DATA_CALLDATA_OFFSET)
        .local_get(data_len)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    router_builder
        .i32_const(DATA_CALLDATA_OFFSET)
        .local_get(data_pointer)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        );

    // Define the return block - all routing logic happens inside this block
    let mut inner_result = Ok(());
    router_builder.block(None, |return_block| {
        inner_result = (|| {
            let return_block_id = return_block.id();

            // If data_len == 0, there isn't even a function selector present in the calldata, so we attempt to default to "receive" by injecting its selector into the argument data.
            // Otherwise (data_len != 0), read selector directly from the args (the standard case).
            return_block
                .local_get(data_len)
                .unop(UnaryOp::I32Eqz)
                .if_else(
                    None,
                    |then| {
                        // data_len == 0: try receive
                        // If no receive function, the selector_variable remains uninitialized and will not match any function
                        if let Some(receive_fn) = functions
                            .iter()
                            .find(|f| f.function_name.as_str() == "receive")
                        {
                            then.i32_const(i32::from_le_bytes(receive_fn.function_selector))
                                .local_set(function_selector);
                        }

                        // Set data_len to 4
                        then.i32_const(4).local_set(data_len);

                        // Allocate memory for the receive function selector, which is the only calldata needed.
                        then.local_get(data_len)
                            .call(compilation_ctx.allocator)
                            .local_tee(data_pointer)
                            .local_get(function_selector)
                            .store(
                                compilation_ctx.memory_id,
                                StoreKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            );
                    },
                    |else_| {
                        // Load selector from memory
                        else_
                            .local_get(data_pointer)
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(function_selector);
                    },
                );

            // Offset args pointer by 4 bytes to exclude selector
            return_block
                .local_get(data_pointer)
                .i32_const(4)
                .binop(BinaryOp::I32Add)
                .local_set(data_pointer);

            // Try to route call based on selector for all public functions. Any successful match
            // will set data_pointer, data_len and break to exit_label.
            for function in functions {
                inner_result = function.build_router_block(
                    return_block,
                    module,
                    function_selector,
                    data_pointer,
                    data_len,
                    return_block_id,
                    compilation_ctx,
                    runtime_error_data,
                );
            }
            // If no function matched we might be in the fallback case:
            if let Some(fallback_fn) = functions
                .iter()
                .find(|f| f.function_name.as_str() == "fallback")
            {
                // Restore the data pointer to the start of the calldata, as the fallback function has no selector
                return_block
                    .local_get(data_pointer)
                    .i32_const(4)
                    .binop(BinaryOp::I32Sub)
                    .local_set(data_pointer);

                // Wrap function to unpack/pack arguments
                inner_result = fallback_fn.wrap_public_function(
                    module,
                    return_block,
                    return_block_id,
                    data_pointer,
                    compilation_ctx,
                    runtime_error_data,
                );

                // Stack: [data_pointer] [data_len]
                // Set final locals and break to exit
                return_block
                    .local_set(data_len)
                    .local_set(data_pointer)
                    .br(return_block_id);
            }

            // --- NO MATCH CASE ---
            // If execution reaches here, no function matched.
            // We set up the error message and fall through to the common return.
            return_block.i64_const(ERROR_NO_FUNCTION_MATCH);
            let ptr = build_abort_error_message(return_block, module, compilation_ctx)?;

            // Set data_pointer to skip header (ptr + 4)
            return_block
                .local_get(ptr)
                .i32_const(4)
                .binop(BinaryOp::I32Add)
                .local_set(data_pointer);

            // Set data_len from the error message length
            return_block
                .local_get(ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(data_len);

            // Set status to 1 (error)
            return_block.i32_const(1).local_set(status);
            // Fall through to end of block...
            Ok(())
        })();
    });

    inner_result?;
    // --- SHARED EXIT LOGIC ---

    // 1. Check for Global Abort
    router_builder.block(None, |abort_block| {
        // Load the abort message pointer from DATA_ABORT_MESSAGE_PTR_OFFSET
        // If not null, an abort occurred and we need to return the error message

        let abort_block_id = abort_block.id();

        // Load the ptr
        let ptr = module.locals.add(ValType::I32);
        abort_block
            .i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_tee(ptr);

        // Check if the ptr is null
        abort_block.unop(UnaryOp::I32Eqz);

        // If the ptr is null, jump to the end of the block, skipping the error message loading
        abort_block.br_if(abort_block_id);

        // Load the abort message length from the ptr and set return data length
        abort_block
            .local_get(ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(data_len);

        // Load the abort message pointer and set data_ptr
        abort_block
            .local_get(ptr)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_set(data_pointer);

        // Set status to 1 (error)
        abort_block.i32_const(1).local_set(status);
    });

    // 2. Write return data
    router_builder
        .local_get(data_pointer)
        .local_get(data_len)
        .call(write_return_data_function);

    // 3. Conditionally commit changes to storage (iff status == 0)
    let commit_changes_to_storage_function = RuntimeFunction::get_commit_changes_to_storage_fn(
        module,
        compilation_ctx,
        runtime_error_data,
        dynamic_fields_global_variables,
    )?;

    router_builder
        .local_get(status)
        .unop(UnaryOp::I32Eqz)
        .if_else(
            None,
            |then| {
                then.call(commit_changes_to_storage_function);
            },
            |_| {},
        );

    // Update the next_free_memory_pointer global to the current next_offset from runtime_error_data
    let next_free_memory_pointer = module
        .globals
        .get_mut(compilation_ctx.globals.next_free_memory_pointer);

    next_free_memory_pointer.kind = GlobalKind::Local(ConstExpr::Value(Value::I32(
        runtime_error_data.get_next_offset(),
    )));

    // Update the available_memory global to account for the memory used by error data
    let available_memory = module
        .globals
        .get_mut(compilation_ctx.globals.available_memory);

    available_memory.kind = GlobalKind::Local(ConstExpr::Value(Value::I32(
        MEMORY_PAGE_SIZE - runtime_error_data.get_next_offset(),
    )));

    // 4. Return
    router_builder.local_get(status).return_();

    let router = router.finish(vec![data_len], &mut module.funcs);
    add_entrypoint(module, router);

    Ok(())
}

/// Add an entrypoint to the module with the interface defined by Stylus
pub fn add_entrypoint(module: &mut Module, func: FunctionId) {
    module.exports.add("user_entrypoint", func);
}

#[cfg(test)]
mod tests {
    use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store, TypedFunc};

    use crate::{
        test_compilation_context, test_tools::build_module,
        translation::intermediate_types::ISignature, utils::display_module,
    };

    use super::*;

    fn add_noop_function<'a>(
        module: &mut Module,
        signature: &'a ISignature,
        compilation_ctx: &CompilationContext,
    ) -> PublicFunction<'a> {
        // Noop function
        let mut noop_builder = FunctionBuilder::new(&mut module.types, &[], &[]);
        noop_builder.func_body();

        let noop = noop_builder.finish(vec![], &mut module.funcs);

        PublicFunction::new(noop, "noop", signature, compilation_ctx).unwrap()
    }

    fn add_noop_2_function<'a>(
        module: &mut Module,
        signature: &'a ISignature,
        compilation_ctx: &CompilationContext,
    ) -> PublicFunction<'a> {
        // Noop function
        let mut noop_builder = FunctionBuilder::new(&mut module.types, &[], &[]);
        noop_builder.func_body();

        let noop = noop_builder.finish(vec![], &mut module.funcs);

        PublicFunction::new(noop, "noop_2", signature, compilation_ctx).unwrap()
    }

    struct ReadArgsData {
        data: Vec<u8>,
    }

    fn setup_wasmtime_module(
        module: &mut Module,
        data: ReadArgsData,
    ) -> (
        Linker<ReadArgsData>,
        Store<ReadArgsData>,
        TypedFunc<i32, i32>,
    ) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();
        let get_memory = move |caller: &mut Caller<'_, ReadArgsData>| match caller
            .get_module_export(&mem_export)
        {
            Some(Extern::Memory(mem)) => mem,
            _ => panic!("failed to find host memory"),
        };

        linker
            .func_wrap(
                "vm_hooks",
                "native_keccak256",
                |mut caller: wasmtime::Caller<'_, ReadArgsData>,
                 input_data_ptr: u32,
                 data_length: u32,
                 return_data_ptr: u32| {
                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut input_data = vec![0; data_length as usize];
                    memory
                        .read(&caller, input_data_ptr as usize, &mut input_data)
                        .unwrap();

                    let hash = alloy_primitives::keccak256(input_data);

                    memory
                        .write(&mut caller, return_data_ptr as usize, hash.as_slice())
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "read_args",
                move |mut caller: Caller<'_, ReadArgsData>, args_ptr: u32| {
                    println!("read_args");

                    let mem = get_memory(&mut caller);

                    let args_data = caller.data().data.clone();
                    println!("args_data: {args_data:?}");

                    mem.write(&mut caller, args_ptr as usize, &args_data)
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "write_result",
                |_return_data_pointer: u32, _return_data_length: u32| {},
            )
            .unwrap();

        linker
            .func_wrap("vm_hooks", "storage_flush_cache", |_: i32| {})
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "tx_origin",
                move |mut caller: Caller<'_, ReadArgsData>, ptr: u32| {
                    println!("tx_origin, writing in {ptr}");

                    let mem = get_memory(&mut caller);

                    // Write 0x7357 address
                    let test_address =
                        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7];

                    mem.write(&mut caller, ptr as usize, test_address).unwrap();
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "emit_log",
                move |mut caller: Caller<'_, ReadArgsData>, ptr: u32, len: u32, _topic: u32| {
                    println!("emit_log, reading from {ptr}, length: {len}");

                    let mem = get_memory(&mut caller);
                    let mut buffer = vec![0; len as usize];

                    mem.read(&mut caller, ptr as usize, &mut buffer).unwrap();

                    println!("read memory: {buffer:?}");
                },
            )
            .unwrap();

        let mut store = Store::new(&engine, data);
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<i32, i32>(&mut store, "user_entrypoint")
            .unwrap();

        (linker, store, entrypoint)
    }

    #[test]
    fn test_build_entrypoint_router_noop() {
        let (mut raw_module, allocator_func, memory_id, compilation_context_globals) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, compilation_context_globals);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        let noop_selector_data = noop.get_selector().to_vec();
        let noop_2_selector_data = noop_2.get_selector().to_vec();

        let mut runtime_error_data = RuntimeErrorData::new();
        build_entrypoint_router(
            &mut raw_module,
            &[noop, noop_2],
            &compilation_ctx,
            &mut runtime_error_data,
            &vec![],
        )
        .unwrap();
        display_module(&mut raw_module);

        let data = ReadArgsData {
            data: noop_selector_data,
        };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 0);

        let data = ReadArgsData {
            data: noop_2_selector_data,
        };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_build_entrypoint_router_no_data() {
        let (mut raw_module, allocator_func, memory_id, compilation_context_globals) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, compilation_context_globals);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        let mut runtime_error_data = RuntimeErrorData::new();
        build_entrypoint_router(
            &mut raw_module,
            &[noop, noop_2],
            &compilation_ctx,
            &mut runtime_error_data,
            &vec![],
        )
        .unwrap();
        display_module(&mut raw_module);

        // Invalid selector
        let data = ReadArgsData { data: vec![] };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        entrypoint.call(&mut store, data_len).unwrap();
    }

    #[test]
    fn test_build_entrypoint_router_no_match() {
        let (mut raw_module, allocator_func, memory_id, compilation_context_globals) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, compilation_context_globals);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        let mut runtime_error_data = RuntimeErrorData::new();
        build_entrypoint_router(
            &mut raw_module,
            &[noop, noop_2],
            &compilation_ctx,
            &mut runtime_error_data,
            &vec![],
        )
        .unwrap();
        display_module(&mut raw_module);

        // Invalid selector
        let data = ReadArgsData { data: vec![0; 4] };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 1);
    }
}
