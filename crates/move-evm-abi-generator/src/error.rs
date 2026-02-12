// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use move_binary_format::file_format::SignatureToken;
use move_bytecode_to_wasm::compilation_context::ModuleId;
use move_symbol_pool::Symbol;

use std::path::PathBuf;

pub struct AbiGeneratorError {
    pub kind: AbiGeneratorErrorKind,
}

#[derive(thiserror::Error, Debug)]
pub enum AbiGeneratorErrorKind {
    // Data lookup errors
    #[error("Module ID not found for path '{0}'")]
    ModuleIdNotFound(PathBuf),

    #[error("Module data not found for module id {0}")]
    ModuleDataNotFound(ModuleId),

    #[error("Could not find dependency module '{0}'")]
    DependencyNotFound(ModuleId),

    #[error("Function '{0}' not found in parsed special attributes")]
    FunctionNotFound(Symbol),

    #[error("Struct not found by index {0} in module {1}")]
    StructNotFoundByIndex(u16, ModuleId),

    #[error("Struct not found by identifier '{0}' in module {1}")]
    StructNotFoundByIdentifier(Symbol, ModuleId),

    #[error("Enum not found by index {0} in module {1}")]
    EnumNotFoundByIndex(u16, ModuleId),

    #[error("Parsed event not found for '{0}' in module {1}")]
    ParsedEventNotFound(Symbol, ModuleId),

    #[error("Parsed struct not found for '{0}' in module {1}")]
    ParsedStructNotFound(Symbol, ModuleId),

    #[error("Parsed enum not found for '{0}' in module {1}")]
    ParsedEnumNotFound(Symbol, ModuleId),

    #[error("Missing type parameter in call to '{0}'")]
    MissingTypeParameter(String),

    // Type processing errors
    #[error(
        "Invalid type found in native emit function: expected Datatype or DatatypeInstantiation, found {0:?}"
    )]
    InvalidEmitType(SignatureToken),

    #[error("Invalid type found in native revert function: expected Datatype, found {0:?}")]
    InvalidRevertType(SignatureToken),

    #[error("Non-simple enum '{0}' found in function {1}")]
    NonSimpleEnumInSignature(Symbol, String),

    #[error("Storage struct '{0}' in module {1} has no fields")]
    StorageStructNoFields(Symbol, ModuleId),

    #[error("Invalid first field for storage struct '{0}' in module {1}: expected {2} or {3}.")]
    StorageStructInvalidFirstField(Symbol, ModuleId, String, String),

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
