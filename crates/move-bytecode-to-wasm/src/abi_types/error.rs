use move_symbol_pool::Symbol;

use crate::{
    compilation_context::CompilationContextError,
    error::{CompilationError, ICEError, ICEErrorKind},
    native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError,
    translation::intermediate_types::error::IntermediateTypeError,
    vm_handled_types::error::VmHandledTypeError,
};

use super::public_function::PublicFunctionValidationError;

#[derive(Debug, thiserror::Error)]
pub enum AbiOperationError {
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

    #[error(
        "expected stylus::object::UID or stylus::object::NamedId as first field in {0} struct (it has key ability)"
    )]
    StorageObjectHasNoId(Symbol),

    #[error(r#"cannot abi unpack/pack enum "{0}", it contains at least one variant with fields"#)]
    EnumIsNotSimple(Symbol),

    #[error("cannot unpack generic type parameter")]
    UnpackingGenericTypeParameter,

    #[error("cannot pack generic type parameter")]
    PackingGenericTypeParameter,

    #[error(
        "found heap type unpacking a reference. this should be handled in the add_unpack_instructions function"
    )]
    UnhandledHeapTypeReference,
}

#[derive(thiserror::Error, Debug)]
pub enum AbiError {
    #[error("there was an error performing abi unpack operation")]
    Unpack(#[source] AbiOperationError),

    #[error("there was an error performing abi pack operation")]
    Pack(#[source] AbiOperationError),

    #[error("abi encoding error")]
    AbiEncoding(#[source] AbiOperationError),

    #[error("there was an error validating a public function")]
    PublicFunction(#[from] PublicFunctionValidationError),

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("compilation context error ocurred while ABI")]
    CompilationContext(#[from] CompilationContextError),

    #[error("an error ocurred while processing an intermediate type")]
    IntermediateType(#[from] IntermediateTypeError),

    #[error("an error ocurred while processing a vm handled type")]
    VmHandledType(#[from] VmHandledTypeError),

    #[error(
        "expected stylus::object::UID or stylus::object::NamedId as first field in {0} struct (it has key ability)"
    )]
    ExpectedUIDOrNamedId(Symbol),

    #[error("unable to get type ABI size")]
    UnableToGetTypeAbiSize,

    #[error("invalid selector size")]
    InvalidSelectorSize,

    #[error(
        "storage object (FrozenStorageObject, OwnedStorageObject, SharedStorageObject) has invalid type paramter"
    )]
    StorageObjectHasInvalidType,
}

impl From<AbiError> for CompilationError {
    fn from(value: AbiError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Abi(value)))
    }
}
