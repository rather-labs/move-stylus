use std::{backtrace::Backtrace, fmt::Display};

use move_compiler::{diagnostics::Diagnostic, shared::files::MappedFiles};
use move_parse_special_attributes::SpecialAttributeError;

use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    constructor::ConstructorError,
    data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    hostio::error::HostIOError,
    native_functions::error::NativeFunctionError,
    translation::{TranslationError, table::FunctionTableError},
    wasm_validation::WasmValidationError,
};

use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

#[derive(thiserror::Error, Debug)]
pub enum DependencyProcessingError {
    #[error("internal compiler error(s) ocurred")]
    ICE(#[from] ICEError),

    #[error("code error ocurred")]
    CodeError(Vec<CodeError>),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("internal compiler error(s) ocurred")]
    ICE(#[from] ICEError),

    #[error("code error ocurred")]
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
    #[error("an error ocurred processing the compilation context")]
    CompilationContext(#[from] CompilationContextError),

    #[error("an error ocurred while translating move bytecode")]
    Translation(#[from] TranslationError),

    #[error("an error ocurred while handling the function table")]
    FunctionTable(#[from] FunctionTableError),

    #[error("an error ocurred while generating a native function's code")]
    NativeFunction(#[from] NativeFunctionError),

    #[error("an error ocurred whie processing a contract's ABI")]
    Abi(#[from] AbiError),

    #[error("an error ocurred while processing a contract's constructor")]
    Constructor(#[from] ConstructorError),

    #[error("an error ocurred while processing building host environment")]
    HostIO(#[from] HostIOError),

    #[error("an error ocurred while processing a contact's dependencies")]
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
    FrozenObjectsCannotBeShared,
    SharedObjectsCannotBeFrozen,
    FrozenObjectsCannotBeTransferred,
    SharedObjectsCannotBeTransferred,
    StorageObjectNotFound,
    Overflow,
    OutOfBounds,
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
            RuntimeError::Overflow => b"Overflow",
            RuntimeError::OutOfBounds => b"Out of bounds",
        }
    }
}
/// Adds the instructions to store the error message pointer at DATA_ABORT_MESSAGE_PTR_OFFSET and return 1 to indicate an error occurred.
pub fn add_handle_error_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    return_i64: bool,
) {
    let encoded_error_ptr = module.locals.add(ValType::I32);
    builder.local_set(encoded_error_ptr);

    // Store the ptr at DATA_ABORT_MESSAGE_PTR_OFFSET
    builder
        .i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
        .local_get(encoded_error_ptr)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Return 1 to indicate an error occurred
    if return_i64 {
        builder.i64_const(1);
    } else {
        builder.i32_const(1);
    }

    builder.return_();
}

/// Adds the instructions to propagate the error by returning if the error message pointer at DATA_ABORT_MESSAGE_PTR_OFFSET is not null.
pub fn add_propagate_error_instructions(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
) {
    // If the function aborts, propagate the error
    builder.block(None, |b| {
        let block_id = b.id();
        b.i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(block_id);

        b.i32_const(1).return_();
    });
}
