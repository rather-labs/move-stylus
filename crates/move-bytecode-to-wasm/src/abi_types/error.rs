use crate::{
    error::{CompilationError, ICEError, ICEErrorKind},
    runtime::error::RuntimeFunctionError,
};

use super::{
    packing::error::AbiPackError, public_function::PublicFunctionValidationError,
    unpacking::error::AbiUnpackError,
};

#[derive(Debug, thiserror::Error)]
pub enum AbiEncodingError {
    #[error("found a reference inside a reference")]
    RefInsideRef,

    #[error("generic type parameter")]
    FoundGenericTypeParameter,

    #[error("signer type cannot be packed as it has no ABI representation")]
    FoundSignerType,

    #[error("cannnot know the size of a generic type parameter at compile time")]
    GenericTypeParameterSize,

    #[error("cannot check if generic type parameter is dynamic at compile time")]
    GenericTypeParameterIsDynamic,

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),
}

#[derive(thiserror::Error, Debug)]
pub enum AbiError {
    #[error("there was an error performing abi unpack operation")]
    Unpack(#[from] AbiUnpackError),

    #[error("there was an error performing abi pack operation")]
    Pack(#[from] AbiPackError),

    #[error("abi encoding error")]
    AbiEncoding(#[from] AbiEncodingError),

    #[error("there was an error validating a public function")]
    PublicFunction(#[from] PublicFunctionValidationError),
}

impl From<AbiError> for CompilationError {
    fn from(value: AbiError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Abi(value)))
    }
}
