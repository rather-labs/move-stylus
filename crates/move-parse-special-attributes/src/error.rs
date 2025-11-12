use std::fmt::{self, Display};

use move_compiler::{
    diag,
    diagnostics::{
        Diagnostic,
        codes::{DiagnosticInfo, Severity, custom},
    },
};
use move_ir_types::location::Loc;

use crate::{
    abi_error::AbiErrorParseError,
    event::EventParseError,
    external_call::{
        error::{ExternalCallFunctionError, ExternalCallStructError},
        external_struct::ExternalStructError,
    },
    function_validation::FunctionValidationError,
};

#[derive(thiserror::Error, Debug)]
pub enum SpecialAttributeErrorKind {
    #[error("Abi error: {0}")]
    AbiError(#[from] AbiErrorParseError),

    #[error("External call error: {0}")]
    ExternalCallFunction(#[from] ExternalCallFunctionError),

    #[error("External call struct error: {0}")]
    ExternalCallStruct(#[from] ExternalCallStructError),

    #[error("Event error: {0}")]
    Event(#[from] EventParseError),

    #[error("External struct error: {0}")]
    ExternalStruct(#[from] ExternalStructError),

    #[error("Function validation error: {0}")]
    FunctionValidation(#[from] FunctionValidationError),

    #[error("Too many attributes found")]
    TooManyAttributes,
}

#[derive(thiserror::Error, Debug)]
pub struct SpecialAttributeError {
    pub kind: SpecialAttributeErrorKind,
    #[allow(dead_code)]
    pub line_of_code: Loc,
}

impl Display for SpecialAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl From<&SpecialAttributeError> for Diagnostic {
    fn from(value: &SpecialAttributeError) -> Self {
        let diagnostic_info: DiagnosticInfo = match &value.kind {
            SpecialAttributeErrorKind::ExternalCallFunction(e) => e.into(),
            SpecialAttributeErrorKind::ExternalCallStruct(e) => e.into(),
            SpecialAttributeErrorKind::Event(e) => e.into(),
            SpecialAttributeErrorKind::ExternalStruct(e) => e.into(),
            SpecialAttributeErrorKind::AbiError(e) => e.into(),
            SpecialAttributeErrorKind::FunctionValidation(e) => e.into(),
            SpecialAttributeErrorKind::TooManyAttributes => custom(
                "Special attributes error",
                Severity::BlockingError,
                3,
                3,
                Box::leak(value.to_string().into_boxed_str()),
            ),
        };

        diag!(diagnostic_info, (value.line_of_code, "".to_string()))
    }
}
