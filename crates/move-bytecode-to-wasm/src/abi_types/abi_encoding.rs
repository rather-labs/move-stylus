use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{
    CompilationContext,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    vm_handled_types::{
        VmHandledType,
        bytes::Bytes,
        named_id::NamedId,
        storage_object::{FrozenStorageObject, OwnedStorageObject, SharedStorageObject},
        string::String_,
        tx_context::TxContext,
        uid::Uid,
    },
};

use super::{error::AbiError, sol_name::SolName};

pub type AbiFunctionSelector = [u8; 4];

pub fn selector<T: AsRef<[u8]>>(bytes: T) -> Result<AbiFunctionSelector, AbiError> {
    keccak256(bytes)[..4]
        .try_into()
        .map_err(|_| AbiError::InvalidSelectorSize)
}

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector<F>(
    function_name: &str,
    signature: &[IntermediateType],
    compilation_ctx: &CompilationContext,
    case_callback: F,
) -> Result<AbiFunctionSelector, AbiError>
where
    F: Fn(&str) -> String,
{
    let parameter_strings = signature
        .iter()
        .map(|s| solidity_name(s, compilation_ctx))
        .collect::<Result<Vec<Option<String>>, AbiError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join(",");

    let function_name = case_callback(function_name);

    selector(format!("{function_name}({parameter_strings})"))
}

fn solidity_name(
    argument: &IntermediateType,
    compilation_ctx: &CompilationContext,
) -> Result<Option<String>, AbiError> {
    let sol_name = match argument {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress => argument.sol_name(compilation_ctx)?,
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
            let enum_ = compilation_ctx.get_enum_by_intermediate_type(argument)?;

            if enum_.is_simple {
                argument.sol_name(compilation_ctx)?
            } else {
                None
            }
        }
        IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
            solidity_name(inner, compilation_ctx)?
        }
        IntermediateType::IVector(inner) => {
            solidity_name(inner, compilation_ctx)?.map(|sol_n| format!("{sol_n}[]"))
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } if TxContext::is_vm_type(module_id, *index, compilation_ctx)? => None,
        IntermediateType::IStruct {
            module_id, index, ..
        } if String_::is_vm_type(module_id, *index, compilation_ctx)? => {
            argument.sol_name(compilation_ctx)?
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } if Bytes::is_vm_type(module_id, *index, compilation_ctx)? => {
            argument.sol_name(compilation_ctx)?
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
            Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string())
        }
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            ..
        } if OwnedStorageObject::is_vm_type(module_id, *index, compilation_ctx)?
            || SharedStorageObject::is_vm_type(module_id, *index, compilation_ctx)?
            || FrozenStorageObject::is_vm_type(module_id, *index, compilation_ctx)? =>
        {
            if types.len() != 1 {
                return Err(AbiError::StorageObjectHasInvalidType);
            }

            let Some(inner_type) = types.first() else {
                return Err(AbiError::StorageObjectHasInvalidType);
            };

            solidity_name(inner_type, compilation_ctx)?
        }
        IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
            let struct_ = compilation_ctx.get_struct_by_intermediate_type(argument)?;

            if struct_.has_key {
                sol_name_storage_ids(&struct_, compilation_ctx)?
            } else {
                struct_fields_sol_name(&struct_, compilation_ctx)?
            }
        }
        IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => None,
    };

    Ok(sol_name)
}

fn sol_name_storage_ids(
    struct_: &IStruct,
    compilation_ctx: &CompilationContext,
) -> Result<Option<String>, AbiError> {
    match struct_.fields.first() {
        Some(IntermediateType::IStruct {
            module_id, index, ..
        }) if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
            Ok(Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string()))
        }
        Some(IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }) if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => Ok(None),

        _ => Err(AbiError::ExpectedUIDOrNamedId(struct_.identifier)),
    }
}

#[inline]
pub fn struct_fields_sol_name(
    struct_: &IStruct,
    compilation_ctx: &CompilationContext,
) -> Result<Option<String>, AbiError> {
    Ok(struct_
        .fields
        .iter()
        .map(|field| solidity_name(field, compilation_ctx))
        .collect::<Result<Option<Vec<String>>, AbiError>>()?
        .map(|fields| fields.join(","))
        .map(|fields| format!("({fields})")))
}
