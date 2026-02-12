use std::{backtrace::Backtrace, fmt::Display};

use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol};
use move_compiler::{diagnostics::Diagnostic, shared::files::MappedFiles};
use move_parse_special_attributes::SpecialAttributeError;

use crate::{
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    constructor::ConstructorError,
    hostio::error::HostIOError,
    native_functions::error::NativeFunctionError,
    translation::{TranslationError, table::FunctionTableError},
    wasm_validation::WasmValidationError,
};

#[derive(thiserror::Error, Debug)]
pub enum DependencyProcessingError {
    #[error("internal compiler error(s) occurred")]
    ICE(#[from] ICEError),

    #[error("code error occurred")]
    CodeError(Vec<CodeError>),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("internal compiler error(s) occurred")]
    ICE(#[from] ICEError),

    #[error("code error occurred")]
    CodeError {
        mapped_files: MappedFiles,
        errors: Vec<CodeError>,
    },

    #[error("no files found to compile")]
    NoFilesFound,
}

impl From<ICEError> for Box<CompilationError> {
    fn from(value: ICEError) -> Self {
        CompilationError::from(value).into()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CodeError {
    #[error("an special attributes error ocured")]
    SpecialAttributesError(#[from] SpecialAttributeError),
}

#[derive(Debug)]
pub struct ICEError {
    pub version: String,
    pub name: String,
    pub kind: ICEErrorKind,
    pub backtrace: Backtrace,
}

impl std::error::Error for ICEError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl ICEError {
    pub fn new(kind: ICEErrorKind) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            name: env!("CARGO_PKG_NAME").to_owned(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

impl Display for ICEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\n{}", self.name, self.version, self.kind)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DependencyError {
    #[error("could not find dependency {0}")]
    DependencyNotFound(String),

    #[error("processed the same dependency ({0}) twice in different contexts")]
    DependencyProcessedMoreThanOnce(ModuleId),
}

impl From<DependencyError> for DependencyProcessingError {
    fn from(value: DependencyError) -> Self {
        DependencyProcessingError::ICE(ICEError::new(value.into()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ICEErrorKind {
    #[error("an error occurred processing the compilation context")]
    CompilationContext(#[from] CompilationContextError),

    #[error("an error occurred while translating move bytecode")]
    Translation(#[from] TranslationError),

    #[error("an error occurred while handling the function table")]
    FunctionTable(#[from] FunctionTableError),

    #[error("an error occurred while generating a native function's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("an error occurred whie processing a contract's ABI")]
    Abi(#[from] AbiError),

    #[error("an error occurred while processing a contract's constructor")]
    Constructor(#[from] ConstructorError),

    #[error("an error occurred while processing building host environment")]
    HostIO(#[from] HostIOError),

    #[error("an error occurred while processing a contact's dependencies")]
    Dependency(#[from] DependencyError),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("wasm validation error")]
    WasmValidation(#[from] WasmValidationError),

    #[error("module not compiled: {0}")]
    ModuleNotCompiled(String),

    #[error("unexpected error: {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<CodeError> for Diagnostic {
    fn from(value: CodeError) -> Self {
        Diagnostic::from(&value)
    }
}

impl From<&CodeError> for Diagnostic {
    fn from(value: &CodeError) -> Self {
        match value {
            CodeError::SpecialAttributesError(special_attribute_error) => {
                special_attribute_error.into()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeError {
    /// Attempted to share an object that is frozen.
    FrozenObjectsCannotBeShared,
    /// Attempted to freeze an object that is already shared.
    SharedObjectsCannotBeFrozen,
    /// Attempted to transfer ownership of an object that is frozen.
    FrozenObjectsCannotBeTransferred,
    /// Attempted to transfer ownership of an object that is shared.
    SharedObjectsCannotBeTransferred,
    /// The requested storage object was not found.
    StorageObjectNotFound,
    /// Attempted to delete an object that is frozen.
    FrozenObjectsCannotBeDeleted,
    /// The type of a storage object does not match the expected type.
    StorageObjectTypeMismatch,
    /// Arithmetic overflow occurred.
    Overflow,
    /// Access was out of bounds (e.g., array index out of range).
    OutOfBounds,
    /// Attempted an out-of-bounds memory access.
    MemoryAccessOutOfBounds,
    /// The size of an enum is too large to handle.
    EnumSizeTooLarge,
}

impl RuntimeError {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            RuntimeError::FrozenObjectsCannotBeShared => b"Frozen objects cannot be shared",
            RuntimeError::SharedObjectsCannotBeFrozen => b"Shared objects cannot be frozen",
            RuntimeError::FrozenObjectsCannotBeTransferred => {
                b"Frozen objects cannot be transferred"
            }
            RuntimeError::SharedObjectsCannotBeTransferred => {
                b"Shared objects cannot be transferred"
            }
            RuntimeError::StorageObjectNotFound => b"Object not found",
            RuntimeError::FrozenObjectsCannotBeDeleted => b"Frozen objects cannot be deleted",
            RuntimeError::StorageObjectTypeMismatch => b"Storage object type mismatch",
            RuntimeError::Overflow => b"Overflow",
            RuntimeError::OutOfBounds => b"Out of bounds",
            RuntimeError::MemoryAccessOutOfBounds => b"Memory access out of bounds",
            RuntimeError::EnumSizeTooLarge => b"Enum size too large",
        }
    }

    pub fn encode_abi(&self) -> Vec<u8> {
        [
            keccak256(b"Error(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&(String::from_utf8_lossy(self.as_bytes()),)),
        ]
        .concat()
    }
}
