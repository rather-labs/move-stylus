use crate::native_functions::error::NativeFunctionError;

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

    #[error("unsupported stack_data_size {0} for IRef")]
    RefInvalidStackDataSize(u32),

    #[error("found a reference inside a reference")]
    RefInsideRef,

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunction(#[from] NativeFunctionError),
}
