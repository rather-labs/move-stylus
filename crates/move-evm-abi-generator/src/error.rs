// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use move_binary_format::file_format::SignatureToken;
use move_core_types::language_storage::ModuleId;
use move_symbol_pool::Symbol;

pub struct AbiGeneratorError {
    pub kind: AbiGeneratorErrorKind,
}

#[derive(thiserror::Error, Debug)]
pub enum AbiGeneratorErrorKind {
    // Data lookup errors
    #[error("Module ID not found for path")]
    ModuleIdNotFound,

    #[error("Module data not found for module ID")]
    ModuleDataNotFound,

    #[error("Could not find dependency module '{0}'")]
    DependencyNotFound(ModuleId),

    #[error("Function '{0}' not found in parsed special attributes")]
    FunctionNotFound(Symbol),

    #[error("Struct not found by index in module")]
    StructNotFoundByIndex,

    #[error("Struct not found by identifier '{0}'")]
    StructNotFoundByIdentifier(Symbol),

    #[error("Enum not found by index in module")]
    EnumNotFoundByIndex,

    #[error("Parsed event not found for '{0}'")]
    ParsedEventNotFound(Symbol),

    #[error("Parsed struct not found for '{0}'")]
    ParsedStructNotFound(Symbol),

    #[error("Parsed enum not found for '{0}'")]
    ParsedEnumNotFound(Symbol),

    #[error("Missing type parameter")]
    MissingTypeParameter,

    // Type processing errors
    #[error(
        "Invalid type found in native emit function: expected Datatype or DatatypeInstantiation, found {0:?}"
    )]
    InvalidEmitType(SignatureToken),

    #[error("Invalid type found in native revert function: expected Datatype, found {0:?}")]
    InvalidRevertType(SignatureToken),

    #[error("Non-simple enum '{0}' found in function signature")]
    NonSimpleEnumInSignature(Symbol),

    #[error("Storage struct first field is not a UID")]
    StorageStructMissingUid,

    #[error("Storage struct first field is not a NamedId")]
    StorageStructMissingNamedId,

    #[error("Storage struct has no fields")]
    StorageStructNoFields,

    #[error("Storage struct first field is not a UID or NamedId")]
    StorageStructInvalidFirstField,

    #[error("Unknown BytesN type: Bytes{0}")]
    UnknownBytesNType(String),

    #[error("Unknown sol types struct: {0}")]
    UnknownSolTypesStruct(String),

    #[error("Expected struct type but found: {0}")]
    ExpectedStructType(String),

    #[error("Expected enum type but found: {0}")]
    ExpectedEnumType(String),

    // JSON ABI generation errors
    #[error("Unexpected Tuple type in JSON ABI generation")]
    TupleInJsonAbi,

    #[error("Unexpected None type in JSON ABI generation")]
    NoneTypeInJsonAbi,

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Struct '{0}' not found in ABI structs for JSON encoding")]
    AbiStructNotFound(Symbol),
}
