use move_parse_special_attributes::function_modifiers::{Function, Visibility};

use crate::types::Type;

/// Converts the input string to camel case.
pub fn snake_to_camel(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    // .len returns byte count but ok in this case!

    #[derive(PartialEq)]
    enum ChIs {
        FirstOfStr,
        NextOfSepMark,
        Other,
    }

    let mut flag = ChIs::FirstOfStr;

    for ch in input.chars() {
        if flag == ChIs::FirstOfStr {
            result.push(ch.to_ascii_lowercase());
            flag = ChIs::Other;
        } else if ch == '_' {
            flag = ChIs::NextOfSepMark;
        } else if flag == ChIs::NextOfSepMark {
            result.push(ch.to_ascii_uppercase());
            flag = ChIs::Other;
        } else {
            result.push(ch);
        }
    }

    result
}

pub(crate) fn process_functions<'special_attrs>(
    contract_abi: &mut String,
    functions: impl Iterator<Item = &'special_attrs Function>,
) {
    for function in functions {
        contract_abi.push_str("function ");
        contract_abi.push_str(&snake_to_camel(&function.name));
        contract_abi.push('(');

        contract_abi.push_str(
            &function
                .signature
                .parameters
                .iter()
                .filter_map(|param| {
                    let abi_type = Type::from(&param.type_);

                    if abi_type == Type::None {
                        None
                    } else {
                        Some(format!("{} {}", abi_type.name(), param.name))
                    }
                })
                .collect::<Vec<String>>()
                .join(", "),
        );

        contract_abi.push(')');
        contract_abi.push(' ');

        let mut modifiers: Vec<&str> = Vec::new();
        function
            .modifiers
            .iter()
            .for_each(|m| modifiers.push(m.as_str()));

        if function.visibility == Visibility::Public {
            modifiers.push("public")
        }

        // All functions we process are entry
        modifiers.push("external");

        contract_abi.push_str(&modifiers.join(" "));

        match Type::from(&function.signature.return_type) {
            Type::Unit => (),
            t @ Type::Tuple(_) => {
                contract_abi.push(' ');
                contract_abi.push_str(&t.name());
            }
            t => {
                contract_abi.push(' ');
                contract_abi.push_str(&format!("({})", t.name()));
            }
        }

        if let Some(' ') = contract_abi.chars().last() {
            contract_abi.pop();
        }

        contract_abi.push(';');

        contract_abi.push('\n');
    }
}
