use move_parse_special_attributes::function_modifiers::FunctionModifier;
use walrus::{
    FunctionBuilder, FunctionId, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::{
        function_encoding::move_signature_to_abi_selector, packing::Packable, unpacking::Unpackable,
    },
    compilation_context::ModuleId,
    data::DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
    hostio::host_functions::{
        call_contract, delegate_call_contract, read_return_data, static_call_contract,
    },
    runtime::RuntimeFunction,
    translation::{
        functions::MappedFunction,
        intermediate_types::{IntermediateType, VmHandledStruct},
    },
    vm_handled_types::{
        VmHandledType,
        contract_call_result::{ContractCallEmptyResult, ContractCallResult},
        uid::Uid,
    },
};

/// Adds a function to perform an external contract call.
///
/// The functions are built using the signature contained in `function_information` and the
/// modifiers declared by the user using the `#[ext(external_call, ..)]` attribute.
///
/// Depending if the declared function is payable or not, the generated function will expect
/// the `value` argument as the first argument. Additionally, if the function is declared with the
/// `gas` argument, it will be passed to the `call_contract` functions, otherwise, the maximum gas
/// (u64::MAX) will be used.
pub fn add_external_contract_call_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
    function_information: &MappedFunction,
    function_modifiers: &[FunctionModifier],
    arguments_types: &[IntermediateType],
) -> FunctionId {
    let name = format!(
        "{}_{}_{}",
        module_id.hash(),
        module_id.module_name,
        function_information.function_id.identifier
    );

    if let Some(function_id) = module.funcs.by_name(&name) {
        return function_id;
    }

    let (read_return_data, _) = read_return_data(module);
    let (call_contract, _) = call_contract(module);
    let (delegate_call_contract, _) = delegate_call_contract(module);
    let (static_call_contract, _) = static_call_contract(module);
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);
    let swap = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

    let arguments = function_information.signature.get_argument_wasm_types();

    let mut function = FunctionBuilder::new(&mut module.types, &arguments, &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let function_args: Vec<LocalId> = arguments.iter().map(|a| module.locals.add(*a)).collect();
    let self_ = function_args
        .first()
        .unwrap_or_else(|| panic!("contract call function has no arguments"));

    // Locals
    let address_ptr = module.locals.add(ValType::I32);
    let is_delegate_call = module.locals.add(ValType::I32);
    let gas = module.locals.add(ValType::I64);
    let value = module.locals.add(ValType::I32);

    builder
        .local_get(*self_)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(*self_);

    // The address to call is the first argument of self (20 bytes)
    builder
        .local_get(*self_)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .local_set(address_ptr);

    builder
        .local_get(*self_)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        )
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(is_delegate_call);

    // Load gas
    builder
        .local_get(*self_)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 8,
            },
        )
        .load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(gas);

    // Load value (if the function is payable) otherwise we load zeroes
    if function_modifiers.contains(&FunctionModifier::Payable) {
        builder
            .local_get(*self_)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 12,
                },
            )
            .local_tee(value)
            .local_get(value)
            .call(swap);
    } else {
        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_set(value);
    }

    let calldata_arguments = &function_information.signature.arguments[1..];

    let calldata_start = module.locals.add(ValType::I32);

    //  Create the function selector
    let selector = move_signature_to_abi_selector(
        &function_information.function_id.identifier,
        calldata_arguments,
        compilation_ctx,
    );

    // Save the function selector before the arguments
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(calldata_start);

    builder
        .i32_const(i32::from_be_bytes(selector))
        .call(swap_i32)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    let calldata_len = module.locals.add(ValType::I32);

    // Pack the arguments in the calldata
    if !calldata_arguments.is_empty() {
        let writer_pointer = module.locals.add(ValType::I32);
        let calldata_reference_pointer = module.locals.add(ValType::I32);

        let mut args_size = 0;
        for signature_token in calldata_arguments {
            // If the function returns multiple values, those values will be encoded as a tuple. By
            // definition, a tuple T is dynamic (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k.
            // The encode size for a dynamically encoded field inside a dynamically encoded tuple is
            // just 32 bytes (the value is the offset to where the values are packed)
            args_size += if signature_token.is_dynamic(compilation_ctx) {
                32
            } else {
                signature_token.encoded_size(compilation_ctx)
            };
        }

        builder
            .i32_const(args_size as i32)
            .call(compilation_ctx.allocator)
            .local_tee(writer_pointer)
            .local_set(calldata_reference_pointer);

        for (argument, wasm_local) in calldata_arguments.iter().zip(&function_args[1..]) {
            let argument = match argument {
                IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => &**inner,
                _ => argument,
            };

            if argument.is_dynamic(compilation_ctx) {
                argument.add_pack_instructions_dynamic(
                    &mut builder,
                    module,
                    *wasm_local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );

                builder
                    .local_get(writer_pointer)
                    .i32_const(32)
                    .binop(BinaryOp::I32Add)
                    .local_set(writer_pointer);
            } else {
                argument.add_pack_instructions(
                    &mut builder,
                    module,
                    *wasm_local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );

                builder
                    .local_get(writer_pointer)
                    .i32_const(argument.encoded_size(compilation_ctx) as i32)
                    .binop(BinaryOp::I32Add)
                    .local_set(writer_pointer);
            }
        }
    }

    // Get the calldata length
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(calldata_start)
        .binop(BinaryOp::I32Sub)
        .local_set(calldata_len);

    let call_contract_result = module.locals.add(ValType::I32);
    let return_data_len = module.locals.add(ValType::I32);
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_set(return_data_len);

    // If the function is pure or view, we use static_call_contract since no state modification is
    // allowed
    if function_modifiers.contains(&FunctionModifier::Pure)
        || function_modifiers.contains(&FunctionModifier::View)
    {
        builder
            .local_get(address_ptr)
            .local_get(calldata_start)
            .local_get(calldata_len)
            .local_get(gas)
            .local_get(return_data_len)
            .call(static_call_contract)
            .local_set(call_contract_result);
    } else {
        builder.local_get(is_delegate_call).if_else(
            None,
            |then_| {
                // Delegate call
                then_
                    .local_get(address_ptr)
                    .local_get(calldata_start)
                    .local_get(calldata_len)
                    .local_get(gas)
                    .local_get(return_data_len)
                    .call(delegate_call_contract)
                    .local_set(call_contract_result);
            },
            |else_| {
                // Regular call
                else_
                    .local_get(address_ptr)
                    .local_get(calldata_start)
                    .local_get(calldata_len)
                    .local_get(value)
                    .local_get(gas)
                    .local_get(return_data_len)
                    .call(call_contract)
                    .local_set(call_contract_result);
            },
        );
    }

    let call_result = module.locals.add(ValType::I32);
    let call_result_code_ptr = module.locals.add(ValType::I32);

    if function_information.signature.returns.len() > 1 {
        panic!(
            "external contract call function {} must return a ContractCallResult<T> or ContractCallEmptyResult with a single type parameter",
            function_information.function_id
        );
    }

    // Depending on the return type, we allocate the proper size for the call result (4 for empty,
    // 8 for result)
    match function_information.signature.returns.first() {
        None => panic!(
            "external contract call function {} must return a ContractCallResult<T> or ContractCallEmptyResult",
            function_information.function_id
        ),
        Some(IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }) if ContractCallResult::is_vm_type(module_id, *index, compilation_ctx) => {
            builder
                .i32_const(8)
                .call(compilation_ctx.allocator)
                .local_set(call_result);
        }
        Some(IntermediateType::IStruct {
            module_id, index, ..
        }) if ContractCallEmptyResult::is_vm_type(module_id, *index, compilation_ctx) => {
            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_set(call_result);
        }
        _ => panic!(
            "external contract call function {} must return a ContractCallResult<T> or ContractCallEmptyResult",
            function_information.function_id
        ),
    }

    // Save the result in the first field of CallResult<>
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(call_result_code_ptr)
        .local_get(call_contract_result)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                offset: 0,
                align: 0,
            },
        );

    builder
        .local_get(call_result)
        .local_get(call_result_code_ptr)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                offset: 0,
                align: 0,
            },
        );

    // If the call succeded (returned 0), we proceed to decode the result
    if matches!(
        function_information.signature.returns.first(),
        Some(IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }) if ContractCallResult::is_vm_type(module_id, *index, compilation_ctx)
    ) {
        builder.block(None, |block| {
            let block_id = block.id();

            // Exit the block if the status != 0
            block
                .local_get(call_contract_result)
                .i32_const(0)
                .binop(BinaryOp::I32Ne)
                .br_if(block_id);

            let return_data_abi_encoded_ptr = module.locals.add(ValType::I32);

            block
                .local_get(return_data_len)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(return_data_len);

            block
                .local_get(return_data_len)
                .call(compilation_ctx.allocator)
                .local_tee(return_data_abi_encoded_ptr);

            block
                .i32_const(0)
                .local_get(return_data_len)
                .call(read_return_data)
                .local_set(return_data_len);

            assert_eq!(
                1,
                function_information.signature.returns.len(),
                "invalid contract call function, it can only return one value"
            );

            if let IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } = &function_information.signature.returns[0]
            {
                if ContractCallResult::is_vm_type(module_id, *index, compilation_ctx) {
                    let calldata_reader_pointer = module.locals.add(ValType::I32);

                    block
                        .local_get(return_data_abi_encoded_ptr)
                        .local_set(calldata_reader_pointer);

                    let result_type = &types[0];

                    // Unpack the value
                    result_type.add_unpack_instructions(
                        block,
                        module,
                        return_data_abi_encoded_ptr,
                        calldata_reader_pointer,
                        compilation_ctx,
                    );

                    let abi_decoded_call_result = if result_type == &IntermediateType::IU64 {
                        module.locals.add(ValType::I64)
                    } else {
                        module.locals.add(ValType::I32)
                    };

                    block.local_set(abi_decoded_call_result);

                    // If the return type is a stack type, we need to create the intermediate pointer
                    // for the struct field, otherwise it is already a pointer, we write it directly
                    let data_ptr = if result_type.is_stack_type() {
                        let call_result_value_ptr = module.locals.add(ValType::I32);
                        let (store_kind, store_len) = if result_type == &IntermediateType::IU64 {
                            (StoreKind::I64 { atomic: false }, 8)
                        } else {
                            (StoreKind::I32 { atomic: false }, 4)
                        };
                        block
                            .i32_const(store_len)
                            .call(compilation_ctx.allocator)
                            .local_tee(call_result_value_ptr)
                            .local_get(abi_decoded_call_result)
                            .store(
                                compilation_ctx.memory_id,
                                store_kind,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            );
                        call_result_value_ptr
                    } else {
                        abi_decoded_call_result
                    };

                    block.local_get(call_result).local_get(data_ptr).store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 4,
                        },
                    );
                } else {
                    panic!(
                        "invalid ContractCallResult type found in function {}",
                        function_information.function_id
                    );
                }
            }
        });
    }

    // Before returning, if the call is a delegated call, we must load from storage the potentially
    // modified storage objects
    // We ignore the first argument since it is the #[external_call] object
    let mut storage_objects = arguments_types
        .iter()
        .enumerate()
        .filter_map(|(i, itype)| {
            let itype = if let IntermediateType::IMutRef(inner) = itype {
                inner
            } else if let IntermediateType::IRef(inner) = itype {
                inner
            } else {
                itype
            };

            match itype {
                IntermediateType::IStruct {
                    module_id,
                    index,
                    vm_handled_struct:
                        VmHandledStruct::StorageId {
                            parent_module_id,
                            parent_index,
                            instance_types,
                        },
                } if Uid::is_vm_type(module_id, *index, compilation_ctx) => {
                    let (parent_struct_itype, parent_struct) =
                        if let Some(instance_types) = instance_types {
                            let itype = IntermediateType::IGenericStructInstance {
                                module_id: parent_module_id.clone(),
                                index: *parent_index,
                                types: instance_types.clone(),
                                vm_handled_struct: VmHandledStruct::None,
                            };

                            let struct_ = compilation_ctx
                                .get_struct_by_index(parent_module_id, *parent_index)
                                .unwrap();
                            struct_.instantiate(instance_types);

                            (itype, struct_)
                        } else {
                            let itype = IntermediateType::IStruct {
                                module_id: parent_module_id.clone(),
                                index: *parent_index,
                                vm_handled_struct: VmHandledStruct::None,
                            };
                            let struct_ = compilation_ctx
                                .get_struct_by_index(parent_module_id, *parent_index)
                                .unwrap();

                            (itype, struct_)
                        };

                    Some((function_args[i], parent_struct_itype, parent_struct))
                }
                _ => None,
            }
        })
        .peekable();

    if storage_objects.peek().is_some() {
        let locate_storage_data_fn =
            RuntimeFunction::LocateStorageData.get(module, Some(compilation_ctx));

        builder.block(None, |block| {
            let block_id = block.id();

            let uid_ptr = module.locals.add(ValType::I32);
            let original_struct_ptr = module.locals.add(ValType::I32);
            let new_struct_ptr = module.locals.add(ValType::I32);

            // Exit the block if it is not a delegate call
            block
                .local_get(is_delegate_call)
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            for (storage_obj_uid, storage_obj_itype, storage_obj) in storage_objects {
                // Read the struct pointer located right before the UID field
                block
                    .local_get(storage_obj_uid)
                    .i32_const(4)
                    .binop(BinaryOp::I32Sub)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(original_struct_ptr);

                // Read the UID value and locate the data in storage
                block
                    .local_get(storage_obj_uid)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(uid_ptr)
                    .i32_const(0)
                    .call(locate_storage_data_fn);

                // The slot for this struct written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET
                let read_and_decode_from_storage_fn = RuntimeFunction::ReadAndDecodeFromStorage
                    .get_generic(module, compilation_ctx, &[&storage_obj_itype]);

                block
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .local_get(uid_ptr)
                    .call(read_and_decode_from_storage_fn)
                    .local_set(new_struct_ptr);

                // Once we return from the reading the allegedly modified object from storage, we
                // replace the old pointer representation with the new one returned by the
                // function.
                // If it happens that the owner changed in the delegated call, the object will NOT
                // be located by locate_storage_data_fn and it will throw an unrechable.
                // That's ok, if the delegate call changed the owner, we can't continue handling
                // the object here
                //
                // By overwriting the struct, we update the data that could have change in the
                // call. For example, if we have the struct
                //
                // publict structr Foo {
                //      id: UID,
                //      value: u64,
                // }
                //
                // The underlying representation in memory will be:
                //
                // 0xX: [ptr_uid, ptr_value]
                //
                // located at address 0xX
                //
                // After the read_and_decode_from_storage_fn execution, we will have in a new
                // memory location 0xY another representation of the struct with the possibly
                // updated balues by the delegate call:
                //
                // 0xY: [ptr_uid_updated, ptr_value_updated]
                //
                // located at address 0xY
                //
                // (NOTE: The uid is not really updated, is just to reflect that those are new
                // pointers)
                //
                // Since the struct located at 0xY contains the updated values by the delegated
                // call but in our current execution, all the references of Foo are pointing to
                // 0xX, we replace all the pointers in 0xX for the pointers of 0xY, since the ones
                // in 0xY are pointing to the updated values. So at the end of the delegate call,
                // the Foo located at 0xX will have the following representation in memory;
                //
                // 0xX: [ptr_uid_updated, ptr_value_updated]
                block
                    .local_get(original_struct_ptr)
                    .local_get(new_struct_ptr)
                    .i32_const(storage_obj.heap_size as i32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
        });
    }

    // After the call we read the data
    builder.local_get(call_result);

    function.finish(function_args, &mut module.funcs)
}
