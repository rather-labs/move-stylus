#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
mod human_readable;
mod json_format;
mod special_types;
mod types;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use move_binary_format::file_format::{
    Bytecode, CompiledModule, DatatypeHandleIndex, SignatureToken, StructDefInstantiationIndex,
    StructDefinitionIndex,
};
use move_bytecode_to_wasm::compilation_context as ctx;
use move_bytecode_to_wasm::{PackageModuleData, compilation_context::ModuleData};
use move_compiler::shared::files::MappedFiles;
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use move_parse_special_attributes::SpecialAttributeError;

use crate::common::snake_to_upper_camel;
use crate::types::Type;

pub struct Abi {
    pub file: PathBuf,
    pub content_json: Option<String>,
    pub content_human_readable: Option<String>,
}

pub fn generate_abi(
    package: &CompiledPackage,
    root_compiled_units: &[&CompiledUnitWithSource],
    package_module_data: &PackageModuleData,
    generate_json: bool,
    generate_human_readable: bool,
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

        // Collect all the calls to emit<> and revert<> function to know which events and errors
        // are emmited in this module so we can put them in the ABI
        let mut processed_modules = HashSet::new();
        let (module_emitted_events, module_errors) = process_events_and_errors(
            package,
            root_compiled_module,
            root_compiled_units,
            &mut processed_modules,
            &package_module_data.modules_data,
        );

        let abi = abi::Abi::new(
            module_data,
            &package_module_data.modules_data,
            &module_emitted_events,
            &module_errors,
        );

        if abi.is_empty() {
            continue;
        }

        let json_abi = if generate_json {
            Some(json_format::process_abi(&abi))
        } else {
            None
        };

        let hr_abi = if generate_human_readable {
            Some(human_readable::process_abi(&abi))
        } else {
            None
        };

        result.push(Abi {
            file: file.to_path_buf(),
            content_json: json_abi,
            content_human_readable: hr_abi,
        });
    }

    Ok(result)
}

const STYLUS_FRAMEWORK_ADDRESS: AccountAddress = AccountAddress::new([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
]);

#[derive(Debug)]
pub(crate) struct FunctionCall {
    module_id: ModuleId,
    identifier: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct EventStruct {
    module_id: ModuleId,
    identifier: String,
    struct_def_instantiation_index: Option<StructDefInstantiationIndex>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct ErrorStruct {
    module_id: ModuleId,
    identifier: String,
}

/// This functions recursively searches for `emit` and `revert` calls to put in the ABI which
/// structs are used for events and errors respectively.
fn process_events_and_errors(
    package: &CompiledPackage,
    root_compiled_module: &CompiledUnitWithSource,
    root_compiled_units: &[&CompiledUnitWithSource],
    processed_modules: &mut HashSet<ModuleId>,
    modules_data: &HashMap<ctx::ModuleId, ModuleData>,
) -> (HashSet<EventStruct>, HashSet<ErrorStruct>) {
    let module = &root_compiled_module.unit.module;

    processed_modules.insert(module.self_id());

    // Process top level functions
    let mut top_level_functions = Vec::new();
    let mut top_level_events = HashSet::new();
    let mut top_level_errors = HashSet::new();
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

                        if module_id.address() == &STYLUS_FRAMEWORK_ADDRESS {
                            if module_id.name().as_str() == "event" && identifier == "emit" {
                                let signature = module.signature_at(instantiation.type_parameters);
                                match &signature.0[0] {
                                    SignatureToken::Datatype(datatype_handle_index) => {
                                        let struct_handle =
                                            module.datatype_handle_at(*datatype_handle_index);
                                        top_level_events.insert(EventStruct {
                                            module_id: module.module_id_for_handle(
                                                module.module_handle_at(struct_handle.module),
                                            ),
                                            identifier: module
                                                .identifier_at(struct_handle.name)
                                                .to_string(),
                                            struct_def_instantiation_index: None,
                                        });
                                    }
                                    SignatureToken::DatatypeInstantiation(data) => {
                                        let (datatype_handle_index, _) = data.as_ref();
                                        let struct_handle =
                                            module.datatype_handle_at(*datatype_handle_index);

                                        let struct_def_instantiation_index =
                                            find_struct_def_instantiation_index(
                                                module,
                                                *datatype_handle_index,
                                            )
                                            .unwrap();

                                        let event_module_id = module.module_id_for_handle(
                                            module.module_handle_at(struct_handle.module),
                                        );

                                        let event_module = modules_data
                                            .get(&ctx::ModuleId {
                                                address: event_module_id
                                                    .address()
                                                    .into_bytes()
                                                    .into(),
                                                module_name: event_module_id.name().to_string(),
                                            })
                                            .unwrap();

                                        let concrete_types = event_module
                                            .structs
                                            .get_generic_struct_types_instances(
                                                &struct_def_instantiation_index,
                                            )
                                            .unwrap();

                                        let concrete_types_names = concrete_types
                                            .iter()
                                            .map(|t| {
                                                Type::from_intermediate_type(t, modules_data).name()
                                            })
                                            .collect::<Vec<String>>()
                                            .join("_");

                                        let event_identifier = snake_to_upper_camel(&format!(
                                            "{}_{}",
                                            module.identifier_at(struct_handle.name),
                                            concrete_types_names
                                        ));

                                        top_level_events.insert(EventStruct {
                                            module_id: event_module_id,
                                            identifier: event_identifier,
                                            struct_def_instantiation_index: Some(
                                                struct_def_instantiation_index,
                                            ),
                                        });
                                    }
                                    _ => panic!(
                                        "invalid type found in emit function {:?}",
                                        signature.0[0]
                                    ),
                                }
                            } else if module_id.name().as_str() == "error" && identifier == "revert"
                            {
                                let signature = module.signature_at(instantiation.type_parameters);
                                match signature.0[0] {
                                    SignatureToken::Datatype(datatype_handle_index) => {
                                        let struct_handle =
                                            module.datatype_handle_at(datatype_handle_index);
                                        top_level_errors.insert(ErrorStruct {
                                            module_id: module.module_id_for_handle(
                                                module.module_handle_at(struct_handle.module),
                                            ),
                                            identifier: module
                                                .identifier_at(struct_handle.name)
                                                .to_string(),
                                        });
                                    }
                                    _ => panic!("invalid type found in revert function"),
                                }
                            }
                        }

                        top_level_functions.push(FunctionCall {
                            module_id,
                            identifier,
                        });
                    }
                    Bytecode::Call(idx) => {
                        let function_handle = module.function_handle_at(*idx);
                        let module_id = module
                            .module_id_for_handle(module.module_handle_at(function_handle.module));
                        let identifier = module.identifier_at(function_handle.name).to_string();
                        top_level_functions.push(FunctionCall {
                            module_id,
                            identifier,
                        });
                    }

                    _ => continue,
                }
            }
        }
    }

    let mut result_events = HashSet::new();
    let mut result_errors = HashSet::new();
    // Recursively process calls
    for function_call in &top_level_functions {
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

            let (events, errors) = process_events_and_errors(
                package,
                child_module,
                root_compiled_units,
                processed_modules,
                modules_data,
            );
            result_events.extend(events);
            result_errors.extend(errors);
        }
    }

    result_events.extend(top_level_events);
    result_errors.extend(top_level_errors);

    (result_events, result_errors)
}

/// Maps a `DatatypeHandleIndex` to a `StructDefInstantiationIndex` by finding the struct definition
/// and then searching for a matching instantiation. Optionally matches type parameters if provided.
fn find_struct_def_instantiation_index(
    module: &CompiledModule,
    datatype_handle_index: DatatypeHandleIndex,
) -> Option<StructDefInstantiationIndex> {
    // Verify the struct definition exists
    module.find_struct_def(datatype_handle_index)?;

    // Get the index of this struct definition
    let struct_def_index = module
        .struct_defs()
        .iter()
        .position(|d| d.struct_handle == datatype_handle_index)
        .map(|idx| StructDefinitionIndex::new(idx as u16))?;

    // Search through struct instantiations to find one that matches
    for (idx, instantiation) in module.struct_instantiations().iter().enumerate() {
        if instantiation.def == struct_def_index {
            // If no type parameters specified, return the first match
            return Some(StructDefInstantiationIndex::new(idx as u16));
        }
    }

    None
}
