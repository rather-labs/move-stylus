use crate::{
    abi_types::error::AbiError,
    error::{CompilationError, ICEError, ICEErrorKind},
};

#[derive(Debug, thiserror::Error)]
pub enum HostIOError {
    #[error("there was an error setting up the host environmnet")]
    Abi(#[from] AbiError),
}

impl From<HostIOError> for CompilationError {
    fn from(value: HostIOError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::HostIO(value)))
    }
}
