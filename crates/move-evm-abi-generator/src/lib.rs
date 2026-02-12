// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

#![allow(dead_code)]
//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod abi;
mod common;
pub mod error;
mod human_readable;
mod json_format;
mod special_types;
mod types;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::error::{AbiGeneratorError, AbiGeneratorErrorKind};
use move_binary_format::file_format::{
    Bytecode, CompiledModule, DatatypeHandleIndex, FunctionHandleIndex, SignatureToken,
    StructDefInstantiationIndex, StructDefinitionIndex,
};
use move_bytecode_to_wasm::{
    PackageModuleData, compilation_context as ctx,
    compilation_context::{ModuleData, module_data::struct_data::IntermediateType},
};
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use move_symbol_pool::Symbol;

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
) -> Result<Vec<Abi>, AbiGeneratorError> {
    let mut result = Vec::new();
    for root_compiled_module in root_compiled_units {
        let file = &root_compiled_module.source_path;
        let module_id = package_module_data
            .modules_paths
            .get(file)
            .ok_or(AbiGeneratorError {
                kind: AbiGeneratorErrorKind::ModuleIdNotFound,
            })?;

        let module_data =
            package_module_data
                .modules_data
                .get(module_id)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound,
                })?;

        // Collect all the calls to emit<> and revert<> function to know which events and errors
        // are emmited in this module so we can put them in the ABI
        let mut processed_modules = HashSet::new();
        let (module_emitted_events, module_errors) = process_events_and_errors(
            package,
            root_compiled_module,
            root_compiled_units,
            &mut processed_modules,
            &package_module_data.modules_data,
        )?;

        let abi = abi::Abi::new(
            module_data,
            &package_module_data.modules_data,
            &module_emitted_events,
            &module_errors,
        )?;

        if abi.is_empty() {
            continue;
        }

        let json_abi = if generate_json {
            Some(json_format::process_abi(&abi)?)
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

const STYLUS_FRAMEWORK_EVENT_MODULE: &str = "event";
const STYLUS_FRAMEWORK_ERROR_MODULE: &str = "error";
const STYLUS_FRAMEWORK_NATIVE_EMIT_FUNCTION: &str = "emit";
const STYLUS_FRAMEWORK_NATIVE_REVERT_FUNCTION: &str = "revert";

#[derive(Debug)]
pub(crate) struct FunctionCall {
    module_id: ModuleId,
    identifier: Symbol,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct EventStruct {
    module_id: ModuleId,
    identifier: Symbol,
    type_parameters: Option<Vec<IntermediateType>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct ErrorStruct {
    module_id: ModuleId,
    identifier: Symbol,
}

/// Processes the events and errors emitted by the module and its children
fn process_events_and_errors(
    package: &CompiledPackage,
    root_compiled_module: &CompiledUnitWithSource,
    root_compiled_units: &[&CompiledUnitWithSource],
    processed_modules: &mut HashSet<ModuleId>,
    modules_data: &HashMap<ctx::ModuleId, ModuleData>,
) -> Result<(HashSet<EventStruct>, HashSet<ErrorStruct>), AbiGeneratorError> {
    let module = &root_compiled_module.unit.module;
    processed_modules.insert(module.self_id());

    let mut function_calls = Vec::new();
    let mut events = HashSet::new();
    let mut errors = HashSet::new();

    // Helper to resolve ModuleId and Symbol from a FunctionHandleIndex
    let resolve_call = |handle_idx: FunctionHandleIndex| {
        let handle = module.function_handle_at(handle_idx);
        let m_id = module.module_id_for_handle(module.module_handle_at(handle.module));
        let name = Symbol::from(module.identifier_at(handle.name).as_str());
        (m_id, name)
    };

    for function in module
        .function_defs()
        .iter()
        .filter_map(|f| f.code.as_ref())
    {
        for instruction in &function.code {
            match instruction {
                Bytecode::CallGeneric(idx) => {
                    let inst = module.function_instantiation_at(*idx);
                    let (m_id, name) = resolve_call(inst.handle);

                    // Handle Stylus Framework specific calls (emit/revert)
                    if m_id.address() == &STYLUS_FRAMEWORK_ADDRESS {
                        let type_params = &module.signature_at(inst.type_parameters).0;

                        match m_id.name().as_str() {
                            STYLUS_FRAMEWORK_EVENT_MODULE
                                if name.as_str() == STYLUS_FRAMEWORK_NATIVE_EMIT_FUNCTION =>
                            {
                                let first_ty = type_params.first().ok_or(AbiGeneratorError {
                                    kind: AbiGeneratorErrorKind::MissingTypeParameter,
                                })?;

                                events.insert(parse_event(module, first_ty, modules_data)?);
                            }
                            STYLUS_FRAMEWORK_ERROR_MODULE
                                if name.as_str() == STYLUS_FRAMEWORK_NATIVE_REVERT_FUNCTION =>
                            {
                                let first_ty = type_params.first().ok_or(AbiGeneratorError {
                                    kind: AbiGeneratorErrorKind::MissingTypeParameter,
                                })?;
                                errors.insert(parse_error(module, first_ty)?);
                            }
                            _ => {}
                        }
                    }
                    function_calls.push(FunctionCall {
                        module_id: m_id,
                        identifier: name,
                    });
                }
                Bytecode::Call(idx) => {
                    let (m_id, name) = resolve_call(*idx);
                    function_calls.push(FunctionCall {
                        module_id: m_id,
                        identifier: name,
                    });
                }
                _ => continue,
            }
        }
    }

    // --- Recursion Phase ---
    for call in &function_calls {
        let target_id = &call.module_id;
        if target_id == &module.self_id() || processed_modules.contains(target_id) {
            continue;
        }

        let child_unit = find_child_module(package, root_compiled_units, target_id)?;
        let (child_events, child_errors) = process_events_and_errors(
            package,
            child_unit,
            root_compiled_units,
            processed_modules,
            modules_data,
        )?;

        events.extend(child_events);
        errors.extend(child_errors);
    }

    Ok((events, errors))
}

/// Helper to find a module in dependencies or root units
fn find_child_module<'a>(
    package: &'a CompiledPackage,
    roots: &[&'a CompiledUnitWithSource],
    target: &ModuleId,
) -> Result<&'a CompiledUnitWithSource, AbiGeneratorError> {
    package
        .deps_compiled_units
        .iter()
        // Destructure the tuple reference directly in the find closure
        .find(|(_, compiled_unit)| compiled_unit.unit.module.self_id() == *target)
        .map(|(_, compiled_unit)| compiled_unit)
        .or_else(|| {
            roots
                .iter()
                .find(|c| c.unit.module.self_id() == *target)
                .copied()
        })
        .ok_or(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::DependencyNotFound(target.clone()),
        })
}

/// Parses an event from a signature token
fn parse_event(
    module: &CompiledModule,
    token: &SignatureToken,
    modules_data: &HashMap<ctx::ModuleId, ModuleData>,
) -> Result<EventStruct, AbiGeneratorError> {
    match token {
        SignatureToken::Datatype(handle_idx) => {
            let handle = module.datatype_handle_at(*handle_idx);
            Ok(EventStruct {
                module_id: module.module_id_for_handle(module.module_handle_at(handle.module)),
                identifier: Symbol::from(module.identifier_at(handle.name).as_str()),
                type_parameters: None,
            })
        }
        SignatureToken::DatatypeInstantiation(data) => {
            let (handle_idx, type_params) = data.as_ref();
            let handle = module.datatype_handle_at(*handle_idx);
            let event_module_id =
                module.module_id_for_handle(module.module_handle_at(handle.module));

            // Resolve module data to get handle mappings for type conversion
            let event_module = modules_data
                .get(&ctx::ModuleId::new(
                    event_module_id.address().into_bytes().into(),
                    event_module_id.name().as_str(),
                ))
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::DependencyNotFound(event_module_id.clone()),
                })?;

            let event_type_parameters = type_params
                .iter()
                .map(|t| {
                    IntermediateType::try_from_signature_token(
                        t,
                        &event_module.datatype_handles_map,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::InvalidEmitType(token.clone()),
                })?;

            Ok(EventStruct {
                module_id: event_module_id,
                identifier: Symbol::from(module.identifier_at(handle.name).as_str()),
                type_parameters: Some(event_type_parameters),
            })
        }
        _ => Err(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::InvalidEmitType(token.clone()),
        }),
    }
}

/// Parses an error from a signature token
fn parse_error(
    module: &CompiledModule,
    token: &SignatureToken,
) -> Result<ErrorStruct, AbiGeneratorError> {
    if let SignatureToken::Datatype(handle_idx) = token {
        let handle = module.datatype_handle_at(*handle_idx);
        Ok(ErrorStruct {
            module_id: module.module_id_for_handle(module.module_handle_at(handle.module)),
            identifier: Symbol::from(module.identifier_at(handle.name).as_str()),
        })
    } else {
        Err(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::InvalidRevertType(token.clone()),
        })
    }
}

/// Maps a `DatatypeHandleIndex` to a `StructDefInstantiationIndex` by finding the struct definition
/// and then searching for a matching instantiation. Matches both the struct definition and type parameters.
fn find_struct_def_instantiation_index(
    module: &CompiledModule,
    datatype_handle_index: DatatypeHandleIndex,
    type_parameters: &[SignatureToken],
) -> Option<StructDefInstantiationIndex> {
    // 1. Locate the StructDefinitionIndex
    let struct_def_index = module
        .struct_defs()
        .iter()
        .position(|d| d.struct_handle == datatype_handle_index)
        .map(|idx| StructDefinitionIndex::new(idx as u16))?;

    // 2. Find the instantiation matching both the index and the type parameters
    module
        .struct_instantiations()
        .iter()
        .enumerate()
        .find(|(_, inst)| {
            if inst.def != struct_def_index {
                return false;
            }

            let inst_params = &module.signature_at(inst.type_parameters).0;

            // Length check + element-wise comparison
            inst_params.len() == type_parameters.len()
                && inst_params
                    .iter()
                    .zip(type_parameters)
                    .all(|(a, b)| signature_tokens_match(a, b))
        })
        .map(|(idx, _)| StructDefInstantiationIndex::new(idx as u16))
}

/// Checks if two signature tokens match (for type parameter comparison)
fn signature_tokens_match(token1: &SignatureToken, token2: &SignatureToken) -> bool {
    use SignatureToken::*;
    match (token1, token2) {
        // Group single-inner recursive types
        (Vector(t1), Vector(t2))
        | (Reference(t1), Reference(t2))
        | (MutableReference(t1), MutableReference(t2)) => signature_tokens_match(t1, t2),

        // Direct index comparisons
        (Datatype(idx1), Datatype(idx2)) => idx1 == idx2,
        (TypeParameter(idx1), TypeParameter(idx2)) => idx1 == idx2,

        // Complex instantiations
        (DatatypeInstantiation(inst1), DatatypeInstantiation(inst2)) => {
            let (id1, params1) = inst1.as_ref();
            let (id2, params2) = inst2.as_ref();

            id1 == id2
                && params1.len() == params2.len()
                && params1
                    .iter()
                    .zip(params2)
                    .all(|(p1, p2)| signature_tokens_match(p1, p2))
        }

        // Catch-all for primitives (U8, Address, Bool, etc.) and mismatches
        (t1, t2) => t1 == t2,
    }
}
