use crate::{
    compilation_context::CompilationContextError, native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError, translation::intermediate_types::IntermediateType,
};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("there was an error decoding data from storage")]
    Decode(#[from] DecodeError),

    #[error("there was an error encoding data from storage")]
    Encode(#[from] EncodeError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("an error ocurred while generating a native function's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("compilation context error")]
    CompilationContext(#[from] CompilationContextError),
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("trying to decode an invalid type")]
    InvalidType(IntermediateType),

    #[error("invalid storage size {0} for {1:?}")]
    InvalidStorageSize(i32, IntermediateType),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("trying to encode an invalid type")]
    InvalidType(IntermediateType),
}
