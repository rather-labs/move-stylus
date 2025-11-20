use std::rc::Rc;

use crate::{
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    runtime::error::RuntimeFunctionError,
    storage::error::StorageError,
    translation::{
        intermediate_types::{IntermediateType, error::IntermediateTypeError},
        table::FunctionId,
    },
    vm_handled_types::error::VmHandledTypeError,
};

#[derive(Debug, thiserror::Error)]
pub enum NativeFunctionError {
    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("an storage error ocurred while translating a function")]
    Storage(#[source] Rc<StorageError>),

    #[error("compilation context error ocurred while processing a native function")]
    CompilationContext(#[from] CompilationContextError),

    #[error("abi error ocurred while processing a native function")]
    Abi(#[source] Rc<AbiError>),

    #[error("an error ocurred while processing an intermediate type")]
    IntermediateType(#[from] IntermediateTypeError),

    #[error("an error ocurred while processing a vm handled type")]
    VmHandledType(#[from] VmHandledTypeError),

    #[error(r#"host function "{0}" not supported yet"#)]
    HostFunctionNotSupported(String),

    #[error(r#"native function "{0}::{1}" not supported yet"#)]
    NativeFunctionNotSupported(ModuleId, String),

    #[error(r#"generic native function "{0}::{1}" not supported yet"#)]
    GenericdNativeFunctionNotSupported(ModuleId, String),

    #[error(r#"missing special attributes for external call "{0}::{1}""#)]
    NotExternalCall(ModuleId, String),

    #[error(r#"contract call function "{0}::{1}" has no arguments"#)]
    ContractCallFunctionNoArgs(ModuleId, String),

    #[error(r#"external contract call function "{0}" must return a ContractCallResult<T> or ContractCallEmptyResult with a single type parameter"#)]
    ContractCallFunctionInvalidReturn(FunctionId),

    #[error(r#"found an struct "{0:?}" that is not a named id in named_ids array"#)]
    ContractCallInvalidNamedId(IntermediateType),

    #[error(
        r#"called get_generic_function_name for function "{0}::{1}" with no generic parameters"#
    )]
    GetGenericFunctionNameNoGenerics(ModuleId, String),

    #[error(r#"there was an error linking "{0}" function, expected IStruct, found {1:?}"#)]
    WrongGenericType(String, IntermediateType),

    #[error("key type not supported {0:?}")]
    DynamicFieldWrongKeyType(IntermediateType),

    // Emit function section
    #[error(r#"trying to emit log with the struct {0} which is not an event"#)]
    EmitFunctionNoEvent(String),

    #[error(r#"invalid event field {0:?}"#)]
    EmitFunctionInvalidEventField(IntermediateType),

    #[error(
        "there was an error instantiating an emit event function: vector does not have abi encoded data"
    )]
    EmitFunctionInvalidVectorData,

    // revert function section
    #[error(r#"trying to revert with the struct "{0}" which is not an error"#)]
    RevertFunctionNoError(String),
}
