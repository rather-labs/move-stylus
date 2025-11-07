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

use human_redable::{process_functions, process_structs};
use move_bytecode_to_wasm::{
    PackageModuleData, compilation_context::module_data::struct_data::IntermediateType,
};
use move_compiler::shared::files::MappedFiles;
use move_parse_special_attributes::SpecialAttributeError;
use types::Type;

#[derive(Default)]
pub(crate) struct Abi {
    /// This contains all the structs that appear as argument o return of functions. Once we
    /// process the functions this will be the structs appearing in the ABi
    struct_to_process: HashSet<(IntermediateType, Option<Vec<Type>>)>,
}

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

        let result = abi::process_functions(module_data, &package_module_data.modules_data);
        println!("{result:#?}");

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
