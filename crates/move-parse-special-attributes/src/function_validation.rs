// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! Function validation module.
//!
//! This module validates Move functions according to Stylus Framework rules:
//! - Generic functions cannot be entrypoints
//! - Event/error types can only be passed to native emit/revert functions
//! - Entry functions cannot return structs with the key ability
//! - UID and NamedId types are reserved and cannot be used as function arguments
//! - Calls to `emit()` must pass an event struct
//! - Calls to `revert()` must pass an error struct

mod error;
mod native_functions;
mod parse_body;

// Re-export public items
pub use error::FunctionValidationError;

use move_compiler::parser::ast::Function;
use move_symbol_pool::Symbol;
use std::collections::{HashMap, HashSet};

use crate::{
    AbiError, Event, ModuleId, Struct_,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    function_modifiers::Signature,
    types::Type,
};

use native_functions::{is_native_emit, is_native_revert, validate_function_calls};
use parse_body::extract_function_calls;

/// Maps function names (or aliases) to their original name and source module.
/// e.g., if `use stylus::error::revert as revert_alias;` is in scope,
/// this would contain: `revert_alias` -> ("revert", ModuleId { address: SF_ADDRESS, module_name: "error" })
pub type FunctionAliasMap = HashMap<Symbol, (Symbol, ModuleId)>;

/// Builds a function alias map from imported members.
/// This allows us to resolve function aliases back to their original names and modules.
pub fn build_function_alias_map(
    imported_members: &HashMap<ModuleId, Vec<(Symbol, Option<Symbol>)>>,
) -> FunctionAliasMap {
    let mut map = FunctionAliasMap::new();

    for (module_id, members) in imported_members {
        for (original_name, alias_opt) in members {
            // Use alias if present, otherwise use the original name
            let key = alias_opt.unwrap_or(*original_name);
            map.insert(key, (*original_name, module_id.clone()));
        }
    }

    map
}

/// Validates that a function is correct:
///
/// - If the function is generic, it cannot be an entrypoint.
/// - If the function has an Event parameter, it must be an emit function; otherwise, it is invalid.
/// - If the function has an AbiError parameter, it must be a revert function; otherwise, it is invalid.
/// - Entry functions cannot return structs with the key ability.
/// - Functions cannot take a UID as arguments, unless it is a function from the Stylus Framework package.
/// - Calls to `emit` must pass an event struct as argument.
/// - Calls to `revert` must pass an error struct as argument.
pub fn validate_function(
    function: &Function,
    events: &HashMap<Symbol, Event>,
    abi_errors: &HashMap<Symbol, AbiError>,
    structs: &[Struct_],
    deps_structs: &HashMap<ModuleId, Vec<Struct_>>,
    imported_members: &HashMap<ModuleId, Vec<(Symbol, Option<Symbol>)>>,
    package_address: [u8; 32],
) -> Result<(), SpecialAttributeError> {
    let signature = crate::function_modifiers::Function::parse_signature(&function.signature);

    // If any of the function's parameters is a UID type and the package address does not match the Stylus Framework address, this function should be rejected as invalid.
    if package_address != crate::reserved_modules::SF_ADDRESS {
        for param in &signature.parameters {
            for struct_name in param.type_.extract_user_data_type_names() {
                if struct_name.as_str() == "UID" {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::InvalidUidArgument,
                        ),
                        line_of_code: function.loc,
                    });
                } else if struct_name.as_str() == "NamedId" {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::InvalidNamedIdArgument,
                        ),
                        line_of_code: function.loc,
                    });
                }
            }
        }
    }

    if function.entry.is_some() {
        // If the function is generic and is entry, it should be rejected as invalid.
        if !function.signature.type_parameters.is_empty() {
            return Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::GenericFunctionsIsEntry,
                ),
                line_of_code: function.loc,
            });
        }

        // Check if return type contains any structs with the key ability
        for struct_name in signature.return_type.extract_user_data_type_names() {
            // First, check if the struct exists in local structs
            let module_struct = structs.iter().find(|s| s.name == struct_name);

            // If not defined in the module, check in imported members
            let imported_struct = module_struct
                .is_none()
                .then(|| {
                    imported_members.iter().find_map(|(module_id, members)| {
                        members.iter().find_map(|(original_name, alias_opt)| {
                            // First check the original name, if not found, check the alias
                            if original_name == &struct_name
                                || alias_opt
                                    .as_ref()
                                    .map(|a| a == &struct_name)
                                    .unwrap_or(false)
                            {
                                // If there's a match, search the struct in the dependency's structs hashmap.
                                // This map supplements the imported members by providing extra information about structs, including whether they have the key ability.
                                deps_structs.get(module_id).and_then(|module_structs| {
                                    module_structs.iter().find(|s| s.name == *original_name)
                                })
                            } else {
                                None
                            }
                        })
                    })
                })
                .flatten();

            // If struct is not found in either local or imported, return error
            match module_struct.or(imported_struct) {
                None => {
                    // Note: here we might encounter the case where the datatype is actually an enum not an struct,
                    // in this case we dont want to return an error, we want to ignore it.
                }
                Some(found_struct) => {
                    // If struct is found and has key ability, return error
                    if found_struct.has_key {
                        return Err(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::FunctionValidation(
                                FunctionValidationError::EntryFunctionReturnsKeyStruct,
                            ),
                            line_of_code: function.loc,
                        });
                    }
                }
            }
        }
    }

    // Event and error types can only be passed as arguments to the native emit/revert functions
    // from the stylus framework. If a non-framework function has an event or error argument, reject it.
    //
    // Note: We skip validation for the native emit/revert functions themselves because they use
    // generic type parameters (e.g., `emit<T: copy + drop>(event: T)`). The generic `T` won't match
    // our `is_event_type` check since it's not a concrete event type registered in the `events` map.

    // Event types can only be passed as arguments to the native `emit` function from `stylus::event` module.
    if !is_native_emit(function, package_address)
        && signature
            .parameters
            .iter()
            .any(|p| p.type_.is_event(events))
    {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::InvalidEventArgument,
            ),
            line_of_code: function.loc,
        });
    }

    // Error types can only be passed as arguments to the native `revert` function from `stylus::error` module.
    if !is_native_revert(function, package_address)
        && signature
            .parameters
            .iter()
            .any(|p| p.type_.is_abi_error(abi_errors))
    {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::InvalidErrorArgument,
            ),
            line_of_code: function.loc,
        });
    }

    // Build a set of known struct names to filter out struct constructors from function calls
    // At the parser level, positional struct constructors (e.g., `MyStruct(a)`) look like function calls
    let known_struct_names: HashSet<Symbol> = structs
        .iter()
        .map(|s| s.name)
        .chain(events.keys().copied())
        .chain(abi_errors.keys().copied())
        .collect();

    // Build function alias map for resolving function aliases (e.g., `revert as revert_alias`)
    let function_alias_map = build_function_alias_map(imported_members);

    // Extract calls (this includes function calls and struct constructors), then filter out struct constructors
    let (all_calls, bindings) = extract_function_calls(function);
    let function_calls: Vec<_> = all_calls
        .into_iter()
        .filter(|call| !known_struct_names.contains(&call.function_name))
        .collect();

    validate_function_calls(
        &function_calls,
        events,
        abi_errors,
        &bindings,
        &function_alias_map,
    )?;

    Ok(())
}

/// Checks if a storage object parameter is valid (must be a struct with key ability)
pub fn check_storage_object_param(
    signature: &Signature,
    identifier: Symbol,
    identifier_loc: move_ir_types::location::Loc,
    module_structs: &[Struct_],
) -> Result<(), SpecialAttributeError> {
    if let Some(param_type_name) = signature.parameters.iter().find_map(|p| {
        if p.name == identifier {
            match &p.type_ {
                Type::UserDataType(name, _) => Some(name),
                Type::Ref(inner) => {
                    if let Type::UserDataType(name, _) = &**inner {
                        Some(name)
                    } else {
                        None
                    }
                }
                Type::MutRef(inner) => {
                    if let Type::UserDataType(name, _) = &**inner {
                        Some(name)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }) {
        if let Some(struct_) = module_structs.iter().find(|s| s.name == *param_type_name) {
            if !struct_.has_key {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::StorageObjectNotKeyedStruct(identifier),
                    ),
                    line_of_code: identifier_loc,
                });
            }
        } else {
            return Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::StorageObjectStructNotFound(identifier),
                ),
                line_of_code: identifier_loc,
            });
        }
    } else {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::ParameterNotFound(identifier),
            ),
            line_of_code: identifier_loc,
        });
    }

    Ok(())
}

/// Checks if a storage object parameter has already been processed (to detect duplicates)
pub fn check_repeated_storage_object_param(
    processed_storage_objects: &mut HashSet<Symbol>,
    identifier: Symbol,
    identifier_loc: move_ir_types::location::Loc,
) -> Result<(), SpecialAttributeError> {
    if processed_storage_objects.contains(&identifier) {
        Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::RepeatedStorageObject(identifier),
            line_of_code: identifier_loc,
        })
    } else {
        // Add to processed storage objects
        processed_storage_objects.insert(identifier);
        Ok(())
    }
}
