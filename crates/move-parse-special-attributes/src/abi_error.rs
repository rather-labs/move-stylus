// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Ability_, Attribute_, StructDefinition},
};

use crate::{
    SpecialAttributeError,
    error::{DIAGNOSTIC_CATEGORY, SpecialAttributeErrorKind},
};

#[derive(Debug)]
pub struct AbiError {
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AbiErrorParseError {
    #[error(r#"not marked as an abierror"#)]
    NotAnAbiError,

    #[error(r#"errors with generic type parameters are not supported"#)]
    GenericAbiError,

    #[error(r#"abi errors with key are not supported"#)]
    AbiErrorWithKey,

    #[error(r#"The built-in error "Error" cannot be re-defined."#)]
    InvalidAbiErrorName,
}

impl From<&AbiErrorParseError> for DiagnosticInfo {
    fn from(value: &AbiErrorParseError) -> Self {
        custom(
            DIAGNOSTIC_CATEGORY,
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

impl TryFrom<&StructDefinition> for AbiError {
    type Error = SpecialAttributeError;

    fn try_from(value: &StructDefinition) -> Result<Self, Self::Error> {
        // Find the attribute we need
        for attribute in &value.attributes {
            for att in &attribute.value {
                let parameterized = match &att.value {
                    Attribute_::Parameterized(n, spanned) if n.value.as_str() == "ext" => {
                        &spanned.value
                    }
                    _ => continue,
                };

                // To be an abi error, the first named parameter must be "abi_error". If we dont find it,
                // continue
                let abi_error = match parameterized.first() {
                    Some(p) if p.value.attribute_name().value.as_str() == "abi_error" => {
                        // Error is a reserved error name as its the one used for clever errors.
                        if value.name.to_string() == "Error" {
                            return Err(SpecialAttributeError {
                                kind: SpecialAttributeErrorKind::AbiError(
                                    AbiErrorParseError::InvalidAbiErrorName,
                                ),
                                line_of_code: value.loc,
                            });
                        }
                        AbiError {
                            name: value.name.to_string(),
                        }
                    }
                    _ => continue,
                };

                if !value.type_parameters.is_empty() {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::AbiError(
                            AbiErrorParseError::GenericAbiError,
                        ),
                        line_of_code: value.loc,
                    });
                }

                // Check if the event has key
                if value.abilities.iter().any(|a| a.value == Ability_::Key) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::AbiError(
                            AbiErrorParseError::AbiErrorWithKey,
                        ),
                        line_of_code: value.loc,
                    });
                }

                // If we have more than one attribute, return an error
                if parameterized.len() > 1 {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::TooManyAttributes,
                        line_of_code: value.loc,
                    });
                }

                return Ok(abi_error);
            }
        }

        Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::AbiError(AbiErrorParseError::NotAnAbiError),
            line_of_code: value.loc,
        })
    }
}
