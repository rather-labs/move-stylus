use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{translation::intermediate_types::IntermediateType, utils::snake_to_camel};

pub type AbiFunctionSelector = [u8; 4];

fn selector<T: AsRef<[u8]>>(bytes: T) -> AbiFunctionSelector {
    keccak256(bytes)[..4].try_into().unwrap()
}

pub trait SolName {
    fn sol_name(&self) -> String;
}

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector<T: SolName>(
    function_name: &str,
    signature: &[T],
) -> AbiFunctionSelector {
    let mut parameter_strings = Vec::new();
    for signature_token in signature.iter() {
        parameter_strings.push(signature_token.sol_name());
    }

    let function_name = snake_to_camel(function_name);

    selector(format!(
        "{}({})",
        function_name,
        parameter_strings.join(",")
    ))
}

impl SolName for IntermediateType {
    fn sol_name(&self) -> String {
        match self {
            IntermediateType::IBool => sol_data::Bool::SOL_NAME.to_string(),
            IntermediateType::IU8 => sol_data::Uint::<8>::SOL_NAME.to_string(),
            IntermediateType::IU16 => sol_data::Uint::<16>::SOL_NAME.to_string(),
            IntermediateType::IU32 => sol_data::Uint::<32>::SOL_NAME.to_string(),
            IntermediateType::IU64 => sol_data::Uint::<64>::SOL_NAME.to_string(),
            IntermediateType::IU128 => sol_data::Uint::<128>::SOL_NAME.to_string(),
            IntermediateType::IU256 => sol_data::Uint::<256>::SOL_NAME.to_string(),
            IntermediateType::IAddress => sol_data::Address::SOL_NAME.to_string(),
            IntermediateType::IVector(inner) => format!("{}[]", inner.sol_name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_signature_to_abi_selector() {
        let signature: &[IntermediateType] = &[IntermediateType::IU8, IntermediateType::IU16];
        assert_eq!(
            move_signature_to_abi_selector("test", signature),
            selector("test(uint8,uint16)")
        );

        let signature: &[IntermediateType] = &[IntermediateType::IAddress, IntermediateType::IU256];
        assert_eq!(
            move_signature_to_abi_selector("transfer", signature),
            selector("transfer(address,uint256)")
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Box::new(IntermediateType::IU128)),
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature),
            selector("testArray(uint128[],bool[])")
        );

        let signature: &[IntermediateType] = &[
            IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                IntermediateType::IU128,
            )))),
            IntermediateType::IVector(Box::new(IntermediateType::IBool)),
        ];
        assert_eq!(
            move_signature_to_abi_selector("test_array", signature),
            selector("testArray(uint128[][],bool[])")
        );
    }
}
