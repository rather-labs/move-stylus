use move_compiler::parser::ast::{Attribute_, AttributeValue_, LeadingNameAccess_, Value_};
use move_symbol_pool::Symbol;

use crate::{
    SpecialAttributeError, error::SpecialAttributeErrorKind, event::EventParseError,
    external_call::external_struct::ExternalStructError,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StructModifier {
    ExternalStruct {
        address: [u8; 32],
        module_name: Symbol,
    },
    ExternalCall,
    Event {
        indexes: u8,
        is_anonymous: bool,
    },
    AbiError,
}

impl StructModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExternalStruct { .. } => "external_struct",
            Self::ExternalCall => "external_call",
            Self::Event { .. } => "event",
            Self::AbiError => "abi_error",
        }
    }

    pub fn parse_struct_modifier(
        attribute: &Attribute_,
    ) -> Result<Option<Self>, SpecialAttributeError> {
        match attribute {
            Attribute_::Parameterized(name, spanned1) => match name.value.as_str() {
                "event" => {
                    let mut is_anonymous = false;
                    let mut indexes = 0;
                    for attr in &spanned1.value {
                        match &attr.value {
                            Attribute_::Name(n) if n.value.as_str() == "anonymous" => {
                                is_anonymous = true;
                            }
                            Attribute_::Assigned(n, spanned1) if n.value.as_str() == "indexes" => {
                                match &spanned1.value {
                                    AttributeValue_::Value(v)
                                        if matches!(v.value, Value_::Num(_)) =>
                                    {
                                        match v.value {
                                            Value_::Num(n) => {
                                                let parsed_indexes =
                                                    n.parse::<u8>().map_err(|_| {
                                                        SpecialAttributeError {
                                                            kind: SpecialAttributeErrorKind::Event(
                                                                EventParseError::InvalidIndexNumber,
                                                            ),
                                                            line_of_code: v.loc,
                                                        }
                                                    })?;
                                                if parsed_indexes <= 4 {
                                                    indexes = parsed_indexes;
                                                } else {
                                                    return Err(SpecialAttributeError {
                                                        kind: SpecialAttributeErrorKind::Event(
                                                            EventParseError::TooManyIndexedFields(
                                                                parsed_indexes,
                                                            ),
                                                        ),
                                                        line_of_code: v.loc,
                                                    });
                                                }
                                            }
                                            _ => {
                                                return Err(SpecialAttributeError {
                                                    kind: SpecialAttributeErrorKind::Event(
                                                        EventParseError::IndexExpectedNumber,
                                                    ),
                                                    line_of_code: v.loc,
                                                });
                                            }
                                        }
                                    }

                                    _ => {
                                        return Err(SpecialAttributeError {
                                            kind: SpecialAttributeErrorKind::Event(
                                                EventParseError::IndexExpectedNumber,
                                            ),
                                            line_of_code: spanned1.loc,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(SpecialAttributeError {
                                    kind: SpecialAttributeErrorKind::Event(
                                        EventParseError::InvalidAttribute,
                                    ),
                                    line_of_code: spanned1.loc,
                                });
                            }
                        }
                    }

                    if !is_anonymous && indexes >= 4 {
                        return Err(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::Event(
                                EventParseError::AnonymousTooManyIndexedFields(indexes),
                            ),
                            line_of_code: spanned1.loc,
                        });
                    }

                    Ok(Some(Self::Event {
                        indexes,
                        is_anonymous,
                    }))
                }

                "external_struct" => {
                    let mut parsed_address = false;
                    let mut parsed_module_name = false;

                    let mut address = [0; 32];
                    let mut module_name = Symbol::from("");

                    for attribute in &spanned1.value {
                        match &attribute.value {
                            // Parse address
                            Attribute_::Assigned(n, spanned1) if n.value.as_str() == "address" => {
                                match &spanned1.value {
                                    AttributeValue_::Value(v)
                                        if matches!(v.value, Value_::Address(_)) =>
                                    {
                                        match v.value {
                                            Value_::Address(addr) => match addr.value {
                                                LeadingNameAccess_::AnonymousAddress(
                                                    numerical_address,
                                                ) => {
                                                    if !parsed_address {
                                                        address = numerical_address.into_inner().into_bytes();
                                                        parsed_address = true;
                                                    } else {
                                                        return Err(SpecialAttributeError {
                                                            kind: SpecialAttributeErrorKind::ExternalStruct(
                                                                ExternalStructError::DuplicatedAddressAttribute,
                                                            ),
                                                            line_of_code: addr.loc,
                                                        });
                                                    }
                                                }
                                                _ => return Err(SpecialAttributeError {
                                                    kind: SpecialAttributeErrorKind::ExternalStruct(
                                                        ExternalStructError::ExpectedNumericalAddress,
                                                    ),
                                                    line_of_code: addr.loc,
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
                                            Value_::ByteString(m_name) => {
                                                if !parsed_module_name {
                                                    module_name = m_name;
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

                    Ok(Some(Self::ExternalStruct {
                        address,
                        module_name,
                    }))
                }

                _ => spanned1
                    .value
                    .iter()
                    .map(|s| Self::parse_struct_modifier(&s.value))
                    .next()
                    .transpose()
                    .map(|opt| opt.flatten()),
            },
            Attribute_::Name(name) => match name.value.as_str() {
                "external_call" => Ok(Some(Self::ExternalCall)),
                "event" => Ok(Some(Self::Event {
                    indexes: 0,
                    is_anonymous: false,
                })),
                "abi_error" => Ok(Some(Self::AbiError)),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}
