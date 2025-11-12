use crate::compilation_context::{CompilationContextError, ModuleId};

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

    // TODO: This should be a code error
    #[error(r#"missing special attributes for external call "{0}::{1}""#)]
    NotExternalCall(ModuleId, String),
}
