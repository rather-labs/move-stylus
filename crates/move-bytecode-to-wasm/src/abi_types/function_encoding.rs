use alloy_primitives::keccak256;
use move_binary_format::file_format::Signature;

use super::type_mapping::map_move_type_to_sol_name;

pub type AbiFunctionSelector = [u8; 4];

fn selector<T: AsRef<[u8]>>(bytes: T) -> AbiFunctionSelector {
    keccak256(bytes)[..4].try_into().unwrap()
}

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
pub fn move_signature_to_abi_selector(
    function_name: &str,
    signature: &Signature,
) -> AbiFunctionSelector {
    let mut parameter_strings = Vec::new();
    for signature_token in signature.0.iter() {
        parameter_strings.push(map_move_type_to_sol_name(signature_token));
    }
    selector(format!(
        "{}({})",
        function_name,
        parameter_strings.join(",")
    ))
}

#[cfg(test)]
mod tests {
    use move_binary_format::file_format::SignatureToken;

    use super::*;

    #[test]
    fn test_move_signature_to_abi_selector() {
        let signature = Signature(vec![SignatureToken::U8, SignatureToken::U16]);
        assert_eq!(
            move_signature_to_abi_selector("test", &signature),
            selector("test(uint8,uint16)")
        );

        let signature = Signature(vec![SignatureToken::Address, SignatureToken::U256]);
        assert_eq!(
            move_signature_to_abi_selector("transfer", &signature),
            selector("transfer(address,uint256)")
        );

        let signature = Signature(vec![
            SignatureToken::Vector(Box::new(SignatureToken::U128)),
            SignatureToken::Vector(Box::new(SignatureToken::Bool)),
        ]);
        assert_eq!(
            move_signature_to_abi_selector("testArray", &signature),
            selector("testArray(uint128[],bool[])")
        );

        let signature = Signature(vec![
            SignatureToken::Vector(Box::new(SignatureToken::Vector(Box::new(
                SignatureToken::U128,
            )))),
            SignatureToken::Vector(Box::new(SignatureToken::Bool)),
        ]);
        assert_eq!(
            move_signature_to_abi_selector("testArray", &signature),
            selector("testArray(uint128[][],bool[])")
        );
    }
}
