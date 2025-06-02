use std::{collections::HashMap, path::Path};

use abi_types::public_function::PublicFunction;
use move_binary_format::file_format::{Constant, Signature, Visibility};
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use translation::{
    functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionTable,
    translate_function,
};
use walrus::FunctionId;
use walrus::MemoryId;
use walrus::{Module, RefType};
use wasm_validation::validate_stylus_wasm;

mod abi_types;
mod hostio;
mod memory;
mod runtime;
mod runtime_error_codes;
#[cfg(test)]
mod test_tools;
mod translation;
mod utils;
mod wasm_helpers;
mod wasm_validation;

/// Compilation context
///
/// Functions are processed in order. To access function information (i.e: arguments or return
/// arguments we must know the index of it)
pub struct CompilationContext<'a> {
    /// Move's connstant pool
    pub constants: &'a [Constant],

    /// Module's functions arguments.
    pub functions_arguments: &'a [Vec<IntermediateType>],

    /// Module's functions Returns.
    pub functions_returns: &'a [Vec<IntermediateType>],

    /// Module's signatures
    pub module_signatures: &'a [Signature],

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

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
    for root_compiled_module in root_compiled_units {
        let module_name = root_compiled_module.unit.name.to_string();
        let root_compiled_module = root_compiled_module.unit.module;

        assert!(
            root_compiled_module.struct_defs.is_empty(),
            "Structs are not supported yet"
        );

        assert!(
            root_compiled_module.enum_defs.is_empty(),
            "Enums are not supported yet"
        );

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

        // Return types of functions in intermediate types. Used to fill the stack type
        let mut functions_returns = Vec::new();
        let mut functions_arguments = Vec::new();

        // Function table
        let function_table_id = module.tables.add_local(false, 0, None, RefType::Funcref);
        let mut function_table = FunctionTable::new(function_table_id);

        for (function_def, function_handle) in root_compiled_module
            .function_defs
            .into_iter()
            .zip(root_compiled_module.function_handles.iter())
        {
            let move_function_arguments =
                &root_compiled_module.signatures[function_handle.parameters.0 as usize];

            functions_arguments.push(
                move_function_arguments
                    .0
                    .iter()
                    .map(IntermediateType::try_from)
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let move_function_return =
                &root_compiled_module.signatures[function_handle.return_.0 as usize];

            functions_returns.push(
                move_function_return
                    .0
                    .iter()
                    .map(IntermediateType::try_from)
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let code_locals = &root_compiled_module.signatures
                [function_def.code.as_ref().unwrap().locals.0 as usize];

            let function_name =
                root_compiled_module.identifiers[function_handle.name.0 as usize].to_string();

            let function_handle_index = function_def.function;
            let mapped_function = MappedFunction::new(
                function_name,
                move_function_arguments,
                move_function_return,
                code_locals,
                function_def,
                &mut module,
            );

            function_table.add(&mut module, mapped_function, function_handle_index);
        }

        let compilation_ctx = CompilationContext {
            constants: &root_compiled_module.constant_pool,
            functions_arguments: &functions_arguments,
            functions_returns: &functions_returns,
            module_signatures: &root_compiled_module.signatures,
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
                ));
            }
        }

        hostio::build_entrypoint_router(&mut module, allocator_func, memory_id, &public_functions);

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
