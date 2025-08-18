use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext, abi_types::public_function::PublicFunction,
    runtime_error_codes::ERROR_NO_FUNCTION_MATCH,
};

use super::host_functions;

/// Builds an entrypoint router for the list of public functions provided
/// and adds it to the module exporting it as `user_entrypoint`
///
/// Status is 0 for success and non-zero for failure.
pub fn build_entrypoint_router(
    module: &mut Module,
    functions: &[PublicFunction],
    compilation_ctx: &CompilationContext,
) {
    let (read_args_function, _) = host_functions::read_args(module);
    let (write_return_data_function, _) = host_functions::write_result(module);
    let (storage_flush_cache_function, _) = host_functions::storage_flush_cache(module);

    let args_len = module.locals.add(ValType::I32);
    let selector_variable = module.locals.add(ValType::I32);
    let args_pointer = module.locals.add(ValType::I32);

    let mut router = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let mut router_builder = router.func_body();

    // TODO: handle case where no args data, now we just panic
    router_builder.block(None, |block| {
        let block_id = block.id();

        // If args len is < 4 there is no selector
        block.local_get(args_len);
        block.i32_const(4);
        block.binop(BinaryOp::I32GeS);
        block.br_if(block_id);
        block.unreachable();
    });

    // Load function args to memory
    router_builder.local_get(args_len);
    router_builder.call(compilation_ctx.allocator);
    router_builder.local_tee(args_pointer);
    router_builder.call(read_args_function);

    // Load selector from first 4 bytes of args
    router_builder.local_get(args_pointer);
    router_builder.load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );
    router_builder.local_set(selector_variable);

    for function in functions {
        function.build_router_block(
            &mut router_builder,
            module,
            selector_variable,
            args_pointer,
            args_len,
            write_return_data_function,
            storage_flush_cache_function,
            compilation_ctx,
        );
    }

    // When no match is found, return error code
    // TODO: allow fallback function definition
    router_builder.i32_const(ERROR_NO_FUNCTION_MATCH);
    router_builder.return_();

    let router = router.finish(vec![args_len], &mut module.funcs);
    add_entrypoint(module, router);
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

        PublicFunction::new(noop, "noop", signature, compilation_ctx)
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

        PublicFunction::new(noop, "noop_2", signature, compilation_ctx)
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
                "read_args",
                move |mut caller: Caller<'_, ReadArgsData>, args_ptr: u32| {
                    println!("read_args");

                    let mem = get_memory(&mut caller);

                    let args_data = caller.data().data.clone();
                    println!("args_data: {:?}", args_data);

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

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx);
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

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx);
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

        build_entrypoint_router(&mut raw_module, &[noop, noop_2], &compilation_ctx);
        display_module(&mut raw_module);

        // Invalid selector
        let data = ReadArgsData { data: vec![0; 4] };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, ERROR_NO_FUNCTION_MATCH);
    }
}
