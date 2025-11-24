use std::rc::Rc;

use crate::{
    compilation_context::CompilationContextError,
    native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError,
    translation::intermediate_types::{IntermediateType, error::IntermediateTypeError},
    vm_handled_types::error::VmHandledTypeError,
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

    #[error("there was an error processing an intermediate type while accessing storage: {0:?}")]
    IntermediateType(#[from] IntermediateTypeError),

    #[error("an error ocurred while processing a vm handled type")]
    VmHandledType(#[from] VmHandledTypeError),

    #[error("found a reference inside struct/enum variant fields")]
    FieldSizeFoundRef(IntermediateType),

    #[error("cannot know the field size of a type parameter")]
    FieldSizeFoundTypeParameter,

    #[error("found reference inside struct with index {struct_index}")]
    FoundReferenceInsideStruct { struct_index: u16 },

    #[error(
        "found type parameter inside struct with index {struct_index} and type parameter index {type_parameter_index}"
    )]
    FoundTypeParameterInsideStruct {
        struct_index: u16,
        type_parameter_index: u16,
    },

    #[error(
        "found type parameter inside enum variant with index {variant_index} and enum index {enum_index}"
    )]
    FoundTypeParameterInsideEnumVariant { enum_index: u16, variant_index: u16 },

    #[error("found type parameter inside enum with index {enum_index}")]
    FoundTypeParameterInsideEnum { enum_index: u16 },

    #[error("found reference inside enum with index {enum_index}")]
    FoundReferenceInsideEnum { enum_index: u16 },
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(
        "there was an error processing an intermediate type {0:?} while decoding data from storage"
    )]
    IntermediateType(#[source] Rc<IntermediateTypeError>),

    #[error("trying to decode an invalid type")]
    InvalidType(IntermediateType),

    #[error("invalid storage size {0} for {1:?}")]
    InvalidStorageSize(i32, IntermediateType),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error(
        "there was an error processing an intermediate type {0:?} while encoding data from storage"
    )]
    IntermediateType(#[source] Rc<IntermediateTypeError>),

    #[error("trying to encode an invalid type")]
    InvalidType(IntermediateType),
}
