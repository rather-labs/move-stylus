use move_binary_format::file_format::Bytecode;

use super::{intermediate_types::IntermediateType, types_stack::TypesStackError};

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Types stack error: {0}")]
    TypesStackError(#[from] TypesStackError),

    #[error("types mistach: expected {expected:?} but found {found:?}")]
    TypeMismatch {
        expected: IntermediateType,
        found: IntermediateType,
    },

    #[error("trying to perform the binary operation \"{operation:?}\" on type {operands_types:?}")]
    InvalidBinaryOperation {
        operation: Bytecode,
        operands_types: IntermediateType,
    },

    #[error("trying to perform the operation \"{operation:?}\" on type {operand_type:?}")]
    InvalidOperation {
        operation: Bytecode,
        operand_type: IntermediateType,
    },

    #[error("unssuported operation: {operation:?}")]
    UnssuportedOperation { operation: Bytecode },
}
