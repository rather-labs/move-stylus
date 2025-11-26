use walrus::{
    FunctionBuilder, FunctionId, GlobalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{
    CompilationContext,
    abi_types::{error_encoding::build_abort_error_message, public_function::PublicFunction},
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
    dynamic_fields_global_variables: &Vec<(GlobalId, IntermediateType)>,
) -> Result<(), HostIOError> {
    let (read_args_function, _) = host_functions::read_args(module);
    let (write_return_data_function, _) = host_functions::write_result(module);

    let args_len = module.locals.add(ValType::I32);
    let selector_variable = module.locals.add(ValType::I32);
    let args_pointer = module.locals.add(ValType::I32);

    let mut router = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let mut router_builder = router.func_body();

    // Find receive and fallback functions if they exist
    let receive_function = functions.iter().find(|f| f.function_name == "receive");
    let fallback_function = functions.iter().find(|f| f.function_name == "fallback");

    // Load function args to memory
    router_builder.local_get(args_len);
    router_builder.call(compilation_ctx.allocator);
    router_builder.local_tee(args_pointer);
    router_builder.call(read_args_function);

    // If args_len < 4, try receive/fallback, injecting the selector in the args data
    // If args_len >= 4: load selector from args (normal case)
    router_builder
        .local_get(args_len)
        .i32_const(4)
        .binop(BinaryOp::I32LtS)
        .if_else(
            None,
            |then| {
                // args_len < 4: check for receive or fallback
                // If args_len == 0: try receive(), if no receive function, unreachable (should be a no match error instead?)
                // Else (args_len > 0 but < 4): try fallback(), if no fallback function, unreachable

                // Determine which function to use and set selector: receive (args_len == 0) or fallback (args_len > 0 but < 4)
                then.local_get(args_len).unop(UnaryOp::I32Eqz).if_else(
                    None,
                    |receive_case| {
                        // args_len == 0: try receive
                        if let Some(receive_fn) = receive_function {
                            receive_case
                                .i32_const(i32::from_le_bytes(receive_fn.function_selector))
                                .local_set(selector_variable);
                        } else {
                            receive_case.unreachable();
                        }
                    },
                    |fallback_case| {
                        // args_len > 0 but < 4: try fallback
                        if let Some(fallback_fn) = fallback_function {
                            fallback_case
                                .i32_const(i32::from_le_bytes(fallback_fn.function_selector))
                                .local_set(selector_variable);
                        } else {
                            fallback_case.unreachable();
                        }
                    },
                );

                // Allocate buffer with selector prefix and update args_pointer/args_len
                // Layout: [selector (4 bytes)][original args (if any)]
                let args_pointer_ = module.locals.add(ValType::I32);
                then.local_get(args_len)
                    .i32_const(4)
                    .binop(BinaryOp::I32Add)
                    .call(compilation_ctx.allocator)
                    .local_set(args_pointer_);

                // Write the function selector to the first 4 bytes of the new buffer
                then.local_get(args_pointer_)
                    .local_get(selector_variable)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // If args_len > 0, copy original args into new buffer starting at offset 4
                then.local_get(args_len).unop(UnaryOp::I32Eqz).if_else(
                    None,
                    |_| {
                        // args_len == 0: nothing to copy
                    },
                    |else_| {
                        // args_len > 0: copy original args
                        else_
                            .local_get(args_pointer_)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_get(args_pointer)
                            .local_get(args_len)
                            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
                    },
                );

                // Update args_pointer to point to the new buffer
                then.local_get(args_pointer_).local_set(args_pointer);

                // Update args_len to args_len + 4
                then.local_get(args_len)
                    .i32_const(4)
                    .binop(BinaryOp::I32Add)
                    .local_set(args_len);
            },
            |else_| {
                // Load selector from memory
                else_
                    .local_get(args_pointer)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(selector_variable);
            },
        );

    // Try to route call based on selector for all public functions. Any successful match
    // will execute the target function and return from the router, so code below this loop
    // only runs if **no function matched**.
    for function in functions {
        function.build_router_block(
            &mut router_builder,
            module,
            selector_variable,
            args_pointer,
            args_len,
            write_return_data_function,
            compilation_ctx,
            dynamic_fields_global_variables,
        )?;
    }

    // If no function matched and we have calldata of length >= 4, we might be in the
    // "fallback with arguments" case:
    //
    // - User sends non-empty calldata that does not start with a 4-byte selector
    // - Our normal selector-based routing finds no match
    // - We still want to try the `fallback` entrypoint, which expects only ABI-encoded args.
    //
    // To reuse the existing `build_router_block` logic (which assumes layout
    // `selector || abi_encoded_args`), we synthesize such a layout in memory:
    //
    //   [ fallback selector (4 bytes) ][ original args_len bytes of calldata ]
    //
    // and then:
    //   - point `args_pointer` to the start of this new buffer
    //   - set `args_len` to args_len + 4
    //   - set `selector_variable` to the fallback's selector
    //
    // The fallback's router block will then:
    //   - see the forced selector and match
    //   - add 4 to `args_pointer` before decoding, so the Move function
    //     still receives the original calldata (without selector) as args.
    if let Some(fallback_fn) = fallback_function {
        // selector_variable = fallback selector
        router_builder
            .i32_const(i32::from_le_bytes(fallback_fn.function_selector))
            .local_set(selector_variable);

        // Allocate buffer with selector prefix and update args_pointer/args_len
        let args_pointer_ = module.locals.add(ValType::I32);

        // new_args_pointer = alloc(args_len + 4)
        router_builder
            .local_get(args_len)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .call(compilation_ctx.allocator)
            .local_tee(args_pointer_);

        // Write the function selector to the first 4 bytes of the new buffer
        router_builder
            .local_get(args_pointer_)
            .local_get(selector_variable)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // memcopy(dst = new_args_pointer + 4, src = args_pointer, len = args_len)
        router_builder
            .local_get(args_pointer_)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_get(args_pointer)
            .local_get(args_len)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

        // args_pointer = new_args_pointer
        router_builder
            .local_get(args_pointer_)
            .local_set(args_pointer);

        // args_len = args_len + 4
        router_builder
            .local_get(args_len)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_set(args_len);

        // Let the fallback router block try to handle this call. If it matches, it will
        // execute and return; otherwise execution will continue to the error path below.
        fallback_fn.build_router_block(
            &mut router_builder,
            module,
            selector_variable,
            args_pointer,
            args_len,
            write_return_data_function,
            compilation_ctx,
            dynamic_fields_global_variables,
        )?;
    }

    // Build no function match error message
    router_builder.i64_const(ERROR_NO_FUNCTION_MATCH);
    let ptr = build_abort_error_message(&mut router_builder, module, compilation_ctx)?;

    // Write error data to memory
    router_builder
        // Skip header
        .local_get(ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        // Load msg length
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        // Write
        .call(write_return_data_function);

    // Push the error code and return
    router_builder.i32_const(1).return_();

    let router = router.finish(vec![args_len], &mut module.funcs);
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
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        let noop_selector_data = noop.get_selector().to_vec();
        let noop_2_selector_data = noop_2.get_selector().to_vec();

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx, &vec![])
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
    #[should_panic(expected = "unreachable")]
    fn test_build_entrypoint_router_no_data() {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx, &vec![])
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
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let signature = ISignature {
            arguments: vec![],
            returns: vec![],
        };
        let noop = add_noop_function(&mut raw_module, &signature, &compilation_ctx);
        let noop_2 = add_noop_2_function(&mut raw_module, &signature, &compilation_ctx);

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx, &vec![])
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
