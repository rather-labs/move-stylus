use std::collections::HashSet;

use move_parse_special_attributes::{
    Struct_,
    function_modifiers::{Function, Visibility},
    types,
};

use crate::{Abi, common::snake_to_camel, types::Type};

pub(crate) fn process_functions<'special_attrs>(
    contract_abi: &mut String,
    functions: impl Iterator<Item = &'special_attrs Function>,
    structs: &'special_attrs [Struct_],
    abi: &mut Abi,
) {
    println!("{structs:?}");
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

                    match abi_type {
                        Type::None => None,
                        Type::UserDefined(ref name, _) => {
                            if let Some(struct_) = &structs.iter().find(|s| s.name == *name) {
                                if matches!(
                                struct_.fields.first(),
                                Some((name, types::Type::UserDataType(type_name, _)))
                                    if name == "id" && (type_name == "UID" || type_name == "NamedId")
                                ) {
                                    Some(format!("bytes32 {}", param.name))
                                } else {
                                    let res = Some(format!("{} {}", abi_type.name(), param.name));
                                    abi.struct_to_process.insert(abi_type.name());
                                    res
                                }
                            } else {
                                let res = Some(format!("{} {}", abi_type.name(), param.name));
                                abi.struct_to_process.insert(abi_type.name());
                                res
                            }
                        }
                        _ => Some(format!("{} {}", abi_type.name(), param.name)),
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
            ref t @ Type::Tuple(ref types) => {
                for type_ in types {
                    if let Type::UserDefined(name, _) = type_ {
                        abi.struct_to_process.insert(name.clone());
                    }
                }
                contract_abi.push(' ');
                contract_abi.push_str(&t.name());
            }
            ref t @ Type::UserDefined(ref name, _) => {
                abi.struct_to_process.insert(name.clone());
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

    println!("{:?}", abi.struct_to_process);
}
