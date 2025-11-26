use std::rc::Rc;

use move_binary_format::file_format::{
    FieldHandleIndex, FieldInstantiationIndex, SignatureIndex, StructDefInstantiationIndex,
    StructDefinitionIndex,
};

use crate::{
    error::{CompilationError, ICEError, ICEErrorKind},
    translation::{
        functions::MappedFunctionError, intermediate_types::error::IntermediateTypeError,
    },
};

use super::{ModuleId, module_data::error::ModuleDataError};

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CompilationContextError {
    #[error("creating a mapped function")]
    MappedFunction(#[source] Rc<MappedFunctionError>),

    #[error("there was an error when processing an intermediate type")]
    IntermediateType(#[source] Rc<IntermediateTypeError>),

    #[error("processing module data")]
    ModuleData(#[from] ModuleDataError),

    #[error("struct with index {0} not found in compilation context")]
    StructNotFound(u16),

    #[error("struct with identifier {0} not found in compilation context")]
    StructByIdentifierNotFound(String),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithFieldIdxNotFound(FieldHandleIndex),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithDefinitionIdxNotFound(StructDefinitionIndex),

    #[error("struct with generic field instance id {0:?} not found in compilation context")]
    GenericStructWithFieldIdxNotFound(FieldInstantiationIndex),

    #[error("generic struct instance with field id {0:?} not found in compilation context")]
    GenericStructWithDefinitionIdxNotFound(StructDefInstantiationIndex),

    #[error("signature with signature index {0:?} not found in compilation context")]
    SignatureNotFound(SignatureIndex),

    #[error("enum with index {0} not found in compilation context")]
    EnumNotFound(u16),

    #[error("enum with enum id {0} not found in compilation context")]
    EnumWithVariantIdxNotFound(u16),

    #[error("module {0} not found")]
    ModuleNotFound(ModuleId),

    #[error("expected struct")]
    ExpectedStruct,

    #[error("expected enum")]
    ExpectedEnum,

    #[error("function with identifier {0} not found in compilation context")]
    FunctionByIdentifierNotFound(String),

    #[error(r#"datatype handle index "{0}" not found"#)]
    DatatypeHanldeIndexNotFound(usize),

    #[error("there can be only a single init function per module")]
    TwoOrMoreInits,

    #[error("found init funciton with no arguments")]
    InitFunctionNoAguments,

    #[error("too many arguments for init function")]
    InitFunctionTooManyArgs,

    #[error("init functions does not have TxContext as parameter")]
    InitFunctionNoTxContext,

    #[error("init function second argument must be a OTW")]
    InitFunctionNoOTW,

    #[error("expected no return values for init function")]
    InitFunctionBadRetrunValues,

    #[error("expected private visibility for init function")]
    InitFunctionBadPrivacy,
}

impl From<CompilationContextError> for CompilationError {
    fn from(value: CompilationContextError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::CompilationContext(value)))
    }
}

impl From<MappedFunctionError> for CompilationContextError {
    fn from(value: MappedFunctionError) -> Self {
        CompilationContextError::MappedFunction(value.into())
    }
}

impl From<IntermediateTypeError> for CompilationContextError {
    fn from(value: IntermediateTypeError) -> Self {
        CompilationContextError::IntermediateType(value.into())
    }
}
