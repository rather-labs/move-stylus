#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
mod human_redable;
mod special_types;
mod types;

use std::path::{Path, PathBuf};

use move_bytecode_to_wasm::PackageModuleData;
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::SpecialAttributeError;

pub struct Abi {
    pub file: PathBuf,
    pub content: String,
}

pub fn generate_abi(
    path: &Path,
    package_module_data: &PackageModuleData,
) -> Result<Vec<Abi>, (MappedFiles, Vec<SpecialAttributeError>)> {
    let path = path.join("sources");

    let mut result = Vec::new();
    for file in path.read_dir().unwrap() {
        let file = file.unwrap().path();
        let module_id = package_module_data
            .modules_paths
            .get(&file)
            .expect("error getting module id");

        let module_data = package_module_data
            .modules_data
            .get(module_id)
            .expect("error getting module data");

        let abi = abi::Abi::new(module_data, &package_module_data.modules_data);

        if abi.is_empty() {
            continue;
        }

        let abi = human_redable::process_abi(&abi);
        result.push(Abi { file, content: abi });
    }

    Ok(result)
}
