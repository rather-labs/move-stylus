use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{
    CompilationContext,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    utils::snake_to_camel,
    vm_handled_types::{VmHandledType, tx_context::TxContext},
};

pub type AbiFunctionSelector = [u8; 4];

fn selector<T: AsRef<[u8]>>(bytes: T) -> AbiFunctionSelector {
    keccak256(bytes)[..4].try_into().unwrap()
}

pub trait SolName {
    /// Returns the corresponding type name in solidity in case it exist
    fn sol_name(&self, compilation_ctx: &CompilationContext) -> Option<String>;
}

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector<T: SolName>(
    function_name: &str,
    signature: &[T],
    compilation_ctx: &CompilationContext,
) -> AbiFunctionSelector {
    let parameter_strings = signature
        .iter()
        .filter_map(|s| s.sol_name(compilation_ctx))
        .collect::<Vec<String>>()
        .join(",");

    let function_name = snake_to_camel(function_name);

    selector(format!("{}({})", function_name, parameter_strings))
}

impl SolName for IntermediateType {
    fn sol_name(&self, compilation_ctx: &CompilationContext) -> Option<String> {
        match self {
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
            IntermediateType::IEnum(_) => Some(sol_data::Uint::<8>::SOL_NAME.to_string()),
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                inner.sol_name(compilation_ctx)
            }
            IntermediateType::IVector(inner) => inner
                .sol_name(compilation_ctx)
                .map(|sol_n| format!("{sol_n}[]")),
            IntermediateType::IStruct { module_id, index }
                if TxContext::is_vm_type(module_id, *index, compilation_ctx) =>
            {
                None
            }
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                if struct_.saved_in_storage {
                    Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string())
                } else {
                    Self::struct_fields_sol_name(struct_, compilation_ctx)
                }
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);

                if struct_instance.saved_in_storage {
                    Some(sol_data::FixedBytes::<32>::SOL_NAME.to_string())
                } else {
                    Self::struct_fields_sol_name(&struct_instance, compilation_ctx)
                }
            }
            IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => None,
        }
    }
}

impl IntermediateType {
    #[inline]
    fn struct_fields_sol_name(
        struct_: &IStruct,
        compilation_ctx: &CompilationContext,
    ) -> Option<String> {
        struct_
            .fields
            .iter()
            .map(|field| field.sol_name(compilation_ctx))
            .collect::<Option<Vec<String>>>()
            .map(|fields| fields.join(","))
            .map(|fields| format!("({fields})"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use move_binary_format::file_format::StructDefinitionIndex;

    use crate::{
        compilation_context::{ModuleData, ModuleId},
        test_compilation_context,
        test_tools::build_module,
        translation::intermediate_types::structs::IStruct,
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
                    },
                ),
            ],
            HashMap::new(),
            false,
            false,
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
            false,
        );

        let mut module_data = ModuleData::default();

        let module_structs = vec![struct_1, struct_2];
        module_data.structs.structs = module_structs;

        let signature: &[IntermediateType] = &[
            IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 0,
            },
            IntermediateType::IVector(Box::new(IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 1,
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
