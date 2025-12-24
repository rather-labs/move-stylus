use walrus::{
    FunctionId, GlobalId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    runtime::RuntimeFunction,
    translation::{
        functions::add_unpack_function_return_values_instructions,
        intermediate_types::{ISignature, IntermediateType},
    },
    vm_handled_types::{VmHandledType, signer::Signer},
};

use super::{
    abi_encoding::AbiFunctionSelector, error::AbiError,
    function_encoding::move_signature_to_abi_selector, packing::build_pack_instructions,
    unpacking::build_unpack_instructions,
};

#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
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
    pub(crate) function_name: String,
    pub(crate) function_selector: AbiFunctionSelector,
    signature: &'a ISignature,
}

impl<'a> PublicFunction<'a> {
    pub fn new(
        function_id: FunctionId,
        function_name: &str,
        signature: &'a ISignature,
        compilation_ctx: &CompilationContext,
    ) -> Result<Self, AbiError> {
        Self::check_signature_arguments(function_name, &signature.arguments)?;

        let function_selector =
            move_signature_to_abi_selector(function_name, &signature.arguments, compilation_ctx)?;

        Ok(Self {
            function_id,
            function_name: function_name.to_string(),
            function_selector,
            signature,
        })
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
        compilation_ctx: &CompilationContext,
        dynamic_fields_global_variables: &Vec<(GlobalId, IntermediateType)>,
    ) -> Result<(), AbiError> {
        let mut inner_result = Ok(());
        let commit_changes_to_storage_function = RuntimeFunction::get_commit_changes_to_storage_fn(
            module,
            compilation_ctx,
            dynamic_fields_global_variables,
        )?;

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
            inner_result = self.wrap_public_function(module, block, args_pointer, compilation_ctx);

            // Stack: [return_data_pointer] [return_data_length] [status]
            let status = module.locals.add(ValType::I32);
            block.local_set(status);

            // Write return data to memory
            // Stack: [return_data_pointer] [return_data_length]
            block.call(write_return_data_function);

            // TODO: This is repeated for every function, we should move this and the return below
            // outside this into the main body of the entrypoint so we don't needlesly repeat code
            block.call(commit_changes_to_storage_function);

            // Return status
            block.local_get(status);
            block.return_();
        });

        inner_result
    }

    /// Wraps the function unpacking input parameters from memory and packing output parameters to memory
    ///
    /// Input parameters are read from memory and unpacked as *abi encoded* values
    /// Output parameters are packed as *abi encoded* values and written to memory
    pub fn wrap_public_function(
        &self,
        module: &mut Module,
        block: &mut InstrSeqBuilder,
        args_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        let status = module.locals.add(ValType::I32);
        let data_ptr = module.locals.add(ValType::I32);
        let data_len = module.locals.add(ValType::I32);

        build_unpack_instructions(
            block,
            module,
            &self.signature.arguments,
            args_pointer,
            compilation_ctx,
        )?;

        block.call(self.function_id);

        // Unpack function return values
        add_unpack_function_return_values_instructions(
            block,
            module,
            &self.signature.returns,
            compilation_ctx.memory_id,
        )?;

        if self.signature.returns.is_empty() {
            // Set data_ptr and data_len to 0
            block.i32_const(0).local_set(data_ptr);
            block.i32_const(0).local_set(data_len);
        } else {
            // Set data_ptr and data_len to the result of packing the return values
            let (data_ptr_, data_len_) =
                build_pack_instructions(block, &self.signature.returns, module, compilation_ctx)?;
            block.local_get(data_ptr_).local_set(data_ptr);
            block.local_get(data_len_).local_set(data_len);
        }

        // Load the abort message pointer from DATA_ABORT_MESSAGE_PTR_OFFSET
        // If not null, an abort occurred and we need to return the error message
        block.block(None, |abort_block| {
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
            abort_block.i32_const(0).binop(BinaryOp::I32Eq);

            // If the ptr is null, jump to the end of the block, skipping the error message loading
            abort_block.br_if(abort_block_id);

            // Load the abort message length from the ptr and set data_len
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
                .local_set(data_ptr);

            // Set status to 1
            abort_block.i32_const(1).local_set(status);
        });

        // [data_ptr][data_len][status]
        block
            .local_get(data_ptr)
            .local_get(data_len)
            .local_get(status);

        Ok(())
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
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{
        ConstExpr, FunctionBuilder, MemoryId,
        ir::{LoadKind, MemArg, Value},
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
                "native_keccak256",
                |mut caller: wasmtime::Caller<'_, ()>,
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
                "write_result",
                move |mut caller: Caller<'_, ()>,
                      return_data_pointer: u32,
                      return_data_length: u32| {
                    println!("write_result");
                    println!("return_data_pointer: {return_data_pointer}");
                    println!("return_data_length: {return_data_length}");

                    let mem = get_memory(&mut caller);

                    let mut buffer = vec![0; return_data_length as usize];
                    mem.read(&mut caller, return_data_pointer as usize, &mut buffer)
                        .unwrap();
                    println!("return_data: {buffer:?}");

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
        let calldata_reader_pointer_global =
            module
                .globals
                .add_local(ValType::I32, true, false, ConstExpr::Value(Value::I32(0)));
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);
        // Build mock router
        let (write_return_data_function, _) = host_functions::write_result(module);

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

        public_function
            .build_router_block(
                &mut mock_router_body,
                module,
                selector,
                args_pointer,
                args_len,
                write_return_data_function,
                &compilation_ctx,
                &vec![],
            )
            .unwrap();

        // if no match, return -1
        mock_router_body.i32_const(-1);
        mock_router_body.return_();

        let mock_entrypoint = mock_router_builder.finish(vec![], &mut module.funcs);
        module.exports.add("mock_entrypoint", mock_entrypoint);
    }

    #[test]
    fn test_build_public_function() {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32, ValType::I64],
            &[ValType::I32],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);
        let param3 = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        let returns = vec![
            IntermediateType::IU32,
            IntermediateType::IU16,
            IntermediateType::IU64,
        ];

        let result_local_id = raw_module.locals.add(ValType::I32);
        func_body.block(None, |result_block| {
            let result_block_id = result_block.id();

            // Compute return values inside the block
            result_block.local_get(param1);
            result_block.i32_const(1);
            result_block.binop(BinaryOp::I32Add);

            result_block.local_get(param2);
            result_block.i32_const(1);
            result_block.binop(BinaryOp::I32Add);

            result_block.local_get(param3);
            result_block.i64_const(1);
            result_block.binop(BinaryOp::I64Add);

            // Now prepare_function_return can consume the return values from the stack
            prepare_function_return(
                &mut raw_module,
                result_block,
                &returns,
                &compilation_ctx,
                result_block_id,
                result_local_id,
            )
            .unwrap();
        });

        // Load the result if there are return values
        if !returns.is_empty() {
            func_body.local_get(result_local_id);
        }

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
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx).unwrap();

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
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let param1 = raw_module.locals.add(ValType::I32);
        let param2 = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        let returns = vec![IntermediateType::IU8, IntermediateType::ISigner];

        let result_local_id = raw_module.locals.add(ValType::I32);
        func_body.block(None, |result_block| {
            let result_block_id = result_block.id();

            // Compute return values inside the block
            result_block.i32_const(1);
            result_block.local_get(param2);
            result_block.binop(BinaryOp::I32Add);

            result_block.local_get(param1);

            // Now prepare_function_return can consume the return values from the stack
            prepare_function_return(
                &mut raw_module,
                result_block,
                &returns,
                &compilation_ctx,
                result_block_id,
                result_local_id,
            )
            .unwrap();
        });

        // Load the result if there are return values
        if !returns.is_empty() {
            func_body.local_get(result_local_id);
        }

        let function = function_builder.finish(vec![param1, param2], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let signature = ISignature {
            arguments: vec![IntermediateType::ISigner, IntermediateType::IU8],
            returns,
        };
        let public_function =
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx).unwrap();

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
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

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
            PublicFunction::new(function, "test_function", &signature, &compilation_ctx).unwrap();

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
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

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
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx).unwrap();
    }

    #[test]
    fn test_fail_public_function_signature() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

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
        let err = PublicFunction::new(function, "test_function", &signature, &compilation_ctx)
            .err()
            .unwrap();

        let err = match err {
            AbiError::PublicFunction(e) => e,
            _ => panic!("expected PublicFunctionValidationError"),
        };
        assert_eq!(
            PublicFunctionValidationError::SignatureArgumentPosition(2, "test_function".to_owned()),
            err
        );
    }

    #[test]
    fn test_fail_public_function_signature_complex_type() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

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

        let err = PublicFunction::new(function, "test_function", &signature, &compilation_ctx)
            .err()
            .unwrap();

        let err = match err {
            AbiError::PublicFunction(e) => e,
            _ => panic!("expected PublicFunctionValidationError"),
        };
        assert_eq!(
            PublicFunctionValidationError::ComplexTypeContainsSigner(3, "test_function".to_owned()),
            err
        );
    }

    #[test]
    fn test_fail_public_function_signature_complex_type_2() {
        let (mut raw_module, allocator, memory_id) = build_module(None);
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

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
        PublicFunction::new(function, "test_function", &signature, &compilation_ctx).err();

        let err = PublicFunction::new(function, "test_function", &signature, &compilation_ctx)
            .err()
            .unwrap();

        let err = match err {
            AbiError::PublicFunction(e) => e,
            _ => panic!("expected PublicFunctionValidationError"),
        };
        assert_eq!(
            PublicFunctionValidationError::ComplexTypeContainsSigner(3, "test_function".to_owned()),
            err
        );
    }
}
