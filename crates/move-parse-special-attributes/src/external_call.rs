//! This module is in charge if checking all the constraints related to marking a function as an
//! external call.

use move_compiler::parser::ast::{Function, FunctionBody_, NameAccessChain_, Type_};
use payable::check_payable_value_argument;

use crate::function_modifiers::FunctionModifier;

mod payable;

fn check_return_value(function: &Function, modifiers: &[FunctionModifier]) -> Result<(), String> {
    if modifiers.contains(&FunctionModifier::Payable) {
        match &function.signature.return_type.value {
            Type_::Apply(spanned) => match &spanned.value {
                NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                    "ContractCallResult" | "ContractCallEmptyResult" => {}
                    other => {
                        return Err(format!(
                            "An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult, found '{}'",
                            other
                        ));
                    }
                },
                _ => {
                    return Err("An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult".to_string());
                }
            },
            _ => {
                return Err("An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult".to_string());
            }
        }
    }
    Ok(())
}

fn body_is_native(function: &Function) -> bool {
    function.body.value == FunctionBody_::Native
}

pub(crate) fn validate_external_call_function(
    function: &Function,
    modifiers: &[FunctionModifier],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if !body_is_native(function) {
        errors.push(format!(
            "External call function '{}' must have be native",
            function.name
        ));
    }

    if modifiers.contains(&FunctionModifier::Payable) {
        if let Err(e) = check_payable_value_argument(function, modifiers) {
            errors.push(e);
        }
    }

    if let Err(e) = check_return_value(function, modifiers) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
