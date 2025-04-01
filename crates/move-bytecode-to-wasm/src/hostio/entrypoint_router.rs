use walrus::{
    FunctionBuilder, FunctionId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::abi_types::function_encoding::AbiFunctionSelector;

use super::host_functions;

/// Builds an entrypoint router for the list of functions provided
/// and adds it to the module exporting it as `user_entrypoint`
///
/// Only Move public functions should be included here and they all should have been normalized as f(i32 pointer, i32 length) -> (i32 pointer, i32 length, i32 status)
/// They receive a pointer to the arguments from memory, and the length of the arguments
/// Returns a pointer to the return data, the length of the return data and a status
/// Status is 0 for success and non-zero for failure.
pub fn build_entrypoint_router(
    module: &mut Module,
    allocator_func: FunctionId,
    memory_id: MemoryId,
    functions: &[(FunctionId, AbiFunctionSelector)],
) {
    let (read_args_function, _) = host_functions::read_args(module);
    let (write_return_data_function, _) = host_functions::write_result(module);

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

    // // Load function args to memory
    router_builder.local_get(args_len);
    router_builder.call(allocator_func);
    router_builder.local_tee(args_pointer);
    router_builder.call(read_args_function);

    // Load selector from first 4 bytes of args
    router_builder.local_get(args_pointer);
    router_builder.load(
        memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );
    router_builder.local_set(selector_variable);

    for (func_id, selector) in functions {
        router_builder.block(None, |block| {
            let block_id = block.id();

            block.local_get(selector_variable);
            block.i32_const(i32::from_le_bytes(*selector));
            block.binop(BinaryOp::I32Ne);
            block.br_if(block_id);

            // Call the function
            block.local_get(args_pointer);
            block.i32_const(4); // to offset for selector
            block.binop(BinaryOp::I32Add);
            block.local_get(args_len);
            block.i32_const(4); // reduce to exclude selector
            block.binop(BinaryOp::I32Sub);
            block.call(*func_id);

            // Stack: [return_data_pointer] [return_data_length] [status]
            let status = module.locals.add(ValType::I32);
            block.local_set(status);

            // Write return data to memory
            // Stack: [return_data_pointer] [return_data_length]
            block.call(write_return_data_function);

            // TODO: flush cache??

            // Return status
            block.local_get(status);
            block.return_();
        });
    }

    // When no match is found, we just panic (TODO: handle fallback)
    router_builder.unreachable();

    let router = router.finish(vec![args_len], &mut module.funcs);
    add_entrypoint(module, router);
}

/// Add an entrypoint to the module with the interface defined by Stylus
pub fn add_entrypoint(module: &mut Module, func: FunctionId) {
    module.exports.add("user_entrypoint", func);
}

#[cfg(test)]
mod tests {
    use move_binary_format::file_format::Signature;
    use walrus::{MemoryId, ModuleConfig, ir::StoreKind};
    use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store, TypedFunc};

    use crate::{
        abi_types::function_encoding::move_signature_to_abi_selector, memory::setup_module_memory,
        utils::display_module,
    };

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn add_noop_return_zero_function(module: &mut Module) -> (FunctionId, AbiFunctionSelector) {
        // Noop function
        let mut noop_builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32, ValType::I32, ValType::I32],
        );
        let mut noop_body = noop_builder.func_body();

        let args_pointer = module.locals.add(ValType::I32);
        let args_length = module.locals.add(ValType::I32);

        // null return data and status 0
        noop_body.i32_const(0);
        noop_body.i32_const(0);
        noop_body.i32_const(0);

        let noop = noop_builder.finish(vec![args_pointer, args_length], &mut module.funcs);

        let function_selector = move_signature_to_abi_selector("noop_zero", &Signature(vec![]));

        (noop, function_selector)
    }

    fn add_noop_return_one_function(module: &mut Module) -> (FunctionId, AbiFunctionSelector) {
        // Noop function
        let mut noop_builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32, ValType::I32, ValType::I32],
        );
        let mut noop_body = noop_builder.func_body();

        let args_pointer = module.locals.add(ValType::I32);
        let args_length = module.locals.add(ValType::I32);

        // null return data and status 1
        noop_body.i32_const(0);
        noop_body.i32_const(0);
        noop_body.i32_const(1);

        let noop = noop_builder.finish(vec![args_pointer, args_length], &mut module.funcs);

        let function_selector = move_signature_to_abi_selector("noop_one", &Signature(vec![]));

        (noop, function_selector)
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

        linker
            .func_wrap(
                "vm_hooks",
                "read_args",
                move |mut caller: Caller<'_, ReadArgsData>, args_ptr: u32| {
                    println!("read_args");

                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

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
                move |mut caller: Caller<'_, ReadArgsData>,
                      return_data_pointer: u32,
                      return_data_length: u32| {
                    println!("write_result");
                    println!("return_data_pointer: {}", return_data_pointer);
                    println!("return_data_length: {}", return_data_length);

                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut buffer = vec![0; return_data_length as usize];
                    mem.read(&mut caller, return_data_pointer as usize, &mut buffer)
                        .unwrap();
                    println!("return_data: {:?}", buffer);

                    Ok(())
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
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let (noop_return_zero, function_selector_return_zero) =
            add_noop_return_zero_function(&mut raw_module);
        let (noop_return_one, function_selector_return_one) =
            add_noop_return_one_function(&mut raw_module);

        build_entrypoint_router(
            &mut raw_module,
            allocator_func,
            memory_id,
            &[
                (noop_return_zero, function_selector_return_zero),
                (noop_return_one, function_selector_return_one),
            ],
        );
        display_module(&mut raw_module);

        let data = ReadArgsData {
            data: function_selector_return_zero.to_vec(),
        };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 0);

        let data = ReadArgsData {
            data: function_selector_return_one.to_vec(),
        };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn test_build_entrypoint_router_no_match() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let (noop_return_zero, function_selector_return_zero) =
            add_noop_return_zero_function(&mut raw_module);
        let (noop_return_one, function_selector_return_one) =
            add_noop_return_one_function(&mut raw_module);

        build_entrypoint_router(
            &mut raw_module,
            allocator_func,
            memory_id,
            &[
                (noop_return_zero, function_selector_return_zero),
                (noop_return_one, function_selector_return_one),
            ],
        );
        display_module(&mut raw_module);

        let data = ReadArgsData { data: vec![0; 4] };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        entrypoint.call(&mut store, data_len).unwrap();
    }

    fn add_data_write_function(
        module: &mut Module,
        allocator_func: FunctionId,
        memory_id: MemoryId,
    ) -> (FunctionId, AbiFunctionSelector) {
        let mut noop_builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32, ValType::I32, ValType::I32],
        );
        let mut noop_body = noop_builder.func_body();

        let args_pointer = module.locals.add(ValType::I32);
        let args_length = module.locals.add(ValType::I32);

        let data = [2; 4];
        let data_size = data.len() as i32;

        let data_pointer = module.locals.add(ValType::I32);
        let data_length = module.locals.add(ValType::I32);

        noop_body.i32_const(data_size);
        noop_body.local_tee(data_length);
        noop_body.call(allocator_func);
        noop_body.local_tee(data_pointer);
        noop_body.i32_const(i32::from_le_bytes(data));
        noop_body.store(
            memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // null return data and status 0
        noop_body.local_get(data_pointer);
        noop_body.local_get(data_length);
        noop_body.i32_const(0);

        let noop = noop_builder.finish(vec![args_pointer, args_length], &mut module.funcs);

        let function_selector = move_signature_to_abi_selector("noop_one", &Signature(vec![]));

        (noop, function_selector)
    }

    #[test]
    fn test_build_entrypoint_router_data_write() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let (data_write, function_selector_data_write) =
            add_data_write_function(&mut raw_module, allocator_func, memory_id);

        build_entrypoint_router(
            &mut raw_module,
            allocator_func,
            memory_id,
            &[(data_write, function_selector_data_write)],
        );
        display_module(&mut raw_module);

        let data = ReadArgsData {
            data: function_selector_data_write.to_vec(),
        };
        let data_len = data.data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data);

        let result = entrypoint.call(&mut store, data_len).unwrap();
        assert_eq!(result, 0);
    }
}
