use move_parse_special_attributes::function_modifiers::FunctionModifier;
use walrus::{
    FunctionBuilder, FunctionId, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::{packing::Packable, unpacking::Unpackable},
    compilation_context::ModuleId,
    hostio::host_functions::{call_contract, read_return_data},
    translation::{functions::MappedFunction, intermediate_types::IntermediateType},
    vm_handled_types::{VmHandledType, contract_call_result::ContractCallResult},
};

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

    println!("Processing {name}");

    if let Some(function_id) = module.funcs.by_name(&name) {
        return function_id;
    }

    let (read_return_data, _) = read_return_data(module);
    let (call_contract, _) = call_contract(module);

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

    // The address to call is the first argument of self
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
        .local_set(address_ptr);

    // Calculate the calldata
    let arguments_signature: &Vec<(&IntermediateType, &LocalId)> = if gas_argument_present {
        if function_modifiers.contains(&FunctionModifier::Payable) {
            &function_information.signature.arguments[3..]
                .iter()
                .zip(&function_args[3..])
                .collect()
        } else {
            &function_information.signature.arguments[2..]
                .iter()
                .zip(&function_args[2..])
                .collect()
        }
    } else {
        &function_information.signature.arguments[1..]
            .iter()
            .zip(&function_args[1..])
            .collect()
    };

    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let calldata_len = module.locals.add(ValType::I32);
    let call_contract_result = module.locals.add(ValType::I32);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(writer_pointer)
        .local_set(calldata_reference_pointer);

    for (argument, wasm_local) in arguments_signature {
        argument.add_pack_instructions(
            &mut builder,
            module,
            **wasm_local,
            writer_pointer,
            calldata_reference_pointer,
            compilation_ctx,
        );
    }

    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_set(calldata_len);

    let return_data_len = module.locals.add(ValType::I32);
    builder
        .i32_const(8)
        .call(compilation_ctx.allocator)
        .local_set(return_data_len);

    builder
        .local_get(address_ptr)
        .local_get(calldata_reference_pointer)
        .local_get(calldata_len)
        .local_get(value)
        .local_get(gas)
        .local_get(return_data_len)
        .call(call_contract)
        .local_set(call_contract_result);

    let call_result = module.locals.add(ValType::I32);
    let call_result_code_ptr = module.locals.add(ValType::I32);
    let call_result_value_ptr = module.locals.add(ValType::I32);

    // Recreate the CallResult<T>

    builder
        .i32_const(8)
        .call(compilation_ctx.allocator)
        .local_set(call_result);

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

    // If the call succeded, we proceed to decode the result
    builder.block(None, |block| {
        let block_id = block.id();

        block
            .local_get(call_contract_result)
            .i32_const(0)
            .binop(BinaryOp::I32Ne)
            .br_if(block_id);

        let return_data_abi_encoded_ptr = module.locals.add(ValType::I32);

        block
            .local_get(return_data_len)
            .call(compilation_ctx.allocator)
            .local_tee(return_data_abi_encoded_ptr);

        block
            .i32_const(0)
            .local_get(return_data_len)
            .call(read_return_data)
            .drop();

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
                println!("----- {types:?}");
                let calldata_reader_pointer = module.locals.add(ValType::I32);

                block
                    .local_get(return_data_abi_encoded_ptr)
                    .local_set(calldata_reader_pointer);

                // Unpack the value
                types[0].add_unpack_instructions(
                    block,
                    module,
                    return_data_abi_encoded_ptr,
                    calldata_reader_pointer,
                    compilation_ctx,
                );

                let abi_decoded_call_result = module.locals.add(ValType::I32);
                block.local_set(abi_decoded_call_result);

                // Save the result in the first field of CallResult<>
                // TODO: Check what happens with stack types
                block
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(call_result_value_ptr)
                    .local_get(abi_decoded_call_result)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            offset: 0,
                            align: 0,
                        },
                    );

                block
                    .local_get(call_result)
                    .local_get(call_result_value_ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            offset: 0,
                            align: 0,
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

    // After the call we read the data
    builder.local_get(call_result);

    function.finish(function_args, &mut module.funcs)
}
