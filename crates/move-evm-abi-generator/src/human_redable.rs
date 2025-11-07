use std::collections::HashMap;

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId, module_data::struct_data::IntermediateType,
};
use move_parse_special_attributes::function_modifiers::Visibility;

use crate::{
    Abi,
    common::snake_to_camel,
    special_types::{
        convert_type_for_signature, convert_type_for_struct_field, is_hidden_in_signature,
    },
    types::{Type, type_contains_generics},
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
                            Type::UserDefined(name, types),
                            IntermediateType::IStruct {
                                module_id, index, ..
                            }
                            | IntermediateType::IGenericStructInstance {
                                module_id, index, ..
                            },
                        ) => {
                            if is_hidden_in_signature(name, Some(module_id)) {
                                None
                            } else {
                                let struct_module = modules_data.get(module_id).unwrap();
                                let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                                /*
                                match struct_.fields.first() {
                                    Some(IntermediateType::IGenericStructInstance { module_id, index, types, vm_handled_struct })

                                }
                                */
                                if let Some(first_field) = struct_.fields.first() {}

                                abi.struct_to_process.insert((itype.clone(), types.clone()));
                                println!("1 inserting {itype:?}");

                                Some(format!(
                                    "{} {}",
                                    convert_type_for_signature(name, itype, modules_data),
                                    param.name
                                ))
                            }
                        }
                        _ => Some(format!(
                            "{} {}",
                            convert_type_for_signature(&abi_type.name(), itype, modules_data),
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
            Type::Tuple(ref ret_types) => {
                let mut names = Vec::new();
                for (type_, itype) in ret_types.iter().zip(&function.signature.returns) {
                    if let Type::UserDefined(_, types) = type_ {
                        abi.struct_to_process.insert((itype.clone(), types.clone()));
                        println!("2 inserting {itype:?}");
                    }
                    names.push(
                        convert_type_for_signature(&type_.name(), itype, modules_data).to_owned(),
                    );
                }
                contract_abi.push(' ');
                contract_abi.push_str(&format!("({})", &names.join(", ")));
            }
            ref t @ Type::UserDefined(_, ref types) => {
                assert_eq!(1, function.signature.returns.len());
                let itype = &function.signature.returns[0];
                abi.struct_to_process.insert((itype.clone(), types.clone()));
                println!("3 inserting {itype:?}");
                contract_abi.push(' ');
                contract_abi.push_str(convert_type_for_signature(&t.name(), itype, modules_data));
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
    println!("AAAA {:?}", abi.struct_to_process);
}

pub(crate) fn process_structs(
    contract_abi: &mut String,
    modules_data: &HashMap<ModuleId, ModuleData>,
    abi: &mut Abi,
) {
    let mut struct_section = String::new();

    for (itype, types) in &abi.struct_to_process {
        // If the struct contains a generic type, means it should not be part of the ABI, since
        // Solidity does not support generics yet
        if type_contains_generics(itype) {
            continue;
        }
        // Get the IStruct

        let (struct_, struct_module) = match itype {
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_module = modules_data
                    .get(module_id)
                    .expect("struct module not found");

                (
                    struct_module.structs.get_by_index(*index).unwrap(),
                    struct_module,
                )
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types: instantiation_types,
                ..
            } => {
                let struct_module = modules_data
                    .get(module_id)
                    .expect("struct module not found");

                let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                (&struct_.instantiate(instantiation_types), struct_module)
            }
            _ => {
                continue;
                // panic!("trying to process a type that is not an struct {t:?}",),
            }
        };

        let parsed_struct = struct_module
            .special_attributes
            .structs
            .iter()
            .find(|f| f.name == *struct_.identifier)
            .expect("struct not found");

        let struct_abi_type = Type::from_intermediate_type(itype, modules_data);
        struct_section.push_str(&format!("    struct {} {{\n", struct_abi_type.name()));

        for (itype, (name, _)) in struct_.fields.iter().zip(&parsed_struct.fields) {
            let abi_type = &Type::from_intermediate_type(itype, modules_data);

            struct_section.push_str(&format!(
                "        {} {};\n",
                convert_type_for_struct_field(&abi_type.name(), itype, modules_data),
                name
            ));
        }
        struct_section.push_str("    }\n\n");
    }

    contract_abi.insert_str(0, &struct_section);
}
