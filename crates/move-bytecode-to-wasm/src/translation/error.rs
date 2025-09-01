use move_binary_format::file_format::{Bytecode, SignatureIndex};

use crate::compilation_context::{CompilationContextError, ModuleId};

use super::{intermediate_types::IntermediateType, types_stack::TypesStackError};

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Types stack error: {0}")]
    TypesStackError(#[from] TypesStackError),

    #[error("Compilation context error: {0}")]
    CompilationContextError(#[from] CompilationContextError),

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

    #[error("unsupported operation: {operation:?}")]
    UnsupportedOperation { operation: Bytecode },

    #[error(
        "unable to perform \"{operation:?}\" on types {operand1:?} and {operand2:?}, expected the same type on types stack"
    )]
    OperationTypeMismatch {
        operand1: IntermediateType,
        operand2: IntermediateType,
        operation: Bytecode,
    },

    #[error(
        "the signature index {signature_index:?} does not point to a valid signature for this operation, it contains {number:?} types but only one is expected"
    )]
    VectorInnerTypeNumberError {
        signature_index: SignatureIndex,
        number: usize,
    },

    #[error("found reference inside struct with index {struct_index}")]
    FoundReferenceInsideStruct { struct_index: u16 },

    #[error(
        "found type parameter inside struct with index {struct_index} and type parameter index {type_parameter_index}"
    )]
    FoundTypeParameterInsideStruct {
        struct_index: u16,
        type_parameter_index: u16,
    },

    #[error("found unknown type inside struct with index {struct_index}")]
    FoundUnknownTypeInsideStruct { struct_index: u16 },

    #[error(r#"found external struct "{identifier}" from module "{module_id}" inside struct when unpacking"#)]
    UnpackingStructFoundExternalStruct {
        identifier: String,
        module_id: ModuleId,
    },

    #[error("found reference inside enum with index {enum_index}")]
    FoundReferenceInsideEnum { enum_index: u16 },

    #[error(
        "trying to pack an enum variant using the generic enum definition with index {enum_index}"
    )]
    PackingGenericEnumVariant { enum_index: u16 },

    #[error(
        "found type parameter inside enum variant with index {variant_index} and enum index {enum_index}"
    )]
    FoundTypeParameterInsideEnumVariant { enum_index: u16, variant_index: u16 },

    #[error(
        "found unknown type inside enum variant with index {variant_index} and enum index {enum_index}"
    )]
    FoundUnknownTypeInsideEnumVariant { enum_index: u16, variant_index: u16 },

    // TODO: identify concrete errors and add its corresponding enum variant
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}
