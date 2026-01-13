use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};
use move_symbol_pool::Symbol;

#[derive(Debug)]
/// This struct represents the properties of a event struct.
pub struct Event {
    /// Event name
    pub name: Symbol,

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
