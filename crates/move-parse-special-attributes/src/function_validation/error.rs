// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! Error types for function validation.

use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};
use move_symbol_pool::Symbol;

use crate::error::DIAGNOSTIC_CATEGORY;

#[derive(thiserror::Error, Debug)]
pub enum FunctionValidationError {
    #[error("Only native emit function can take an event struct as an argument")]
    InvalidEventArgument,

    #[error("Only native revert function can take an error struct as an argument")]
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

    #[error("Storage object '{0}' must be a struct with the key ability")]
    StorageObjectNotKeyedStruct(Symbol),

    #[error("Storage object struct '{0}' not found")]
    StorageObjectStructNotFound(Symbol),

    #[error("Parameter '{0}' not found in function signature")]
    ParameterNotFound(Symbol),

    #[error("Struct not found in local or imported modules")]
    StructNotFound,

    #[error("init function cannot be entry")]
    InitFunctionCannotBeEntry,

    #[error("emit() requires an argument that is a struct marked with #[event]")]
    NativeEmitNoArgument,

    #[error("revert() requires an argument that is a struct marked with #[abi_error]")]
    NativeRevertNoArgument,

    #[error("emit() was called with a non-event argument")]
    NativeEmitNotEventArgument,

    #[error("revert() was called with a non-error argument")]
    NativeRevertNotErrorArgument,
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
