use std::collections::HashMap;

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use move_parse_special_attributes::{
    Struct_,
    function_modifiers::{Function, Visibility},
    types,
};

use crate::{Abi, common::snake_to_camel, types::Type};

pub(crate) fn process_functions(
    contract_abi: &mut String,
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
    abi: &mut Abi,
) {
    // First we filter the functions we are ging to process
    let functions = processing_module
        .functions
        .information
        .iter()
        .filter(|f| f.is_entry);

    let structs = &processing_module.special_attributes.structs;

    // println!("{structs:?}");
    for function in functions {
        let function_name = &function.function_id.identifier;
        let parsed_function = processing_module
            .special_attributes
            .functions
            .iter()
            .find(|f| f.name == *function_name)
            .expect("function not found");

        contract_abi.push_str("function ");
        contract_abi.push_str(&snake_to_camel(function_name));
        contract_abi.push('(');

        contract_abi.push_str(
            &parsed_function
                .signature
                .parameters
                .iter()
                .zip(&function.signature.arguments)
                .filter_map(|(param, itype)| {
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

        let mut modifiers: Vec<&str> = processing_module
            .special_attributes
            .functions
            .iter()
            .find(|f| f.name == *function_name)
            .map(|f| f.modifiers.iter().map(|m| m.as_str()).collect())
            .unwrap_or_default();

        /*
        if function.visibility == Visibility::Public {
            modifiers.push("public")
        }
        */

        // All functions we process are entry
        modifiers.push("external");

        contract_abi.push_str(&modifiers.join(" "));

        /*
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
        */

        contract_abi.push(';');

        contract_abi.push('\n');
    }

    println!("{:?}", abi.struct_to_process);
}
