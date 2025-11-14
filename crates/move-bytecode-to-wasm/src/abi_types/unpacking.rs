use error::AbiUnpackError;
use walrus::{InstrSeqBuilder, LocalId, Module, ValType};

use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_OBJECT, STYLUS_FRAMEWORK_ADDRESS},
    },
    data::DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
    native_functions::NativeFunction,
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        boolean::IBool,
        enums::IEnum,
        heap_integers::{IU128, IU256},
        reference::{IMutRef, IRef},
        simple_integers::{IU8, IU16, IU32, IU64},
        structs::IStruct,
        vector::IVector,
    },
    vm_handled_types::{
        VmHandledType, named_id::NamedId, string::String_, tx_context::TxContext, uid::Uid,
    },
};

pub(crate) mod error;
mod unpack_enum;
mod unpack_heap_int;
mod unpack_native_int;
mod unpack_reference;
mod unpack_string;
mod unpack_struct;
mod unpack_vector;

pub trait Unpackable {
    /// Adds the instructions to unpack the abi encoded type to WASM function parameters
    ///
    /// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
    /// and the pointer is pushed onto the stack in the parameter location.
    ///
    /// The reader pointer should be updated internally when a value is read from the args
    /// The calldata reader pointer should never be updated, it is considered static for each type value
    ///
    /// The stack at the end contains the value(or pointer to the value) as **i32/i64**
    fn add_unpack_instructions(
        &self,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiUnpackError>;
}

/// Builds the instructions to unpack the abi encoded values to WASM function parameters
///
/// Each parameter is decoded and loaded in the WASM stack. Complex data types are kept in memory
/// and the pointer is pushed onto the stack in the parameter location.
pub fn build_unpack_instructions<T: Unpackable>(
    function_builder: &mut InstrSeqBuilder,
    module: &mut Module,
    function_arguments_signature: &[T],
    args_pointer: LocalId,
    compilation_ctx: &CompilationContext,
) -> Result<(), AbiUnpackError> {
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    function_builder.local_get(args_pointer);
    function_builder.local_tee(reader_pointer);
    function_builder.local_set(calldata_reader_pointer);

    // The ABI encoded params are always a tuple
    // Static types are stored in-place, but dynamic types are referenced to the call data
    for signature_token in function_arguments_signature.iter() {
        signature_token.add_unpack_instructions(
            function_builder,
            module,
            reader_pointer,
            calldata_reader_pointer,
            compilation_ctx,
        )?;
    }

    Ok(())
}

impl Unpackable for IntermediateType {
    fn add_unpack_instructions(
        &self,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiUnpackError> {
        match self {
            IntermediateType::IBool => IBool::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU8 => IU8::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU16 => IU16::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU32 => IU32::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU64 => IU64::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU128 => IU128::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IU256 => IU256::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IAddress => IAddress::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IVector(inner) => IVector::add_unpack_instructions(
                inner,
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?,
            // The signer must not be unpacked here, since it can't be part of the calldata. It is
            // injected directly by the VM into the stack
            IntermediateType::ISigner => (),
            IntermediateType::IRef(inner) => IRef::add_unpack_instructions(
                inner,
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?,
            IntermediateType::IMutRef(inner) => IMutRef::add_unpack_instructions(
                inner,
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?,

            IntermediateType::IStruct {
                module_id, index, ..
            } if TxContext::is_vm_type(module_id, *index, compilation_ctx) => {
                TxContext::inject(function_builder, module, compilation_ctx);
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } if String_::is_vm_type(module_id, *index, compilation_ctx) => {
                String_::add_unpack_instructions(
                    function_builder,
                    module,
                    reader_pointer,
                    calldata_reader_pointer,
                    compilation_ctx,
                );
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let struct_ = compilation_ctx
                    .get_struct_by_intermediate_type(self)
                    .unwrap();

                if struct_.has_key {
                    load_struct_storage_id(
                        function_builder,
                        module,
                        reader_pointer,
                        calldata_reader_pointer,
                        compilation_ctx,
                        &struct_,
                    )?;

                    add_unpack_from_storage_instructions(
                        function_builder,
                        module,
                        compilation_ctx,
                        self,
                        false,
                    );
                } else {
                    // TODO: Check if the struct is TxContext. If it is, panic since the only valid
                    // TxContext is the one defined in the stylus framework.

                    struct_.add_unpack_instructions(
                        function_builder,
                        module,
                        reader_pointer,
                        calldata_reader_pointer,
                        compilation_ctx,
                    )?;
                }
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self).unwrap();
                if !enum_.is_simple {
                    return Err(AbiUnpackError::EnumIsNotSimple(enum_.identifier.to_owned()));
                }

                IEnum::add_unpack_instructions(
                    &enum_,
                    function_builder,
                    module,
                    reader_pointer,
                    compilation_ctx,
                )
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(AbiUnpackError::UnpackingGenericTypeParameter);
            }
        }
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
    calldata_reader_pointer: LocalId,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> Result<(), AbiUnpackError> {
    match struct_.fields.first() {
        Some(IntermediateType::IStruct {
            module_id, index, ..
        }) if Uid::is_vm_type(module_id, *index, compilation_ctx) => {
            // First we add the instructions to unpack the UID. We use address to unpack it because ids are
            // 32 bytes static, same as an address
            IAddress::add_unpack_instructions(
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );
        }
        Some(IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            ..
        }) if NamedId::is_vm_type(module_id, *index, compilation_ctx) => {
            // We use the native function that computes the ID to leave it in the stack so it can
            // be used by `add_unpack_from_storage_instructions`
            let compute_named_id_fn = NativeFunction::get_generic(
                NativeFunction::NATIVE_COMPUTE_NAMED_ID,
                module,
                compilation_ctx,
                &ModuleId {
                    address: STYLUS_FRAMEWORK_ADDRESS,
                    module_name: SF_MODULE_NAME_OBJECT.to_owned(),
                },
                types,
            )?;

            function_builder.call(compute_named_id_fn);
        }
        _ => {
            return Err(AbiUnpackError::StorageObjectHasNoId(
                struct_.identifier.clone(),
            ));
        }
    }
    Ok(())
}

/// This function searches in the storage for the structure that belongs to the object UID passed
/// as argument.
fn add_unpack_from_storage_instructions(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    unpack_frozen: bool,
) {
    // Search for the object in the objects mappings
    let locate_storage_data_fn =
        RuntimeFunction::LocateStorageData.get(module, Some(compilation_ctx));

    let uid_ptr = module.locals.add(ValType::I32);
    builder.local_tee(uid_ptr);

    if unpack_frozen {
        builder.i32_const(1);
    } else {
        builder.i32_const(0);
    }

    builder.call(locate_storage_data_fn);

    // Read the object
    let read_and_decode_from_storage_fn =
        RuntimeFunction::ReadAndDecodeFromStorage.get_generic(module, compilation_ctx, &[itype]);

    // Copy the slot number into a local to avoid overwriting it later
    let slot_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .call(read_and_decode_from_storage_fn);
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
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            &compilation_ctx,
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
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I64, ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IU64,
                IntermediateType::IU16,
                IntermediateType::IBool,
            ],
            args_pointer,
            &compilation_ctx,
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
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module
            .types
            .add(&[ValType::I32, ValType::I32, ValType::I64], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        // Args data should already be stored in memory
        build_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            args_pointer,
            &compilation_ctx,
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
