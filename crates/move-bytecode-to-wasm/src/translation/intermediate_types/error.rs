use move_binary_format::file_format::DatatypeHandleIndex;
use walrus::ValType;

use crate::{
    compilation_context::CompilationContextError, runtime::error::RuntimeFunctionError,
    wasm_builder_extensions::WasmBuilderExtensionError,
};

use super::IntermediateType;

#[derive(thiserror::Error, Debug)]
pub enum IntermediateTypeError {
    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("compilation context error")]
    CompilationContextError(#[from] CompilationContextError),

    #[error("wasm extensions error")]
    WasmBuilderExtension(#[from] WasmBuilderExtensionError),

    #[error("found type parameter where concrete type was expected")]
    FoundTypeParameter,

    #[error("No user defined data with handler index: {0:?} found")]
    UserDefinedTypeNotFound(DatatypeHandleIndex),

    // Constant loading errors
    #[error("signer type cannot be loaded as a constant")]
    SignerCannotBeConstant,

    #[error("cannot load a constant for a reference type")]
    CannotLoadConstantForReferenceType,

    #[error("structs cannot be loaded as constants")]
    StructsCannotBeConstants,

    #[error("enum variants cannot be loaded as constants")]
    EnumVariantsCannotBeConstants,

    #[error(r#"trying to introduce copy instructions for "signer" type"#)]
    SignerCannotBeCopied,

    #[error("cannot perform ReadRef on {0:?}")]
    CannotReadRefOfType(IntermediateType),

    #[error("found reference inside enum with index {0} and variant index{1}")]
    FoundReferenceInsideEnum(u16, u16),

    #[error("found reference inside struct with index {0}")]
    FoundReferenceInsideStruct(u16),

    #[error("found a reference of a reference")]
    FoundReferenceOfReference,

    #[error("reference types are not supported in TypeName")]
    FoundReferenceInTypeName,

    #[error("cannot perform WriteRef on signer type")]
    CannotWriteRefOnSigner,

    #[error("cast error: trying to cast {0:?}")]
    InvalidCast(IntermediateType),

    #[error("unssuported ValType {0:?}")]
    UnsupportedValType(ValType),

    #[error("empty bytes in vector")]
    EmptyBytesInVector,

    #[error("could not process byte array, wrong size")]
    CouldNotProcessByteArray,

    // Vectors
    #[error("Unsupported data size for vector: {0}")]
    VectorInvalidDataSize(usize),

    #[error("store offset is not aligned with the needed bytes, needed {needed} got {actual}")]
    VectorStoreOffsetNotAligned { needed: usize, actual: usize },

    #[error("type {0:?} not supported in vectors")]
    VectorUnnsuportedType(IntermediateType),

    #[error("found vector of references")]
    FoundVectorOfReferences,

    #[error("found vector of signer")]
    FoundVectorOfSigner,

    #[error("constant address is larger than 20 bytes")]
    ConstantAddressTooLarge,

    // Enums
    #[error("found type parameter inside enum")]
    FoundTypeParameterInEnum,
}
