// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module contains the native implementation for `std::signer::address_of()`.
//!
//! This function returns the address of the signer.
use super::NativeFunction;
use crate::{
    CompilationContext, compilation_context::ModuleId,
    native_functions::error::NativeFunctionError, translation::intermediate_types::signer::ISigner,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

/// Returns the signer's address.
///
/// It allocates fresh memory and copy the 32-byte address data into it.
pub fn add_signer_address_of_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
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
    let new_ptr = module.locals.add(ValType::I32);

    // Allocate 32 bytes and copy the signer's address into it.
    builder
        .i32_const(ISigner::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_tee(new_ptr)
        .local_get(signer_ptr)
        .i32_const(ISigner::HEAP_SIZE)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Return the new, independent pointer.
    builder.local_get(new_ptr);

    Ok(function.finish(vec![signer_ptr], &mut module.funcs))
}
