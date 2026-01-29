// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module contains all the functions retaled to transaction information.
use super::NativeFunction;
use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    hostio::host_functions::{account_balance, account_code_size},
    native_functions::error::NativeFunctionError,
    runtime::RuntimeFunction,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn add_native_account_code_size_fn(
    module: &mut Module,
    _compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let (account_code_size_function_id, _) = account_code_size(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_ACCOUNT_CODE_SIZE,
            module_id,
        ))
        .func_body();

    let address_ptr = module.locals.add(ValType::I32);

    // Skip the first 12 empty bytes
    builder
        .local_get(address_ptr)
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .call(account_code_size_function_id);

    function.finish(vec![address_ptr], &mut module.funcs)
}

pub fn add_native_account_balance_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let (account_balance_function_id, _) = account_balance(module);
    let swap_i256_bytes_function =
        RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx), None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_ACCOUNT_BALANCE,
            module_id,
        ))
        .func_body();

    let address_ptr = module.locals.add(ValType::I32);

    let balance_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(balance_ptr);

    builder
        .local_get(address_ptr)
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .local_get(balance_ptr)
        .call(account_balance_function_id);

    builder
        .local_get(balance_ptr)
        .local_get(balance_ptr)
        .call(swap_i256_bytes_function)
        .local_get(balance_ptr);

    Ok(function.finish(vec![address_ptr], &mut module.funcs))
}
