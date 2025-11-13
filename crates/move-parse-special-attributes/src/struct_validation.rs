use std::collections::HashMap;

use crate::{
    Struct_,
    abi_error::AbiError,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    event::Event,
    types::Type,
};

/// Validates that no struct fields contain events or errors.
/// Returns a vector of errors if any are found.
pub fn validate_structs(
    structs: &[Struct_],
    events: &HashMap<String, Event>,
    abi_errors: &HashMap<String, AbiError>,
) -> Vec<SpecialAttributeError> {
    let mut errors = Vec::new();

    // Recursively checks if a type is or contains an event or error.
    // Returns which one was found (event or error) along with its name, None otherwise.
    fn check_type(
        ty: &Type,
        events: &HashMap<String, Event>,
        abi_errors: &HashMap<String, AbiError>,
    ) -> Option<(bool, String)> {
        match ty {
            Type::UserDataType(name, _) => {
                // Check if the type itself is an event or error
                if events.contains_key(name) {
                    return Some((true, name.clone()));
                }
                if abi_errors.contains_key(name) {
                    return Some((false, name.clone()));
                }
                None
            }
            Type::Vector(inner) => check_type(inner, events, abi_errors),
            Type::Tuple(types) => {
                for t in types {
                    if let Some(found) = check_type(t, events, abi_errors) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    for struct_ in structs {
        for (_, field_type) in &struct_.fields {
            if let Some((is_event, name)) = check_type(field_type, events, abi_errors) {
                let error = if is_event {
                    SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::NestedEvent(name.to_string()),
                        line_of_code: struct_.loc,
                    }
                } else {
                    SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::NestedError(name.to_string()),
                        line_of_code: struct_.loc,
                    }
                };
                errors.push(error);
            }
        }
    }

    errors
}
