//! This module is in charge if checking all the constraints related to marking a function as an
//! external call.

use error::ExternalCallError;
use move_compiler::parser::ast::{Function, FunctionBody_, NameAccessChain_, Type_};
use payable::check_payable_value_argument;

use crate::{
    SpecialAttributeError, error::SpecialAttributeErrorKind, function_modifiers::FunctionModifier,
};

pub mod error;
mod payable;

fn check_return_value(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Option<SpecialAttributeError> {
    if modifiers.contains(&FunctionModifier::Payable) {
        match &function.signature.return_type.value {
            Type_::Apply(spanned) => match &spanned.value {
                NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                    "ContractCallResult" | "ContractCallEmptyResult" => {}
                    other => {
                        return Some(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::ExternalCall(
                                ExternalCallError::InvalidReturnType(other.to_string()),
                            ),
                            line_of_code: path_entry.name.loc,
                        });
                    }
                },
                NameAccessChain_::Path(path_entry) => {
                    return Some(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalCall(
                            ExternalCallError::InvalidReturnType(path_entry.to_string()),
                        ),
                        line_of_code: spanned.loc,
                    });
                }
            },
            other => {
                return Some(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::ExternalCall(
                        ExternalCallError::InvalidReturnType(other.to_string()),
                    ),
                    line_of_code: function.signature.return_type.loc,
                });
            }
        }
    }

    None
}

fn body_is_native(function: &Function) -> bool {
    function.body.value == FunctionBody_::Native
}

pub(crate) fn validate_external_call_function(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Result<(), Vec<SpecialAttributeError>> {
    let mut errors = Vec::new();

    if !body_is_native(function) {
        errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalCall(ExternalCallError::FunctionIsNotNative),
            line_of_code: function.loc,
        })
    }

    if modifiers.contains(&FunctionModifier::Payable) {
        if let Some(e) = check_payable_value_argument(function, modifiers) {
            errors.push(e);
        }
    }

    if let Some(e) = check_return_value(function, modifiers) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
