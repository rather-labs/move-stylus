#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
mod human_redable;
mod special_types;
mod types;

use std::{collections::HashSet, path::PathBuf};

use move_binary_format::file_format::{Bytecode, Signature};
use move_bytecode_to_wasm::PackageModuleData;
use move_compiler::shared::files::MappedFiles;
use move_core_types::language_storage::ModuleId;
use move_package::compilation::{
    compiled_package::{CompiledPackage, CompiledUnitWithSource},
    package_layout::CompiledPackageLayout,
};
use move_parse_special_attributes::SpecialAttributeError;

pub struct Abi {
    pub file: PathBuf,
    pub content: String,
}

pub fn generate_abi(
    package: &CompiledPackage,
    root_compiled_units: &[&CompiledUnitWithSource],
    package_module_data: &PackageModuleData,
) -> Result<Vec<Abi>, (MappedFiles, Vec<SpecialAttributeError>)> {
    let mut result = Vec::new();
    for root_compiled_module in root_compiled_units {
        let file = &root_compiled_module.source_path;
        let module_id = package_module_data
            .modules_paths
            .get(file)
            .expect("error getting module id");

        let module_data = package_module_data
            .modules_data
            .get(module_id)
            .expect("error getting module data");

        // Collect all the calls to emit<> function to know which events are emmited in this module
        // so we can put them in the ABI
        let mut processed_modules = HashSet::new();
        collect_generic_function_calls(
            package,
            root_compiled_module,
            root_compiled_units,
            &mut processed_modules,
        );

        let abi = abi::Abi::new(module_data, &package_module_data.modules_data);

        if abi.is_empty() {
            continue;
        }

        let abi = human_redable::process_abi(&abi);
        result.push(Abi {
            file: file.to_path_buf(),
            content: abi,
        });
    }

    Ok(result)
}

#[derive(Debug)]
struct FunctionCall {
    module_id: ModuleId,
    identifier: String,
    signature: Signature,
}

#[derive(Debug)]
struct EventStruct {
    module_id: ModuleId,
    identifier: String,
    signature: Signature,
}

fn collect_generic_function_calls(
    package: &CompiledPackage,
    root_compiled_module: &CompiledUnitWithSource,
    root_compiled_units: &[&CompiledUnitWithSource],
    processed_modules: &mut HashSet<ModuleId>,
) -> Vec<FunctionCall> {
    let module = &root_compiled_module.unit.module;

    processed_modules.insert(module.self_id());

    // Process top level functions
    let mut result = Vec::new();
    let mut top_level = Vec::new();
    for function in module.function_defs() {
        if let Some(ref code) = function.code {
            for instruction in &code.code {
                match instruction {
                    Bytecode::CallGeneric(idx) => {
                        let instantiation = module.function_instantiation_at(*idx);
                        let function_handle = module.function_handle_at(instantiation.handle);
                        let module_id = module
                            .module_id_for_handle(module.module_handle_at(function_handle.module));
                        let identifier = module.identifier_at(function_handle.name).to_string();

                        let signature = module.signature_at(instantiation.type_parameters).map();
                        top_level.push(FunctionCall {
                            module_id,
                            identifier,
                            signature,
                        });
                        /*
                        println!("Instantaition: {instantiation:?}");
                        println!("Function: {function_handle:?}");
                        println!("Module: {module:?}");
                        */
                    }
                    _ => continue,
                }
            }
        }
    }

    processed_modules.insert(module.self_id());

    // Recursively process calls
    for function_call in &top_level {
        if function_call.module_id != module.self_id()
            && !processed_modules.contains(&function_call.module_id)
        {
            let child_module = package
                .deps_compiled_units
                .iter()
                .find(|(_, c)| c.unit.module.self_id() == function_call.module_id)
                .map(|(_, c)| c)
                .or_else(|| {
                    root_compiled_units
                        .iter()
                        .find(|c| c.unit.module.self_id() == function_call.module_id)
                        .copied()
                })
                .unwrap_or_else(|| panic!("Could not find dependency {}", function_call.module_id));

            let child_module_result = collect_generic_function_calls(
                package,
                child_module,
                root_compiled_units,
                processed_modules,
            );
            result.extend(child_module_result);
        }
    }

    result.extend(top_level);

    println!("\n\n{result:#?}\n\n");

    result
}
