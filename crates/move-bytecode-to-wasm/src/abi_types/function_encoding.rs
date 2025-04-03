use move_binary_format::file_format::{Signature, SignatureToken};

use super::hashing::selector;

pub type AbiFunctionSelector = [u8; 4];

/// Calculate the function selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
pub fn move_signature_to_abi_selector(
    function_name: &str,
    signature: &Signature,
) -> AbiFunctionSelector {
    let mut parameter_strings = Vec::new();
    for signature_token in signature.0.iter() {
        parameter_strings.push(move_token_to_abi_type_string(signature_token));
    }
    selector(format!(
        "{}({})",
        function_name,
        parameter_strings.join(",")
    ))
}

fn move_token_to_abi_type_string(signature_token: &SignatureToken) -> String {
    match signature_token {
        SignatureToken::Bool => "bool".to_string(),
        SignatureToken::U8 => "uint8".to_string(),
        SignatureToken::U16 => "uint16".to_string(),
        SignatureToken::U32 => "uint32".to_string(),
        SignatureToken::U64 => "uint64".to_string(),
        SignatureToken::U128 => "uint128".to_string(),
        SignatureToken::U256 => "uint256".to_string(),
        SignatureToken::Address => "address".to_string(),
        SignatureToken::Vector(boxed_signature_token) => {
            format!("{}[]", move_token_to_abi_type_string(boxed_signature_token))
        }
        SignatureToken::Signer => panic!("Signer is not supported"), // TODO: review how to handle this on public functions
        SignatureToken::Datatype(_) => panic!("Datatype is not supported yet"), // TODO
        SignatureToken::TypeParameter(_) => panic!("TypeParameter is not supported"), // TODO
        SignatureToken::DatatypeInstantiation(_) => {
            panic!("DatatypeInstantiation is not supported") // TODO
        }
        SignatureToken::Reference(_) => {
            panic!("Reference is not allowed as a public function argument")
        }
        SignatureToken::MutableReference(_) => {
            panic!("MutableReference is not allowed as a public function argument")
        }
    }
}

#[cfg(test)]
mod tests {
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
    }
}

