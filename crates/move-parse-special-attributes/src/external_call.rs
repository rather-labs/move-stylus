//! This module is in charge if checking all the constraints related to marking a function as an
//! external call.

use move_compiler::parser::ast::{Function, NameAccessChain_, Type_};

use crate::function_modifiers::FunctionModifier;

fn check_payable_value_argument(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Result<(), String> {
    if modifiers.contains(&FunctionModifier::Payable) {
        if let Some(argument) = function.signature.parameters.get(1) {
            let arg_name = argument.1.0.value.as_str();
            if arg_name != "value" {
                return Err(format!(
                    "The second argument of a payable external call function must be named 'value', found '{}'",
                    arg_name
                ));
            }

            match &argument.2.value {
                Type_::Apply(spanned) => match &spanned.value {
                    NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                        "u256" => {}
                        other => {
                            return Err(format!(
                                "The 'value' argument of a payable external call function must be of type 'u256', found '{}'",
                                other
                            ));
                        }
                    },
                    _ => {
                        return Err("The 'value' argument of a payable external call function must be of type 'u256'".to_string());
                    }
                },
                _ => {
                    return Err("The 'value' argument of a payable external call function must be of type 'u256'".to_string());
                }
            }
        } else {
            return Err("A payable external call function must have a second argument named 'value' of type 'u256'".to_string());
        }
    }
    Ok(())
}

pub(crate) fn validate_external_call_function(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = check_payable_value_argument(function, modifiers) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
