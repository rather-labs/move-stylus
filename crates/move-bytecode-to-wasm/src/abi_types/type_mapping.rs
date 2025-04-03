use alloy_sol_types::{SolType, sol_data};
use move_binary_format::file_format::SignatureToken;

pub fn map_move_type_to_sol_name(signature_token: &SignatureToken) -> String {
    match signature_token {
        SignatureToken::Bool => sol_data::Bool::SOL_NAME.to_string(),
        SignatureToken::U8 => sol_data::Uint::<8>::SOL_NAME.to_string(),
        SignatureToken::U16 => sol_data::Uint::<16>::SOL_NAME.to_string(),
        SignatureToken::U32 => sol_data::Uint::<32>::SOL_NAME.to_string(),
        SignatureToken::U64 => sol_data::Uint::<64>::SOL_NAME.to_string(),
        SignatureToken::U128 => sol_data::Uint::<128>::SOL_NAME.to_string(),
        SignatureToken::U256 => sol_data::Uint::<256>::SOL_NAME.to_string(),
        SignatureToken::Address => sol_data::Address::SOL_NAME.to_string(),
        SignatureToken::Vector(boxed_signature_token) => {
            format!("{}[]", map_move_type_to_sol_name(boxed_signature_token))
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
