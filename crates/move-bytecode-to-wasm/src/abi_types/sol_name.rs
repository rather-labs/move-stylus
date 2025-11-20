use alloy_sol_types::{SolType, sol_data};

use crate::{
    CompilationContext,
    translation::intermediate_types::IntermediateType,
    vm_handled_types::{VmHandledType, string::String_},
};

use super::error::AbiError;

pub trait SolName {
    /// Returns the corresponding type name in solidity in case it exist
    fn sol_name(&self, compilation_ctx: &CompilationContext) -> Result<Option<String>, AbiError>;
}

impl SolName for IntermediateType {
    fn sol_name(&self, compilation_ctx: &CompilationContext) -> Result<Option<String>, AbiError> {
        let name = match self {
            IntermediateType::IBool => Some(sol_data::Bool::SOL_NAME.to_string()),
            IntermediateType::IU8 => Some(sol_data::Uint::<8>::SOL_NAME.to_string()),
            IntermediateType::IU16 => Some(sol_data::Uint::<16>::SOL_NAME.to_string()),
            IntermediateType::IU32 => Some(sol_data::Uint::<32>::SOL_NAME.to_string()),
            IntermediateType::IU64 => Some(sol_data::Uint::<64>::SOL_NAME.to_string()),
            IntermediateType::IU128 => Some(sol_data::Uint::<128>::SOL_NAME.to_string()),
            IntermediateType::IU256 => Some(sol_data::Uint::<256>::SOL_NAME.to_string()),
            IntermediateType::IAddress => Some(sol_data::Address::SOL_NAME.to_string()),
            // According to the official documentation, enum types are encoded as uint8
            // TODO: check if the enum is simple
            IntermediateType::IEnum { .. } => Some(sol_data::Uint::<8>::SOL_NAME.to_string()),
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                inner.sol_name(compilation_ctx)?
            }
            IntermediateType::IVector(inner) => inner
                .sol_name(compilation_ctx)?
                .map(|sol_n| format!("{sol_n}[]")),
            IntermediateType::IStruct {
                module_id, index, ..
            } if String_::is_vm_type(module_id, *index, compilation_ctx)? => {
                Some(sol_data::String::SOL_NAME.to_string())
            }
            // Depening on the contect, structs can be interpreted in different ways (i.e events vs
            // function selector)
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                None
            }
            IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => None,
            IntermediateType::IGenericEnumInstance { .. } => None,
        };

        Ok(name)
    }
}
