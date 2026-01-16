use crate::{
    CompilationContext, translation::intermediate_types::IntermediateType, utils::snake_to_camel,
};

use super::{
    abi_encoding::{self, AbiFunctionSelector},
    error::AbiError,
};

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector(
    function_name: &str,
    signature: &[IntermediateType],
    compilation_ctx: &CompilationContext,
) -> Result<AbiFunctionSelector, AbiError> {
    abi_encoding::move_signature_to_abi_selector(
        function_name,
        signature,
        compilation_ctx,
        snake_to_camel,
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

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

    use super::abi_encoding::selector;
    use super::*;

    #[test]
    fn test_move_signature_to_abi_selector() {
        let (_, allocator_func, memory_id, ctx_globals) = build_module(None);
        let mut compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let signature: &[IntermediateType] = &[IntermediateType::IU8, IntermediateType::IU16];
        assert_eq!(
            move_signature_to_abi_selector("test", signature, &compilation_ctx).unwrap(),
            selector("test(uint8,uint16)").unwrap()
        );

        let signature: &[IntermediateType] = &[IntermediateType::IAddress, IntermediateType::IU256];
        assert_eq!(
            move_signature_to_abi_selector("transfer", signature, &compilation_ctx).unwrap(),
            selector("transfer(address,uint256)").unwrap()
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::ISigner,
            IntermediateType::IAddress,
            IntermediateType::IU64,
            IntermediateType::IVector(Arc::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("set_owner", signature, &compilation_ctx).unwrap(),
            selector("setOwner(address,uint64,bool[])").unwrap()
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
            IntermediateType::IVector(Arc::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature, &compilation_ctx).unwrap(),
            selector("testArray(uint128[],bool[])").unwrap()
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
                IntermediateType::IU128,
            )))),
            IntermediateType::IVector(Arc::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature, &compilation_ctx).unwrap(),
            selector("testArray(uint128[][],bool[])").unwrap()
        );

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

        let module_structs = vec![struct_1, struct_2];
        module_data.structs.structs = module_structs;

        let signature: &[IntermediateType] = &[
            IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 0,
                vm_handled_struct: VmHandledStruct::None,
            },
            IntermediateType::IVector(Arc::new(IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 1,
                vm_handled_struct: VmHandledStruct::None,
            })),
        ];

        compilation_ctx.root_module_data = &module_data;
        assert_eq!(
            move_signature_to_abi_selector("test_struct", signature, &compilation_ctx).unwrap(),
            selector(
                "testStruct((address,uint32[],uint128[],bool,uint8,uint16,uint32,uint64,uint128,uint256,(uint32,uint128)),(uint32,uint128)[])"
            ).unwrap()
        );
    }
}
