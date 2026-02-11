use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    FunctionValidationError, error::SpecialAttributeErrorKind, process_special_attributes,
};
use move_symbol_pool::Symbol;

#[test]
pub fn test_storage_objects() {
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
    let file = std::path::Path::new("tests/structs/sources/storage_objects.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) = process_special_attributes(
        &absolute,
        package_address,
        &std::collections::HashMap::new(),
        &address_alias_instantiation,
    ) else {
        panic!("Expected error due to invalid errors and events usage");
    };

    assert_eq!(special_attributes_errors.len(), 18);

    assert_eq!(
        3,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(&e.kind, SpecialAttributeErrorKind::RepeatedStorageObject(_)))
            .count()
    );

    assert_eq!(
        11,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::ParameterNotFound(_),
                )
            ))
            .count()
    );

    assert_eq!(
        3,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::StructWithoutKey(_),
                )
            ))
            .count()
    );
}
