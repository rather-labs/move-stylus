// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! Native function call detection and validation.
//!
//! This module handles validation of calls to native Stylus Framework functions:
//! - `emit()` from `stylus::event` - must be called with an event struct
//! - `revert()` from `stylus::error` - must be called with an error struct

use std::collections::HashMap;

use move_compiler::parser::ast::{Function, FunctionBody_};
use move_symbol_pool::Symbol;

use crate::{
    AbiError, Event,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    function_validation::FunctionAliasMap,
};

use super::{
    error::FunctionValidationError,
    parse_body::{ExtractedFunctionCall, VariableBindings, extract_struct_name_from_exp},
};

/// Checks if a function is the native `emit` function from `stylus::event` module.
///
/// The native emit function has the signature: `public native fun emit<T: copy + drop>(event: T)`
/// and is defined in the stylus framework package.
pub fn is_native_emit(function: &Function, package_address: [u8; 32]) -> bool {
    function.name.to_string() == "emit"
        && function.body.value == FunctionBody_::Native
        && package_address == crate::reserved_modules::SF_ADDRESS
}

/// Checks if a function is the native `revert` function from `stylus::error` module.
///
/// The native revert function has the signature: `public native fun revert<T: copy + drop>(error: T)`
/// and is defined in the stylus framework package.
pub fn is_native_revert(function: &Function, package_address: [u8; 32]) -> bool {
    function.name.to_string() == "revert"
        && function.body.value == FunctionBody_::Native
        && package_address == crate::reserved_modules::SF_ADDRESS
}

/// Checks if a function call is to the native `emit` function from `stylus::event` module.
///
/// This handles both:
/// - Direct imports: `use stylus::event::emit;`
/// - Aliased imports: `use stylus::event::emit as emit_alias;`
pub fn is_native_emit_call(
    call: &ExtractedFunctionCall,
    function_alias_map: &FunctionAliasMap,
) -> bool {
    // Check if this function name (or alias) resolves to emit from stylus::event
    if let Some((original_name, module_id)) = function_alias_map.get(&call.function_name) {
        return original_name.as_str() == "emit"
            && module_id.address == crate::reserved_modules::SF_ADDRESS
            && module_id.module_name.as_str() == "event";
    }

    false
}

/// Checks if a function call is to the native `revert` function from `stylus::error` module.
///
/// This handles both:
/// - Direct imports: `use stylus::error::revert;`
/// - Aliased imports: `use stylus::error::revert as revert_alias;`
pub fn is_native_revert_call(
    call: &ExtractedFunctionCall,
    function_alias_map: &FunctionAliasMap,
) -> bool {
    // Check if this function name (or alias) resolves to revert from stylus::error
    if let Some((original_name, module_id)) = function_alias_map.get(&call.function_name) {
        return original_name.as_str() == "revert"
            && module_id.address == crate::reserved_modules::SF_ADDRESS
            && module_id.module_name.as_str() == "error";
    }

    false
}

/// Validates function calls to emit, revert, and borrow_uid
///
/// - `emit()` must be called with a struct marked as #[ext(event(...))]
/// - `revert()` must be called with a struct marked as #[ext(abi_error)]
pub fn validate_native_function_calls(
    calls: &[ExtractedFunctionCall],
    events: &HashMap<Symbol, Event>,
    abi_errors: &HashMap<Symbol, AbiError>,
    bindings: &VariableBindings,
    function_alias_map: &FunctionAliasMap,
) -> Result<(), SpecialAttributeError> {
    for call in calls {
        if is_native_emit_call(call, function_alias_map) {
            // If the function is the stylus framework's native `emit` function, then we know it has exactly one argument.
            // This is a sanity check to ensure the function is called correctly. It should never fail.
            if call.arguments.len() != 1 {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::EmitWrongArgumentCount,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Check if the argument is a struct marked as an event
            let is_valid_event = extract_struct_name_from_exp(&call.arguments[0], bindings)
                .is_some_and(|name| events.contains_key(&name));

            if !is_valid_event {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::EmitArgumentNotEvent,
                    ),
                    line_of_code: call.loc,
                });
            }
        } else if is_native_revert_call(call, function_alias_map) {
            // If the function is the stylus framework's native `revert` function, then we know it has exactly one argument.
            // This is a sanity check to ensure the function is called correctly. It should never fail.
            if call.arguments.len() != 1 {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::RevertWrongArgumentCount,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Check if the argument is a struct marked as an abi_error
            let is_valid_error = extract_struct_name_from_exp(&call.arguments[0], bindings)
                .is_some_and(|name| abi_errors.contains_key(&name));

            if !is_valid_error {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::RevertArgumentNotAbiError,
                    ),
                    line_of_code: call.loc,
                });
            }
        }
        // Other function calls don't need special validation
    }

    Ok(())
}
