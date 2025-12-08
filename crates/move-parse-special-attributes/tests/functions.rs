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

    // Build HashMap with ModuleId -> Vec<Struct_>
    let mut deps_structs = std::collections::HashMap::new();

    // First, process misc_external.move to get its structs
    let external_file = std::path::Path::new("tests/functions/sources/misc_external.move");
    let external_absolute: PathBuf = fs::canonicalize(external_file).unwrap();

    let external_special_attributes = process_special_attributes(
        &external_absolute,
        package_address,
        &std::collections::HashMap::new(),
    )
    .expect("misc_external.move should process successfully");

    deps_structs.insert(
        external_special_attributes.module_name.clone(),
        external_special_attributes.structs,
    );

    // First, process misc_external.move to get its structs
    let external_file = std::path::Path::new("tests/functions/sources/misc_external_2.move");
    let external_absolute: PathBuf = fs::canonicalize(external_file).unwrap();

    let external_special_attributes = process_special_attributes(
        &external_absolute,
        package_address,
        &std::collections::HashMap::new(),
    )
    .expect("misc_external_2.move should process successfully");

    deps_structs.insert(
        external_special_attributes.module_name.clone(),
        external_special_attributes.structs,
    );

    // Now process misc.move with the dependency structs
    let file = std::path::Path::new("tests/functions/sources/misc.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err((_, special_attributes_errors)) =
        process_special_attributes(&absolute, package_address, &deps_structs)
    else {
        panic!("Expected error due to invalid function validation");
    };

    assert_eq!(special_attributes_errors.len(), 5);

    assert_eq!(
        3,
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
