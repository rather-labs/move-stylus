use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    ExternalCallError, error::SpecialAttributeErrorKind, process_special_attributes,
};

#[test]
pub fn test_external_call_payable() {
    let file = std::path::Path::new("tests/external_call/sources/external_call_payable.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err(special_attributes_errors) = process_special_attributes(&absolute) else {
        panic!("Expected error due to invalid external_call functions");
    };

    assert_eq!(special_attributes_errors.len(), 3);

    assert_eq!(1, special_attributes_errors.iter().filter(|e| matches!(
        &e.kind,
        SpecialAttributeErrorKind::ExternalCall(ExternalCallError::InvalidValueArgumentType(t)) if t == "u128"
    )).count());

    assert_eq!(1, special_attributes_errors.iter().filter(|e| matches!(
        &e.kind,
        SpecialAttributeErrorKind::ExternalCall(ExternalCallError::InvalidValueArgumentName(name)) if name == "amount"
    )).count());

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::ExternalCall(ExternalCallError::ValueArgumentMissing)
            ))
            .count()
    );
}

#[test]
pub fn test_external_call_general() {
    let file = std::path::Path::new("tests/external_call/sources/external_call.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err(special_attributes_errors) = process_special_attributes(&absolute) else {
        panic!("Expected error due to invalid external_call functions");
    };

    assert_eq!(special_attributes_errors.len(), 2);

    assert_eq!(
        1,
        special_attributes_errors
            .iter()
            .filter(|e| matches!(
                &e.kind,
                SpecialAttributeErrorKind::ExternalCall(ExternalCallError::FunctionIsNotNative)
            ))
            .count()
    );

    assert_eq!(1, special_attributes_errors.iter().filter(|e| matches!(
        &e.kind,
        SpecialAttributeErrorKind::ExternalCall(ExternalCallError::InvalidReturnType(t)) if t == "u64"
    )).count());
}
