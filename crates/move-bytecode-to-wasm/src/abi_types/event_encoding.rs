use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data::FixedBytes};

use crate::{
    CompilationContext,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    vm_handled_types::{
        VmHandledType, bytes::Bytes, id::Id, string::String_, tx_context::TxContext,
    },
};

use super::error::AbiError;
use super::sol_name::SolName;

type AbiEventSignatureHash = [u8; 32];

pub fn move_signature_to_event_signature_hash(
    struct_: &IStruct,
    compilation_ctx: &CompilationContext,
) -> Result<AbiEventSignatureHash, AbiError> {
    let field_strings = struct_
        .fields
        .iter()
        .map(|s| solidity_name(s, compilation_ctx))
        .collect::<Result<Vec<Option<String>>, AbiError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join(",");

    Ok(*keccak256(
        format!("{}({})", struct_.identifier, field_strings).as_bytes(),
    ))
}

fn solidity_name(
    argument: &IntermediateType,
    compilation_ctx: &CompilationContext,
) -> Result<Option<String>, AbiError> {
    Ok(match argument {
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
        } if Id::is_vm_type(module_id, *index, compilation_ctx)? => {
            Some(FixedBytes::<32>::SOL_NAME.to_string())
        }
        IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
            let struct_ = compilation_ctx.get_struct_by_intermediate_type(argument)?;

            struct_fields_sol_name(&struct_, compilation_ctx)?
        }
        IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => None,
    })
}

#[inline]
pub fn struct_fields_sol_name(
    struct_: &IStruct,
    compilation_ctx: &CompilationContext,
) -> Result<Option<String>, AbiError> {
    Ok(struct_
        .fields
        .iter()
        .map(|field| SolName::sol_name(field, compilation_ctx))
        .collect::<Result<Option<Vec<String>>, AbiError>>()?
        .map(|fields| fields.join(","))
        .map(|fields| format!("({fields})")))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use move_binary_format::file_format::StructDefinitionIndex;
    use rstest::rstest;

    use crate::{
        compilation_context::{ModuleData, ModuleId},
        test_compilation_context,
        test_tools::build_module,
        translation::intermediate_types::{
            VmHandledStruct,
            structs::{IStruct, IStructType},
        },
    };

    use super::*;

    #[test]
    fn test_move_signature_to_event_signature_hash_nested() {
        let (_, allocator_func, memory_id, ctx_globals) = build_module(None);
        let mut compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let struct_1 = IStruct::new(
            StructDefinitionIndex::new(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IAddress),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
                ),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
                ),
                (None, IntermediateType::IBool),
                (None, IntermediateType::IU8),
                (None, IntermediateType::IU16),
                (None, IntermediateType::IU32),
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (None, IntermediateType::IU256),
                (
                    None,
                    IntermediateType::IStruct {
                        module_id: ModuleId::default(),
                        index: 1,
                        vm_handled_struct: VmHandledStruct::None,
                    },
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let struct_2 = IStruct::new(
            StructDefinitionIndex::new(1),
            "TestStruct2",
            vec![
                (None, IntermediateType::IU32),
                (None, IntermediateType::IU128),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();

        let module_structs = vec![struct_1.clone(), struct_2];
        module_data.structs.structs = module_structs;

        compilation_ctx.root_module_data = &module_data;
        assert_eq!(
            move_signature_to_event_signature_hash(&struct_1, &compilation_ctx).unwrap(),
            *keccak256(
                "TestStruct(address,uint32[],uint128[],bool,uint8,uint16,uint32,uint64,uint128,uint256,(uint32,uint128))"
            )
        );
    }

    #[rstest]
    #[case(
        &IStruct::new(
            StructDefinitionIndex::new(0),
            "Approval",
            vec![
                (None, IntermediateType::IAddress),
                (None, IntermediateType::IAddress),
                (None, IntermediateType::IU256),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        ),
        *keccak256(b"Approval(address,address,uint256)")
    )]
    #[case(
        &IStruct::new(
            StructDefinitionIndex::new(0),
            "Transfer",
            vec![
                (None, IntermediateType::IAddress),
                (None, IntermediateType::IAddress),
                (None, IntermediateType::IU256),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        ),
        *keccak256(b"Transfer(address,address,uint256)")
    )]
    #[case(
        &IStruct::new(
            StructDefinitionIndex::new(0),
            "Empty",
            vec![],
            HashMap::new(),
            false,
            IStructType::Common,
        ),
        *keccak256(b"Empty()")
    )]
    fn test_move_signature_to_event_signature_hash(
        #[case] event_struct: &IStruct,
        #[case] expected: AbiEventSignatureHash,
    ) {
        let (_, allocator_func, memory_id, ctx_globals) = build_module(None);
        let mut compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let mut module_data = ModuleData::default();

        let module_structs = vec![event_struct.clone()];
        module_data.structs.structs = module_structs;

        compilation_ctx.root_module_data = &module_data;

        assert_eq!(
            move_signature_to_event_signature_hash(event_struct, &compilation_ctx).unwrap(),
            expected
        );
    }
}
