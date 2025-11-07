#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
mod human_redable;
mod special_types;
mod types;

use std::{collections::HashSet, path::Path};

use human_redable::process_functions;
use move_bytecode_to_wasm::{
    PackageModuleData, compilation_context::module_data::struct_data::IntermediateType,
};
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::SpecialAttributeError;
use types::Type;

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

        let (functions, structs_to_process) =
            abi::process_functions(module_data, &package_module_data.modules_data);

        let mut processed_structs = HashSet::new();
        let structs = abi::process_structs(
            structs_to_process,
            &package_module_data.modules_data,
            &mut processed_structs,
        );

        // println!("{structs:#?}");

        let mut result = String::new();

        let abi = abi::Abi { functions, structs };

        human_redable::process_structs(&mut result, &abi);
        human_redable::process_functions(&mut result, &abi);
        println!("{result}");

        /*
        let mut abi = Abi::default();


        let mut result = String::new();
        process_functions(
            &mut result,
            module_data,
            &package_module_data.modules_data,
            &mut abi,
        );


        process_structs(&mut result, &package_module_data.modules_data, &mut abi);

        println!("{result}");
        */
    }

    Ok(())
}
