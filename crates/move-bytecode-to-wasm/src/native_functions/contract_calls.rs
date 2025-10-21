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
    hostio::host_functions::{
        call_contract, delegate_call_contract, emit_log, read_return_data, static_call_contract,
    },
    runtime::RuntimeFunction,
    translation::{functions::MappedFunction, intermediate_types::IntermediateType},
    vm_handled_types::{
        VmHandledType,
        contract_call_result::{ContractCallEmptyResult, ContractCallResult},
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
    gas_argument_present: bool,
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

    let (emit_log_function, _) = emit_log(module);
    let (read_return_data, _) = read_return_data(module);
    let (call_contract, _) = call_contract(module);
    let (delegate_call_contract, _) = delegate_call_contract(module);
    let (static_call_contract, _) = static_call_contract(module);
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);

    let arguments = function_information.signature.get_argument_wasm_types();

    let mut function = FunctionBuilder::new(&mut module.types, &arguments, &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let function_args: Vec<LocalId> = arguments.iter().map(|a| module.locals.add(*a)).collect();
    let self_ = function_args
        .first()
        .unwrap_or_else(|| panic!("contract call function has no arguments"));

    let value = if function_modifiers.contains(&FunctionModifier::Payable) {
        function_args.get(1)
    } else {
        None
    };

    // If value is not specified we allocate 32 bytes and pass that as the value (will be 0)
    let value = if let Some(value) = value {
        *value
    } else {
        let value = module.locals.add(ValType::I32);
        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_set(value);
        value
    };

    // If gas is not present, we set it to the max possible
    let gas = if gas_argument_present {
        if function_modifiers.contains(&FunctionModifier::Payable) {
            function_args.get(2)
        } else {
            function_args.get(1)
        }
    } else {
        None
    };

    let gas = if let Some(gas) = gas {
        *gas
    } else {
        let gas = module.locals.add(ValType::I64);
        builder.i64_const(u64::MAX as i64).local_set(gas);
        gas
    };

    // Locals
    let address_ptr = module.locals.add(ValType::I32);
    let is_delegate_call = module.locals.add(ValType::I32);

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

    /*
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
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_function);
    */

    // Calculate the from where the arguments enter the calldata. Depending on how the call is
    // configured we omit some parameters at the beggining that are not part of the callee
    // signature
    let arguments_from = if gas_argument_present {
        if function_modifiers.contains(&FunctionModifier::Payable) {
            3
        } else {
            2
        }
    } else {
        1
    };

    let calldata_arguments = &function_information.signature.arguments[arguments_from..];

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

        for (argument, wasm_local) in calldata_arguments
            .iter()
            .zip(&function_args[arguments_from..])
        {
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
                println!("Packing static argument {:?}", argument);
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

            // Return data len is in big endian, we read it and change endianess
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
                        block
                            .i32_const(4)
                            .call(compilation_ctx.allocator)
                            .local_tee(call_result_value_ptr)
                            .local_get(abi_decoded_call_result)
                            .store(
                                compilation_ctx.memory_id,
                                if result_type == &IntermediateType::IU64 {
                                    StoreKind::I64 { atomic: false }
                                } else {
                                    StoreKind::I32 { atomic: false }
                                },
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

    // After the call we read the data
    builder.local_get(call_result);

    function.finish(function_args, &mut module.funcs)
}
