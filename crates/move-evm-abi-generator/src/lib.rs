#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
mod human_redable;
mod special_types;
mod types;

use std::path::Path;

use move_bytecode_to_wasm::PackageModuleData;
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::SpecialAttributeError;

pub fn generate_abi(
    path: &Path,
    package_module_data: &PackageModuleData,
) -> Result<(), (MappedFiles, Vec<SpecialAttributeError>)> {
    let path = path.join("sources");

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

        let abi = abi::get_module_abi(module_data, &package_module_data.modules_data);

        let mut result = String::new();

        human_redable::process_structs(&mut result, &abi);
        human_redable::process_functions(&mut result, &abi);
        println!("{result}");
    }

    Ok(())
}
