use std::fmt::{self, Display};

use move_compiler::diagnostics::Diagnostic;
use move_ir_types::location::Loc;

use crate::external_call::error::ExternalCallError;

#[derive(thiserror::Error, Debug)]
pub enum SpecialAttributeErrorKind {
    #[error("External call error: {0}")]
    ExternalCall(#[from] ExternalCallError),
}

#[derive(thiserror::Error, Debug)]
pub struct SpecialAttributeError {
    pub(crate) kind: SpecialAttributeErrorKind,
    pub(crate) line_of_code: Loc,
}

impl Display for SpecialAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

// TODO: We need to implement this to report errors in the same format as the move compiler does
impl From<SpecialAttributeError> for Diagnostic {
    fn from(_value: SpecialAttributeError) -> Self {
        todo!()
    }
}
