use move_compiler::parser::ast::{Function, NameAccessChain_, Type_};

use crate::{
    SpecialAttributeError, error::SpecialAttributeErrorKind, function_modifiers::FunctionModifier,
};

use super::error::ExternalCallError;

pub fn check_payable_value_argument(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Option<SpecialAttributeError> {
    if modifiers.contains(&FunctionModifier::Payable) {
        if let Some(argument) = function.signature.parameters.get(1) {
            let arg_name = argument.1.0.value.as_str();
            if arg_name != "value" {
                return Some(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::ExternalCall(
                        ExternalCallError::InvalidValueArgumentName(argument.1.0.value.to_string()),
                    ),
                    line_of_code: argument.1.0.loc,
                });
            }

            match &argument.2.value {
                Type_::Apply(spanned) => match &spanned.value {
                    NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                        "u256" => {}
                        other => {
                            return Some(SpecialAttributeError {
                                kind: SpecialAttributeErrorKind::ExternalCall(
                                    ExternalCallError::InvalidValueArgumentType(other.to_string()),
                                ),
                                line_of_code: path_entry.name.loc,
                            });
                        }
                    },
                    NameAccessChain_::Path(path_entry) => {
                        return Some(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::ExternalCall(
                                ExternalCallError::InvalidValueArgumentType(path_entry.to_string()),
                            ),
                            line_of_code: spanned.loc,
                        });
                    }
                },
                other => {
                    return Some(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalCall(
                            ExternalCallError::InvalidValueArgumentType(other.to_string()),
                        ),
                        line_of_code: function.signature.return_type.loc,
                    });
                }
            }
        } else {
            return Some(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::ExternalCall(
                    ExternalCallError::ValueArgumentMissing,
                ),
                line_of_code: function.loc,
            });
        }
    }
    None
}
