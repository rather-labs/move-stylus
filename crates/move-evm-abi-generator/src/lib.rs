#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod common;
mod human_redable;
mod types;

use std::{collections::HashSet, path::Path};

use human_redable::process_functions;
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::{SpecialAttributeError, process_special_attributes};

#[derive(Default)]
pub(crate) struct Abi {
    /// This contains all the structs that appear as argument o return of functions. Once we
    /// process the functions this will be the structs appearing in the ABi
    struct_to_process: HashSet<String>,
}

pub fn generate_abi(path: &Path) -> Result<(), (MappedFiles, Vec<SpecialAttributeError>)> {
    let path = path.join("sources");

    for file in path.read_dir().unwrap() {
        println!("processing {:?}", &file.as_ref().unwrap().path());
        let special_attributes = process_special_attributes(&file.unwrap().path())?;

        let mut abi = Abi::default();

        let mut result = String::new();
        let entry_functions = special_attributes.functions.iter().filter(|f| f.is_entry);
        process_functions(
            &mut result,
            entry_functions,
            &special_attributes.structs,
            &mut abi,
        );

        println!("{result}");
    }

    Ok(())
}
