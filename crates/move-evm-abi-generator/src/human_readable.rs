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
            .map(|param| {
                format!(
                    "{} {}",
                    format_type_name_for_abi(&param.type_, abi),
                    param.identifier
                )
            })
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
                contract_abi.push_str(&format_type_name_for_abi(&function.return_types, abi));
            } else {
                contract_abi.push('(');
                contract_abi.push_str(&format_type_name_for_abi(&function.return_types, abi));
                contract_abi.push(')');
            }
        }

        contract_abi.push(';');
        contract_abi.push('\n');
    }
}

pub fn process_structs(contract_abi: &mut String, abi: &Abi) {
    for struct_ in &abi.structs {
        // Add underscore suffix if it's also an event or error
        let identifier = if is_struct_identifier_conflict(&struct_.identifier, abi) {
            format!("{}_", struct_.identifier)
        } else {
            struct_.identifier.clone()
        };

        // Declaration
        contract_abi.push_str("    struct ");
        contract_abi.push_str(&identifier);
        contract_abi.push_str(" {\n");
        for field in &struct_.fields {
            contract_abi.push_str("        ");
            contract_abi.push_str(&format_type_name_for_abi(&field.type_, abi));
            contract_abi.push(' ');
            contract_abi.push_str(&field.identifier);
            contract_abi.push_str(";\n");
        }

        contract_abi.push_str("    }\n\n");
    }
}

pub fn process_events(contract_abi: &mut String, abi: &Abi) {
    for event in &abi.events {
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
                        &f.type_.name(),
                        if f.indexed { " indexed " } else { " " },
                        &f.identifier
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
    for error in &abi.abi_errors {
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

/// Helper function to check if a struct identifier matches any event or error identifier
fn is_struct_identifier_conflict(identifier: &str, abi: &Abi) -> bool {
    abi.events.iter().any(|e| e.identifier == identifier)
        || abi.abi_errors.iter().any(|e| e.identifier == identifier)
}

/// Helper function to format type name for display, adding underscore suffix to struct identifiers
/// that match events or errors to avoid naming conflicts
fn format_type_name_for_abi(ty: &Type, abi: &Abi) -> String {
    match ty {
        Type::Struct { identifier, .. } => {
            // Add underscore after the struct type name if identifier conflicts with event/error
            let ty_name = ty.name();
            if is_struct_identifier_conflict(identifier, abi) {
                format!("{}_", ty_name)
            } else {
                ty_name
            }
        }
        Type::Array(inner) => {
            format!("{}[]", format_type_name_for_abi(inner, abi))
        }
        Type::Tuple(items) => {
            format!(
                "({})",
                items
                    .iter()
                    .map(|i| format_type_name_for_abi(i, abi))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
        _ => ty.name(),
    }
}
