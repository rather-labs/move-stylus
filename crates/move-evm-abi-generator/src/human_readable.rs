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

    process_events(&mut result, abi);
    process_abi_errors(&mut result, abi);
    process_structs(&mut result, abi);
    process_functions(&mut result, abi);

    result.push_str("\n}");

    result
}

pub fn process_functions(contract_abi: &mut String, abi: &Abi) {
    // Sort functions by identifier for deterministic output
    let mut function_indices: Vec<usize> = (0..abi.functions.len()).collect();
    function_indices.sort_by_key(|&i| &abi.functions[i].identifier);

    for &i in &function_indices {
        let function = &abi.functions[i];
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
            .map(|param| format!("{} {}", param.type_.name(), param.identifier))
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
    // Sort structs by identifier for deterministic output
    let mut struct_indices: Vec<usize> = (0..abi.structs.len()).collect();
    struct_indices.sort_by_key(|&i| &abi.structs[i].identifier);

    for &i in &struct_indices {
        let struct_ = &abi.structs[i];
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

pub fn process_events(contract_abi: &mut String, abi: &Abi) {
    // Sort events by identifier for deterministic output
    let mut event_indices: Vec<usize> = (0..abi.events.len()).collect();
    event_indices.sort_by_key(|&i| &abi.events[i].identifier);

    for &i in &event_indices {
        let event = &abi.events[i];
        // Declaration
        contract_abi.push_str("    event ");
        contract_abi.push_str(&event.identifier);
        contract_abi.push('(');
        contract_abi.push_str(
            &event
                .fields
                .iter()
                .map(|f| {
                    format!(
                        "{}{}{}",
                        &f.named_type.type_.name(),
                        if f.indexed { " indexed " } else { " " },
                        &f.named_type.identifier
                    )
                })
                .collect::<Vec<String>>()
                .join(", "),
        );

        contract_abi.push_str(");\n");
    }
    contract_abi.push('\n');
}

pub fn process_abi_errors(contract_abi: &mut String, abi: &Abi) {
    // Sort errors by identifier for deterministic output
    let mut error_indices: Vec<usize> = (0..abi.abi_errors.len()).collect();
    error_indices.sort_by_key(|&i| &abi.abi_errors[i].identifier);

    for &i in &error_indices {
        let error = &abi.abi_errors[i];
        // Declaration
        contract_abi.push_str("    error ");
        contract_abi.push_str(&error.identifier);
        contract_abi.push('(');
        contract_abi.push_str(
            &error
                .fields
                .iter()
                .map(|f| format!("{}{}{}", &f.type_.name(), " ", &f.identifier))
                .collect::<Vec<String>>()
                .join(", "),
        );

        contract_abi.push_str(");\n");
    }
    contract_abi.push('\n');
}
