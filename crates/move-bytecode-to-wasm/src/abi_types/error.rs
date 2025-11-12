use crate::error::{CompilationError, ICEError, ICEErrorKind};

use super::{
    packing::error::AbiPackError, public_function::PublicFunctionValidationError,
    unpacking::error::AbiUnpackError,
};

#[derive(thiserror::Error, Debug)]
pub enum AbiError {
    #[error("there was an error performing abi unpack operation")]
    Unpack(#[from] AbiUnpackError),

    #[error("there was an error performing abi pack operation")]
    Pack(#[from] AbiPackError),

    #[error("there was an error validating a public function")]
    PublicFunction(#[from] PublicFunctionValidationError),
}

impl From<AbiError> for CompilationError {
    fn from(value: AbiError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Abi(value)))
    }
}
