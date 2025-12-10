use crate::{
    abi_error::AbiError,
    error::{SpecialAttributeError, SpecialAttributeErrorKind},
    event::Event,
    reserved_modules::{SF_ADDRESS, SF_RESERVED_STRUCTS},
    types::Type,
};
use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};
use move_ir_types::location::Loc;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum StructValidationError {
    #[error("Struct '{0}' with key ability must have UID or NamedId as its first field.")]
    StructWithKeyMissingUidField(String),

    #[error("Struct '{0}' with key ability must have its first field named 'id'.")]
    StructWithKeyFirstFieldWrongName(String),

    #[error(
        "Struct '{0}' with key ability cannot have UID or NamedId in fields other than the first."
    )]
    MoreThanOneUidFields(String),

    #[error("Struct '{0}' without key ability cannot have UID or NamedId fields.")]
    StructWithoutKeyHasUidField(String),

    #[error("Events cannot be nested. Found {0} in struct.")]
    NestedEvent(String),

    #[error("Errors cannot be nested. Found {0} in struct.")]
    NestedError(String),

    #[error(
        "Struct '{0}' is reserved by the Stylus Framework and cannot be defined in module '{1}'."
    )]
    FrameworkReservedStruct(String, String),
}

impl From<&StructValidationError> for DiagnosticInfo {
    fn from(value: &StructValidationError) -> Self {
        custom(
            "Struct validation error",
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

/// Validates a struct completely: checks if it's reserved, validates UID/NamedId placement,
/// and checks for nested events/errors in fields.
pub fn validate_struct(
    struct_: &crate::Struct_,
    module_name: &str,
    package_address: [u8; 32],
    events: &HashMap<String, Event>,
    abi_errors: &HashMap<String, AbiError>,
) -> Vec<SpecialAttributeError> {
    let mut errors = Vec::new();

    // 1. Check if struct is reserved by Stylus Framework
    errors.extend(check_if_stylus_framework_reserved(
        struct_,
        module_name,
        package_address,
    ));

    // 2. Validate UID/NamedId field placement rules
    errors.extend(validate_uid_and_named_id_placement(
        struct_,
        package_address,
    ));

    // 3. Validate that no fields contain nested events or errors
    errors.extend(check_for_nested_events_or_errors(
        struct_, events, abi_errors,
    ));

    errors
}

/// Validates UID/NamedId field placement rules for structs
fn validate_uid_and_named_id_placement(
    struct_: &crate::Struct_,
    package_address: [u8; 32],
) -> Vec<SpecialAttributeError> {
    let mut errors = Vec::new();

    // Only validate structs not from Stylus Framework
    if package_address != SF_ADDRESS {
        if struct_.has_key {
            // Struct with key ability: first field must be UID or NamedId named "id", no other field can be
            // For now we allow empty structs to have the key ability, but that may change in the future.
            if !struct_.fields.is_empty() {
                let first_field_name = &struct_.fields[0].0;
                let first_field_type = &struct_.fields[0].1;
                let is_first_uid_or_named_id = matches!(
                    first_field_type,
                    Type::UserDataType(name, _) if name == "UID" || name == "NamedId"
                );

                if !is_first_uid_or_named_id {
                    errors.push(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::StructValidation(
                            StructValidationError::StructWithKeyMissingUidField(
                                struct_.name.clone(),
                            ),
                        ),
                        line_of_code: struct_.loc,
                    });
                } else if first_field_name != "id" {
                    errors.push(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::StructValidation(
                            StructValidationError::StructWithKeyFirstFieldWrongName(
                                struct_.name.clone(),
                            ),
                        ),
                        line_of_code: struct_.loc,
                    });
                }

                // Check if any other field is UID or NamedId
                for (_, field_type) in struct_.fields.iter().skip(1) {
                    if matches!(
                        field_type,
                        Type::UserDataType(name, _) if name == "UID" || name == "NamedId"
                    ) {
                        errors.push(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::StructValidation(
                                StructValidationError::MoreThanOneUidFields(struct_.name.clone()),
                            ),
                            line_of_code: struct_.loc,
                        });
                        break;
                    }
                }
            }
        } else {
            // Struct without key ability: no field can be UID or NamedId
            for (_, field_type) in &struct_.fields {
                if matches!(
                    field_type,
                    Type::UserDataType(name, _) if name == "UID" || name == "NamedId"
                ) {
                    errors.push(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::StructValidation(
                            StructValidationError::StructWithoutKeyHasUidField(
                                struct_.name.clone(),
                            ),
                        ),
                        line_of_code: struct_.loc,
                    });
                    break;
                }
            }
        }
    }

    errors
}

/// Checks if the struct is reserved by the Stylus Framework
fn check_if_stylus_framework_reserved(
    struct_: &crate::Struct_,
    module_name: &str,
    package_address: [u8; 32],
) -> Vec<SpecialAttributeError> {
    let mut errors = Vec::new();

    // Check if the struct is reserved by the Stylus Framework
    if package_address != SF_ADDRESS && SF_RESERVED_STRUCTS.contains(&struct_.name.as_str()) {
        errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::StructValidation(
                StructValidationError::FrameworkReservedStruct(
                    struct_.name.clone(),
                    module_name.to_string(),
                ),
            ),
            line_of_code: struct_.loc,
        });
    }

    errors
}

/// Checks if any field of a struct contains nested events or errors
/// Returns a vector of all errors found.
fn check_for_nested_events_or_errors(
    struct_: &crate::Struct_,
    events: &HashMap<String, Event>,
    abi_errors: &HashMap<String, AbiError>,
) -> Vec<SpecialAttributeError> {
    let mut errors = Vec::new();

    for (_, field_type) in &struct_.fields {
        if let Some(error) =
            check_type_for_nested_events_or_errors(field_type, events, abi_errors, struct_.loc)
        {
            errors.push(error);
        }
    }

    errors
}

/// Recursively checks if a type (or nested types) is an event or an error
/// If so, returns a SpecialAttributeError.
fn check_type_for_nested_events_or_errors(
    ty: &Type,
    events: &HashMap<String, Event>,
    abi_errors: &HashMap<String, AbiError>,
    loc: Loc,
) -> Option<SpecialAttributeError> {
    match ty {
        Type::UserDataType(name, _) => {
            // Check if the type itself is an event or error
            if events.contains_key(name) {
                return Some(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::StructValidation(
                        StructValidationError::NestedEvent(name.to_string()),
                    ),
                    line_of_code: loc,
                });
            }
            if abi_errors.contains_key(name) {
                return Some(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::StructValidation(
                        StructValidationError::NestedError(name.to_string()),
                    ),
                    line_of_code: loc,
                });
            }
            None
        }
        Type::Vector(inner) => {
            check_type_for_nested_events_or_errors(inner, events, abi_errors, loc)
        }
        Type::Tuple(types) => {
            for t in types {
                if let Some(error) =
                    check_type_for_nested_events_or_errors(t, events, abi_errors, loc)
                {
                    return Some(error);
                }
            }
            None
        }
        _ => None,
    }
}
