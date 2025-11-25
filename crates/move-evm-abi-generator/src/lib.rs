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
use move_bytecode_to_wasm::{
    PackageModuleData,
    compilation_context::{ModuleData, module_data::struct_data::IntermediateType},
};
use move_compiler::shared::files::MappedFiles;
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use move_parse_special_attributes::SpecialAttributeError;

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
    type_parameters: Option<Vec<IntermediateType>>,
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
                                            type_parameters: None,
                                        });
                                    }
                                    SignatureToken::DatatypeInstantiation(data) => {
                                        let (datatype_handle_index, type_parameters) =
                                            data.as_ref();
                                        let struct_handle =
                                            module.datatype_handle_at(*datatype_handle_index);

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

                                        let event_type_parameters = type_parameters
                                            .iter()
                                            .map(|t| IntermediateType::try_from_signature_token(t, &event_module.datatype_handles_map))
                                            .collect::<std::result::Result<Vec<IntermediateType>, _>>()
                                            .unwrap();

                                        // The identifier is the same accross instantiations because events can be overloaded in the ABI
                                        let event_identifier =
                                            module.identifier_at(struct_handle.name).to_string();

                                        top_level_events.insert(EventStruct {
                                            module_id: event_module_id,
                                            identifier: event_identifier,
                                            type_parameters: Some(event_type_parameters),
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
/// and then searching for a matching instantiation. Matches both the struct definition and type parameters.
fn find_struct_def_instantiation_index(
    module: &CompiledModule,
    datatype_handle_index: DatatypeHandleIndex,
    type_parameters: &[SignatureToken],
) -> Option<StructDefInstantiationIndex> {
    // Verify the struct definition exists
    module.find_struct_def(datatype_handle_index)?;

    // Get the index of this struct definition
    let struct_def_index = module
        .struct_defs()
        .iter()
        .position(|d| d.struct_handle == datatype_handle_index)
        .map(|idx| StructDefinitionIndex::new(idx as u16))?;

    // Search through struct instantiations to find one that matches both the struct definition and type parameters
    for (idx, instantiation) in module.struct_instantiations().iter().enumerate() {
        if instantiation.def == struct_def_index {
            // Get the type parameters for this instantiation
            let instantiation_type_params = &module.signature_at(instantiation.type_parameters).0;

            // Check if the type parameters match
            if instantiation_type_params.len() == type_parameters.len() {
                let mut matches = true;
                for (inst_param, expected_param) in
                    instantiation_type_params.iter().zip(type_parameters.iter())
                {
                    if !signature_tokens_match(inst_param, expected_param) {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    return Some(StructDefInstantiationIndex::new(idx as u16));
                }
            }
        }
    }

    None
}

/// Checks if two signature tokens match (for type parameter comparison)
fn signature_tokens_match(token1: &SignatureToken, token2: &SignatureToken) -> bool {
    match (token1, token2) {
        (SignatureToken::Vector(inner1), SignatureToken::Vector(inner2)) => {
            signature_tokens_match(inner1, inner2)
        }
        (SignatureToken::Datatype(idx1), SignatureToken::Datatype(idx2)) => idx1 == idx2,
        (
            SignatureToken::DatatypeInstantiation(inst1),
            SignatureToken::DatatypeInstantiation(inst2),
        ) => {
            let (idx1, params1) = inst1.as_ref();
            let (idx2, params2) = inst2.as_ref();
            if idx1 != idx2 || params1.len() != params2.len() {
                return false;
            }
            params1
                .iter()
                .zip(params2.iter())
                .all(|(p1, p2)| signature_tokens_match(p1, p2))
        }
        (SignatureToken::TypeParameter(idx1), SignatureToken::TypeParameter(idx2)) => idx1 == idx2,
        (SignatureToken::Reference(inner1), SignatureToken::Reference(inner2)) => {
            signature_tokens_match(inner1, inner2)
        }
        (SignatureToken::MutableReference(inner1), SignatureToken::MutableReference(inner2)) => {
            signature_tokens_match(inner1, inner2)
        }
        (t1, t2) => t1 == t2,
    }
}
