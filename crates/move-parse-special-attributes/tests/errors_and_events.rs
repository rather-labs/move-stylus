use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    abi_error::AbiErrorParseError, event::EventParseError, error::SpecialAttributeErrorKind,
    FunctionValidationError, process_special_attributes,
};

#[test]
pub fn test_errors_and_events() {
    let file = std::path::Path::new("tests/errors_and_events/sources/misc.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) = process_special_attributes(&absolute) else {
        panic!("Expected error due to invalid errors and events usage");
    };

    assert_eq!(special_attributes_errors.len(), 4);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::AbiError(
                    AbiErrorParseError::AbiErrorWithKey
                )
            ))
            .count()
    );

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::Event(EventParseError::EventWithKey)
            ))
            .count()
    );

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::InvalidRevertFunction
                )
            ))
            .count()
    );

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::InvalidEmitFunction
                )
            ))
            .count()
    );
}
