use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{
    CompilationContext,
    compilation_context::ExternalModuleData,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    utils::snake_to_camel,
};

use super::vm_handled_datatypes::TxContext;

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
    let mut parameter_strings = Vec::new();
    for (i, signature_token) in signature.iter().enumerate() {
        if let Some(sol_name) = signature_token.sol_name(compilation_ctx) {
            parameter_strings.push(sol_name);
        }
        // This error should never happen. The panic! placed here is just a safeguard. If this code
        // gets executed means two things:
        // 1. A check failed in PublicFunction::check_signature_arguments.
        // 2. A `signer` type was found in a public function signature, but it is not the first
        //    argument.
        else if i != 0 {
            panic!(
                r#"function signature "{function_name}" can't be represented in Solidity's ABI format"#
            );
        }
    }
    let parameter_strings = parameter_strings.join(",");

    let function_name = snake_to_camel(function_name);

    selector(format!("{}({})", function_name, parameter_strings))
}

impl SolName for IntermediateType {
    fn sol_name(&self, compilation_ctx: &CompilationContext) -> Option<String> {
        match self {
            IntermediateType::IBool => Some(sol_data::Bool::SOL_NAME.to_string()),
            // According to the official documentation, enum types are encoded as uint8
            IntermediateType::IU8 | IntermediateType::IEnum(_) => {
                Some(sol_data::Uint::<8>::SOL_NAME.to_string())
            }
            IntermediateType::IU16 => Some(sol_data::Uint::<16>::SOL_NAME.to_string()),
            IntermediateType::IU32 => Some(sol_data::Uint::<32>::SOL_NAME.to_string()),
            IntermediateType::IU64 => Some(sol_data::Uint::<64>::SOL_NAME.to_string()),
            IntermediateType::IU128 => Some(sol_data::Uint::<128>::SOL_NAME.to_string()),
            IntermediateType::IU256 => Some(sol_data::Uint::<256>::SOL_NAME.to_string()),
            IntermediateType::IAddress => Some(sol_data::Address::SOL_NAME.to_string()),
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                inner.sol_name(compilation_ctx)
            }
            IntermediateType::IVector(inner) => inner
                .sol_name(compilation_ctx)
                .map(|sol_n| format!("{sol_n}[]")),
            IntermediateType::IStruct(index) => {
                let struct_ = compilation_ctx
                    .root_module_data
                    .structs
                    .get_by_index(*index)
                    .unwrap();
                Self::struct_fields_sol_name(struct_, compilation_ctx)
            }
            IntermediateType::IGenericStructInstance(index, types) => {
                let struct_ = compilation_ctx
                    .root_module_data
                    .structs
                    .get_by_index(*index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);

                Self::struct_fields_sol_name(&struct_instance, compilation_ctx)
            }
            IntermediateType::ISigner => None,
            IntermediateType::ITypeParameter(_) => None,
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
            } => {
                let external_data = compilation_ctx
                    .get_external_module_data(module_id, identifier)
                    .unwrap();
                match external_data {
                    // TxContext should not be part of the function signature, since it is injected
                    // by the VM.
                    ExternalModuleData::Struct(_)
                        if TxContext::struct_is_tx_context(module_id, identifier) =>
                    {
                        Some("".to_owned())
                    }
                    ExternalModuleData::Struct(istruct) => {
                        Self::struct_fields_sol_name(istruct, compilation_ctx)
                    }
                    ExternalModuleData::Enum(_ienum) => todo!(),
                }
            }
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
        compilation_context::ModuleData, test_compilation_context, test_tools::build_module,
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
                (None, IntermediateType::IStruct(1)),
            ],
            HashMap::new(),
        );

        let struct_2 = IStruct::new(
            StructDefinitionIndex::new(1),
            "TestStruct2".to_string(),
            vec![
                (None, IntermediateType::IU32),
                (None, IntermediateType::IU128),
            ],
            HashMap::new(),
        );

        let mut module_data = ModuleData::default();

        let module_structs = vec![struct_1, struct_2];
        module_data.structs.structs = module_structs;

        let signature: &[IntermediateType] = &[
            IntermediateType::IStruct(0),
            IntermediateType::IVector(Box::new(IntermediateType::IStruct(1))),
        ];

        compilation_ctx.root_module_data = &module_data;
        assert_eq!(
            move_signature_to_abi_selector("test_struct", signature, &compilation_ctx),
            selector(
                "testStruct((address,uint32[],uint128[],bool,uint8,uint16,uint32,uint64,uint128,uint256,(uint32,uint128)),(uint32,uint128)[])"
            )
        );
    }

    #[test]
    #[should_panic(
        expected = r#"function signature "test_invalid_signature" can't be represented in Solidity's ABI format"#
    )]
    fn test_move_signature_to_abi_selector_invalid_1() {
        let (_, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let signature: &[IntermediateType] = &[
            IntermediateType::IU64,
            IntermediateType::ISigner,
            IntermediateType::IAddress,
            IntermediateType::IU64,
        ];
        move_signature_to_abi_selector("test_invalid_signature", signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"function signature "test_invalid_signature" can't be represented in Solidity's ABI format"#
    )]
    fn test_move_signature_to_abi_selector_invalid_2() {
        let (_, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let signature: &[IntermediateType] = &[
            IntermediateType::ISigner,
            IntermediateType::IAddress,
            IntermediateType::IU64,
            IntermediateType::ISigner,
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
            IntermediateType::ISigner,
        ];
        move_signature_to_abi_selector("test_invalid_signature", signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"function signature "test_invalid_signature" can't be represented in Solidity's ABI format"#
    )]
    fn test_move_signature_to_abi_selector_invalid_3() {
        let (_, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let signature: &[IntermediateType] = &[
            IntermediateType::IAddress,
            IntermediateType::IU64,
            IntermediateType::IVector(Box::new(IntermediateType::ISigner)),
        ];
        move_signature_to_abi_selector("test_invalid_signature", signature, &compilation_ctx);
    }

    #[test]
    #[should_panic(
        expected = r#"function signature "test_invalid_signature" can't be represented in Solidity's ABI format"#
    )]
    fn test_move_signature_to_abi_selector_invalid_4() {
        let (_, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let signature: &[IntermediateType] = &[
            IntermediateType::IAddress,
            IntermediateType::IU64,
            IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                IntermediateType::ISigner,
            )))),
        ];
        move_signature_to_abi_selector("test_invalid_signature", signature, &compilation_ctx);
    }
}
