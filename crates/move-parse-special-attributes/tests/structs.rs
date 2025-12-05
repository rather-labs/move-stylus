use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    StructValidationError, error::SpecialAttributeErrorKind, process_special_attributes,
};

#[test]
pub fn test_struct_validation() {
    let package_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let file = std::path::Path::new("tests/structs/sources/struct_validations.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) =
        process_special_attributes(&absolute, package_address)
    else {
        panic!("Expected error due to invalid struct validation");
    };

    assert_eq!(special_attributes_errors.len(), 16);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::StructValidation(
                    StructValidationError::StructWithKeyFirstFieldWrongName(_)
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
                SpecialAttributeErrorKind::StructValidation(
                    StructValidationError::StructWithKeyMissingUidField(_)
                )
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
                    StructValidationError::MoreThanOneUidFields(_)
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
                SpecialAttributeErrorKind::StructValidation(
                    StructValidationError::StructWithoutKeyHasUidField(_)
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
                SpecialAttributeErrorKind::StructValidation(StructValidationError::NestedEvent(_))
            ))
            .count()
    );

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::StructValidation(StructValidationError::NestedError(_))
            ))
            .count()
    );

    assert_eq!(
        9,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::StructValidation(
                    StructValidationError::FrameworkReservedStruct(_, _)
                )
            ))
            .count()
    );
}
