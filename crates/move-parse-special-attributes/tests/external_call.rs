use std::{fs, path::PathBuf};

use move_parse_special_attributes::{
    ExternalCallFunctionError, error::SpecialAttributeErrorKind, process_special_attributes,
};

#[test]
pub fn test_external_call_general() {
    let file = std::path::Path::new("tests/external_call/sources/external_call.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let Err(special_attributes_errors) = process_special_attributes(&absolute) else {
        panic!("Expected error due to invalid external_call functions");
    };

    println!("{special_attributes_errors:#?}");

    assert_eq!(special_attributes_errors.len(), 2);

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
}
