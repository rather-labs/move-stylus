use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{
    CompilationContext,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    utils::snake_to_camel,
    vm_handled_types::{
        VmHandledType, named_id::NamedId, string::String_, tx_context::TxContext, uid::Uid,
    },
};

use super::sol_name::SolName;

pub type AbiFunctionSelector = [u8; 4];

fn selector<T: AsRef<[u8]>>(bytes: T) -> AbiFunctionSelector {
    keccak256(bytes)[..4].try_into().unwrap()
}

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector(
    function_name: &str,
    signature: &[IntermediateType],
    compilation_ctx: &CompilationContext,
) -> AbiFunctionSelector {
    let parameter_strings = signature
        .iter()
        .filter_map(|s| solidity_name(s, compilation_ctx))
        .collect::<Vec<String>>()
        .join(",");

    let function_name = snake_to_camel(function_name);

    selector(format!("{}({})", function_name, parameter_strings))
}

fn solidity_name(
    argument: &IntermediateType,
    compilation_ctx: &CompilationContext,
) -> Option<String> {
    match argument {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress => argument.sol_name(compilation_ctx),
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
            let enum_ = compilation_ctx
                .get_enum_by_intermediate_type(argument)
                .unwrap();
            if enum_.is_simple {
                argument.sol_name(compilation_ctx)
            } else {
                None
            }
        }
        IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
            solidity_name(inner, compilation_ctx)
        }
        IntermediateType::IVector(inner) => {
            solidity_name(inner, compilation_ctx).map(|sol_n| format!("{sol_n}[]"))
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } if TxContext::is_vm_type(module_id, *index, compilation_ctx) => None,
        IntermediateType::IStruct {
            module_id, index, ..
        } if String_::is_vm_type(module_id, *index, compilation_ctx) => {
            argument.sol_name(compilation_ctx)
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } if Uid::is_vm_type(module_id, *index, compilation_ctx) => {
            Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string())
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } => {
            let struct_ = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .unwrap();

            if struct_.has_key {
                sol_name_storage_ids(struct_, compilation_ctx)
            } else {
                struct_fields_sol_name(struct_, compilation_ctx)
            }
        }
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            ..
        } => {
            let struct_ = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .unwrap();
            let struct_instance = struct_.instantiate(types);

            if struct_instance.has_key {
                sol_name_storage_ids(struct_, compilation_ctx)
            } else {
                struct_fields_sol_name(&struct_instance, compilation_ctx)
            }
        }
        IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => None,
    }
}

fn sol_name_storage_ids(struct_: &IStruct, compilation_ctx: &CompilationContext) -> Option<String> {
    match struct_.fields.first() {
        Some(IntermediateType::IStruct {
            module_id, index, ..
        }) if Uid::is_vm_type(module_id, *index, compilation_ctx) => {
            Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string())
        }
        Some(IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }) if NamedId::is_vm_type(module_id, *index, compilation_ctx) => None,

        _ => panic!(
            "expected stylus::object::UID or stylus::object::NamedId as first field in {} struct (it has key ability)",
            struct_.identifier
        ),
    }
}

#[inline]
fn struct_fields_sol_name(
    struct_: &IStruct,
    compilation_ctx: &CompilationContext,
) -> Option<String> {
    struct_
        .fields
        .iter()
        .map(|field| solidity_name(field, compilation_ctx))
        .collect::<Option<Vec<String>>>()
        .map(|fields| fields.join(","))
        .map(|fields| format!("({fields})"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use move_binary_format::file_format::StructDefinitionIndex;

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
    fn test_move_signature_to_abi_selector() {
        let (_, allocator_func, memory_id) = build_module(None);
        let mut compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let signature: &[IntermediateType] = &[IntermediateType::IU8, IntermediateType::IU16];
        assert_eq!(
            move_signature_to_abi_selector("test", signature, &compilation_ctx),
            selector("test(uint8,uint16)")
        );

        let signature: &[IntermediateType] = &[IntermediateType::IAddress, IntermediateType::IU256];
        assert_eq!(
            move_signature_to_abi_selector("transfer", signature, &compilation_ctx),
            selector("transfer(address,uint256)")
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::ISigner,
            IntermediateType::IAddress,
            IntermediateType::IU64,
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("set_owner", signature, &compilation_ctx),
            selector("setOwner(address,uint64,bool[])")
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Box::new(IntermediateType::IU128)),
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature, &compilation_ctx),
            selector("testArray(uint128[],bool[])")
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                IntermediateType::IU128,
            )))),
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature, &compilation_ctx),
            selector("testArray(uint128[][],bool[])")
        );

        let struct_1 = IStruct::new(
            StructDefinitionIndex::new(0),
            "TestStruct".to_string(),
            vec![
                (None, IntermediateType::IAddress),
                (
                    None,
                    IntermediateType::IVector(Box::new(IntermediateType::IU32)),
                ),
                (
                    None,
                    IntermediateType::IVector(Box::new(IntermediateType::IU128)),
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
            "TestStruct2".to_string(),
            vec![
                (None, IntermediateType::IU32),
                (None, IntermediateType::IU128),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();

        let module_structs = vec![struct_1, struct_2];
        module_data.structs.structs = module_structs;

        let signature: &[IntermediateType] = &[
            IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 0,
                vm_handled_struct: VmHandledStruct::None,
            },
            IntermediateType::IVector(Box::new(IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 1,
                vm_handled_struct: VmHandledStruct::None,
            })),
        ];

        compilation_ctx.root_module_data = &module_data;
        assert_eq!(
            move_signature_to_abi_selector("test_struct", signature, &compilation_ctx),
            selector(
                "testStruct((address,uint32[],uint128[],bool,uint8,uint16,uint32,uint64,uint128,uint256,(uint32,uint128)),(uint32,uint128)[])"
            )
        );
    }
}
