use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Attribute_, StructDefinition},
};

use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind};

#[derive(Debug)]
pub struct AbiError {
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AbiErrorParseError {
    #[error(r#"not marked as an abierror"#)]
    NotAnAbiError,
}

impl From<&AbiErrorParseError> for DiagnosticInfo {
    fn from(value: &AbiErrorParseError) -> Self {
        custom(
            "Abi error parsing error",
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
        // Find the attribute we neekd
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
                    Some(p) if p.value.attribute_name().value.as_str() == "abi_error" => AbiError {
                        name: value.name.to_string(),
                    },
                    _ => continue,
                };

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
