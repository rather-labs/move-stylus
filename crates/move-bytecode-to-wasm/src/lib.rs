use abi_types::public_function::PublicFunction;
pub(crate) use compilation_context::{CompilationContext, UserDefinedType};
use compilation_context::{ModuleData, ModuleId};
use constructor::inject_constructor;
use move_binary_format::file_format::FunctionDefinition;
use move_package::{
    compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource},
    source_package::parsed_manifest::PackageName,
};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};
use translation::{
    intermediate_types::IntermediateType,
    table::{FunctionId, FunctionTable},
    translate_function,
};

use walrus::{Module, RefType};
use wasm_validation::validate_stylus_wasm;

pub(crate) mod abi_types;
mod compilation_context;
mod constructor;
mod data;
mod generics;
mod hostio;
mod memory;
mod native_functions;
mod runtime;
mod runtime_error_codes;
mod storage;
mod translation;
mod utils;
mod vm_handled_types;
mod wasm_builder_extensions;
mod wasm_helpers;
mod wasm_validation;

#[cfg(feature = "inject-host-debug-fns")]
use walrus::ValType;

#[cfg(test)]
mod test_tools;

pub type GlobalFunctionTable<'move_package> =
    HashMap<FunctionId, &'move_package FunctionDefinition>;

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

    // Contains all a reference for all functions definitions in case we need to process them and
    // statically link them
    let mut function_definitions: GlobalFunctionTable = HashMap::new();

    // TODO: a lot of clones, we must create a symbol pool
    for root_compiled_module in &root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        println!("compiling module {module_name}...");
        let root_compiled_module = &root_compiled_module.unit.module;

        let root_module_id = ModuleId {
            address: root_compiled_module.address().into_bytes().into(),
            module_name: module_name.clone(),
        };

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

        #[cfg(feature = "inject-host-debug-fns")]
        inject_debug_fns(&mut module);

        // Function table
        let function_table_id = module.tables.add_local(false, 0, None, RefType::Funcref);
        let mut function_table = FunctionTable::new(function_table_id);

        // Process the dependency tree
        process_dependency_tree(
            &mut modules_data,
            &package.deps_compiled_units,
            &root_compiled_units,
            &root_compiled_module.immediate_dependencies(),
            &mut function_definitions,
        );

        let root_module_data = ModuleData::build_module_data(
            root_module_id.clone(),
            root_compiled_module,
            &package.deps_compiled_units,
            &root_compiled_units,
            &mut function_definitions,
        );

        let compilation_ctx =
            CompilationContext::new(&root_module_data, &modules_data, memory_id, allocator_func);

        let mut public_functions = Vec::new();
        for function_information in root_module_data
            .functions
            .information
            .iter()
            .filter(|fi| fi.function_id.module_id == root_module_id && !fi.is_generic)
        {
            translate_and_link_functions(
                &function_information.function_id,
                &mut function_table,
                &function_definitions,
                &mut module,
                &compilation_ctx,
            );

            let wasm_function_id = function_table
                .get_by_function_id(&function_information.function_id)
                .unwrap()
                .wasm_function_id
                .unwrap();

            if function_information.is_entry {
                public_functions.push(PublicFunction::new(
                    wasm_function_id,
                    &function_information.function_id.identifier,
                    &function_information.signature,
                    &compilation_ctx,
                ));
            }
        }

        // Inject constructor function.
        inject_constructor(
            &mut function_table,
            &mut module,
            &compilation_ctx,
            &mut public_functions,
        );

        hostio::build_entrypoint_router(&mut module, &public_functions, &compilation_ctx);

        function_table.ensure_all_functions_added().unwrap();
        validate_stylus_wasm(&mut module).unwrap();

        modules.insert(module_name, module);
        modules_data.insert(root_module_id.clone(), root_module_data);
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

/// This functions process the dependency tree for the root module.
///
/// It builds `ModuleData` for every module in the dependency tree and saves it in a HashMap.
pub fn process_dependency_tree<'move_package>(
    dependencies_data: &mut HashMap<ModuleId, ModuleData>,
    deps_compiled_units: &'move_package [(PackageName, CompiledUnitWithSource)],
    root_compiled_units: &'move_package [CompiledUnitWithSource],
    dependencies: &[move_core_types::language_storage::ModuleId],
    function_definitions: &mut GlobalFunctionTable<'move_package>,
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

        let immediate_dependencies = &dependency_module.immediate_dependencies();
        // If the the dependency has dependency, we process them first
        if !immediate_dependencies.is_empty() {
            process_dependency_tree(
                dependencies_data,
                deps_compiled_units,
                root_compiled_units,
                immediate_dependencies,
                function_definitions,
            );
        }

        let dependency_module_data = ModuleData::build_module_data(
            module_id.clone(),
            dependency_module,
            deps_compiled_units,
            root_compiled_units,
            function_definitions,
        );

        let processed_dependency = dependencies_data.insert(module_id, dependency_module_data);

        assert!(
            processed_dependency.is_none(),
            "processed the same dep twice in different contexts"
        );
    }
}

/// Trnaslates a function to WASM and links it to the WASM module
///
/// It also recursively translates and links all the functions called by this function
fn translate_and_link_functions(
    function_id: &FunctionId,
    function_table: &mut FunctionTable,
    function_definitions: &GlobalFunctionTable,
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
) {
    // Obtain the function information and module's data
    let (function_information, module_data) = if let Some(fi) = compilation_ctx
        .root_module_data
        .functions
        .information
        .iter()
        .find(|f| {
            f.function_id.module_id == function_id.module_id
                && f.function_id.identifier == function_id.identifier
        }) {
        (fi, compilation_ctx.root_module_data)
    } else {
        let module_data = compilation_ctx
            .deps_data
            .get(&function_id.module_id)
            .unwrap();

        let fi = module_data
            .functions
            .information
            .iter()
            .find(|f| f.function_id.identifier == function_id.identifier)
            .unwrap();

        (fi, module_data)
    };

    // If the function is generic, we instantiate the concrete types so we can translate it
    let function_information = if function_information.is_generic {
        &function_information.instantiate(function_id.type_instantiations.as_ref().unwrap())
    } else {
        function_information
    };

    // Process function defined in this module
    // First we check if there is already an entry for this function
    if let Some(table_entry) = function_table.get_by_function_id(&function_information.function_id)
    {
        // If it has asigned a wasm function id means that we already translated it, so we skip
        // it
        if table_entry.wasm_function_id.is_some() {
            return;
        }
    }
    // If it is not present, we add an entry for it
    else {
        function_table.add(module, function_id.clone(), function_information);
    }

    let function_definition = function_definitions
        // TODO do this in nother way
        .get(&function_id.get_generic_fn_id())
        .unwrap_or_else(|| panic!("could not find function definition for {}", function_id));

    // If the function contains code we translate it
    // If it does not it means is a native function, we do nothing, it is linked and called
    // directly in the translation function
    if let Some(move_bytecode) = function_definition.code.as_ref() {
        let (wasm_function_id, functions_to_link) = translate_function(
            module,
            compilation_ctx,
            module_data,
            function_table,
            function_information,
            move_bytecode,
        )
        .unwrap_or_else(|_| panic!("there was an error translating {}", function_id));

        function_table
            .add_to_wasm_table(module, function_id, wasm_function_id)
            .expect("there was an error adding the module's functions to the function table");

        // Recursively translate and link functions called by this function
        functions_to_link.iter().for_each(|function_id| {
            translate_and_link_functions(
                function_id,
                function_table,
                function_definitions,
                module,
                compilation_ctx,
            )
        });
    }
}

// TODO: Move somewhere else
pub fn get_generic_function_name(base_name: &str, generics: &[&IntermediateType]) -> String {
    if generics.is_empty() {
        panic!("generic_function_name called with no generics");
    }

    let mut hasher = DefaultHasher::new();
    generics.iter().for_each(|t| t.hash(&mut hasher));
    let hash = format!("{:x}", hasher.finish());
    format!("{base_name}_{hash}")
}

#[cfg(feature = "inject-host-debug-fns")]
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

#[cfg(feature = "inject-host-debug-fns")]
#[macro_export]
macro_rules! declare_host_debug_functions {
    ($module: ident) => {
        (
            $module.imports.get_func("", "print_i32").unwrap(),
            $module.imports.get_func("", "print_i64").unwrap(),
            $module.imports.get_func("", "print_memory_from").unwrap(),
            $module.imports.get_func("", "print_separator").unwrap(),
            $module.imports.get_func("", "print_u128").unwrap(),
        )
    };
}
