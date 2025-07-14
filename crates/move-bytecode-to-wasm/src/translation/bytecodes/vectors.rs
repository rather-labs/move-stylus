use move_binary_format::file_format::SignatureIndex;

use crate::{
    CompilationContext,
    translation::{TranslationError, intermediate_types::IntermediateType},
};

/// Converts the signature index pointing to a Move's Signature token that represents the inner
/// type of a vector and convets it to an IntermediateType.
///
/// This is used as a safety check to ensure that the inner type of a vector obtained from Move's
/// compilation matches the one contained in the types stack.
pub fn get_inner_type_from_signature(
    signature_index: &SignatureIndex,
    compilation_ctx: &CompilationContext,
) -> Result<IntermediateType, TranslationError> {
    let signatures = compilation_ctx.get_signatures_by_index(*signature_index)?;

    let signature = if signatures.len() != 1 {
        return Err(TranslationError::VectorInnerTypeNumberError {
            signature_index: *signature_index,
            number: signatures.len(),
        });
    } else {
        &signatures[0]
    };

    Ok(IntermediateType::try_from_signature_token(
        signature,
        compilation_ctx.root_module_data.datatype_handles_map,
    )?)
}
