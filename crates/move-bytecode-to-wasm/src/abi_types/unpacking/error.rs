use crate::{
    abi_types::error::AbiEncodingError, native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError,
};

#[derive(Debug, thiserror::Error)]
pub enum AbiUnpackError {
    #[error(
        "expected stylus::object::UID or stylus::object::NamedId as first field in {0} struct (it has key ability)"
    )]
    StorageObjectHasNoId(String),

    #[error(r#"cannot abi unpack enum "{0}", it contains at least one variant with fields"#)]
    EnumIsNotSimple(String),

    #[error("cannot unpack generic type parameter")]
    UnpackingGenericTypeParameter,

    #[error("found a reference inside a reference")]
    RefInsideRef,

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("abi encoding error")]
    AbiEncoding(#[from] AbiEncodingError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),
}
