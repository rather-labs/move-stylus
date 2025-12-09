use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    ExternalCallFunctionError, ExternalCallStructError, error::SpecialAttributeErrorKind,
    process_special_attributes,
};

#[test]
pub fn test_external_call_general() {
    let package_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let file = std::path::Path::new("tests/external_call/sources/external_call.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

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

    let Err((_, special_attributes_errors)) = process_special_attributes(
        &absolute,
        package_address,
        &std::collections::HashMap::new(),
        &address_alias_instantiation,
    ) else {
        panic!("Expected error due to invalid external_call functions");
    };

    assert_eq!(special_attributes_errors.len(), 8);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::ExternalCallFunction(
                    ExternalCallFunctionError::FunctionIsNotNative
                )
            ))
            .count()
    );

    assert_eq!(1, special_attributes_errors.iter().filter(|e| matches!(
        &e.kind,
        SpecialAttributeErrorKind::ExternalCallFunction(ExternalCallFunctionError::InvalidReturnType(t)) if t == "u64"
    )).count());

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::ExternalCallFunction(
                    ExternalCallFunctionError::InvalidFirstArgument
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
                SpecialAttributeErrorKind::ExternalCallStruct(
                    ExternalCallStructError::MissingConfiguration
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
                SpecialAttributeErrorKind::ExternalCallStruct(
                    ExternalCallStructError::InvalidConfigurationField
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
                SpecialAttributeErrorKind::ExternalCallStruct(
                    ExternalCallStructError::TooManyFields
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
                SpecialAttributeErrorKind::ExternalCallStruct(
                    ExternalCallStructError::MissingAbilityDrop
                )
            ))
            .count()
    );
}
