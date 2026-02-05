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
/// - Direct imports: `use stylus::event::emit;` then `emit(...)`
/// - Aliased imports: `use stylus::event::emit as emit_alias;` then `emit_alias(...)`
pub fn is_emit_call(call: &ExtractedFunctionCall, function_alias_map: &FunctionAliasMap) -> bool {
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
/// - Direct imports: `use stylus::error::revert;` then `revert(...)`
/// - Aliased imports: `use stylus::error::revert as revert_alias;` then `revert_alias(...)`
pub fn is_revert_call(call: &ExtractedFunctionCall, function_alias_map: &FunctionAliasMap) -> bool {
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
/// - `emit()` must be called with a struct marked as #[event]
/// - `revert()` must be called with a struct marked as #[abi_error]
pub fn validate_function_calls(
    calls: &[ExtractedFunctionCall],
    events: &HashMap<Symbol, Event>,
    abi_errors: &HashMap<Symbol, AbiError>,
    bindings: &VariableBindings,
    function_alias_map: &FunctionAliasMap,
) -> Result<(), SpecialAttributeError> {
    for call in calls {
        if is_emit_call(call, function_alias_map) {
            // emit() should have exactly one argument that is an event struct
            if call.arguments.is_empty() {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::NativeEmitNoArgument,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Try to extract the struct name from the first argument
            if let Some(struct_name) = extract_struct_name_from_exp(&call.arguments[0], bindings) {
                // Check if the struct is marked as an event
                if !events.contains_key(&struct_name) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::NativeEmitNotEventArgument,
                        ),
                        line_of_code: call.loc,
                    });
                }
            } else {
                panic!("struct_name not found for emit call");
            }
        } else if is_revert_call(call, function_alias_map) {
            // revert() should have exactly one argument that is an error struct
            if call.arguments.is_empty() {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::NativeRevertNoArgument,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Try to extract the struct name from the first argument
            if let Some(struct_name) = extract_struct_name_from_exp(&call.arguments[0], bindings) {
                // Check if the struct is marked as an abi_error
                if !abi_errors.contains_key(&struct_name) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::NativeRevertNotErrorArgument,
                        ),
                        line_of_code: call.loc,
                    });
                }
            } else {
                panic!("struct_name not found for revert call");
            }
        }
        // Other function calls don't need special validation
    }

    Ok(())
}
