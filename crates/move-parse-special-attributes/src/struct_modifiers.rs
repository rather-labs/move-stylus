use move_compiler::parser::ast::{Attribute_, AttributeValue_, Value_};

use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind, event::EventParseError};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StructModifier {
    ExternalStruct,
    ExternalCall,
    Event { indexes: u8, is_anonymous: bool },
    AbiError,
}

impl StructModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExternalStruct => "external_struct",
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
                    println!("Parsing event attributes: {:?}", spanned1.value);
                    for attr in &spanned1.value {
                        println!("Parsing event attribute: {:?}", attr);
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
                _ => spanned1
                    .value
                    .iter()
                    .map(|s| Self::parse_struct_modifier(&s.value))
                    .next()
                    .transpose()
                    .map(|opt| opt.flatten()),
            },
            Attribute_::Name(name) => match name.value.as_str() {
                "external_struct" => Ok(Some(Self::ExternalStruct)),
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
