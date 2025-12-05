use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    FunctionValidationError, error::SpecialAttributeErrorKind, process_special_attributes,
};

#[test]
pub fn test_function_validation() {
    let package_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let file = std::path::Path::new("tests/functions/sources/misc.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) =
        process_special_attributes(&absolute, package_address)
    else {
        panic!("Expected error due to invalid function validation");
    };

    assert_eq!(special_attributes_errors.len(), 3);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::EntryFunctionReturnsKeyStruct
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
                    FunctionValidationError::InvalidUidArgument
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
                    FunctionValidationError::InvalidNamedIdArgument
                )
            ))
            .count()
    );
}
