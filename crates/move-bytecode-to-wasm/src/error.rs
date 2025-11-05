use std::fmt::Display;

use move_compiler::{diagnostics::Diagnostic, shared::files::MappedFiles};
use move_parse_special_attributes::SpecialAttributeError;

use crate::compilation_context::CompilationContextError;

#[derive(thiserror::Error, Debug)]
pub struct CompilationError {
    pub files: MappedFiles,

    pub kind: CompilationErrorKind,
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A compilation error has ocurred: {}", self.kind)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationErrorKind {
    #[error(
        "An internal compiler error has ocurred. If this keeps happening, please open an issue in\n<gh url>"
    )]
    ICE(#[from] ICEError),

    #[error("internal compiler error(s) ocurred")]
    CodeError(Vec<CodeError>),
}

#[derive(thiserror::Error, Debug)]
pub enum CodeError {
    #[error("an special attributes error ocured")]
    SpecialAttributesError(#[from] SpecialAttributeError),
}

#[derive(thiserror::Error, Debug)]
pub enum ICEError {
    #[error("an error ocurred processing the compilation context")]
    CompilationContext(#[from] CompilationContextError),
}

impl From<CodeError> for Diagnostic {
    fn from(value: CodeError) -> Self {
        match value {
            CodeError::SpecialAttributesError(ref special_attribute_error) => {
                special_attribute_error.into()
            }
        }
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
