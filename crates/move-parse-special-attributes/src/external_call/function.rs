use std::collections::HashSet;

use move_compiler::parser::ast::{Function, FunctionBody_, NameAccessChain_, Type_};
use move_symbol_pool::Symbol;

use crate::{
    ExternalCallFunctionError, SpecialAttributeError, error::SpecialAttributeErrorKind,
    function_modifiers::FunctionModifier, shared::get_single_type_name,
};

const FN_RESULT_EMPTY_STRUCT_NAME: &str = "ContractCallEmptyResult";
const FN_RESULT_STRUCT_NAME: &str = "ContractCallResult";

fn check_return_value(function: &Function) -> Option<SpecialAttributeError> {
    match &function.signature.return_type.value {
        Type_::Apply(spanned) => match &spanned.value {
            NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                FN_RESULT_EMPTY_STRUCT_NAME | FN_RESULT_STRUCT_NAME => {}
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

fn check_first_parameter_is_external_struct(
    function: &Function,
    external_call_structs: &HashSet<Symbol>,
) -> bool {
    if let Some((_, _, type_)) = function.signature.parameters.first() {
        if let Some(name) = get_single_type_name(&type_.value) {
            external_call_structs.contains(&name)
        } else {
            false
        }
    } else {
        false
    }
}

pub(crate) fn validate_external_call_function(
    function: &Function,
    _modifiers: &[FunctionModifier],
    external_call_structs: &HashSet<Symbol>,
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

    if !check_first_parameter_is_external_struct(function, external_call_structs) {
        errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalCallFunction(
                ExternalCallFunctionError::InvalidFirstArgument,
            ),
            line_of_code: function.loc,
        })
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
