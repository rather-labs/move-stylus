use std::{collections::HashMap, path::Path};

use abi_types::public_function::PublicFunction;
use move_binary_format::file_format::{
    Constant, DatatypeHandleIndex, FieldHandleIndex, Signature, StructDefinitionIndex, Visibility,
};
use move_binary_format::internals::ModuleIndex;
use move_package::compilation::compiled_package::{CompiledPackage, CompiledUnitWithSource};
use translation::intermediate_types::structs::IStruct;
use translation::{
    functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionTable,
    translate_function,
};
use walrus::FunctionId;
use walrus::MemoryId;
use walrus::ValType;
use walrus::{Module, RefType};
use wasm_validation::validate_stylus_wasm;

mod abi_types;
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

    /// Module's structs: contains all the user defined structs
    pub module_structs: &'a [IStruct],

    /// Maps a field index to its corresponding struct
    pub fields_to_struct_map: &'a HashMap<FieldHandleIndex, StructDefinitionIndex>,

    // This Hashmap maps the move's datatype handles to our internal representation of those
    // types. The datatype handles are used interally by move to look for user defined data
    // types
    pub datatype_handles_map: &'a HashMap<DatatypeHandleIndex, UserDefinedType>,

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

pub enum UserDefinedType {
    Struct(usize, String),
    Enum(usize),
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
            root_compiled_module.enum_defs.is_empty(),
            "Enums are not supported yet"
        );

        let mut datatype_handles_map = HashMap::new();

        for (index, datatype_handle) in root_compiled_module.datatype_handles().iter().enumerate() {
            let idx = DatatypeHandleIndex::new(index as u16);

            // Assert the index we constructed is ok
            assert_eq!(
                *datatype_handle,
                root_compiled_module.datatype_handles()[idx.into_index()]
            );

            let addition_result = if let Some(position) = root_compiled_module
                .struct_defs()
                .iter()
                .position(|s| s.struct_handle == idx)
            {
                let name = root_compiled_module.identifiers()[datatype_handle.name.0 as usize]
                    .as_str()
                    .to_owned();

                datatype_handles_map.insert(idx, UserDefinedType::Struct(position, name))
            } else if let Some(position) = root_compiled_module
                .enum_defs()
                .iter()
                .position(|e| e.enum_handle == idx)
            {
                datatype_handles_map.insert(idx, UserDefinedType::Enum(position))
            } else {
                panic!("datatype handle index {index} not found");
            };

            assert!(
                addition_result.is_none(),
                "user defined data with handle {:?} already defined",
                idx
            );
        }

        // Module's structs
        let mut module_structs: Vec<IStruct> = vec![];
        let mut fields_to_struct_map = HashMap::new();
        for (index, struct_def) in root_compiled_module.struct_defs().iter().enumerate() {
            let struct_index = StructDefinitionIndex::new(index as u16);
            let mut fields_map = HashMap::new();
            let mut all_fields = Vec::new();
            if let Some(fields) = struct_def.fields() {
                for (field_index, field) in fields.iter().enumerate() {
                    let intermediate_type = IntermediateType::try_from_signature_token(
                        &field.signature.0,
                        &datatype_handles_map,
                    )
                    .unwrap();

                    let field_index = root_compiled_module
                        .field_handles()
                        .iter()
                        .position(|f| f.field == field_index as u16 && f.owner == struct_index)
                        .map(|i| FieldHandleIndex::new(i as u16));

                    // If field_index is None means the field is never referenced in the code
                    if let Some(field_index) = field_index {
                        let res = fields_map.insert(field_index, intermediate_type.clone());
                        assert!(
                            res.is_none(),
                            "there was an error creating a field in struct {struct_index}, field with index {field_index} already exist"
                        );
                        let res = fields_to_struct_map.insert(field_index, struct_index);
                        assert!(
                            res.is_none(),
                            "there was an error mapping field {field_index} to struct {struct_index}, already mapped"
                        );
                        all_fields.push((Some(field_index), intermediate_type));
                    } else {
                        all_fields.push((None, intermediate_type));
                    }
                }
            }

            let UserDefinedType::Struct(_, ref name) =
                datatype_handles_map[&struct_def.struct_handle]
            else {
                panic!(
                    "user defined type with datatype handle index {} not found",
                    struct_def.struct_handle
                )
            };

            module_structs.push(IStruct::new(
                name.clone(),
                struct_index,
                all_fields,
                fields_map,
            ));
        }

        let (mut module, allocator_func, memory_id) = hostio::new_module_with_host();

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
                    .map(|s| IntermediateType::try_from_signature_token(s, &datatype_handles_map))
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let move_function_return =
                &root_compiled_module.signatures[function_handle.return_.0 as usize];

            functions_returns.push(
                move_function_return
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, &datatype_handles_map))
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
                &datatype_handles_map,
                &mut module,
            );

            function_table.add(&mut module, mapped_function, function_handle_index);
        }

        let compilation_ctx = CompilationContext {
            constants: &root_compiled_module.constant_pool,
            functions_arguments: &functions_arguments,
            functions_returns: &functions_returns,
            module_signatures: &root_compiled_module.signatures,
            module_structs: &module_structs,
            datatype_handles_map: &datatype_handles_map,
            fields_to_struct_map: &fields_to_struct_map,
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
