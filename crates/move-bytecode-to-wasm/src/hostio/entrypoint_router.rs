use walrus::{
    FunctionBuilder, FunctionId, GlobalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{
    CompilationContext,
    abi_types::{error_encoding::build_abort_error_message, public_function::PublicFunction},
    data::DATA_CALLDATA_OFFSET,
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
    let receive_function = functions
        .iter()
        .find(|f| f.function_name.as_str() == "receive");
    let fallback_function = functions
        .iter()
        .find(|f| f.function_name.as_str() == "fallback");

    // Load function args to memory
    router_builder
        .local_get(args_len)
        .call(compilation_ctx.allocator)
        .local_tee(args_pointer)
        .call(read_args_function);

    // Store the calldata length and pointer to the data segment
    router_builder
        .i32_const(DATA_CALLDATA_OFFSET)
        .local_get(args_len)
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
        .local_get(args_pointer)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        );

    // If args_len == 0, try receive, injecting the selector in the args data
    // If args_len != 0: load selector from args (normal case)
    router_builder
        .local_get(args_len)
        .unop(UnaryOp::I32Eqz)
        .if_else(
            None,
            |then| {
                // args_len == 0: try receive
                // If no receive function, the selector_variable remains uninitialized and will not match any function
                if let Some(receive_fn) = receive_function {
                    then.i32_const(i32::from_le_bytes(receive_fn.function_selector))
                        .local_set(selector_variable);
                }

                // Update args_len to 4
                then.i32_const(4).local_set(args_len);

                // Store the new args_len to memory
                then.i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_get(args_len)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // Allocate buffer with selector prefix
                then.local_get(args_len)
                    .call(compilation_ctx.allocator)
                    .local_tee(args_pointer)
                    .local_get(selector_variable)
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
    // will execute the target function and return from the router.
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

    // If no function matched we might be in the fallback case:
    if let Some(fallback_fn) = fallback_function {
        let commit_changes_to_storage_function = RuntimeFunction::get_commit_changes_to_storage_fn(
            module,
            compilation_ctx,
            dynamic_fields_global_variables,
        )?;

        // Wrap function to pack/unpack parameters
        fallback_fn.wrap_public_function(
            module,
            &mut router_builder,
            args_pointer,
            compilation_ctx,
        )?;

        // Stack: [return_data_pointer] [return_data_length] [status]
        let status = module.locals.add(ValType::I32);
        router_builder.local_set(status);

        // Write return data to memory
        router_builder.call(write_return_data_function);

        router_builder.call(commit_changes_to_storage_function);

        // Return status
        router_builder.local_get(status);
        router_builder.return_();
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

    use walrus::{ConstExpr, ir::Value};

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
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);
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
    // #[should_panic(expected = "unreachable")]
    fn test_build_entrypoint_router_no_data() {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);
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
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);
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
