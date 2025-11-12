use std::rc::Rc;

use crate::{
    abi_types::error::AbiError,
    compilation_context::{CompilationContextError, ModuleId},
    translation::table::FunctionId,
};

#[derive(Debug, thiserror::Error)]
pub enum NativeFunctionError {
    #[error(r#"host function "{0}" not supported yet"#)]
    HostFunctionNotSupported(String),

    #[error(r#"native function "{0}::{1}" not supported yet"#)]
    NativeFunctionNotSupported(ModuleId, String),

    #[error(r#"generic native function "{0}::{1}" not supported yet"#)]
    GenericdNativeFunctionNotSupported(ModuleId, String),

    #[error("compilation context error ocurred while processing a native function")]
    CompilationContext(#[from] CompilationContextError),

    #[error("abi error ocurred while processing a native function")]
    Abi(Rc<AbiError>),

    #[error(r#"missing special attributes for external call "{0}::{1}""#)]
    NotExternalCall(ModuleId, String),

    #[error(r#"contract call function "{0}::{1}" has no arguments"#)]
    ContractCallFunctionNoArgs(ModuleId, String),

    #[error(r#"external contract call function "{0}" must return a ContractCallResult<T> or ContractCallEmptyResult with a single type parameter"#)]
    ContractCallFunctionInvalidReturn(FunctionId),
}
