//! This module is in charge of injecting the datatypes that can only be created or are
//! automatically injected by the VM, such as the primitive type Signer or the TxContext struct
//! from the stylus framework.
pub mod bytes;
pub mod contract_call_result;
pub mod dynamic_fields;
pub mod error;
pub mod named_id;
pub mod signer;
pub mod string;
pub mod table;
pub mod tx_context;
pub mod uid;

use error::VmHandledTypeError;
use walrus::{InstrSeqBuilder, Module};

use crate::{
    CompilationContext, compilation_context::ModuleId,
    translation::intermediate_types::IntermediateType,
};

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
    fn is_vm_type(
        module_id: &ModuleId,
        index: u16,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError>;
}

/// Auxiliary funtion that returns true if an `IntermediateType` is a valid UID or NamedId struct
pub fn is_uid_or_named_id(
    itype: &IntermediateType,
    compilation_ctx: &CompilationContext,
) -> Result<bool, VmHandledTypeError> {
    match itype {
        IntermediateType::IStruct {
            module_id, index, ..
        }
        | IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } => Ok(uid::Uid::is_vm_type(module_id, *index, compilation_ctx)?
            || named_id::NamedId::is_vm_type(module_id, *index, compilation_ctx)?),
        _ => Ok(false),
    }
}
