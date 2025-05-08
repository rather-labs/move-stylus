use std::{collections::HashMap, path::Path};

use abi_types::public_function::PublicFunction;
use move_binary_format::file_format::Visibility;
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use translation::functions::MappedFunction;
use walrus::Module;
use wasm_validation::validate_stylus_wasm;

mod abi_types;
mod hostio;
mod memory;
mod runtime_error_codes;
mod translation;
mod utils;
mod wasm_helpers;
mod wasm_validation;

pub fn translate_single_module(package: &CompiledPackage, module_name: &str) -> Module {
    let mut modules = translate_package(package, Some(module_name.to_string()));
    modules.remove(module_name).expect("Module not compiled")
}

pub fn translate_package(
    package: &CompiledPackage,
    module_name: Option<String>,
) -> HashMap<String, Module> {
    let root_compiled_units: Vec<&CompiledUnitWithSource> = if let Some(module_name) = module_name {
        package
            .root_compiled_units
            .iter()
            .filter(move |unit| unit.unit.name.to_string() == module_name)
            .collect()
    } else {
        package.root_compiled_units.iter().collect()
    };

    assert!(
        !root_compiled_units.is_empty(),
        "Module not found in package"
    );

    let mut modules = HashMap::new();
    for root_compiled_module in root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        let root_compiled_module = &root_compiled_module.unit.module;

        assert!(
            root_compiled_module.struct_defs.is_empty(),
            "Structs are not supported yet"
        );

        assert!(
            root_compiled_module.enum_defs.is_empty(),
            "Enums are not supported yet"
        );

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

        // All functions are defined empty to get their corresponding Ids
        let mut mapped_functions = Vec::new();
        for (function_def, function_handle) in root_compiled_module
            .function_defs
            .iter()
            .zip(root_compiled_module.function_handles.iter())
        {
            let move_function_arguments =
                &root_compiled_module.signatures[function_handle.parameters.0 as usize];
            let move_function_return =
                &root_compiled_module.signatures[function_handle.return_.0 as usize];

            let function_name =
                root_compiled_module.identifiers[function_handle.name.0 as usize].to_string();

            mapped_functions.push(MappedFunction::new(
                function_name,
                move_function_arguments,
                move_function_return,
                function_def,
                &mut module,
                &root_compiled_module.signatures,
            ));
        }

        let mut public_functions = Vec::new();
        let function_ids = mapped_functions.iter().map(|f| f.id).collect::<Vec<_>>();
        for mapped_function in mapped_functions {
            mapped_function
                .translate_function(
                    &mut module,
                    &root_compiled_module.constant_pool,
                    &function_ids,
                    memory_id,
                    allocator_func,
                )
                .unwrap();

            if mapped_function.move_definition.visibility == Visibility::Public {
                public_functions.push(PublicFunction::new(
                    mapped_function.id,
                    &mapped_function.name,
                    mapped_function.signature,
                ));
            }
        }

        hostio::build_entrypoint_router(&mut module, allocator_func, memory_id, &public_functions);

        validate_stylus_wasm(&mut module).unwrap();

        modules.insert(module_name, module);
    }

    modules
}

pub fn translate_package_cli(package: &CompiledPackage, rerooted_path: &Path) {
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
