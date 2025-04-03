use std::path::Path;

use abi_types::public_function::PublicFunction;
use move_package::compilation::compiled_package::CompiledPackage;
use translation::map_signature;
use wasm_validation::validate_stylus_wasm;

mod abi_types;
mod hostio;
mod memory;
mod translation;
mod utils;
mod wasm_validation;

pub fn translate_package(package: &CompiledPackage, rerooted_path: &Path) {
    println!("package: {:#?}", package);

    let build_directory = rerooted_path.join("build/wasm");
    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory).unwrap();

    let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

    assert!(
        package.root_compiled_units.len() == 1,
        "Compilation for multiple packages is not supported yet"
    );

    let root_compiled_module = &package.root_compiled_units[0].unit.module;

    assert!(
        root_compiled_module.struct_defs.is_empty(),
        "Structs are not supported yet"
    );

    assert!(
        root_compiled_module.enum_defs.is_empty(),
        "Enums are not supported yet"
    );

    let mut public_functions = Vec::new();
    for (function_def, function_handle) in root_compiled_module
        .function_defs
        .iter()
        .zip(root_compiled_module.function_handles.iter())
    {
        let move_function_arguments =
            &root_compiled_module.signatures[function_handle.parameters.0 as usize];
        let move_function_return =
            &root_compiled_module.signatures[function_handle.return_.0 as usize];

        let function_arguments = map_signature(move_function_arguments);
        let function_return = map_signature(move_function_return);

        let function_id = translation::translate_function(
            function_def,
            &function_arguments,
            &function_return,
            &root_compiled_module.constant_pool,
            &mut module,
        )
        .unwrap();

        let function_name =
            root_compiled_module.identifiers[function_handle.name.0 as usize].to_string();

        public_functions.push(PublicFunction::new(
            function_id,
            &function_name,
            move_function_arguments,
            move_function_return,
        ));
    }

    hostio::build_entrypoint_router(&mut module, allocator_func, memory_id, &public_functions);

    module
        .emit_wasm_file(build_directory.join("output.wasm"))
        .unwrap();

    validate_stylus_wasm(&mut module).unwrap();

    // Convert to WAT format
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    std::fs::write(build_directory.join("output.wat"), wat.as_bytes())
        .expect("Failed to write WAT file");
}
