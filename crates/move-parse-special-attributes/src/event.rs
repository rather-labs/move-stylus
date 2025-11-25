use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Ability_, Attribute_, AttributeValue_, StructDefinition, Value_},
};

use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind};

#[derive(Debug)]
/// This struct represents the properties of a event struct.
pub struct Event {
    /// Event name
    pub name: String,

    /// `true` if the  event is anonymous, otherwise `false`
    pub is_anonymous: bool,

    /// Indexed parameters. Indexed parameters are parsed in order and there can be max of 4
    /// anonymous events and 3 for non-anonymous parameters.
    pub indexes: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum EventParseError {
    #[error(r#"invalid event attribute"#)]
    InvalidAttribute,

    #[error("expected number as index")]
    IndexExpectedNumber,

    #[error(r#"too many indexed parameters (found {0}, max 3)"#)]
    TooManyIndexedFields(u8),

    #[error(r#"has an invalid index number"#)]
    InvalidIndexNumber,

    #[error(r#"too many indexed parameters (found {0}, max 4)"#)]
    AnonymousTooManyIndexedFields(u8),

    #[error(r#"not marked as an event"#)]
    NotAnEvent,

    #[error(r#"events with key are not supported"#)]
    EventWithKey,
}

impl From<&EventParseError> for DiagnosticInfo {
    fn from(value: &EventParseError) -> Self {
        custom(
            "Event struct error",
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

impl TryFrom<&StructDefinition> for Event {
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

                // To be an event, the first named parameter must be "event". If we dont find it,
                // continue
                let mut event = match parameterized.first() {
                    Some(p) if p.value.attribute_name().value.as_str() == "event" => Event {
                        name: value.name.to_string(),
                        is_anonymous: false,
                        indexes: 0,
                    },
                    _ => continue,
                };

                // Check if the event has the key ability
                if value.abilities.iter().any(|a| a.value == Ability_::Key) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::Event(EventParseError::EventWithKey),
                        line_of_code: value.loc,
                    });
                }

                for attribute in parameterized.iter().skip(1) {
                    match &attribute.value {
                        Attribute_::Name(n) if n.value.as_str() == "anonymous" => {
                            event.is_anonymous = true
                        }
                        Attribute_::Assigned(n, spanned1) if n.value.as_str() == "indexes" => {
                            match &spanned1.value {
                                AttributeValue_::Value(v) if matches!(v.value, Value_::Num(_)) => {
                                    match v.value {
                                        Value_::Num(n) => {
                                            let indexes = n.parse::<u8>().map_err(|_| {
                                                SpecialAttributeError {
                                                    kind: SpecialAttributeErrorKind::Event(
                                                        EventParseError::InvalidIndexNumber,
                                                    ),
                                                    line_of_code: v.loc,
                                                }
                                            })?;
                                            if indexes <= 4 {
                                                event.indexes = indexes
                                            } else {
                                                return Err(SpecialAttributeError {
                                                    kind: SpecialAttributeErrorKind::Event(
                                                        EventParseError::TooManyIndexedFields(
                                                            indexes,
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
                                line_of_code: attribute.loc,
                            });
                        }
                    }
                }

                if !event.is_anonymous && event.indexes == 4 {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::Event(
                            EventParseError::AnonymousTooManyIndexedFields(event.indexes),
                        ),
                        line_of_code: attribute.loc,
                    });
                }

                return Ok(event);
            }
        }

        Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::Event(EventParseError::NotAnEvent),
            line_of_code: value.loc,
        })
    }
}
