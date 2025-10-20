use std::{fs, path::PathBuf};

use move_parse_special_attributes::process_special_attributes;

#[test]
pub fn test_external_call_payable() {
    let file = std::path::Path::new("tests/external_call/sources/external_call_payable.move");
    let absolute: PathBuf = fs::canonicalize(file).unwrap();

    let special_attributes = process_special_attributes(&absolute);

    println!("{:?}", special_attributes);
}
