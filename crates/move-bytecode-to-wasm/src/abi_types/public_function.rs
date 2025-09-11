use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::{DATA_ABORT_FLAG_OFFSET, DATA_ERROR_CODE_OFFSET},
    runtime::RuntimeFunction,
    runtime_error_codes::ERROR_SELECTOR,
    translation::{
        functions::add_unpack_function_return_values_instructions,
        intermediate_types::{ISignature, IntermediateType},
    },
    vm_handled_types::{VmHandledType, signer::Signer},
};

use super::{
    function_encoding::{AbiFunctionSelector, move_signature_to_abi_selector},
    packing::build_pack_instructions,
    unpacking::build_unpack_instructions,
};

#[derive(thiserror::Error, Debug)]
pub enum PublicFunctionValidationError {
    #[error(r#"error in argument {0} of function "{1}", only one "signer" argument at the beginning is admitted"#)]
    SignatureArgumentPosition(usize, String),

    #[error(
        r#"error in argument {0} of function "{1}", complex types can't contain the type "signer""#
    )]
    ComplexTypeContainsSigner(usize, String),
}

/// This struct wraps a Move function interface and its internal WASM representation
/// in order to expose it to the entrypoint router to be called externally.
///
/// It allows functions to be executed as contracts calls, by unpacking the arguments using `read_args` from the host,
/// injecting these arguments in the functions and packing the return values using `write_result` host function.
pub struct PublicFunction<'a> {
    function_id: FunctionId,
    function_selector: AbiFunctionSelector,
    signature: &'a ISignature,
}

impl<'a> PublicFunction<'a> {
    pub fn new(
        function_id: FunctionId,
        function_name: &str,
        signature: &'a ISignature,
        compilation_ctx: &CompilationContext,
    ) -> Self {
        Self::check_signature_arguments(function_name, &signature.arguments)
            .unwrap_or_else(|e| panic!("ABI error: {e}"));

        let function_selector =
            move_signature_to_abi_selector(function_name, &signature.arguments, compilation_ctx);

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
        compilation_ctx: &CompilationContext,
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

            // If the first argument's type is signer, we inject the tx.origin into the stack as a
            // first parameter
            match self.signature.arguments.first() {
                Some(IntermediateType::ISigner) => {
                    Signer::inject(block, module, compilation_ctx);
                }
                Some(IntermediateType::IRef(inner)) if **inner == IntermediateType::ISigner => {
                    Signer::inject(block, module, compilation_ctx);
                }
                _ => {
                    // If there's no signer, reduce args length by 4 bytes to exclude selector,
                    // otherwise we reuse the selector's 4 bytes (32 bits) for the signer pointer
                    block.local_get(args_len);
                    block.i32_const(4);
                    block.binop(BinaryOp::I32Sub);
                    block.local_set(args_len);
                }
            }

            // Wrap function to pack/unpack parameters
            self.wrap_public_function(module, block, args_pointer, compilation_ctx);

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
        compilation_ctx: &CompilationContext,
    ) {
        let data_ptr = module.locals.add(ValType::I32);
        let data_len = module.locals.add(ValType::I32);
        let status = module.locals.add(ValType::I32);

        // Clear abort flag data
        block.i32_const(DATA_ABORT_FLAG_OFFSET).i32_const(0).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Clear error code data
        block.i32_const(DATA_ERROR_CODE_OFFSET).i64_const(0).store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        build_unpack_instructions(
            block,
            module,
            &self.signature.arguments,
            args_pointer,
            compilation_ctx,
        );

        block.call(self.function_id);

        // Unpack function return values
        add_unpack_function_return_values_instructions(
            block,
            module,
            &self.signature.returns,
            compilation_ctx.memory_id,
        );

        // Success path
        if self.signature.returns.is_empty() {
            // Set data_ptr and data_len to 0
            block.i32_const(0).local_set(data_ptr);
            block.i32_const(0).local_set(data_len);
        } else {
            // Set data_ptr and data_len to the result of packing the return values
            let (data_ptr_, data_len_) =
                build_pack_instructions(block, &self.signature.returns, module, compilation_ctx);

            block.local_get(data_ptr_).local_set(data_ptr);
            block.local_get(data_len_).local_set(data_len);
        }

        // Check if the abort flag is set
        block
            .i32_const(DATA_ABORT_FLAG_OFFSET)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .i32_const(0)
            .binop(BinaryOp::I32Ne);

        block.if_else(
            None,
            |then_| {
                // Abort path
                then_.i32_const(1).local_set(status);

                // Load the u64 abort code from memory
                let error_code = module.locals.add(ValType::I64);
                then_
                    .i32_const(DATA_ERROR_CODE_OFFSET)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(error_code);

                // Build error message
                Self::build_abort_error_message(
                    then_,
                    module,
                    compilation_ctx,
                    error_code,
                    data_ptr,
                    data_len,
                );
            },
            |else_| {
                // Success path

                // Set status to 0 to flag success
                else_.i32_const(0).local_set(status);
            },
        );

        // [data_ptr][data_len][status]
        block
            .local_get(data_ptr)
            .local_get(data_len)
            .local_get(status);
    }

    /// This function checks if the arguments of a public functions is valid. A signature is not
    /// valid in the following cases:
    ///
    /// - It contains more than one `signer`: In an EVM context, there is only one signer per transaction.
    /// - It contains a `signer` argument but it is not the first argument: The Move specification
    ///   states that, [if a public function contains a `signer` argument, it must be placed as the
    ///   first argument](https://move-language.github.io/move/signer.html#comparison-to-address).
    /// - It has any complex type (i.e: vector) that contains a signer type: The signer is
    ///   injected by the VM only if the first argument of the function is a `signer`.
    fn check_signature_arguments(
        function_name: &str,
        arguments: &[IntermediateType],
    ) -> Result<(), PublicFunctionValidationError> {
        for (i, argument) in arguments.iter().enumerate() {
            match argument {
                IntermediateType::ISigner => {
                    if i != 0 {
                        return Err(PublicFunctionValidationError::SignatureArgumentPosition(
                            i + 1,
                            function_name.to_owned(),
                        ));
                    }
                }
                IntermediateType::IVector(it) if Self::find_signature_type(it) => {
                    return Err(PublicFunctionValidationError::ComplexTypeContainsSigner(
                        i + 1,
                        function_name.to_owned(),
                    ));
                }
                // TODO: add TxContext as last parameter
                _ => continue,
            }
        }

        Ok(())
    }

    // Recursively checks if a type contains the `signature` type. This is used to look for the
    // type in complex types such as vector or structs
    fn find_signature_type(argument: &IntermediateType) -> bool {
        match argument {
            IntermediateType::ISigner => true,
            IntermediateType::IVector(intermediate_type) => {
                Self::find_signature_type(intermediate_type)
            }
            _ => false,
        }
    }

    /// Builds an error message for abort instructions with the error code converted to decimal.
    fn build_abort_error_message(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        error_code: LocalId,
        data_ptr: LocalId,
        data_len: LocalId,
    ) {
        // Message prefix
        const PREFIX: &[u8] = b"Abort instruction reached: error code ";

        // Convert error code to decimal string
        let u64_to_dec_ascii = RuntimeFunction::U64ToAsciiBase10.get(module, Some(compilation_ctx));
        let error_ptr = module.locals.add(ValType::I32);
        let error_len = module.locals.add(ValType::I32);

        // Convert error code to decimal string
        builder
            .local_get(error_code)
            .call(u64_to_dec_ascii)
            .local_tee(error_ptr);

        // Load the length of the decimal string
        builder
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(error_len);

        // Load the pointer to the decimal string
        builder
            .local_get(error_ptr)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_set(error_ptr);

        // Calculate message length and total allocation size
        let msg_raw_len = module.locals.add(ValType::I32);
        let msg_total_len = module.locals.add(ValType::I32);
        const HEAD_OFFSET: u32 = 35; // Position of head word (0x20)
        const LENGTH_OFFSET: u32 = 64; // Position of length word
        const MSG_START: u32 = 4 + 32 + 32; // selector(4) + head(32) + len(32)

        // msg_raw_len = PREFIX.len() + decimal_len
        builder
            .i32_const(PREFIX.len() as i32)
            .local_get(error_len)
            .binop(BinaryOp::I32Add)
            .local_set(msg_raw_len);

        // total_len = selector(4) + head(32) + len(32) + padded_msg_len
        builder
            .local_get(msg_raw_len)
            .i32_const(31)
            .binop(BinaryOp::I32Add)
            .i32_const(!31)
            .binop(BinaryOp::I32And)
            .i32_const(MSG_START as i32)
            .binop(BinaryOp::I32Add)
            .local_set(msg_total_len);

        // Allocate memory
        builder
            .local_get(msg_total_len)
            .local_tee(data_len)
            .call(compilation_ctx.allocator)
            .local_set(data_ptr);

        // Write error selector (first 4 bytes)
        for (i, b) in ERROR_SELECTOR.iter().enumerate() {
            builder.local_get(data_ptr).i32_const(*b as i32).store(
                compilation_ctx.memory_id,
                StoreKind::I32_8 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i as u32,
                },
            );
        }

        // Write head word (offset to data = 0x20) in the last byte of the 32-byte word
        builder.local_get(data_ptr).i32_const(32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: HEAD_OFFSET,
            },
        );

        // Write length word (big-endian, in the LAST 4 bytes of the 32-byte word)
        let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);
        builder
            .local_get(data_ptr)
            .local_get(msg_raw_len)
            .call(swap_i32)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: LENGTH_OFFSET,
                },
            );

        // Write prefix
        for (i, &b) in PREFIX.iter().enumerate() {
            builder.local_get(data_ptr).i32_const(b as i32).store(
                compilation_ctx.memory_id,
                StoreKind::I32_8 { atomic: false },
                MemArg {
                    align: 0,
                    offset: MSG_START + i as u32,
                },
            );
        }

        // Append decimal digits after the prefix
        builder
            .local_get(data_ptr)
            .i32_const(MSG_START as i32 + PREFIX.len() as i32)
            .binop(BinaryOp::I32Add)
            .local_get(error_ptr)
            .local_get(error_len)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{
        FunctionBuilder, MemoryId,
        ir::{LoadKind, MemArg},
    };
    use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store, TypedFunc};

    use crate::{
        hostio::host_functions,
        test_compilation_context,
        test_tools::build_module,
        translation::{functions::prepare_function_return, intermediate_types::IntermediateType},
        utils::display_module,
    };

    use super::*;

    fn setup_wasmtime_module(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        expected_result: Vec<u8>,
    ) -> (Linker<()>, Store<()>, TypedFunc<(), i32>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();
        let get_memory =
            move |caller: &mut Caller<'_, ()>| match caller.get_module_export(&mem_export) {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find host memory"),
            };

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

                    let mem = get_memory(&mut caller);

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

        linker
            .func_wrap(
                "vm_hooks",
                "tx_origin",
                move |mut caller: Caller<'_, ()>, ptr: u32| {
                    println!("tx_origin, writing in {ptr}");

                    let mem = get_memory(&mut caller);

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
                move |mut caller: Caller<'_, ()>, ptr: u32, len: u32, _topic: u32| {
                    println!("emit_log, reading from {ptr}, length: {len}");

                    let mem = get_memory(&mut caller);
                    let mut buffer = vec![0; len as usize];

                    mem.read(&mut caller, ptr as usize, &mut buffer).unwrap();

                    println!("read memory: {buffer:?}");
                },
            )
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
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
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
            &compilation_ctx,
        );

        // if no match, return -1
        mock_router_body.i32_const(-1);
        mock_router_body.return_();

        let mock_entrypoint = mock_router_builder.finish(vec![], &mut module.funcs);
        module.exports.add("mock_entrypoint", mock_entrypoint);
    }

    #[test]
    fn test_build_public_function() {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, allocator);
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
        prepare_function_return(&mut raw_module, &mut func_body, &returns, &compilation_ctx);

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            returns,
        };
        let public_function =
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx);

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        data = [public_function.get_selector().to_vec(), data].concat();
        let data_len = data.len() as i32;

        // Build mock router
        build_mock_router(
            &mut raw_module,
            &public_function,
            data_len,
            allocator,
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
    #[should_panic]
    fn test_build_public_function_with_signer() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(1);
        func_body.local_get(param2);
        func_body.binop(BinaryOp::I32Add);

        func_body.local_get(param1);

        let returns = vec![IntermediateType::IU8, IntermediateType::ISigner];
        prepare_function_return(&mut raw_module, &mut func_body, &returns, &compilation_ctx);

        let function = function_builder.finish(vec![param1, param2], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![IntermediateType::ISigner, IntermediateType::IU8],
            returns,
        };
        let public_function =
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx);

        let mut data = <sol!((uint8,))>::abi_encode_params(&(1,));
        data = [public_function.get_selector().to_vec(), data].concat();
        let data_len = data.len() as i32;

        // Build mock router
        build_mock_router(
            &mut raw_module,
            &public_function,
            data_len,
            allocator,
            memory_id,
        );

        display_module(&mut raw_module);

        let expected_result = <sol!((uint8, address))>::abi_encode_params(&(
            2,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7],
        ));

        let (_, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, expected_result);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_build_entrypoint_router_no_match() {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

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

        let signature = ISignature {
            arguments: vec![
                IntermediateType::IU32,
                IntermediateType::IU32,
                IntermediateType::IU64,
            ],
            returns: vec![IntermediateType::IU32],
        };
        let public_function =
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx);

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

    // TODO: At the moment we are just checking that this does not fail when tranlsating the
    // signature to EVM ABI. Once the signer address injection is in place, create a test that
    // injects a mock address as signer and execute the function
    #[test]
    fn public_function_with_signature() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32, ValType::I64],
            &[],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);
        let param3 = raw_module.locals.add(ValType::I64);

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![
                IntermediateType::ISigner,
                IntermediateType::IBool,
                IntermediateType::IU64,
            ],
            returns: vec![],
        };
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"ABI error: error in argument 2 of function "test_function", only one "signer" argument at the beginning is admitted"#
    )]
    fn test_fail_public_function_signature() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I64], &[]);

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);
        let param3 = raw_module.locals.add(ValType::I64);

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![
                IntermediateType::IBool,
                IntermediateType::ISigner,
                IntermediateType::IU64,
            ],
            returns: vec![],
        };
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"ABI error: error in argument 3 of function "test_function", complex types can't contain the type "signer""#
    )]
    fn test_fail_public_function_signature_complex_type() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I64], &[]);

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I64);
        let param3 = raw_module.locals.add(ValType::I32);

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![
                IntermediateType::IBool,
                IntermediateType::IU64,
                IntermediateType::IVector(Box::new(IntermediateType::ISigner)),
            ],
            returns: vec![],
        };
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"ABI error: error in argument 3 of function "test_function", complex types can't contain the type "signer""#
    )]
    fn test_fail_public_function_signature_complex_type_2() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I64], &[]);

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I64);
        let param3 = raw_module.locals.add(ValType::I32);

        let function = function_builder.finish(vec![param1, param2, param3], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![
                IntermediateType::IBool,
                IntermediateType::IU64,
                IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                    IntermediateType::ISigner,
                )))),
            ],
            returns: vec![],
        };
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx);
    }
}
