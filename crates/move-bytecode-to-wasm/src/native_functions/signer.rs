// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module contains the native implementation for `std::signer::address_of()`.
//!
//! This function returns the address of the signer.
use super::NativeFunction;
use crate::{
    CompilationContext, compilation_context::ModuleId, native_functions::error::NativeFunctionError,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

// This function simply echoes the signer pointer.
pub fn add_signer_address_of_fn(
    module: &mut Module,
    _compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_SIGNER_ADDRESS_OF,
            module_id,
        ))
        .func_body();

    let signer_ptr = module.locals.add(ValType::I32);

    builder.local_get(signer_ptr);

    Ok(function.finish(vec![signer_ptr], &mut module.funcs))
}
