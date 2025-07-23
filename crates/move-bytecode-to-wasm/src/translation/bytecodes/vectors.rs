use move_binary_format::file_format::SignatureIndex;

use crate::{
    compilation_context::module_data::ModuleData,
    translation::{TranslationError, intermediate_types::IntermediateType},
};

/// Converts the signature index pointing to a Move's Signature token that represents the inner
/// type of a vector and convets it to an IntermediateType.
///
/// This is used as a safety check to ensure that the inner type of a vector obtained from Move's
/// compilation matches the one contained in the types stack.
pub fn get_inner_type_from_signature(
    signature_index: &SignatureIndex,
    module_data: &ModuleData,
) -> Result<IntermediateType, TranslationError> {
    let signatures = module_data.get_signatures_by_index(*signature_index)?;

    if signatures.len() != 1 {
        Err(TranslationError::VectorInnerTypeNumberError {
            signature_index: *signature_index,
            number: signatures.len(),
        })
    } else {
        Ok(signatures[0].clone())
    }
}
