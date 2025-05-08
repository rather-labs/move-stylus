use walrus::{FunctionId, InstrSeqBuilder, LocalId, Module, ValType, ir::BinaryOp};

use crate::translation::{
    functions::add_unpack_function_return_values_instructions, intermediate_types::ISignature,
};

use super::{
    function_encoding::{AbiFunctionSelector, move_signature_to_abi_selector},
    packing::build_pack_instructions,
    unpacking::build_unpack_instructions,
};

/// This struct wraps a Move function interface and its internal WASM representation
/// in order to expose it to the entrypoint router to be called externally.
///
/// It allows functions to be executed as contracts calls, by unpacking the arguments using `read_args` from the host,
/// injecting these arguments in the functions and packing the return values using `write_result` host function.
pub struct PublicFunction {
    function_id: FunctionId,
    function_selector: AbiFunctionSelector,
    signature: ISignature,
}

impl PublicFunction {
    pub fn new(function_id: FunctionId, function_name: &str, signature: ISignature) -> Self {
        let function_selector = move_signature_to_abi_selector(function_name, &signature.arguments);

        Self {
            function_id,
            function_selector,
            signature,
        }
    }

    #[cfg(test)]
    pub fn get_selector(&self) -> &AbiFunctionSelector {
        &self.function_selector
    }

    /// Builds the router block for the function
    ///
    /// Executes the wrapped function if the selector matches
    #[allow(clippy::too_many_arguments)]
    pub fn build_router_block(
        &self,
        router_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        selector_variable: LocalId,
        args_pointer: LocalId,
        args_len: LocalId,
        write_return_data_function: FunctionId,
        storage_flush_cache_function: FunctionId,
        allocator_func: FunctionId,
    ) {
        router_builder.block(None, |block| {
            let block_id = block.id();

            block.local_get(selector_variable);
            block.i32_const(i32::from_le_bytes(self.function_selector));
            block.binop(BinaryOp::I32Ne);
            block.br_if(block_id);

            // Offset args pointer by 4 bytes to exclude selector
            block.local_get(args_pointer);
            block.i32_const(4);
            block.binop(BinaryOp::I32Add);
            block.local_set(args_pointer);

            // Reduce args length by 4 bytes to exclude selector
            block.local_get(args_len);
            block.i32_const(4);
            block.binop(BinaryOp::I32Sub);
            block.local_set(args_len);

            // Wrap function to pack/unpack parameters
            self.wrap_public_function(module, block, args_pointer, allocator_func);

            // Stack: [return_data_pointer] [return_data_length] [status]
            let status = module.locals.add(ValType::I32);
            block.local_set(status);

            // Write return data to memory
            // Stack: [return_data_pointer] [return_data_length]
            block.call(write_return_data_function);

            block.i32_const(0); // Do not clear cache
            block.call(storage_flush_cache_function);

            // Return status
            block.local_get(status);
            block.return_();
        });
    }

    /// Wraps the function unpacking input parameters from memory and packing output parameters to memory
    ///
    /// Input parameters are read from memory and unpacked as *abi encoded* values
    /// Output parameters are packed as *abi encoded* values and written to memory
    fn wrap_public_function(
        &self,
        module: &mut Module,
        block: &mut InstrSeqBuilder,
        args_pointer: LocalId,
        allocator_func: FunctionId,
    ) {
        let memory_id = module.get_memory_id().expect("memory not found");

        build_unpack_instructions(
            block,
            module,
            &self.signature.arguments,
            args_pointer,
            memory_id,
            allocator_func,
        );
        block.call(self.function_id);
        add_unpack_function_return_values_instructions(
            block,
            &mut module.locals,
            &self.signature.returns,
            memory_id,
        );

        build_pack_instructions(
            block,
            &self.signature.returns,
            module,
            memory_id,
            allocator_func,
        );

        // TODO: Define error handling strategy, for now it will always result in traps
        // So it will only reach this point in the case of success
        block.i32_const(0);
    }
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, sol};
    use walrus::{
        FunctionBuilder, MemoryId, ModuleConfig,
        ir::{LoadKind, MemArg},
    };
    use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store, TypedFunc};

    use crate::{
        hostio::host_functions,
        memory::setup_module_memory,
        translation::{functions::prepare_function_return, intermediate_types::IntermediateType},
        utils::display_module,
    };

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        expected_result: Vec<u8>,
    ) -> (Linker<()>, Store<()>, TypedFunc<(), i32>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "write_result",
                move |mut caller: Caller<'_, ()>,
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

                    assert_eq!(buffer, expected_result);

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap("vm_hooks", "storage_flush_cache", |_: i32| Ok(()))
            .unwrap();

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<(), i32>(&mut store, "mock_entrypoint")
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

    fn build_mock_router(
        module: &mut Module,
        public_function: &PublicFunction,
        data_len: i32,
        allocator_func: FunctionId,
        memory_id: MemoryId,
    ) {
        // Build mock router
        let (write_return_data_function, _) = host_functions::write_result(module);
        let (storage_flush_cache_function, _) = host_functions::storage_flush_cache(module);

        let selector = module.locals.add(ValType::I32);
        let args_pointer = module.locals.add(ValType::I32);
        let args_len = module.locals.add(ValType::I32);

        let mut mock_router_builder = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

        let mut mock_router_body = mock_router_builder.func_body();

        // Allocate memory to compensate for the forced memory initialization
        mock_router_body.i32_const(data_len);
        mock_router_body.call(allocator_func);
        mock_router_body.drop();

        mock_router_body.i32_const(0);
        mock_router_body.local_set(args_pointer);

        mock_router_body.i32_const(data_len);
        mock_router_body.local_set(args_len);

        // Load selector from first 4 bytes of args
        mock_router_body.local_get(args_pointer);
        mock_router_body.load(
            memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
        mock_router_body.local_set(selector);

        public_function.build_router_block(
            &mut mock_router_body,
            module,
            selector,
            args_pointer,
            args_len,
            write_return_data_function,
            storage_flush_cache_function,
            allocator_func,
        );

        // if no match, return -1
        mock_router_body.i32_const(-1);
        mock_router_body.return_();

        let mock_entrypoint = mock_router_builder.finish(vec![], &mut module.funcs);
        module.exports.add("mock_entrypoint", mock_entrypoint);
    }

    #[test]
    fn test_build_public_function() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32, ValType::I64],
            &[ValType::I32],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);
        let param3 = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.local_get(param1);
        func_body.i32_const(1);
        func_body.binop(BinaryOp::I32Add);

        func_body.local_get(param2);
        func_body.i32_const(1);
        func_body.binop(BinaryOp::I32Add);

        func_body.local_get(param3);
        func_body.i64_const(1);
        func_body.binop(BinaryOp::I64Add);

        let returns = vec![
            IntermediateType::IU32,
            IntermediateType::IU16,
            IntermediateType::IU64,
        ];
        prepare_function_return(
            &mut raw_module.locals,
            &mut func_body,
            &returns,
            memory_id,
            allocator_func,
        );

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let public_function = PublicFunction::new(
            function,
            "test_function",
            ISignature {
                arguments: vec![
                    IntermediateType::IBool,
                    IntermediateType::IU16,
                    IntermediateType::IU64,
                ],
                returns,
            },
        );

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        data = [public_function.get_selector().to_vec(), data].concat();
        let data_len = data.len() as i32;

        // Build mock router
        build_mock_router(
            &mut raw_module,
            &public_function,
            data_len,
            allocator_func,
            memory_id,
        );

        display_module(&mut raw_module);

        let expected_result =
            <sol!((uint32, uint16, uint64))>::abi_encode_params(&(2, 1235, 123456789012346));

        let (_, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, expected_result);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_build_entrypoint_router_no_match() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32, ValType::I64],
            &[ValType::I32],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);
        let param3 = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.local_get(param1);
        func_body.i32_const(1);
        func_body.binop(BinaryOp::I32Add);

        func_body.local_get(param2);
        func_body.i32_const(1);
        func_body.binop(BinaryOp::I32Add);
        func_body.drop();

        func_body.local_get(param3);
        func_body.i64_const(1);
        func_body.binop(BinaryOp::I64Add);
        func_body.drop();

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let public_function = PublicFunction::new(
            function,
            "test_function",
            ISignature {
                arguments: vec![
                    IntermediateType::IU32,
                    IntermediateType::IU32,
                    IntermediateType::IU64,
                ],
                returns: vec![IntermediateType::IU32],
            },
        );

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        data = [public_function.get_selector().to_vec(), data].concat();
        // This will make the selector invalid
        data[0] = 0;
        let data_len = data.len() as i32;

        // Build mock router
        build_mock_router(
            &mut raw_module,
            &public_function,
            data_len,
            allocator_func,
            memory_id,
        );

        display_module(&mut raw_module);

        let (_, mut store, entrypoint) = setup_wasmtime_module(&mut raw_module, data, vec![]);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, -1);
    }
}
