use std::{collections::HashMap, path::Path};

use abi_types::public_function::PublicFunction;
pub(crate) use compilation_context::{CompilationContext, UserDefinedType};
use compilation_context::{ModuleData, ModuleId};
use move_binary_format::file_format::Visibility;
use move_package::{
    compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource},
    source_package::parsed_manifest::PackageName,
};
use translation::translate_function;
use walrus::{Module, ValType};
use wasm_validation::validate_stylus_wasm;

pub(crate) mod abi_types;
mod compilation_context;
mod hostio;
mod memory;
mod runtime;
mod runtime_error_codes;
mod translation;
mod utils;
mod wasm_builder_extensions;
mod wasm_helpers;
mod wasm_validation;

#[cfg(test)]
mod test_tools;

pub fn translate_single_module(package: CompiledPackage, module_name: &str) -> Module {
    let mut modules = translate_package(package, Some(module_name.to_string()));

    modules.remove(module_name).expect("Module not compiled")
}

pub fn translate_package(
    package: CompiledPackage,
    module_name: Option<String>,
) -> HashMap<String, Module> {
    let root_compiled_units: Vec<CompiledUnitWithSource> = if let Some(module_name) = module_name {
        package
            .root_compiled_units
            .into_iter()
            .filter(move |unit| unit.unit.name.to_string() == module_name)
            .collect()
    } else {
        package.root_compiled_units.into_iter().collect()
    };

    assert!(
        !root_compiled_units.is_empty(),
        "Module not found in package"
    );

    let mut modules = HashMap::new();

    // Contains the module data for all the root package and its dependencies
    let mut modules_data: HashMap<ModuleId, ModuleData> = HashMap::new();

    // TODO: a lot of cloenes, we must create a symbol pool
    for root_compiled_module in root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        let root_compiled_module = root_compiled_module.unit.module;

        println!("compiling module {module_name}...");

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();
        inject_debug_fns(&mut module);

        // Process the dependency tree
        process_dependency_tree(
            &mut modules_data,
            &package.deps_compiled_units,
            &root_compiled_module.immediate_dependencies(),
            &mut module,
        );

        let (root_module_data, mut function_table) =
            ModuleData::build_module_data(&root_compiled_module, &mut module);

        let root_module_id = ModuleId {
            address: root_compiled_module.address().into_bytes().into(),
            module_name: module_name.clone(),
        };
        modules_data.insert(root_module_id.clone(), root_module_data);

        let compilation_ctx = CompilationContext {
            root_module_data: &modules_data[&root_module_id],
            deps_data: &modules_data,
            memory_id,
            allocator: allocator_func,
        };

        let mut public_functions = Vec::new();
        let mut function_ids = Vec::new();

        for index in 0..function_table.len() {
            let function_id =
                translate_function(&mut module, index, &compilation_ctx, &mut function_table)
                    .unwrap();
            function_ids.push(function_id);
        }

        for (index, function_id) in function_ids.iter().enumerate() {
            let entry = function_table.get(index).unwrap();
            let mapped_function = &entry.function;

            if mapped_function.function_definition.visibility == Visibility::Public {
                public_functions.push(PublicFunction::new(
                    *function_id,
                    &mapped_function.name,
                    &mapped_function.signature,
                    &compilation_ctx,
                ));
            }
        }

        hostio::build_entrypoint_router(&mut module, &public_functions, &compilation_ctx);

        // Fill the WASM table with the function ids
        for (index, function_id) in function_ids.into_iter().enumerate() {
            function_table
                .add_to_wasm_table(&mut module, index, function_id)
                .expect("there was an error adding the module's functions to the function table");
        }

        function_table.ensure_all_functions_added().unwrap();
        validate_stylus_wasm(&mut module).unwrap();

        modules.insert(module_name, module);
    }

    modules
}

pub fn translate_package_cli(package: CompiledPackage, rerooted_path: &Path) {
    let build_directory = rerooted_path.join("build/wasm");
    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory).unwrap();

    let mut modules = translate_package(package, None);
    for (module_name, module) in modules.iter_mut() {
        module
            .emit_wasm_file(build_directory.join(format!("{}.wasm", module_name)))
            .unwrap();

        // Convert to WAT format
        let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
        std::fs::write(
            build_directory.join(format!("{}.wat", module_name)),
            wat.as_bytes(),
        )
        .expect("Failed to write WAT file");
    }
}

#[cfg(feature = "inject-host-debug-fns")]
#[macro_export]
macro_rules! declare_host_debug_functions {
    ($module: ident) => {
        (
            $module.imports.get_func("", "print_i32").unwrap(),
            $module.imports.get_func("", "print_memory_from").unwrap(),
            $module.imports.get_func("", "print_separator").unwrap(),
            $module.imports.get_func("", "print_u128").unwrap(),
        )
    };
}

/// This functions process the dependency tree for the root module.
///
/// It builds `ModuleData` for every module in the dependency tree and saves it in a HashMap.
pub fn process_dependency_tree(
    dependencies_data: &mut HashMap<ModuleId, ModuleData>,
    deps_compiled_units: &[(PackageName, CompiledUnitWithSource)],
    dependencies: &[move_core_types::language_storage::ModuleId],
    module: &mut Module,
) {
    for dependency in dependencies {
        let module_id = ModuleId {
            module_name: dependency.name().to_string(),
            address: dependency.address().into_bytes().into(),
        };
        print!("\tprocessing dependency {module_id}...",);
        // If the HashMap contains the key, we already processed that dependency
        if dependencies_data.contains_key(&module_id) {
            println!(" [cached]");
            continue;
        } else {
            println!();
        }

        // Find the dependency inside Move's compiled package
        let dependency_module = deps_compiled_units
            .iter()
            .find(|(_, module)| {
                module.unit.name().as_str() == dependency.name().as_str()
                    && module.unit.address.into_bytes() == **dependency.address()
            })
            .map(|(_, module)| module)
            .unwrap_or_else(|| panic!("could not find dependency {}", dependency.name()));

        let dependency_module = &dependency_module.unit.module;

        // If the the dependency has dependency, we process them first
        if !dependency_module.immediate_dependencies().is_empty() {
            process_dependency_tree(
                dependencies_data,
                deps_compiled_units,
                &dependency_module.immediate_dependencies(),
                module,
            );
        }

        let (dependency_module_data, _dependency_fn_table) =
            ModuleData::build_module_data(dependency_module, module);

        let processed_dependency = dependencies_data.insert(module_id, dependency_module_data);

        assert!(
            processed_dependency.is_none(),
            "processed the same dep twice in different contexts"
        );
    }
}

fn inject_debug_fns(module: &mut walrus::Module) {
    if cfg!(feature = "inject-host-debug-fns") {
        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_i32", func_ty);

        let func_ty = module.types.add(&[ValType::I32], &[]);
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
