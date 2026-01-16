use alloy_sol_types::{SolType, sol_data};
use walrus::{InstrSeqBuilder, LocalId, Module, ValType, ir::InstrSeqId};

use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiOperationError},
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_OBJECT, STYLUS_FRAMEWORK_ADDRESS},
    },
    data::RuntimeErrorData,
    native_functions::NativeFunction,
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    vm_handled_types::{
        VmHandledType, bytes::Bytes, named_id::NamedId, string::String_, tx_context::TxContext,
        uid::Uid,
    },
    wasm_builder_extensions::WasmBuilderExtension,
};

pub trait Unpackable {
    /// Adds the instructions to unpack the abi encoded type to WASM function parameters
    ///
    /// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
    /// and the pointer is pushed onto the stack in the parameter location.
    ///
    /// The reader pointer should be updated internally when a value is read from the args
    /// The calldata base pointer should never be updated, it is considered static for each type value
    ///
    /// The stack at the end contains the value(or pointer to the value) as **i32/i64**
    #[allow(clippy::too_many_arguments)]
    fn add_unpack_instructions(
        &self,
        parent_type: Option<&IntermediateType>,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        return_block_id: Option<InstrSeqId>,
        reader_pointer: LocalId,
        calldata_base_pointer: LocalId,
        compilation_ctx: &CompilationContext,
        runtime_error_data: Option<&mut RuntimeErrorData>,
    ) -> Result<(), AbiError>;
}

/// Builds the instructions to unpack the abi encoded values to WASM function parameters
///
/// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
/// and the pointer is pushed onto the stack in the parameter location.
pub fn build_unpack_instructions<T: Unpackable>(
    function_builder: &mut InstrSeqBuilder,
    return_block_id: InstrSeqId,
    module: &mut Module,
    function_arguments_signature: &[T],
    args_pointer: LocalId,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<(), AbiError> {
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_base_pointer = module.locals.add(ValType::I32);

    function_builder
        .local_get(args_pointer)
        .local_tee(reader_pointer)
        .local_set(calldata_base_pointer);

    // Set the global calldata reader pointer to the reader pointer
    function_builder
        .local_get(reader_pointer)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    // The ABI encoded params are always a tuple
    // Static types are stored in-place, but dynamic types are referenced to the call data
    for signature_token in function_arguments_signature.iter() {
        signature_token.add_unpack_instructions(
            None,
            function_builder,
            module,
            Some(return_block_id),
            reader_pointer,
            calldata_base_pointer,
            compilation_ctx,
            Some(runtime_error_data),
        )?;
    }

    Ok(())
}

impl Unpackable for IntermediateType {
    fn add_unpack_instructions(
        &self,
        parent_type: Option<&IntermediateType>,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        return_block_id: Option<InstrSeqId>,
        reader_pointer: LocalId,
        calldata_base_pointer: LocalId,
        compilation_ctx: &CompilationContext,
        runtime_error_data: Option<&mut RuntimeErrorData>,
    ) -> Result<(), AbiError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
                let encoded_size = match self {
                    IntermediateType::IBool => sol_data::Bool::ENCODED_SIZE,
                    IntermediateType::IU8 => sol_data::Uint::<8>::ENCODED_SIZE,
                    IntermediateType::IU16 => sol_data::Uint::<16>::ENCODED_SIZE,
                    IntermediateType::IU32 => sol_data::Uint::<32>::ENCODED_SIZE,
                    _ => None,
                }
                .ok_or(AbiError::UnableToGetTypeAbiSize)?;

                let unpack_u32_function =
                    RuntimeFunction::UnpackU32.get(module, Some(compilation_ctx), None)?;
                builder
                    .local_get(reader_pointer)
                    .i32_const(encoded_size as i32)
                    .call(unpack_u32_function);
            }
            IntermediateType::IU64 => {
                let unpack_i64_function =
                    RuntimeFunction::UnpackU64.get(module, Some(compilation_ctx), None)?;
                builder.local_get(reader_pointer).call(unpack_i64_function);
            }
            IntermediateType::IU128 => {
                let unpack_u128_function =
                    RuntimeFunction::UnpackU128.get(module, Some(compilation_ctx), None)?;
                builder.local_get(reader_pointer).call(unpack_u128_function);
            }
            IntermediateType::IU256 => {
                let unpack_u256_function =
                    RuntimeFunction::UnpackU256.get(module, Some(compilation_ctx), None)?;
                builder.local_get(reader_pointer).call(unpack_u256_function);
            }
            IntermediateType::IAddress => {
                let unpack_address_function =
                    RuntimeFunction::UnpackAddress.get(module, Some(compilation_ctx), None)?;
                builder
                    .local_get(reader_pointer)
                    .call(unpack_address_function);
            }
            IntermediateType::IVector(inner) => {
                let unpack_vector_fn = RuntimeFunction::UnpackVector.get_generic(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &[inner],
                )?;

                builder
                    .local_get(reader_pointer)
                    .local_get(calldata_base_pointer)
                    .call(unpack_vector_fn);
            }
            // The signer must not be unpacked here, since it can't be part of the calldata. It is
            // injected directly by the VM into the stack
            IntermediateType::ISigner => (),
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                match inner.as_ref() {
                    IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress
                    | IntermediateType::ISigner
                    | IntermediateType::IVector(_)
                    | IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. }
                    | IntermediateType::IEnum { .. }
                    | IntermediateType::IGenericEnumInstance { .. } => {
                        // For heap-allocated types, directly invoke the unpack function of the referenced inner type
                        inner.add_unpack_instructions(
                            Some(self),
                            builder,
                            module,
                            return_block_id,
                            reader_pointer,
                            calldata_base_pointer,
                            compilation_ctx,
                            runtime_error_data,
                        )?;
                    }
                    _ => {
                        // For stack types, call the unpack reference runtime fn,
                        // which has the instructions to allocate a middle pointer to the unpacked value
                        let unpack_reference_function = RuntimeFunction::UnpackReference
                            .get_generic(module, compilation_ctx, runtime_error_data, &[inner])?;

                        builder
                            .local_get(reader_pointer)
                            .local_get(calldata_base_pointer);

                        call_unpack_or_runtime_function(
                            builder,
                            compilation_ctx,
                            unpack_reference_function,
                            &RuntimeFunction::UnpackReference,
                            return_block_id,
                        );
                    }
                }
            }

            IntermediateType::IStruct {
                module_id, index, ..
            } if TxContext::is_vm_type(module_id, *index, compilation_ctx)? => {
                TxContext::inject(builder, module, compilation_ctx)?;
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } if String_::is_vm_type(module_id, *index, compilation_ctx)? => {
                let unpack_string_function = RuntimeFunction::UnpackString.get(
                    module,
                    Some(compilation_ctx),
                    runtime_error_data,
                )?;
                builder
                    .local_get(reader_pointer)
                    .local_get(calldata_base_pointer)
                    .call(unpack_string_function);
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } if Bytes::is_vm_type(module_id, *index, compilation_ctx)? => {
                let unpack_bytes_function =
                    RuntimeFunction::UnpackBytes.get(module, Some(compilation_ctx), None)?;
                builder
                    .local_get(reader_pointer)
                    .call(unpack_bytes_function);
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let struct_ = compilation_ctx.get_struct_by_intermediate_type(self)?;

                if struct_.has_key {
                    load_struct_storage_id(
                        builder,
                        module,
                        reader_pointer,
                        compilation_ctx,
                        &struct_,
                    )?;

                    // If the inner type is a storage struct, we need to pass the flag unpack_frozen.
                    // If the parent type is an immutable reference, we need to unpack frozen objects, so we push a 1 to the stack. Else we push a 0 to the stack.
                    if parent_type.is_some_and(|p| matches!(p, IntermediateType::IRef(_))) {
                        builder.i32_const(1);
                    } else {
                        builder.i32_const(0);
                    }

                    let unpack_storage_struct_function = RuntimeFunction::UnpackStorageStruct
                        .get_generic(module, compilation_ctx, runtime_error_data, &[self])?;

                    // Unpack the storage struct
                    call_unpack_or_runtime_function(
                        builder,
                        compilation_ctx,
                        unpack_storage_struct_function,
                        &RuntimeFunction::UnpackStorageStruct,
                        return_block_id,
                    );
                } else {
                    let unpack_struct_function = RuntimeFunction::UnpackStruct.get_generic(
                        module,
                        compilation_ctx,
                        runtime_error_data,
                        &[self],
                    )?;

                    builder
                        .local_get(reader_pointer)
                        .local_get(calldata_base_pointer);

                    call_unpack_or_runtime_function(
                        builder,
                        compilation_ctx,
                        unpack_struct_function,
                        &RuntimeFunction::UnpackStruct,
                        return_block_id,
                    );
                }
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let unpack_enum_function = RuntimeFunction::UnpackEnum.get_generic(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &[self],
                )?;
                builder.local_get(reader_pointer).call(unpack_enum_function);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(AbiError::Unpack(
                    AbiOperationError::UnpackingGenericTypeParameter,
                ));
            }
        }

        // Update the local reader pointer value to the global reader pointer, which is modified when unpacking
        builder
            .global_get(compilation_ctx.globals.calldata_reader_pointer)
            .local_set(reader_pointer);
        Ok(())
    }
}

/// This function leaves in the stack a pointer containing the ID for the storage struct about to be
/// unpacked.
///
/// If the first field is a `UID`, we just unpack a IAddress.
/// If the first field is a `NamedId` we compute the named id.
/// If none of the above is the first field, we found a compiler error and abort
fn load_struct_storage_id(
    function_builder: &mut InstrSeqBuilder,
    module: &mut Module,
    reader_pointer: LocalId,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> Result<(), AbiError> {
    match struct_.fields.first() {
        Some(IntermediateType::IStruct {
            module_id, index, ..
        }) if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
            // First we add the instructions to unpack the UID. We use address to unpack it because ids are
            // 32 bytes static, same as an address
            let unpack_address_function =
                RuntimeFunction::UnpackAddress.get(module, Some(compilation_ctx), None)?;
            function_builder
                .local_get(reader_pointer)
                .call(unpack_address_function);
        }
        Some(IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            ..
        }) if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
            // We use the native function that computes the ID to leave it in the stack so it can
            // be used by `add_unpack_from_storage_instructions`
            let compute_named_id_fn = NativeFunction::get_generic(
                NativeFunction::NATIVE_COMPUTE_NAMED_ID,
                module,
                compilation_ctx,
                None,
                &ModuleId::new(STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT),
                types,
            )
            .map_err(AbiError::NativeFunction)?;

            function_builder.call(compute_named_id_fn);
        }
        _ => {
            Err(AbiError::Unpack(AbiOperationError::StorageObjectHasNoId(
                struct_.identifier,
            )))?;
        }
    }
    Ok(())
}

/// Helper function to conditionally call either `call_unpack_function` or `call_runtime_function`
/// based on whether a return block ID is provided.
fn call_unpack_or_runtime_function(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    function_id: walrus::FunctionId,
    runtime_fn: &RuntimeFunction,
    return_block_id: Option<InstrSeqId>,
) {
    if let Some(return_block_id) = return_block_id {
        builder.call_unpack_function(compilation_ctx, function_id, runtime_fn, return_block_id);
    } else {
        builder.call_runtime_function(compilation_ctx, function_id, runtime_fn);
    }
}
#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::{Engine, Linker};

    use crate::{
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        utils::display_module,
    };

    use super::*;

    fn validator(param: u32, param2: u32, param3: u64) {
        println!("validator: {param}, {param2}, {param3}");

        assert_eq!(param, 1);
        assert_eq!(param2, 1234);
        assert_eq!(param3, 123456789012345);
    }

    #[test]
    fn test_build_unpack_instructions() {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        let return_block_id = func_body.id();
        // Args data should already be stored in memory
        let mut runtime_error_data = RuntimeErrorData::new();
        build_unpack_instructions(
            &mut func_body,
            return_block_id,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            &compilation_ctx,
            &mut runtime_error_data,
        )
        .unwrap();

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        println!("data: {data:?}");
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker.func_wrap("", "validator", validator).unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            data,
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_reversed() {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I64, ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        let return_block_id = func_body.id();
        // Args data should already be stored in memory
        let mut runtime_error_data = RuntimeErrorData::new();
        build_unpack_instructions(
            &mut func_body,
            return_block_id,
            &mut raw_module,
            &[
                IntermediateType::IU64,
                IntermediateType::IU16,
                IntermediateType::IBool,
            ],
            args_pointer,
            &compilation_ctx,
            &mut runtime_error_data,
        )
        .unwrap();

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((uint64, uint16, bool))>::abi_encode_params(&(123456789012345, 1234, true));
        println!("data: {data:?}");
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker
            .func_wrap("", "validator", |param: u64, param2: u32, param3: u32| {
                println!("validator: {param}, {param2}, {param3}");

                assert_eq!(param3, 1);
                assert_eq!(param2, 1234);
                assert_eq!(param, 123456789012345);
            })
            .unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            data,
            "test_function",
            Some(linker),
        );
        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_unpack_instructions_offset_memory() {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        let return_block_id = func_body.id();
        // Args data should already be stored in memory
        let mut runtime_error_data = RuntimeErrorData::new();
        build_unpack_instructions(
            &mut func_body,
            return_block_id,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            &compilation_ctx,
            &mut runtime_error_data,
        )
        .unwrap();

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        // Offset data by 10 bytes
        data = [vec![0; 10], data].concat();
        println!("data: {data:?}");
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker.func_wrap("", "validator", validator).unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            data,
            "test_function",
            Some(linker),
        );
        entrypoint.call(&mut store, (10, data_len - 10)).unwrap();
    }
}
