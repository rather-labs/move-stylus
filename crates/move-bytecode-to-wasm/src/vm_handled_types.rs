//! This module is in charge of injecting the datatypes that can only be created or are
//! automatically injected by the VM, such as the primitive type Signer or the TxContext struct
//! from the stylus framework.

pub mod signer;
pub mod tx_context;
pub mod uid;

use walrus::{InstrSeqBuilder, Module};

use crate::{CompilationContext, compilation_context::ModuleId};

pub trait VmHandledType {
    const IDENTIFIER: &str;

    /// Injects the VM Handled type
    fn inject(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    );

    /// Checks if the type is the reserved one or one declared by the user with the same name.
    ///
    /// Panics if the type is not the vm one
    fn is_vm_type(module_id: &ModuleId, index: u16, compilation_ctx: &CompilationContext) -> bool;
}
