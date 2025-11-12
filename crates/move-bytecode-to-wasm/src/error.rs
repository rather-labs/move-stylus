use std::{backtrace::Backtrace, fmt::Display};

use move_compiler::{diagnostics::Diagnostic, shared::files::MappedFiles};
use move_parse_special_attributes::SpecialAttributeError;

use crate::{
    abi_types::error::AbiError,
    compilation_context::CompilationContextError,
    native_functions::error::NativeFunctionError,
    translation::{TranslationError, table::FunctionTableError},
};

#[derive(thiserror::Error, Debug)]
pub enum DependencyError {
    #[error("An internal compiler error (ICE) has ocurred.\n{0}")]
    ICE(#[from] ICEError),

    #[error("internal compiler error(s) ocurred")]
    CodeError(Vec<CodeError>),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("An internal compiler error (ICE) has ocurred.\n{0}")]
    ICE(#[from] ICEError),

    #[error("internal compiler error(s) ocurred")]
    CodeError {
        mapped_files: MappedFiles,
        errors: Vec<CodeError>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CodeError {
    #[error("an special attributes error ocured")]
    SpecialAttributesError(#[from] SpecialAttributeError),
}

#[derive(Debug)]
pub struct ICEError {
    pub kind: ICEErrorKind,
    pub backtrace: Backtrace,
}

impl std::error::Error for ICEError {}

impl ICEError {
    pub fn new(kind: ICEErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

impl Display for ICEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{}\nPlease open an issue in Github <project url> with this message.\n\n{}"#,
            self.kind, self.backtrace
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ICEErrorKind {
    #[error("an error ocurred processing the compilation context")]
    CompilationContext(#[from] CompilationContextError),

    #[error("an error ocurred while translating move bytecode")]
    Translation(#[from] TranslationError),

    #[error("an error ocurred while handling the function table")]
    FunctionTable(#[from] FunctionTableError),

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("an error ocurred while processing a contract's ABI")]
    Abi(#[from] AbiError),
}

impl From<CodeError> for Diagnostic {
    fn from(value: CodeError) -> Self {
        Diagnostic::from(&value)
    }
}

impl From<&CodeError> for Diagnostic {
    fn from(value: &CodeError) -> Self {
        match value {
            CodeError::SpecialAttributesError(special_attribute_error) => {
                special_attribute_error.into()
            }
        }
    }
}
