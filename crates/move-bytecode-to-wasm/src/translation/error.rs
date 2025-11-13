use std::rc::Rc;

use move_binary_format::file_format::{Bytecode, SignatureIndex};
use relooper::BranchMode;
use walrus::{LocalId, ValType};

use crate::{
    compilation_context::{CompilationContextError, ModuleId},
    error::{CompilationError, ICEError, ICEErrorKind},
    native_functions::error::NativeFunctionError,
};

use super::{
    intermediate_types::IntermediateType, table::FunctionId, types_stack::TypesStackError,
};

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Types stack error: {0}")]
    TypesStackError(#[from] TypesStackError),

    #[error("Compilation context error: {0}")]
    CompilationContextError(#[from] CompilationContextError),

    #[error("an error ocurred while generating a native funciton's code")]
    NativeFunctionError(#[from] NativeFunctionError),

    #[error(r#"function "{0}" not found in global functions table"#)]
    FunctionDefinitionNotFound(FunctionId),

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

    #[error("enum with index {enum_index} size not computed")]
    EnumSizeNotComputed { enum_index: u16 },

    #[error("calling field borrow mut without type instantiations")]
    DynamicFieldBorrowMutNoTypeInstantiations,

    #[error(
        "there was an error processing field borrow, expected struct from module {expected_module_id} and index {expected_struct_index} and got module {actual_module_id} and index {actual_struct_index}"
    )]
    BorrowFieldStructMismatch {
        expected_module_id: ModuleId,
        expected_struct_index: u16,
        actual_module_id: ModuleId,
        actual_struct_index: u16,
    },

    #[error("could not instantiate generic types")]
    CouldNotInstantiateGenericTypes,

    #[error("expected generic struct instance, found {0:?}")]
    ExpectedGenericStructInstance(IntermediateType),

    #[error("expected generic struct instance")]
    ExpectedGenericStructInstanceNotFound,

    #[error("branch target not found")]
    BranchTargetNotFound(u16),

    #[error("a translation error ocurred translating instruction {0:?}\n{1}")]
    AtInstruction(Bytecode, Rc<TranslationError>),

    #[error(r#"could not find original local "{0:?}" in function information"#)]
    LocalNotFound(LocalId),

    // Flow
    #[error("switch: more than one case returns a value, Move should have merged them")]
    SwitchMoreThanOneCase,

    #[error("IfElse result mismatch: then={0:?}, else={1:?}")]
    IfElseMismatch(Vec<ValType>, Vec<ValType>),

    #[error("unsupported branch mode: {0:?}")]
    UnssuportedBranchMode(BranchMode),

    #[error("jump table for IfElse flow should contain exactly 2 elements")]
    IfElseJumpTableBranchesNumberMismatch,

    // Unknown

    // TODO: identify concrete errors and add its corresponding enum variant
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl From<TranslationError> for CompilationError {
    fn from(value: TranslationError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Translation(value)))
    }
}
