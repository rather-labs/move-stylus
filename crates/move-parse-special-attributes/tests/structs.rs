use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    StructValidationError, error::SpecialAttributeErrorKind, process_special_attributes,
};
use move_symbol_pool::Symbol;

#[test]
pub fn test_struct_validation() {
    let package_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let address_alias_instantiation = std::collections::HashMap::from([
        (
            Symbol::from("std"),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1,
            ],
        ),
        (
            Symbol::from("stylus"),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 2,
            ],
        ),
        (
            Symbol::from("test"),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
        ),
    ]);
    let file = std::path::Path::new("tests/structs/sources/struct_validations.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) = process_special_attributes(
        &absolute,
        package_address,
        &std::collections::HashMap::new(),
        &address_alias_instantiation,
    ) else {
        panic!("Expected error due to invalid struct validation");
    };

    assert_eq!(special_attributes_errors.len(), 15);

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
        8,
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
