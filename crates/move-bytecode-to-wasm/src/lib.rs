// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

pub(crate) mod abi_types;
pub mod compilation_context;
mod constructor;
pub mod data;
pub mod error;
mod generics;
mod hasher;
mod hostio;
mod memory;
mod native_functions;
mod runtime;
mod storage;
mod translation;
mod utils;
mod vm_handled_types;
mod wasm_builder_extensions;
mod wasm_validation;

#[cfg(any(test, feature = "inject-host-debug-fns"))]
mod test_tools;

use abi_types::public_function::PublicFunction;
pub(crate) use compilation_context::{CompilationContext, UserDefinedType};
use compilation_context::{ModuleData, ModuleId};
use constructor::inject_constructor;
use data::RuntimeErrorData;
use error::{
    CodeError, CompilationError, DependencyError, DependencyProcessingError, ICEError, ICEErrorKind,
};
use hostio::entrypoint_router::build_entrypoint_router;
use move_package::{
    compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource},
    source_package::parsed_manifest::PackageName,
};
use move_parse_special_attributes::process_special_attributes;
use move_symbol_pool::Symbol;
use std::{collections::HashMap, path::PathBuf};
use translation::{
    TranslationError, intermediate_types::IntermediateType, table::FunctionTable,
    translate_and_link_functions,
};

use walrus::{GlobalId, Module, RefType};
use wasm_validation::validate_stylus_wasm;

pub use translation::functions::MappedFunction;

pub fn translate_single_module<'move_compiled_package>(
    package: &'move_compiled_package CompiledPackage,
    module_name: &str,
    modules_data: &mut HashMap<ModuleId, ModuleData<'move_compiled_package>>,
) -> Result<Module, CompilationError> {
    let mut modules = translate_package(
        package,
        Some(module_name.to_string()),
        modules_data,
        false,
        true,
    )?;

    Ok(modules
        .remove(module_name)
        .ok_or_else(|| ICEError::new(ICEErrorKind::ModuleNotCompiled(module_name.to_string())))?)
}

pub fn translate_package<'move_package>(
    package: &'move_package CompiledPackage,
    module_name: Option<String>,
    modules_data: &mut HashMap<ModuleId, ModuleData<'move_package>>,
    verbose: bool,
    test_mode: bool,
) -> Result<HashMap<String, Module>, CompilationError> {
    // HashMap of package name to address
    // This includes all the dependencies of the root package
    let address_alias_instantiation: HashMap<Symbol, [u8; 32]> = package
        .compiled_package_info
        .address_alias_instantiation
        .iter()
        .map(|(key, value)| (*key, value.into_bytes()))
        .collect();

    let root_compiled_units: Vec<&'move_package CompiledUnitWithSource> =
        if let Some(module_name) = module_name {
            package
                .root_compiled_units
                .iter()
                .filter(move |unit| unit.unit.name.to_string() == module_name)
                .collect()
        } else {
            package.root_compiled_units.iter().collect()
        };

    if root_compiled_units.is_empty() {
        return Err(CompilationError::NoFilesFound);
    }

    let mut modules = HashMap::new();
    let mut errors = Vec::new();

    for root_compiled_module in &root_compiled_units {
        // This is used to keep track of dynamic fields were retrieved from storage as mutable.
        // This vector is used at the end of the entrypoint function to commit the possible changes
        // made to those variables. The variables will be declared in the source code even if the
        // code does not follow a path that executes a borrow_mut function for dynamic fields. The
        // value of those variables will be:
        // - a pointer to the dynamic field
        // - -1 if the code never executed borrow_mut for that variable
        let mut dynamic_fields_global_variables: Vec<(GlobalId, IntermediateType)> = Vec::new();

        let module_name = root_compiled_module.unit.name.to_string();
        println!("\x1B[1m\x1B[32mCOMPILING\x1B[0m {module_name}");
        let root_compiled_module_unit = &root_compiled_module.unit.module;

        // Extract package address from CompiledPackage
        let package_address = root_compiled_module_unit.address().into_bytes();

        let root_module_id = ModuleId::new(package_address.into(), module_name.as_str());

        let (mut module, allocator_func, memory_id, compilation_context_globals) =
            hostio::new_module_with_host();

        // Function table
        let function_table_id = module.tables.add_local(false, 0, None, RefType::Funcref);
        let mut function_table = FunctionTable::new(function_table_id);

        // Process the dependency tree
        if let Err(dependencies_errors) = process_dependency_tree(
            modules_data,
            &package.deps_compiled_units,
            &root_compiled_units,
            &root_compiled_module_unit.immediate_dependencies(),
            &address_alias_instantiation,
            verbose,
            test_mode,
        ) {
            match dependencies_errors {
                DependencyProcessingError::ICE(ice_error) => {
                    return Err(CompilationError::ICE(ice_error));
                }
                DependencyProcessingError::CodeError(code_errors) => {
                    errors.extend(code_errors);
                    continue;
                }
            }
        }

        // Build a HashMap of structs by module id from all dependencies.
        // This allows proper validation of entry function return values, ensuring they do not return imported structs with the key ability.
        let deps_structs = build_dependency_structs_map(modules_data);

        let special_attributes = match process_special_attributes(
            &root_compiled_module.source_path,
            package_address,
            &deps_structs,
            &address_alias_instantiation,
        ) {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let root_module_data = ModuleData::build_module_data(
            root_module_id,
            root_compiled_module,
            &package.deps_compiled_units,
            &root_compiled_units,
            special_attributes,
            test_mode,
        )?;

        let compilation_ctx = CompilationContext::new(
            &root_module_data,
            modules_data,
            memory_id,
            allocator_func,
            compilation_context_globals,
        );

        let mut runtime_error_data = RuntimeErrorData::new();

        let mut public_functions = Vec::new();
        for function_information in root_module_data.functions.information.iter().filter(|fi| {
            fi.function_id.module_id == root_module_id && !fi.is_generic && !fi.is_native
        }) {
            translate_and_link_functions(
                &function_information.function_id,
                &mut function_table,
                modules_data,
                &mut module,
                &compilation_ctx,
                &mut runtime_error_data,
                &mut dynamic_fields_global_variables,
            )?;

            if function_information.is_entry {
                let wasm_function_id = function_table
                    .get_by_function_id(&function_information.function_id)
                    .ok_or(TranslationError::EntryFunctionNotFound)?
                    .wasm_function_id
                    .ok_or(TranslationError::EntryFunctionWasmIdNotFound)?;

                public_functions.push(PublicFunction::new(
                    wasm_function_id,
                    &function_information.function_id.identifier,
                    &function_information.signature,
                    &compilation_ctx,
                )?);
            }
        }

        // Inject constructor function.
        inject_constructor(
            &mut function_table,
            &mut module,
            &compilation_ctx,
            &mut public_functions,
        )?;

        build_entrypoint_router(
            &mut module,
            &public_functions,
            &compilation_ctx,
            &mut runtime_error_data,
            &dynamic_fields_global_variables,
        )?;

        function_table.ensure_all_functions_added()?;
        validate_stylus_wasm(&mut module)?;

        modules.insert(module_name, module);
        modules_data.insert(root_module_id, root_module_data);
    }

    if errors.is_empty() {
        Ok(modules)
    } else {
        Err(CompilationError::CodeError {
            mapped_files: package.file_map.clone(),
            errors,
        })
    }
}

#[derive(Debug)]
pub struct PackageModuleData<'move_package> {
    pub modules_paths: HashMap<PathBuf, ModuleId>,
    pub modules_data: HashMap<ModuleId, ModuleData<'move_package>>,
}

pub fn package_module_data<'move_package>(
    package: &'move_package CompiledPackage,
    root_compiled_units: &'move_package [&CompiledUnitWithSource],
    verbose: bool,
    test_mode: bool,
) -> Result<PackageModuleData<'move_package>, CompilationError> {
    // HashMap of package name to address
    let address_alias_instantiation: HashMap<Symbol, [u8; 32]> = package
        .compiled_package_info
        .address_alias_instantiation
        .iter()
        .map(|(key, value)| (*key, value.into_bytes()))
        .collect();

    let mut modules_data = HashMap::new();
    let mut modules_paths = HashMap::new();
    let mut errors = Vec::new();

    for root_compiled_module in root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        let root_compiled_module_unit = &root_compiled_module.unit.module;

        let package_address = root_compiled_module_unit.address().into_bytes();
        let root_module_id = ModuleId::new(package_address.into(), module_name.as_str());

        // Process the dependency tree
        if let Err(dependencies_errors) = process_dependency_tree(
            &mut modules_data,
            &package.deps_compiled_units,
            root_compiled_units,
            &root_compiled_module_unit.immediate_dependencies(),
            &address_alias_instantiation,
            verbose,
            test_mode,
        ) {
            match dependencies_errors {
                DependencyProcessingError::ICE(ice_error) => {
                    return Err(CompilationError::ICE(ice_error));
                }
                DependencyProcessingError::CodeError(code_errors) => {
                    errors.extend(code_errors);
                    continue;
                }
            }
        }

        // Create a mapping from each dependency's module id to the list of structs obtained via the special attributes crate.
        // This allows proper validation of entry function return values, ensuring they do not return imported structs with the key ability.
        let deps_structs = build_dependency_structs_map(&modules_data);

        let special_attributes = match process_special_attributes(
            &root_compiled_module.source_path,
            package_address,
            &deps_structs,
            &address_alias_instantiation,
        ) {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let root_module_data = ModuleData::build_module_data(
            root_module_id,
            root_compiled_module,
            &package.deps_compiled_units,
            root_compiled_units,
            special_attributes,
            test_mode,
        )?;

        modules_data.insert(root_module_id, root_module_data);
        modules_paths.insert(root_compiled_module.source_path.clone(), root_module_id);
    }

    if errors.is_empty() {
        Ok(PackageModuleData {
            modules_data,
            modules_paths,
        })
    } else {
        Err(CompilationError::CodeError {
            mapped_files: package.file_map.clone(),
            errors,
        })
    }
}

/// This functions process the dependency tree for the root module.
///
/// It builds `ModuleData` for every module in the dependency tree and saves it in a HashMap.
pub fn process_dependency_tree<'move_package>(
    dependencies_data: &mut HashMap<ModuleId, ModuleData<'move_package>>,
    deps_compiled_units: &'move_package [(PackageName, CompiledUnitWithSource)],
    root_compiled_units: &[&'move_package CompiledUnitWithSource],
    dependencies: &[move_core_types::language_storage::ModuleId],
    address_alias_instantiation: &HashMap<Symbol, [u8; 32]>,
    verbose: bool,
    test_mode: bool,
) -> Result<(), DependencyProcessingError> {
    let mut errors = Vec::new();
    for dependency in dependencies {
        let dependency_address = dependency.address().into_bytes();
        let module_id = ModuleId::new(dependency_address.into(), dependency.name().as_str());

        // If the HashMap contains the key, we already processed that dependency
        if !dependencies_data.contains_key(&module_id) {
            if verbose {
                println!("  \x1B[1m\x1B[34mPROCESSING DEPENDENCY\x1B[0m {module_id}");
            }
        } else {
            if verbose {
                println!("  \x1B[1m\x1B[34mPROCESSING DEPENDENCY\x1B[0m {module_id} [cached]");
            }
            continue;
        }

        // Find the dependency inside Move's compiled package
        let dependency_module = deps_compiled_units
            .iter()
            .find(|(_, module)| {
                module.unit.name().as_str() == dependency.name().as_str()
                    && module.unit.address.into_bytes() == **dependency.address()
            })
            .map(|(_, module)| module)
            .ok_or_else(|| DependencyError::DependencyNotFound(dependency.name().to_string()))?;

        let immediate_dependencies = &dependency_module.unit.module.immediate_dependencies();
        // If the the dependency has dependency, we process them first
        let dependencies_process_result = if !immediate_dependencies.is_empty() {
            process_dependency_tree(
                dependencies_data,
                deps_compiled_units,
                root_compiled_units,
                immediate_dependencies,
                address_alias_instantiation,
                verbose,
                test_mode,
            )
        } else {
            Ok(())
        };

        if let Err(dependencies_errors) = dependencies_process_result {
            match dependencies_errors {
                DependencyProcessingError::ICE(ice_error) => {
                    return Err(DependencyProcessingError::ICE(ice_error));
                }
                DependencyProcessingError::CodeError(code_errors) => {
                    errors.extend(code_errors);
                    continue;
                }
            }
        }

        // Create a mapping from each dependency's module id to the list of structs obtained via the special attributes crate.
        // This allows proper validation of entry function return values, ensuring they do not return imported structs with the key ability.
        let deps_structs = build_dependency_structs_map(dependencies_data);

        let special_attributes = match process_special_attributes(
            &dependency_module.source_path,
            dependency_address,
            &deps_structs,
            address_alias_instantiation,
        ) {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let dependency_module_data = ModuleData::build_module_data(
            module_id,
            dependency_module,
            deps_compiled_units,
            root_compiled_units,
            special_attributes,
            test_mode,
        )
        .map_err(|e| {
            DependencyProcessingError::ICE(ICEError::new(ICEErrorKind::CompilationContext(e)))
        })?;

        let processed_dependency = dependencies_data.insert(module_id, dependency_module_data);

        if processed_dependency.is_some() {
            Err(DependencyError::DependencyProcessedMoreThanOnce(module_id))?;
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(DependencyProcessingError::CodeError(errors))
    }
}

/// Builds a HashMap mapping module IDs to their structs from all dependencies.
///
/// This function extracts struct information from `ModuleData` and converts it into
/// a format compatible with `move_parse_special_attributes::ModuleId`. This mapping
/// is essential for validating entry function return values, ensuring they do not
/// return imported structs with the `key` ability.
fn build_dependency_structs_map(
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> HashMap<move_parse_special_attributes::ModuleId, Vec<move_parse_special_attributes::Struct_>> {
    let mut deps_structs = HashMap::new();
    for md in modules_data.values() {
        deps_structs.insert(
            move_parse_special_attributes::ModuleId {
                address: <[u8; 32]>::try_from(md.id.address.as_slice()).unwrap(),
                module_name: md.id.module_name,
            },
            md.special_attributes.structs.clone(),
        );
    }
    deps_structs
}
