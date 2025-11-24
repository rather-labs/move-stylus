use std::{backtrace::Backtrace, fmt::Display};

use move_compiler::{diagnostics::Diagnostic, shared::files::MappedFiles};
use move_parse_special_attributes::SpecialAttributeError;

use crate::{
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    constructor::ConstructorError,
    hostio::error::HostIOError,
    native_functions::error::NativeFunctionError,
    translation::{TranslationError, table::FunctionTableError},
};

#[derive(thiserror::Error, Debug)]
pub enum DependencyProcessingError {
    #[error("internal compiler error(s) ocurred")]
    ICE(#[from] ICEError),

    #[error("code error ocurred")]
    CodeError(Vec<CodeError>),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("internal compiler error(s) ocurred")]
    ICE(#[from] ICEError),

    #[error("code error ocurred")]
    CodeError {
        mapped_files: MappedFiles,
        errors: Vec<CodeError>,
    },

    #[error("no files found to compile")]
    NoFilesFound,
}

#[derive(thiserror::Error, Debug)]
pub enum CodeError {
    #[error("an special attributes error ocured")]
    SpecialAttributesError(#[from] SpecialAttributeError),
}

#[derive(Debug)]
pub struct ICEError {
    pub version: String,
    pub name: String,
    pub kind: ICEErrorKind,
    pub backtrace: Backtrace,
}

impl std::error::Error for ICEError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl ICEError {
    pub fn new(kind: ICEErrorKind) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            name: env!("CARGO_PKG_NAME").to_owned(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

impl Display for ICEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\n{}", self.name, self.version, self.kind)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DependencyError {
    #[error("could not find dependency {0}")]
    DependencyNotFound(String),

    #[error("processed the same dependency ({0}) twice in different contexts")]
    DependencyProcessedMoreThanOnce(ModuleId),
}

impl From<DependencyError> for DependencyProcessingError {
    fn from(value: DependencyError) -> Self {
        DependencyProcessingError::ICE(ICEError::new(value.into()))
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

    #[error("an error ocurred while generating a native function's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("an error ocurred whie processing a contract's ABI")]
    Abi(#[from] AbiError),

    #[error("an error ocurred while processing a contract's constructor")]
    Constructor(#[from] ConstructorError),

    #[error("an error ocurred while processing building host environment")]
    HostIO(#[from] HostIOError),

    #[error("an error ocurred while processing a contact's dependencies")]
    Dependency(#[from] DependencyError),
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
