// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use std::fmt::{self, Display};

use move_symbol_pool::Symbol;

#[derive(thiserror::Error, Debug)]
pub enum AbiGeneratorErrorKind {
    // Data lookup errors
    #[error("Module ID not found for path")]
    ModuleIdNotFound,

    #[error("Module data not found for module ID")]
    ModuleDataNotFound,

    #[error("Could not find dependency module '{0}'")]
    DependencyNotFound(String),

    #[error("Function '{0}' not found in parsed special attributes")]
    FunctionNotFound(Symbol),

    #[error("Struct not found by index in module")]
    StructNotFoundByIndex,

    #[error("Struct not found by identifier '{0}'")]
    StructNotFoundByIdentifier(Symbol),

    #[error("Enum not found by index in module")]
    EnumNotFoundByIndex,

    #[error("Event special attributes not found for struct '{0}'")]
    EventAttributesNotFound(Symbol),

    #[error("Parsed struct not found for '{0}'")]
    ParsedStructNotFound(Symbol),

    // Type processing errors
    #[error(
        "Invalid type found in emit function: expected Datatype or DatatypeInstantiation, found {0}"
    )]
    InvalidEmitType(String),

    #[error("Invalid type found in revert function: expected Datatype")]
    InvalidRevertType,

    #[error("Non-simple enum '{0}' found in function signature")]
    NonSimpleEnumInSignature(Symbol),

    #[error("Storage struct has no UID as first parameter")]
    StorageStructNoUid,

    #[error("Storage struct has no NamedId as first parameter")]
    StorageStructNoNamedId,

    #[error("Storage struct has no valid ID as first parameter")]
    StorageStructNoId,

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

#[derive(thiserror::Error, Debug)]
pub struct AbiGeneratorError {
    pub kind: AbiGeneratorErrorKind,
}

impl Display for AbiGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl From<AbiGeneratorErrorKind> for AbiGeneratorError {
    fn from(kind: AbiGeneratorErrorKind) -> Self {
        Self { kind }
    }
}
