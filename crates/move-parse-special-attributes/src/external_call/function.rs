use move_compiler::parser::ast::{Function, FunctionBody_, NameAccessChain_, Type_};

use crate::{
    ExternalCallFunctionError, SpecialAttributeError, error::SpecialAttributeErrorKind,
    function_modifiers::FunctionModifier,
};

fn check_return_value(function: &Function) -> Option<SpecialAttributeError> {
    match &function.signature.return_type.value {
        Type_::Apply(spanned) => match &spanned.value {
            NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                "ContractCallResult" | "ContractCallEmptyResult" => {}
                other => {
                    return Some(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalCallFunction(
                            ExternalCallFunctionError::InvalidReturnType(other.to_string()),
                        ),
                        line_of_code: path_entry.name.loc,
                    });
                }
            },
            NameAccessChain_::Path(path_entry) => {
                return Some(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::ExternalCallFunction(
                        ExternalCallFunctionError::InvalidReturnType(path_entry.to_string()),
                    ),
                    line_of_code: spanned.loc,
                });
            }
        },
        other => {
            return Some(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::ExternalCallFunction(
                    ExternalCallFunctionError::InvalidReturnType(other.to_string()),
                ),
                line_of_code: function.signature.return_type.loc,
            });
        }
    }

    None
}

fn body_is_native(function: &Function) -> bool {
    function.body.value == FunctionBody_::Native
}

pub(crate) fn validate_external_call_function(
    function: &Function,
    _modifiers: &[FunctionModifier],
) -> Result<(), Vec<SpecialAttributeError>> {
    let mut errors = Vec::new();

    if !body_is_native(function) {
        errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalCallFunction(
                ExternalCallFunctionError::FunctionIsNotNative,
            ),
            line_of_code: function.loc,
        })
    }

    // TODO: Check for invalid modifiers

    if let Some(e) = check_return_value(function) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
