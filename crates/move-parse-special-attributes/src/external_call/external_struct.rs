// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};
use move_symbol_pool::Symbol;

use crate::error::DIAGNOSTIC_CATEGORY;

#[derive(Debug)]
pub struct ExternalStruct {
    pub name: Symbol,
    pub address: [u8; 32],
    pub module_name: Symbol,
}

#[derive(Debug, thiserror::Error)]
pub enum ExternalStructError {
    #[error("duplicated address attribute")]
    DuplicatedAddressAttribute,

    #[error("expected numerical address")]
    ExpectedNumericalAddress,

    #[error("expected address")]
    ExpectedAddress,

    #[error("invalid attribute")]
    InvalidAttribute,

    #[error("not an external struct")]
    NotAnExternalStruct,

    #[error("duplicated module_name attribute")]
    DuplicatedModuleNameAttribute,

    #[error("expected byte string for module_name")]
    ExpectedByteString,

    #[error("address attribute not defined")]
    AddressNotDefined,

    #[error("module_name attribute not defined")]
    ModuleNameNotDefined,
}

impl From<&ExternalStructError> for DiagnosticInfo {
    fn from(value: &ExternalStructError) -> Self {
        custom(
            DIAGNOSTIC_CATEGORY,
            Severity::BlockingError,
            4,
            4,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}
