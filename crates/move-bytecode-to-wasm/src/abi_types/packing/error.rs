use crate::{
    abi_types::error::AbiEncodingError, native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError,
};

#[derive(Debug, thiserror::Error)]
pub enum AbiPackError {
    #[error(
        "expected stylus::object::UID or stylus::object::NamedId as first field in {0} struct (it has key ability)"
    )]
    StorageObjectHasNoId(String),

    #[error(r#"cannot abi pack enum "{0}", it contains at least one variant with fields"#)]
    EnumIsNotSimple(String),

    #[error("cannot pack generic type parameter")]
    PackingGenericTypeParameter,

    #[error("cannnot know the size of a generic type parameter at compile time")]
    GenericTypeParameterSize,

    #[error("cannot check if generic type parameter is dynamic at compile time")]
    GenericTypeParameterIsDynamic,

    #[error("found a reference inside a reference")]
    RefInsideRef,

    #[error("signer type cannot be packed as it has no ABI representation")]
    FoundSignerType,

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("abi encoding error")]
    AbiEncoding(#[from] AbiEncodingError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),
}
