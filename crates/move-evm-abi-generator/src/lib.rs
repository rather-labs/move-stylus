#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod common;
mod human_redable;
mod types;

use std::path::Path;

use human_redable::process_functions;
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::{SpecialAttributeError, process_special_attributes};

pub fn generate_abi(path: &Path) -> Result<(), (MappedFiles, Vec<SpecialAttributeError>)> {
    let path = path.join("sources");

    for file in path.read_dir().unwrap() {
        println!("processing {:?}", &file.as_ref().unwrap().path());
        let special_attributes = process_special_attributes(&file.unwrap().path())?;

        let mut result = String::new();
        let entry_functions = special_attributes.functions.iter().filter(|f| f.is_entry);
        process_functions(&mut result, entry_functions);

        println!("{result}");
    }

    Ok(())
}
