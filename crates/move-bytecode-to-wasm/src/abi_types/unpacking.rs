use walrus::{InstrSeqBuilder, LocalId, Module, ValType};

use crate::{
    CompilationContext,
    compilation_context::ExternalModuleData,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        boolean::IBool,
        enums::IEnum,
        heap_integers::{IU128, IU256},
        reference::{IMutRef, IRef},
        simple_integers::{IU8, IU16, IU32, IU64},
        vector::IVector,
    },
};

use super::vm_handled_datatypes::TxContext;

mod unpack_enum;
mod unpack_heap_int;
mod unpack_native_int;
mod unpack_reference;
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
    );
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
) {
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
        );
    }
}

impl Unpackable for IntermediateType {
    fn add_unpack_instructions(
        &self,
        function_builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
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
            ),
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
            ),
            IntermediateType::IMutRef(inner) => IMutRef::add_unpack_instructions(
                inner,
                function_builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            ),
            IntermediateType::IStruct(index) => {
                let struct_ = compilation_ctx
                    .root_module_data
                    .structs
                    .get_by_index(*index)
                    .unwrap();

                // TODO: Check if the struct is TxContext. If it is, panic since the only valid
                // TxContext is the one defined in the stylus framework.

                struct_.add_unpack_instructions(
                    function_builder,
                    module,
                    reader_pointer,
                    calldata_reader_pointer,
                    compilation_ctx,
                );
            }
            IntermediateType::IGenericStructInstance(index, types) => {
                let struct_ = compilation_ctx
                    .root_module_data
                    .structs
                    .get_by_index(*index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);
                struct_instance.add_unpack_instructions(
                    function_builder,
                    module,
                    reader_pointer,
                    calldata_reader_pointer,
                    compilation_ctx,
                )
            }
            IntermediateType::IEnum(enum_index) => {
                let enum_ = compilation_ctx
                    .root_module_data
                    .enums
                    .get_enum_by_index(*enum_index)
                    .unwrap();
                if !enum_.is_simple {
                    panic!(
                        "cannot abi unpack enum with index {enum_index}, it contains at least one variant with fields"
                    );
                }
                IEnum::add_unpack_instructions(
                    enum_,
                    function_builder,
                    module,
                    reader_pointer,
                    compilation_ctx,
                )
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
            } => {
                let external_data = compilation_ctx
                    .get_external_module_data(module_id, identifier)
                    .unwrap();

                match external_data {
                    ExternalModuleData::Struct(istruct) => {
                        if TxContext::struct_is_tx_context(module_id, identifier) {
                            TxContext::inject_tx_context(
                                function_builder,
                                compilation_ctx.allocator,
                            );
                        } else {
                            istruct.add_unpack_instructions(
                                function_builder,
                                module,
                                reader_pointer,
                                calldata_reader_pointer,
                                compilation_ctx,
                            )
                        }
                    }
                    ExternalModuleData::Enum(ienum) => {
                        if !ienum.is_simple {
                            panic!(
                                "cannot abi external module's enum {identifier}, it contains at least one variant with fields"
                            );
                        }
                        IEnum::add_unpack_instructions(
                            ienum,
                            function_builder,
                            module,
                            reader_pointer,
                            compilation_ctx,
                        )
                    }
                }
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot unpack generic type parameter");
            }
        }
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
        println!("validator: {}, {}, {}", param, param2, param3);

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
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
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
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((uint64, uint16, bool))>::abi_encode_params(&(123456789012345, 1234, true));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker
            .func_wrap("", "validator", |param: u64, param2: u32, param3: u32| {
                println!("validator: {}, {}, {}", param, param2, param3);

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
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let mut data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        // Offset data by 10 bytes
        data = [vec![0; 10], data].concat();
        println!("data: {:?}", data);
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
