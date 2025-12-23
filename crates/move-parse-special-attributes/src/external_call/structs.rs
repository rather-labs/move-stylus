use move_compiler::parser::ast::{Ability_, StructDefinition};

use crate::{
    SpecialAttributeError,
    error::SpecialAttributeErrorKind,
    shared::{contains_abilities, get_single_type_name},
};

use super::error::ExternalCallStructError;

const CONFIGURATION_TYPE_NAME: &str = "CrossContractCall";

fn check_fields(s: &StructDefinition) -> Option<Vec<SpecialAttributeError>> {
    let mut errors = Vec::new();

    match &s.fields {
        move_compiler::parser::ast::StructFields::Positional(fields) => {
            if fields.is_empty() {
                errors.push(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::ExternalCallStruct(
                        ExternalCallStructError::MissingConfiguration,
                    ),
                    line_of_code: s.loc,
                });
            } else if fields.len() > 1 {
                errors.push(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::ExternalCallStruct(
                        ExternalCallStructError::TooManyFields,
                    ),
                    line_of_code: s.loc,
                });
            } else {
                let (_, type_) = &fields[0];
                if let Some(name) = get_single_type_name(&type_.value) {
                    if name.as_str() != CONFIGURATION_TYPE_NAME {
                        errors.push(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::ExternalCallStruct(
                                ExternalCallStructError::InvalidConfigurationField,
                            ),
                            line_of_code: type_.loc,
                        })
                    }
                } else {
                    errors.push(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalCallStruct(
                            ExternalCallStructError::InvalidConfigurationField,
                        ),
                        line_of_code: type_.loc,
                    })
                }
            }
        }
        _ => errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalCallStruct(
                ExternalCallStructError::InvalidConfigurationField,
            ),
            line_of_code: s.loc,
        }),
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

pub(crate) fn validate_external_call_struct(
    s: &StructDefinition,
) -> Result<(), Vec<SpecialAttributeError>> {
    let mut errors = Vec::new();

    // Validate it has de drop ability
    if !contains_abilities(
        &[Ability_::Drop],
        &s.abilities
            .iter()
            .map(|a| a.value)
            .collect::<Vec<Ability_>>(),
    ) {
        errors.push(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalCallStruct(
                ExternalCallStructError::MissingAbilityDrop,
            ),
            line_of_code: s.loc,
        });
    }

    // Check the fields
    if let Some(e) = check_fields(s) {
        errors.extend(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
