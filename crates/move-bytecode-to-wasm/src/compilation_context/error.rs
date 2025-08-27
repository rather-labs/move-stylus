use move_binary_format::file_format::{
    FieldHandleIndex, FieldInstantiationIndex, SignatureIndex, StructDefInstantiationIndex,
    StructDefinitionIndex,
};

use super::ModuleId;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CompilationContextError {
    #[error("struct with index {0} not found in compilation context")]
    StructNotFound(u16),

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

    #[error("module {0:?} not found")]
    ModuleNotFound(ModuleId),

    #[error("expected struct")]
    ExpectedStruct,
}
