use crate::error::{CompilationErrorKind, ICEError, ICEErrorKind};

use super::{public_function::PublicFunctionValidationError, unpacking::error::AbiUnpackError};

#[derive(thiserror::Error, Debug)]
pub enum AbiError {
    #[error("there was an error performing abi unpack operation")]
    Unpack(#[from] AbiUnpackError),

    #[error("there was an error validating a public function")]
    PublicFunction(#[from] PublicFunctionValidationError),
}

impl From<AbiError> for CompilationErrorKind {
    fn from(value: AbiError) -> Self {
        CompilationErrorKind::ICE(ICEError::new(ICEErrorKind::Abi(value)))
    }
}
