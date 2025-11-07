use std::collections::HashMap;

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId, module_data::struct_data::IntermediateType,
};

use crate::{
    abi::{Abi, Visibility},
    special_types::convert_type_for_struct_field,
    types::{Type, type_contains_generics},
};

pub(crate) fn process_functions(contract_abi: &mut String, abi: &Abi) {
    for function in &abi.functions {
        if function.visibility == Visibility::Private && !function.is_entry {
            continue;
        }
        // Identifier
        contract_abi.push_str("    function ");
        contract_abi.push_str(&function.identifier);

        // Params
        contract_abi.push('(');
        let formatted_parameters = function
            .parameters
            .iter()
            .map(|param| format!("{} {}", &param.type_.name(), param.identifier))
            .collect::<Vec<String>>();

        contract_abi.push_str(&formatted_parameters.join(", "));
        contract_abi.push(')');

        let mut modifiers: Vec<&str> = function.modifiers.iter().map(|m| m.as_str()).collect();

        // Modifiers
        if function.visibility == Visibility::Public {
            modifiers.push("public")
        }

        // All functions we process are entry
        if function.is_entry {
            modifiers.push("external");
        }

        if !modifiers.is_empty() {
            contract_abi.push(' ');
            contract_abi.push_str(&modifiers.join(" "));
        }

        // Return
        if function.return_types != Type::None {
            contract_abi.push(' ');

            if let Type::Tuple(_) = function.return_types {
                contract_abi.push_str(&function.return_types.name());
            } else {
                contract_abi.push('(');
                contract_abi.push_str(&function.return_types.name());
                contract_abi.push(')');
            }
        }

        contract_abi.push(';');
        contract_abi.push('\n');
    }
}

/*
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
*/
