use alloy_primitives::keccak256;
use alloy_sol_types::{SolType, sol_data};

use crate::{
    translation::intermediate_types::{
        IParam,
        address::IAddress,
        boolean::IBool,
        heap_integers::{IU128, IU256},
        simple_integers::{IU8, IU16, IU32, IU64},
    },
    utils::snake_to_camel,
};

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
pub fn move_signature_to_abi_selector(
    function_name: &str,
    signature: &[Box<dyn IParam>],
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

impl SolName for IBool {
    fn sol_name(&self) -> String {
        sol_data::Bool::SOL_NAME.to_string()
    }
}

impl SolName for IAddress {
    fn sol_name(&self) -> String {
        sol_data::Address::SOL_NAME.to_string()
    }
}

impl SolName for IU8 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<8>::SOL_NAME.to_string()
    }
}

impl SolName for IU16 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<16>::SOL_NAME.to_string()
    }
}

impl SolName for IU32 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<32>::SOL_NAME.to_string()
    }
}

impl SolName for IU64 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<64>::SOL_NAME.to_string()
    }
}

impl SolName for IU128 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<128>::SOL_NAME.to_string()
    }
}

impl SolName for IU256 {
    fn sol_name(&self) -> String {
        sol_data::Uint::<256>::SOL_NAME.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::translation::intermediate_types::{
        address::IAddress,
        heap_integers::IU256,
        simple_integers::{IU8, IU16},
    };

    use super::*;

    #[test]
    fn test_move_signature_to_abi_selector() {
        let signature: &[Box<dyn IParam>] = &[Box::new(IU8), Box::new(IU16)];
        assert_eq!(
            move_signature_to_abi_selector("test", signature),
            selector("test(uint8,uint16)")
        );

        let signature: &[Box<dyn IParam>] = &[Box::new(IAddress), Box::new(IU256)];
        assert_eq!(
            move_signature_to_abi_selector("transfer", signature),
            selector("transfer(address,uint256)")
        );
    }
}
