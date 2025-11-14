use crate::{
    abi_types::error::AbiError,
    error::{CompilationError, ICEError, ICEErrorKind},
    runtime::error::RuntimeFunctionError,
};

#[derive(Debug, thiserror::Error)]
pub enum HostIOError {
    #[error("there was an error setting up the host environmnet")]
    Abi(#[from] AbiError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),
}

impl From<HostIOError> for CompilationError {
    fn from(value: HostIOError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::HostIO(value)))
    }
}
