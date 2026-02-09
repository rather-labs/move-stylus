// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! Error types for function validation.

use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};
use move_symbol_pool::Symbol;

use crate::error::DIAGNOSTIC_CATEGORY;

#[derive(thiserror::Error, Debug)]
pub enum FunctionValidationError {
    #[error("Only Stylus Framework's `emit` function can take an event struct as an argument")]
    InvalidEventArgument,

    #[error("Only Stylus Framework's `revert` function can take an error struct as an argument")]
    InvalidErrorArgument,

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

    #[error("Struct '{0}' must have the key ability to be a storage object")]
    StructWithoutKey(Symbol),

    #[error("Struct '{0}' not found")]
    StructNotFound(Symbol),

    #[error("Parameter '{0}' not found in function signature")]
    ParameterNotFound(Symbol),

    #[error("init function cannot be entry")]
    InitFunctionCannotBeEntry,

    #[error("emit() requires exactly one argument")]
    EmitWrongArgumentCount,

    #[error("revert() requires exactly one argument")]
    RevertWrongArgumentCount,

    #[error("emit() argument must be a struct marked with #[event]")]
    EmitArgumentNotEvent,

    #[error("revert() argument must be a struct marked with #[abi_error]")]
    RevertArgumentNotAbiError,
}

impl From<&FunctionValidationError> for DiagnosticInfo {
    fn from(value: &FunctionValidationError) -> Self {
        custom(
            DIAGNOSTIC_CATEGORY,
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}
