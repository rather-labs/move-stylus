//! This module contains all the functions retaled to transaction information.
use super::NativeFunction;
use crate::runtime::RuntimeFunction;
use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    data::DATA_CALLDATA_OFFSET,
    hostio::host_functions::{block_basefee, msg_sender, msg_value, tx_gas_price},
    native_functions::error::NativeFunctionError,
    translation::intermediate_types::{address::IAddress, heap_integers::IU256},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

/// Defines native functions that are wrappers for host functions.
macro_rules! define_host_fn_native_fn_wrapper {
    ($name: ident, $host_fn: ident, $native_fn_name: expr, $alloc_size: expr) => {
        pub fn $name(
            module: &mut walrus::Module,
            compilation_ctx: &$crate::CompilationContext,
            module_id: &$crate::compilation_context::ModuleId,
        ) -> walrus::FunctionId {
            let (host_function_id, _) = $host_fn(module);

            let mut function =
                walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);

            let ptr = module.locals.add(walrus::ValType::I32);

            let name = $crate::native_functions::NativeFunction::get_function_name(
                $native_fn_name,
                module_id,
            );
            let mut builder = function.name(name).func_body();

            builder
                .i32_const($alloc_size)
                .call(compilation_ctx.allocator)
                .local_tee(ptr)
                .call(host_function_id)
                .local_get(ptr);

            function.finish(vec![], &mut module.funcs)
        }
    };
}

pub fn add_native_sender_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let (msg_sender_function_id, _) = msg_sender(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let address_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_SENDER,
            module_id,
        ))
        .func_body();

    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_tee(address_ptr)
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .call(msg_sender_function_id)
        .local_get(address_ptr);

    function.finish(vec![], &mut module.funcs)
}

define_host_fn_native_fn_wrapper!(
    add_native_msg_value_fn,
    msg_value,
    NativeFunction::NATIVE_MSG_VALUE,
    IU256::HEAP_SIZE
);

define_host_fn_native_fn_wrapper!(
    add_native_block_basefee_fn,
    block_basefee,
    NativeFunction::NATIVE_BLOCK_BASEFEE,
    IU256::HEAP_SIZE
);

define_host_fn_native_fn_wrapper!(
    add_native_tx_gas_price_fn,
    tx_gas_price,
    NativeFunction::NATIVE_GAS_PRICE,
    IU256::HEAP_SIZE
);

pub fn add_native_data_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let bytes_to_vec_fn = RuntimeFunction::BytesToVec
        .get(module, Some(compilation_ctx), None)
        .map_err(NativeFunctionError::from)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_DATA,
            module_id,
        ))
        .func_body();

    // Cast the calldata to vector<u8> and return the ptr
    builder
        .i32_const(DATA_CALLDATA_OFFSET)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        )
        .i32_const(DATA_CALLDATA_OFFSET)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .call(bytes_to_vec_fn);

    Ok(function.finish(vec![], &mut module.funcs))
}
