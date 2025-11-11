use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Attribute_, AttributeValue_, LeadingNameAccess_, StructDefinition, Value_},
};

use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind};

#[derive(Debug)]
pub struct ExternalStruct {
    pub name: String,
    pub address: [u8; 32],
    pub module_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ExternalStructError {
    #[error("duplicated address attribute")]
    DuplicatedAddressAttribute,

    #[error("expected numerical address")]
    ExpectedNumericalAddress,

    #[error("expected address")]
    ExpectedAddress,

    #[error("invalid attribute")]
    InvalidAttribute,

    #[error("not an external struct")]
    NotAnExternalStruct,

    #[error("duplicated module_name attribute")]
    DuplicatedModuleNameAttribute,

    #[error("expected byte string for module_name")]
    ExpectedByteString,

    #[error("address attribute not defined")]
    AddressNotDefined,

    #[error("module_name attribute not defined")]
    ModuleNameNotDefined,
}

impl From<&ExternalStructError> for DiagnosticInfo {
    fn from(value: &ExternalStructError) -> Self {
        custom(
            "External struct error",
            Severity::BlockingError,
            4,
            4,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

impl TryFrom<&StructDefinition> for ExternalStruct {
    type Error = SpecialAttributeError;

    fn try_from(value: &StructDefinition) -> Result<Self, Self::Error> {
        let mut parsed_address = false;
        let mut parsed_module_name = false;
        // Find the attribute we need
        for attribute in &value.attributes {
            for att in &attribute.value {
                let parameterized = match &att.value {
                    Attribute_::Parameterized(n, spanned) if n.value.as_str() == "ext" => {
                        &spanned.value
                    }
                    _ => continue,
                };

                // To be an event, the first named parameter must be "event". If we dont find it,
                // continue
                let mut external_struct = match parameterized.first() {
                    Some(p) if p.value.attribute_name().value.as_str() == "external_struct" => {
                        ExternalStruct {
                            name: value.name.to_string(),
                            address: [0u8; 32],
                            module_name: String::new(),
                        }
                    }
                    _ => continue,
                };

                for attribute in parameterized.iter().skip(1) {
                    match &attribute.value {
                        // Parse address
                        Attribute_::Assigned(n, spanned1) if n.value.as_str() == "address" => {
                            match &spanned1.value {
                                AttributeValue_::Value(v)
                                    if matches!(v.value, Value_::Address(_)) =>
                                {
                                    match v.value {
                                        Value_::Address(address) => match address.value {
                                            LeadingNameAccess_::AnonymousAddress(
                                                numerical_address,
                                            ) => {
                                                if !parsed_address {
                                                    external_struct.address =
                                                        numerical_address.into_inner().into_bytes();
                                                    parsed_address = true;
                                                } else {
                                                    return Err(SpecialAttributeError {
                                                        kind: SpecialAttributeErrorKind::ExternalStruct(
                                                            ExternalStructError::DuplicatedAddressAttribute,
                                                        ),
                                                        line_of_code: address.loc,
                                                    });
                                                }
                                            }
                                            _ => return Err(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::ExternalStruct(
                                                    ExternalStructError::ExpectedNumericalAddress,
                                                ),
                                                line_of_code: address.loc,
                                            }),
                                        },
                                        _ => {
                                            return Err(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::ExternalStruct(
                                                    ExternalStructError::ExpectedAddress,
                                                ),
                                                line_of_code: v.loc,
                                            });
                                        }
                                    }
                                }

                                _ => {
                                    return Err(SpecialAttributeError {
                                        kind: SpecialAttributeErrorKind::ExternalStruct(
                                            ExternalStructError::ExpectedAddress,
                                        ),
                                        line_of_code: spanned1.loc,
                                    });
                                }
                            }
                        }
                        // Parse module name
                        Attribute_::Assigned(n, spanned1) if n.value.as_str() == "module_name" => {
                            match &spanned1.value {
                                AttributeValue_::Value(v)
                                    if matches!(v.value, Value_::ByteString(_)) =>
                                {
                                    match v.value {
                                        Value_::ByteString(module_name) => {
                                            if !parsed_module_name {
                                                external_struct.module_name =
                                                    module_name.as_str().to_string();
                                                parsed_module_name = true;
                                            } else {
                                                return Err(SpecialAttributeError {
                                                        kind: SpecialAttributeErrorKind::ExternalStruct(
                                                            ExternalStructError::DuplicatedModuleNameAttribute,
                                                        ),
                                                        line_of_code: v.loc,
                                                    });
                                            }
                                        }
                                        _ => {
                                            return Err(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::ExternalStruct(
                                                    ExternalStructError::ExpectedByteString,
                                                ),
                                                line_of_code: v.loc,
                                            });
                                        }
                                    }
                                }
                                _ => {
                                    return Err(SpecialAttributeError {
                                        kind: SpecialAttributeErrorKind::ExternalStruct(
                                            ExternalStructError::ExpectedByteString,
                                        ),
                                        line_of_code: spanned1.loc,
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(SpecialAttributeError {
                                kind: SpecialAttributeErrorKind::ExternalStruct(
                                    ExternalStructError::InvalidAttribute,
                                ),
                                line_of_code: attribute.loc,
                            });
                        }
                    }
                }

                if !parsed_address {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalStruct(
                            ExternalStructError::AddressNotDefined,
                        ),
                        line_of_code: value.loc,
                    });
                }

                if !parsed_module_name {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::ExternalStruct(
                            ExternalStructError::ModuleNameNotDefined,
                        ),
                        line_of_code: value.loc,
                    });
                }

                return Ok(external_struct);
            }
        }

        Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::ExternalStruct(
                ExternalStructError::NotAnExternalStruct,
            ),
            line_of_code: value.loc,
        })
    }
}
