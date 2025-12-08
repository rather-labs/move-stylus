use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Function, FunctionBody_},
};

use crate::{
    AbiError, Event, Struct_,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    types::Type,
};

use std::collections::HashMap;
#[derive(thiserror::Error, Debug)]
pub enum FunctionValidationError {
    #[error("Function with Event type parameter must be a native emit function")]
    InvalidEmitFunction,

    #[error("Function with Error type parameter must be a native revert function")]
    InvalidRevertFunction,

    #[error("Generic functions cannot be entrypoints")]
    GenericFunctionsIsEntry,

    #[error("Entry functions cannot return structs with the key ability")]
    EntryFunctionReturnsKeyStruct,

    #[error("Invalid UID argument. UID is a reserved type and cannot be used as an argument.")]
    InvalidUidArgument,

    #[error(
        "Invalid NamedId argument. NamedId is a reserved type and cannot be used as an argument."
    )]
    InvalidNamedIdArgument,

    #[error("Struct not found in local or imported modules")]
    StructNotFound,
}

impl From<&FunctionValidationError> for DiagnosticInfo {
    fn from(value: &FunctionValidationError) -> Self {
        custom(
            "Function validation error",
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

/// Checks if a type is an Event by comparing its name with known events
fn is_event_type(type_: &Type, events: &HashMap<String, Event>) -> bool {
    match type_ {
        Type::UserDataType(name, _) => events.contains_key(name),
        _ => false,
    }
}

/// Checks if a type is an AbiError by comparing its name with known abi_errors
fn is_abi_error_type(type_: &Type, abi_errors: &HashMap<String, AbiError>) -> bool {
    match type_ {
        Type::UserDataType(name, _) => abi_errors.contains_key(name),
        _ => false,
    }
}

/// Validates that a function with Event type parameter is a native emit function
fn validate_emit_function(
    function: &Function,
    events: &HashMap<String, Event>,
) -> Result<(), SpecialAttributeError> {
    let err = SpecialAttributeError {
        kind: SpecialAttributeErrorKind::FunctionValidation(
            FunctionValidationError::InvalidEmitFunction,
        ),
        line_of_code: function.loc,
    };
    // Check function name
    if function.name.to_string() != "emit" {
        return Err(err);
    }

    // Check if function is native
    if function.body.value != FunctionBody_::Native {
        return Err(err);
    }

    // Check if function is public
    if !matches!(
        function.visibility,
        move_compiler::parser::ast::Visibility::Public(_)
    ) {
        return Err(err);
    }

    // Check that there's exactly one parameter
    if function.signature.parameters.len() != 1 {
        return Err(err);
    }

    // Check that the parameter is an Event type
    let param_type = Type::parse_type(&function.signature.parameters[0].2.value);
    if !is_event_type(&param_type, events) {
        return Err(err);
    }

    // Check that return type is Unit
    let return_type = Type::parse_type(&function.signature.return_type.value);
    if return_type != Type::Unit {
        return Err(err);
    }

    Ok(())
}

/// Validates that a function with an Error type parameter is a native revert function
fn validate_revert_function(
    function: &Function,
    abi_errors: &HashMap<String, AbiError>,
) -> Result<(), SpecialAttributeError> {
    let err = SpecialAttributeError {
        kind: SpecialAttributeErrorKind::FunctionValidation(
            FunctionValidationError::InvalidRevertFunction,
        ),
        line_of_code: function.loc,
    };
    // Check function name
    if function.name.to_string() != "revert" {
        return Err(err);
    }

    // Check if function is native
    if function.body.value != FunctionBody_::Native {
        return Err(err);
    }

    // Check if function is public
    if !matches!(
        function.visibility,
        move_compiler::parser::ast::Visibility::Public(_)
    ) {
        return Err(err);
    }

    // Check that there's exactly one parameter
    if function.signature.parameters.len() != 1 {
        return Err(err);
    }

    // Check that the parameter is an AbiError type
    let param_type = Type::parse_type(&function.signature.parameters[0].2.value);
    if !is_abi_error_type(&param_type, abi_errors) {
        return Err(err);
    }

    // Check that return type is Unit
    let return_type = Type::parse_type(&function.signature.return_type.value);
    if return_type != Type::Unit {
        return Err(err);
    }

    Ok(())
}

/// Extracts all struct names from a type (recursively handles vectors, tuples, etc.)
fn extract_struct_names(type_: &Type) -> Vec<String> {
    match type_ {
        Type::UserDataType(name, _) => vec![name.clone()],
        Type::Vector(inner) => extract_struct_names(inner),
        Type::Tuple(types) => types.iter().flat_map(extract_struct_names).collect(),
        _ => Vec::new(),
    }
}

/// Validates that a function is correct:
///
/// - If the function is generic, it cannot be an entrypoint.
/// - If the function has an Event parameter, it must be an emit function; otherwise, it is invalid.
/// - If the function has an AbiError parameter, it must be a revert function; otherwise, it is invalid.
/// - Entry functions cannot return structs with the key ability.
/// - Functions cannot take a UID as arguments, unless it is a function from the Stylus Framework package.
pub fn validate_function(
    function: &Function,
    events: &HashMap<String, Event>,
    abi_errors: &HashMap<String, AbiError>,
    structs: &[Struct_],
    deps_structs: &HashMap<String, Vec<Struct_>>,
    imported_members: &HashMap<String, Vec<(String, Option<String>)>>,
    package_address: [u8; 32],
) -> Result<(), SpecialAttributeError> {
    let signature = crate::function_modifiers::Function::parse_signature(&function.signature);

    // If any of the function's parameters is a UID type and the package address does not match the Stylus Framework address, this function should be rejected as invalid.
    if package_address != crate::reserved_modules::SF_ADDRESS {
        for param in &signature.parameters {
            for struct_name in extract_struct_names(&param.type_) {
                if struct_name == "UID" {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::InvalidUidArgument,
                        ),
                        line_of_code: function.loc,
                    });
                } else if struct_name == "NamedId" {
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
        for struct_name in extract_struct_names(&signature.return_type) {
            // First, check if the struct exists in local structs
            let module_struct = structs.iter().find(|s| s.name == struct_name);

            // If not found locally, check in imported members
            let imported_struct = module_struct
                .is_none()
                .then(|| {
                    imported_members.iter().find_map(|(module_name, members)| {
                        members.iter().find_map(|(original_name, alias_opt)| {
                            // First check the original name, if not found, check the alias
                            if alias_opt
                                .as_ref()
                                .map(|a| a == &struct_name)
                                .unwrap_or(false)
                                || original_name == &struct_name
                            {
                                // If there's a match, search the struct in the dependency's structs hashmap.
                                // This map supplements the imported members by providing extra information about structs, including whether they have the key ability.
                                // TODO: ideally we should use module_id instead of module_name, but at this point the ast doesnt resolve named addresses, and from the compiler we get numerical addresses.
                                deps_structs.get(module_name).and_then(|module_structs| {
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
                    // TODO: here we might encounter the case where the datatype is actually an enum not an struct,
                    // in this case we dont want to return an error, we want to ignore it.
                    // return Err(SpecialAttributeError {
                    //     kind: SpecialAttributeErrorKind::FunctionValidation(
                    //         FunctionValidationError::StructNotFound,
                    //     ),
                    //     line_of_code: function.loc,
                    // });
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

    for param in &signature.parameters {
        if is_event_type(&param.type_, events) {
            return validate_emit_function(function, events);
        }
        if is_abi_error_type(&param.type_, abi_errors) {
            return validate_revert_function(function, abi_errors);
        }
    }

    Ok(())
}
