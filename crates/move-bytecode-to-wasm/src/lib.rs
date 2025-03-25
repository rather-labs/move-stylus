use std::path::Path;

use move_package::compilation::compiled_package::CompiledPackage;

mod hostio;
mod translation;

pub fn translate_package(package: &CompiledPackage, rerooted_path: &Path) {
    println!("package: {:#?}", package);

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

    assert!(
        root_compiled_module.function_defs.len() == 1,
        "Compilation for multiple functions is not supported yet"
    );

    let function_def = &root_compiled_module.function_defs[0];
    let function_handle = &root_compiled_module.function_handles[0];

    let build_directory = rerooted_path.join("build/wasm");
    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory).unwrap();

    let mut module = hostio::new_module_with_host();

    let function_id = translation::translate_function(
        function_def,
        function_handle,
        &root_compiled_module.constant_pool,
        &mut module,
        &root_compiled_module.signatures,
    )
    .unwrap();

    hostio::add_entrypoint(&mut module, function_id);

    module
        .emit_wasm_file(build_directory.join("output.wasm"))
        .unwrap();

    validate_wasm(&module.emit_wasm());

    // Convert to WAT format
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    std::fs::write(build_directory.join("output.wat"), wat.as_bytes())
        .expect("Failed to write WAT file");
}

/// Validate the Wasm module using the wasmparser crate
/// TODO: Validate Stylus specific constraints
fn validate_wasm(wasm: &[u8]) {
    let mut validator = wasmparser::Validator::new();

    validator
        .validate_all(wasm)
        .expect("Failed to validate Wasm");
}
