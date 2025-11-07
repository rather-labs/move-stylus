use crate::{
    abi::{Abi, Visibility},
    common::snake_to_upper_camel,
    types::Type,
};

pub fn process_abi(abi: &Abi) -> String {
    let mut result = String::new();

    result.push_str("contract ");
    result.push_str(&snake_to_upper_camel(&abi.contract_name));
    result.push_str(" {\n\n");

    process_structs(&mut result, abi);
    process_functions(&mut result, abi);

    result.push_str("\n}");

    result
}

pub fn process_functions(contract_abi: &mut String, abi: &Abi) {
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

pub fn process_structs(contract_abi: &mut String, abi: &Abi) {
    for struct_ in &abi.structs {
        // Declaration
        contract_abi.push_str("    struct ");
        contract_abi.push_str(&struct_.identifier);
        contract_abi.push_str(" {\n");
        for field in &struct_.fields {
            contract_abi.push_str("        ");
            contract_abi.push_str(&field.type_.name());
            contract_abi.push(' ');
            contract_abi.push_str(&field.identifier);
            contract_abi.push_str(";\n");
        }

        contract_abi.push_str("    }\n\n");
    }
}
