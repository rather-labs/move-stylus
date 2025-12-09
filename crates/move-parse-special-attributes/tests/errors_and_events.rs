use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    FunctionValidationError, StructValidationError, abi_error::AbiErrorParseError,
    error::SpecialAttributeErrorKind, event::EventParseError, process_special_attributes,
};

#[test]
pub fn test_errors_and_events() {
    let package_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let address_alias_instantiation = std::collections::HashMap::from([
        (
            "std".to_string(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1,
            ],
        ),
        (
            "stylus".to_string(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 2,
            ],
        ),
        (
            "test".to_string(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
        ),
    ]);
    let file = std::path::Path::new("tests/errors_and_events/sources/misc.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) = process_special_attributes(
        &absolute,
        package_address,
        &std::collections::HashMap::new(),
        &address_alias_instantiation,
    ) else {
        panic!("Expected error due to invalid errors and events usage");
    };

    assert_eq!(special_attributes_errors.len(), 7);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::AbiError(AbiErrorParseError::AbiErrorWithKey)
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

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::AbiError(AbiErrorParseError::InvalidAbiErrorName)
            ))
            .count()
    );

    assert_eq!(
        2,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::StructValidation(
                    StructValidationError::StructWithKeyMissingUidField(_)
                )
            ))
            .count()
    );
}
