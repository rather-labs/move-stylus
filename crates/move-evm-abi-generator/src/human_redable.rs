use std::collections::HashMap;

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId, module_data::struct_data::IntermediateType,
};
use move_parse_special_attributes::function_modifiers::Visibility;

use crate::{
    Abi,
    common::snake_to_camel,
    special_types::{convert_type, is_hidden_in_signature},
    types::Type,
};

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

    for function in functions {
        let function_name = &function.function_id.identifier;
        let parsed_function = processing_module
            .special_attributes
            .functions
            .iter()
            .find(|f| f.name == *function_name)
            .expect("function not found");

        contract_abi.push_str("    function ");
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

                    // Remove the references if any
                    let itype = match itype {
                        IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                            inner.as_ref()
                        }
                        _ => itype,
                    };

                    match (&abi_type, itype) {
                        (Type::None, _) => None,
                        (
                            Type::UserDefined(name, _),
                            IntermediateType::IStruct { module_id, .. }
                            | IntermediateType::IGenericStructInstance { module_id, .. }
                            | IntermediateType::IEnum { module_id, .. }
                            | IntermediateType::IGenericEnumInstance { module_id, .. },
                        ) => {
                            if is_hidden_in_signature(name, Some(module_id)) {
                                None
                            } else {
                                abi.struct_to_process.insert(itype.clone());

                                Some(format!(
                                    "{} {}",
                                    convert_type(name, itype, modules_data),
                                    param.name
                                ))
                            }
                        }
                        _ => Some(format!(
                            "{} {}",
                            convert_type(&abi_type.name(), itype, modules_data),
                            param.name
                        )),
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

        if parsed_function.visibility == Visibility::Public {
            modifiers.push("public")
        }

        // All functions we process are entry
        modifiers.push("external");

        contract_abi.push_str(&modifiers.join(" "));

        match Type::from(&parsed_function.signature.return_type) {
            Type::Unit => (),
            Type::Tuple(ref types) => {
                let mut names = Vec::new();
                for (type_, itype) in types.iter().zip(&function.signature.returns) {
                    if let Type::UserDefined(_, _) = type_ {
                        abi.struct_to_process.insert(itype.clone());
                    }
                    names.push(convert_type(&type_.name(), itype, modules_data).to_owned());
                }
                contract_abi.push(' ');
                contract_abi.push_str(&format!("({})", &names.join(", ")));
            }
            ref t @ Type::UserDefined(_, _) => {
                assert_eq!(1, function.signature.returns.len());
                let itype = &function.signature.returns[0];
                abi.struct_to_process.insert(itype.clone());
                contract_abi.push(' ');
                contract_abi.push_str(convert_type(&t.name(), itype, modules_data));
            }
            t => {
                contract_abi.push(' ');
                contract_abi.push_str(&format!("({})", &t.name()));
            }
        }

        if let Some(' ') = contract_abi.chars().last() {
            contract_abi.pop();
        }

        contract_abi.push(';');
        contract_abi.push('\n');
    }
}

// pub fn process_structs() {}
