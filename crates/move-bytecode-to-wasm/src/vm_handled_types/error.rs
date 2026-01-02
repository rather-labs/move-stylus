use move_symbol_pool::Symbol;

use crate::{compilation_context::CompilationContextError, runtime::error::RuntimeFunctionError};

#[derive(Debug, thiserror::Error)]
pub enum VmHandledTypeError {
    #[error("compilation context error")]
    CompilationContextError(#[from] CompilationContextError),

    #[error("runtime function error")]
    RuntimeFunctionError(#[from] RuntimeFunctionError),

    #[error(r#"invalid "{0}" found, only the one from the stylus framework is valid"#)]
    InvalidFrameworkType(Symbol),

    #[error(r#"invalid "{0}" found, only the one from the standard library is valid"#)]
    InvalidStdLibType(Symbol),
}
