use std::rc::Rc;

use move_binary_format::file_format::{Bytecode, SignatureIndex};
use move_symbol_pool::Symbol;
use relooper::BranchMode;
use walrus::{LocalId, ValType};

use crate::{
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    error::{CompilationError, ICEError, ICEErrorKind},
    native_functions::error::NativeFunctionError,
    runtime::error::RuntimeFunctionError,
    vm_handled_types::error::VmHandledTypeError,
};

use super::{
    intermediate_types::{IntermediateType, error::IntermediateTypeError},
    table::{FunctionId, FunctionTableError},
    types_stack::TypesStackError,
};

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("types stack error")]
    TypesStackError(#[from] TypesStackError),

    #[error("compilation context error")]
    CompilationContextError(#[from] CompilationContextError),

    #[error("an error ocurred while generating a native function's code")]
    NativeFunctionError(#[from] NativeFunctionError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("an abi error ocurred while translating a function")]
    AbiEncoding(#[from] AbiError),

    #[error("an error ocurred while processing an intermediate type")]
    IntermediateType(#[from] IntermediateTypeError),

    #[error("an error ocurred while using the function table")]
    FunctionTable(#[from] FunctionTableError),

    #[error("an error ocurred while processing a vm handled type")]
    VmHandledType(#[from] VmHandledTypeError),

    #[error("a translation error ocurred translating instruction {0:?}")]
    AtInstruction(Bytecode, #[source] Rc<TranslationError>),

    #[error(r#"function "{0}" not found in global functions table"#)]
    FunctionDefinitionNotFound(FunctionId),

    #[error("could not process byte array, wrong size")]
    CouldNotProcessByteArray,

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

    #[error("found reference inside enum with index {0}")]
    FoundReferenceInsideEnum(u16),

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

    #[error("processing CallGeneric bytecode without type instantiations")]
    CallGenericWihtoutTypeInstantiations,

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

    #[error(r#"could not find original local "{0:?}" in function information"#)]
    LocalNotFound(LocalId),

    #[error("trying to peform an operation on a type parameter")]
    FoundTypeParameter,

    #[error("entry function not found in function table")]
    EntryFunctionNotFound,

    #[error("entry function WASM ID not found in function table")]
    EntryFunctionWasmIdNotFound,

    // Flow
    #[error("switch: more than one case returns a value, Move should have merged them")]
    SwitchMoreThanOneCase,

    #[error("switch: all cases must be Simple in a Switch flow")]
    SwitchCasesNotSimple,

    #[error("IfElse result mismatch: then={0:?}, else={1:?}")]
    IfElseMismatch(Vec<ValType>, Vec<ValType>),

    #[error("unsupported branch mode: {0:?}")]
    UnssuportedBranchMode(BranchMode),

    #[error("jump table for IfElse flow should contain exactly 2 elements")]
    IfElseJumpTableBranchesNumberMismatch,

    #[error("only Simple flow has label")]
    NotSimpleFlowWithLabel,

    #[error("jump table not found while translating Switch flow!")]
    JumpTableNotFound,

    #[error("Missing block id for jump-table label")]
    MissingBlockIdForJumpTableLabel,

    #[error("block context not found")]
    BlockContextNotFound,

    // Misc
    #[error("invalid intermediate type {0:?} found in unpack function")]
    InvalidTypeInUnpackFunction(IntermediateType),

    #[error("constant data not consumed")]
    ConstantDataNotConsumed,

    #[error("{field_id} not found in {struct_identifier}")]
    StructFieldNotFound {
        field_id: usize,
        struct_identifier: String,
    },

    #[error("{field_id} offset not found in {struct_identifier}")]
    StructFieldOffsetNotFound {
        field_id: usize,
        struct_identifier: String,
    },

    #[error("multiple WASM return values not supported, found {0} return values")]
    MultipleWasmReturnValues(usize),

    #[error("generic function {0}::{1} has no type instantiations")]
    GenericFunctionNoTypeInstantiations(ModuleId, Symbol),
}

impl From<TranslationError> for CompilationError {
    fn from(value: TranslationError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Translation(value)))
    }
}
