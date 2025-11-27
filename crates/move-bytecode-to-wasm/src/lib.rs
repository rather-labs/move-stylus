use abi_types::public_function::PublicFunction;
pub(crate) use compilation_context::{CompilationContext, UserDefinedType};
use compilation_context::{ModuleData, ModuleId};
use constructor::inject_constructor;
use error::{
    CodeError, CompilationError, DependencyError, DependencyProcessingError, ICEError, ICEErrorKind,
};
use move_binary_format::file_format::FunctionDefinition;
use move_package::{
    compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource},
    source_package::parsed_manifest::PackageName,
};
use move_parse_special_attributes::process_special_attributes;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use translation::{
    TranslationError,
    intermediate_types::IntermediateType,
    table::{FunctionId, FunctionTable},
    translate_and_link_functions,
};

use walrus::{GlobalId, Module, RefType};
use wasm_validation::validate_stylus_wasm;

pub(crate) mod abi_types;
pub mod compilation_context;
mod constructor;
mod data;
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

pub use translation::functions::MappedFunction;

#[cfg(feature = "inject-host-debug-fns")]
use walrus::ValType;

#[cfg(test)]
mod test_tools;

pub type GlobalFunctionTable<'move_package> =
    HashMap<FunctionId, &'move_package FunctionDefinition>;

pub fn translate_single_module(
    package: CompiledPackage,
    module_name: &str,
) -> Result<Module, CompilationError> {
    let mut modules = translate_package(package, Some(module_name.to_string()))?;

    Ok(modules
        .remove(module_name)
        .ok_or_else(|| ICEError::new(ICEErrorKind::ModuleNotCompiled(module_name.to_string())))?)
}

pub fn translate_package(
    package: CompiledPackage,
    module_name: Option<String>,
) -> Result<HashMap<String, Module>, CompilationError> {
    let root_compiled_units: Vec<&CompiledUnitWithSource> = if let Some(module_name) = module_name {
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

    // Contains the module data for all the root package and its dependencies
    let mut modules_data: HashMap<ModuleId, ModuleData> = HashMap::new();

    // Contains all a reference for all functions definitions in case we need to process them and
    // statically link them
    let mut function_definitions: GlobalFunctionTable = HashMap::new();

    let mut errors = Vec::new();

    // TODO: a lot of clones, we must create a symbol pool
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

        let root_module_id = ModuleId {
            address: root_compiled_module_unit.address().into_bytes().into(),
            module_name: module_name.clone(),
        };

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

        #[cfg(feature = "inject-host-debug-fns")]
        inject_debug_fns(&mut module);

        // Function table
        let function_table_id = module.tables.add_local(false, 0, None, RefType::Funcref);
        let mut function_table = FunctionTable::new(function_table_id);

        // Process the dependency tree
        if let Err(dependencies_errors) = process_dependency_tree(
            &mut modules_data,
            &package.deps_compiled_units,
            &root_compiled_units,
            &root_compiled_module_unit.immediate_dependencies(),
            &mut function_definitions,
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

        let special_attributes = match process_special_attributes(&root_compiled_module.source_path)
        {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let root_module_data = ModuleData::build_module_data(
            root_module_id.clone(),
            root_compiled_module,
            &package.deps_compiled_units,
            &root_compiled_units,
            &mut function_definitions,
            special_attributes,
        )?;

        let compilation_ctx =
            CompilationContext::new(&root_module_data, &modules_data, memory_id, allocator_func);

        let mut public_functions = Vec::new();
        for function_information in root_module_data.functions.information.iter().filter(|fi| {
            fi.function_id.module_id == root_module_id && !fi.is_generic && !fi.is_native
        }) {
            translate_and_link_functions(
                &function_information.function_id,
                &mut function_table,
                &function_definitions,
                &mut module,
                &compilation_ctx,
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

        hostio::build_entrypoint_router(
            &mut module,
            &public_functions,
            &compilation_ctx,
            &dynamic_fields_global_variables,
        )?;

        function_table.ensure_all_functions_added()?;
        validate_stylus_wasm(&mut module)?;

        modules.insert(module_name, module);
        modules_data.insert(root_module_id.clone(), root_module_data);
    }

    if errors.is_empty() {
        Ok(modules)
    } else {
        Err(CompilationError::CodeError {
            mapped_files: package.file_map,
            errors,
        })
    }
}

#[derive(Debug)]
pub struct PackageModuleData {
    pub modules_paths: HashMap<PathBuf, ModuleId>,
    pub modules_data: HashMap<ModuleId, ModuleData>,
}

pub fn package_module_data(
    package: &CompiledPackage,
    module_name: Option<String>,
) -> Result<PackageModuleData, CompilationError> {
    let mut modules_data = HashMap::new();
    let mut modules_paths = HashMap::new();
    let mut errors = Vec::new();

    // This is not used in this function but is used in the others
    let mut function_definitions: GlobalFunctionTable = HashMap::new();

    let root_compiled_units: Vec<&CompiledUnitWithSource> = if let Some(module_name) = module_name {
        package
            .root_compiled_units
            .iter()
            .filter(move |unit| unit.unit.name.to_string() == module_name)
            .collect()
    } else {
        package.root_compiled_units.iter().collect()
    };

    for root_compiled_module in &root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        let root_compiled_module_unit = &root_compiled_module.unit.module;

        let root_module_id = ModuleId {
            address: root_compiled_module_unit.address().into_bytes().into(),
            module_name: module_name.clone(),
        };

        // Process the dependency tree
        if let Err(dependencies_errors) = process_dependency_tree(
            &mut modules_data,
            &package.deps_compiled_units,
            &root_compiled_units,
            &root_compiled_module_unit.immediate_dependencies(),
            &mut function_definitions,
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

        let special_attributes = match process_special_attributes(&root_compiled_module.source_path)
        {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let root_module_data = ModuleData::build_module_data(
            root_module_id.clone(),
            root_compiled_module,
            &package.deps_compiled_units,
            &root_compiled_units,
            &mut function_definitions,
            special_attributes,
        )?;

        modules_data.insert(root_module_id.clone(), root_module_data);
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

pub fn translate_package_cli(
    package: CompiledPackage,
    rerooted_path: &Path,
    install_dir: Option<PathBuf>,
    emit_wat: bool,
) -> Result<(), CompilationError> {
    let build_directory = if let Some(install_dir) = install_dir {
        install_dir.join(format!(
            "build/{}/wasm",
            package.compiled_package_info.package_name
        ))
    } else {
        rerooted_path.join(format!(
            "build/{}/wasm",
            package.compiled_package_info.package_name
        ))
    };

    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory)
        .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

    let mut modules = translate_package(package, None)?;

    for (module_name, module) in modules.iter_mut() {
        module
            .emit_wasm_file(build_directory.join(format!("{module_name}.wasm")))
            .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

        if emit_wat {
            // Convert to WAT format
            let wat = wasmprinter::print_bytes(module.emit_wasm())
                .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;
            std::fs::write(
                build_directory.join(format!("{module_name}.wat")),
                wat.as_bytes(),
            )
            .map_err(|e| ICEError::new(ICEErrorKind::Io(e)))?;
        }
    }

    Ok(())
}

/// This functions process the dependency tree for the root module.
///
/// It builds `ModuleData` for every module in the dependency tree and saves it in a HashMap.
pub fn process_dependency_tree<'move_package>(
    dependencies_data: &mut HashMap<ModuleId, ModuleData>,
    deps_compiled_units: &'move_package [(PackageName, CompiledUnitWithSource)],
    root_compiled_units: &'move_package [&CompiledUnitWithSource],
    dependencies: &[move_core_types::language_storage::ModuleId],
    function_definitions: &mut GlobalFunctionTable<'move_package>,
) -> Result<(), DependencyProcessingError> {
    let mut errors = Vec::new();
    for dependency in dependencies {
        let module_id = ModuleId {
            module_name: dependency.name().to_string(),
            address: dependency.address().into_bytes().into(),
        };
        // If the HashMap contains the key, we already processed that dependency
        if !dependencies_data.contains_key(&module_id) {
            // println!("  \x1B[1m\x1B[32mPROCESSING DEPENDENCY\x1B[0m {module_id}");
        } else {
            // println!("  \x1B[1m\x1B[32mPROCESSING DEPENDENCY\x1B[0m {module_id} [cached]");
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
                function_definitions,
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

        let special_attributes = match process_special_attributes(&dependency_module.source_path) {
            Ok(sa) => sa,
            Err((_mf, e)) => {
                errors.extend(e.into_iter().map(CodeError::SpecialAttributesError));
                continue;
            }
        };

        let dependency_module_data = ModuleData::build_module_data(
            module_id.clone(),
            dependency_module,
            deps_compiled_units,
            root_compiled_units,
            function_definitions,
            special_attributes,
        )
        .map_err(|e| {
            DependencyProcessingError::ICE(ICEError::new(ICEErrorKind::CompilationContext(e)))
        })?;

        let processed_dependency =
            dependencies_data.insert(module_id.clone(), dependency_module_data);

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

// TODO: Move to translation.rs

#[cfg(feature = "inject-host-debug-fns")]
fn inject_debug_fns(module: &mut walrus::Module) {
    if cfg!(feature = "inject-host-debug-fns") {
        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_i32", func_ty);

        let func_ty = module.types.add(&[ValType::I32, ValType::I32], &[]);
        module.add_import_func("", "print_memory_from", func_ty);

        let func_ty = module.types.add(&[ValType::I64], &[]);
        module.add_import_func("", "print_i64", func_ty);

        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_u128", func_ty);

        let func_ty = module.types.add(&[], &[]);
        module.add_import_func("", "print_separator", func_ty);

        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_address", func_ty);
    }
}

#[cfg(feature = "inject-host-debug-fns")]
#[macro_export]
macro_rules! declare_host_debug_functions {
    ($module: ident) => {
        (
            $module.imports.get_func("", "print_i32").unwrap(),
            $module.imports.get_func("", "print_i64").unwrap(),
            $module.imports.get_func("", "print_memory_from").unwrap(),
            $module.imports.get_func("", "print_address").unwrap(),
            $module.imports.get_func("", "print_separator").unwrap(),
            $module.imports.get_func("", "print_u128").unwrap(),
        )
    };
}
