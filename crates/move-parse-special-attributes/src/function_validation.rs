use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Function, FunctionBody_},
};

use crate::{
    AbiError, Event,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    types::Type,
};

#[derive(thiserror::Error, Debug)]
pub enum FunctionValidationError {
    #[error("Function with Event type parameter must be a native emit function")]
    InvalidEmitFunction,

    #[error("Function with Error type parameter must be a native revert function")]
    InvalidRevertFunction,

    #[error("Generic functions cannot be entrypoints")]
    GenericFunctionsIsEntry,
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
fn is_event_type(type_: &Type, events: &std::collections::HashMap<String, Event>) -> bool {
    match type_ {
        Type::UserDataType(name, _) => events.contains_key(name),
        _ => false,
    }
}

/// Checks if a type is an AbiError by comparing its name with known abi_errors
fn is_abi_error_type(
    type_: &Type,
    abi_errors: &std::collections::HashMap<String, AbiError>,
) -> bool {
    match type_ {
        Type::UserDataType(name, _) => abi_errors.contains_key(name),
        _ => false,
    }
}

/// Validates that a function with Event type parameter is a native emit function
fn validate_emit_function(
    function: &Function,
    events: &std::collections::HashMap<String, Event>,
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
    abi_errors: &std::collections::HashMap<String, AbiError>,
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

/// Validates that a function is correct:
///
/// - If the function is generic, it cannot be an entrypoint.
/// - If the function has an Event parameter, it must be an emit function; otherwise, it is invalid.
/// - If the function has an AbiError parameter, it must be a revert function; otherwise, it is invalid.
/// - If neither type is present, the function is always considered valid.
pub fn validate_function(
    function: &Function,
    events: &std::collections::HashMap<String, Event>,
    abi_errors: &std::collections::HashMap<String, AbiError>,
) -> Result<(), SpecialAttributeError> {
    if !function.signature.type_parameters.is_empty() && function.entry.is_some() {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::GenericFunctionsIsEntry,
            ),
            line_of_code: function.loc,
        });
    }
    let signature = crate::function_modifiers::Function::parse_signature(&function.signature);

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
