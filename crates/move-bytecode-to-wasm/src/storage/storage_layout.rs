use crate::{
    CompilationContext,
    translation::TranslationError,
    translation::intermediate_types::{IntermediateType, enums::IEnum, structs::IStruct},
};
use walrus::ir::BinaryOp;
use walrus::{InstrSeqBuilder, LocalId, Module, ValType};

const SLOT_SIZE: u32 = 32;

/// Emits WASM instructions to compute the total storage size for an enum (maximum of all variants).
///
/// Arguments:
/// - `module`: Walrus module being built
/// - `builder`: Instruction sequence builder to append to
/// - `enum_`: the enum whose size to compute
/// - `written_bytes_in_slot`: LocalId containing bytes already used in the storage slot (affects layout)
/// - `compilation_ctx`: context for type resolution
///
/// Returns `Ok(Some(LocalId))` with a local containing the computed size, or `Ok(None)` if generic type encountered.
/// The returned local will contain the computed size in bytes.
pub fn compute_enum_storage_size(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    enum_: &IEnum,
    written_bytes_in_slot: LocalId,
    compilation_ctx: &CompilationContext,
) -> Result<LocalId, TranslationError> {
    // Increment the used bytes by 1 to account for the variant index
    // Compute: written_bytes = (written_bytes + 1) % SLOT_SIZE
    let used_bytes_in_slot = module.locals.add(ValType::I32);
    builder
        .local_get(written_bytes_in_slot)
        .i32_const(1)
        .binop(BinaryOp::I32Add)
        .i32_const(SLOT_SIZE as i32)
        .binop(BinaryOp::I32RemS)
        .local_set(used_bytes_in_slot);

    // Compute the size of each variant and pick the largest one
    let enum_size = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(enum_size);

    for variant in &enum_.variants {
        let variant_size = compute_fields_storage_size(
            module,
            builder,
            &variant.fields,
            used_bytes_in_slot,
            compilation_ctx,
            || TranslationError::FoundReferenceInsideEnum {
                enum_index: variant.belongs_to,
            },
            || TranslationError::FoundTypeParameterInsideEnumVariant {
                enum_index: enum_.index,
                variant_index: variant.belongs_to,
            },
        )
        .unwrap();

        // If the variant size is computable, compare it to the current enum size
        // If the variant size is greater than the current enum size, set the enum size to the variant size
        builder
            .local_get(enum_size)
            .local_get(variant_size)
            .binop(BinaryOp::I32GeS)
            .if_else(
                None,
                |_| {
                    // enum_size >= variant_size, do nothing
                },
                |else_| {
                    // enum_size < variant_size, set enum_size to variant_size
                    else_.local_get(variant_size).local_set(enum_size);
                },
            );
    }

    // Add 1 to account for the variant index
    builder
        .local_get(enum_size)
        .i32_const(1)
        .binop(BinaryOp::I32Add)
        .local_set(enum_size);

    Ok(enum_size)
}

/// Emits WASM instructions to compute the total storage size for a struct.
///
/// Arguments:
/// - `module`: Walrus module being built
/// - `builder`: Instruction sequence builder to append to
/// - `struct_`: the struct whose storage size is being calculated
/// - `written_bytes_in_slot`: LocalId containing bytes already used in the current slot (affects layout)
/// - `compilation_ctx`: context used to resolve types and perform lookups
///
/// Filters out UID or NamedId fields (which are not stored) and computes the total size of the remaining fields.
/// Returns `Ok(Some(LocalId))` with a local containing the computed size, or `Ok(None)` if any field contains a generic type parameter.
pub fn compute_struct_storage_size(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    struct_: &IStruct,
    written_bytes_in_slot: LocalId,
    compilation_ctx: &CompilationContext,
) -> Result<LocalId, TranslationError> {
    // Filter out UIDs (not stored)
    let filtered_fields: Vec<IntermediateType> = struct_
        .fields
        .iter()
        .filter(|f| !f.is_uid_or_named_id(compilation_ctx))
        .cloned()
        .collect();

    // Compute: written_bytes = written_bytes % SLOT_SIZE
    let used_bytes_in_slot = module.locals.add(ValType::I32);
    builder
        .local_get(written_bytes_in_slot)
        .i32_const(SLOT_SIZE as i32)
        .binop(BinaryOp::I32RemS)
        .local_set(used_bytes_in_slot);

    compute_fields_storage_size(
        module,
        builder,
        &filtered_fields,
        used_bytes_in_slot,
        compilation_ctx,
        || TranslationError::FoundReferenceInsideStruct {
            struct_index: struct_.index(),
        },
        || TranslationError::FoundTypeParameterInsideStruct {
            struct_index: struct_.index(),
            type_parameter_index: 0,
        },
    )
}

/// Internal helper to emit WASM instructions to compute total storage size for a sequence of fields.
/// The `on_ref` closure defines the error to emit when a reference is found
/// (differs for enums vs structs).
fn compute_fields_storage_size<F, G>(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    fields: &[IntermediateType],
    used_bytes_in_slot: LocalId,
    compilation_ctx: &CompilationContext,
    on_ref: F,
    on_generic: G,
) -> Result<LocalId, TranslationError>
where
    F: Fn() -> TranslationError,
    G: Fn() -> TranslationError,
{
    let total_bytes = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(total_bytes);

    let used_bytes = module.locals.add(ValType::I32);
    builder.local_get(used_bytes_in_slot).local_set(used_bytes);

    for field in fields {
        match field {
            IntermediateType::ITypeParameter(_) => {
                return Err(on_generic());
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(on_ref());
            }
            IntermediateType::IGenericStructInstance { .. } | IntermediateType::IStruct { .. } => {
                let struct_ = compilation_ctx
                    .get_struct_by_intermediate_type(field)
                    .expect("struct not found");

                if struct_.has_key {
                    // Parent stores 32-byte UID when child has key
                    // total_bytes += SLOT_SIZE + (SLOT_SIZE - used_bytes) % SLOT_SIZE

                    // total_bytes += SLOT_SIZE
                    builder
                        .local_get(total_bytes)
                        .i32_const(SLOT_SIZE as i32)
                        .binop(BinaryOp::I32Add);

                    // Padding
                    builder
                        .i32_const(SLOT_SIZE as i32)
                        .local_get(used_bytes)
                        .binop(BinaryOp::I32Sub)
                        .i32_const(SLOT_SIZE as i32)
                        .binop(BinaryOp::I32RemS);

                    // total_bytes += padding
                    builder.binop(BinaryOp::I32Add).local_set(total_bytes);

                    // used_bytes = SLOT_SIZE
                    builder.i32_const(SLOT_SIZE as i32).local_set(used_bytes);
                } else {
                    let struct_size = compute_struct_storage_size(
                        module,
                        builder,
                        &struct_,
                        used_bytes,
                        compilation_ctx,
                    )?;

                    // total_bytes += size
                    builder
                        .local_get(total_bytes)
                        .local_get(struct_size)
                        .binop(BinaryOp::I32Add)
                        .local_set(total_bytes);

                    // used_bytes = (used_bytes + size) % SLOT_SIZE
                    builder
                        .local_get(used_bytes)
                        .local_get(struct_size)
                        .binop(BinaryOp::I32Add)
                        .i32_const(SLOT_SIZE as i32)
                        .binop(BinaryOp::I32RemS)
                        .local_set(used_bytes);
                }
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx
                    .get_enum_by_intermediate_type(field)
                    .expect("enum not found");

                let enum_size = compute_enum_storage_size(
                    module,
                    builder,
                    &enum_,
                    used_bytes,
                    compilation_ctx,
                )?;

                // total_bytes += size
                builder
                    .local_get(total_bytes)
                    .local_get(enum_size)
                    .binop(BinaryOp::I32Add)
                    .local_set(total_bytes);

                // used_bytes = (used_bytes + size) % SLOT_SIZE
                builder
                    .local_get(used_bytes)
                    .local_get(enum_size)
                    .binop(BinaryOp::I32Add)
                    .i32_const(SLOT_SIZE as i32)
                    .binop(BinaryOp::I32RemS)
                    .local_set(used_bytes);
            }
            _ => {
                let field_size = field_size(field, compilation_ctx) as i32;

                // free_bytes = SLOT_SIZE - used_bytes
                let free_bytes = module.locals.add(ValType::I32);
                builder
                    .i32_const(SLOT_SIZE as i32)
                    .local_get(used_bytes)
                    .binop(BinaryOp::I32Sub)
                    .local_set(free_bytes);

                // if field_size_bytes > free_bytes
                builder
                    .i32_const(field_size)
                    .local_get(free_bytes)
                    .binop(BinaryOp::I32GtS)
                    .if_else(
                        None,
                        |then| {
                            // total_bytes += field_size_bytes
                            then.local_get(total_bytes)
                                .i32_const(field_size)
                                .binop(BinaryOp::I32Add);

                            // free_bytes % SLOT_SIZE);
                            then.local_get(free_bytes)
                                .i32_const(SLOT_SIZE as i32)
                                .binop(BinaryOp::I32RemS);

                            // total_bytes += (free_bytes % SLOT_SIZE);
                            then.binop(BinaryOp::I32Add).local_set(total_bytes);

                            // used_bytes = field_size
                            then.i32_const(field_size).local_set(used_bytes);
                        },
                        |else_| {
                            // total_bytes += field_size_bytes
                            else_
                                .local_get(total_bytes)
                                .i32_const(field_size)
                                .binop(BinaryOp::I32Add)
                                .local_set(total_bytes);

                            // used_bytes += field_size_bytes
                            else_
                                .local_get(used_bytes)
                                .i32_const(field_size)
                                .binop(BinaryOp::I32Add)
                                .local_set(used_bytes);
                        },
                    );
            }
        }
    }

    Ok(total_bytes)
}

/// Returns the storage-encoded size in bytes for a given intermediate type.
///
/// Note:
/// - For structs without `key`, size is 0 because their inline size depends on fields;
///   callers compute layout using field-by-field accumulation.
/// - For structs with `key`, at least 32 bytes are used to store the UID reference.
pub fn field_size(field: &IntermediateType, compilation_ctx: &CompilationContext) -> u32 {
    match field {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => 1,
        IntermediateType::IU16 => 2,
        IntermediateType::IU32 => 4,
        IntermediateType::IU64 => 8,
        IntermediateType::IU128 => 16,
        IntermediateType::IU256 => 32,
        IntermediateType::IAddress | IntermediateType::ISigner => 20,
        // Dynamic data occupies the whole slot, but the data is saved somewhere else
        IntermediateType::IVector(_) => 32,
        field if field.is_uid_or_named_id(compilation_ctx) => 32,

        // Structs default to size 0 since their size depends on whether their fields are dynamic or static.
        // The store function will handle this. If a struct has the 'key' ability, it at least occupies 32 bytes for the UID.
        // The store function will manage the rest of the fields.
        IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }
        | IntermediateType::IStruct {
            module_id, index, ..
        } => {
            let s = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .expect("struct not found");

            if s.has_key { 32 } else { 0 }
        }
        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            panic!("found reference inside struct")
        }
        IntermediateType::ITypeParameter(_) => {
            panic!("cannot know the field size of a type parameter");
        }
    }
}

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use crate::compilation_context::{ModuleData, ModuleId};
    use crate::test_tools::build_module;
    use crate::translation::intermediate_types::VmHandledStruct;
    use crate::translation::intermediate_types::enums::IEnumVariant;
    use crate::translation::intermediate_types::structs::IStructType;
    use move_binary_format::file_format::StructDefinitionIndex;
    use rstest::rstest;
    use std::collections::HashMap;
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::{Engine, Linker, Module as WasmModule, Store};

    // Helper to create a compilation context with registered structs and enums for WASM tests
    fn create_wasm_test_ctx(
        structs: Vec<IStruct>,
        enums: Vec<IEnum>,
    ) -> CompilationContext<'static> {
        use std::ptr::addr_of_mut;

        let memory_id: walrus::MemoryId = unsafe { std::mem::zeroed() };
        let allocator: walrus::FunctionId = unsafe { std::mem::zeroed() };

        let module_id = ModuleId::default();
        let mut module_data = ModuleData::default();
        module_data.id = module_id;

        unsafe {
            let structs_data_ptr = addr_of_mut!(module_data.structs) as *mut u8;
            let structs_vec_ptr = structs_data_ptr as *mut Vec<IStruct>;
            std::ptr::write(structs_vec_ptr, structs);
        }
        module_data.enums.enums = enums;

        let root = Box::leak(Box::new(module_data));
        let deps = Box::leak(Box::new(HashMap::new()));
        CompilationContext::new(root, deps, memory_id, allocator)
    }

    // Helper to execute a WASM function that computes storage size for a list of fields
    // Creates a function that takes written_bytes_in_slot as parameter and returns the computed size
    fn execute_compute_fields_storage_size(
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        fields: &[IntermediateType],
        used_bytes_in_slot: u32,
        export_suffix: u32,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let mut function = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32], // used_bytes_in_slot parameter
            &[ValType::I32], // return computed size
        );

        let mut builder = function.func_body();
        let used_bytes_local = module.locals.add(ValType::I32);

        // Call compute_fields_storage_size
        let size_local = compute_fields_storage_size(
            module,
            &mut builder,
            fields,
            used_bytes_local,
            compilation_ctx,
            || TranslationError::FoundReferenceInsideEnum { enum_index: 0 },
            || TranslationError::FoundTypeParameterInsideEnumVariant {
                enum_index: 0,
                variant_index: 0,
            },
        )?;

        // Return the computed size
        builder.local_get(size_local);

        // The used_bytes_local is mapped to the function parameter
        let test_func = function.finish(vec![used_bytes_local], &mut module.funcs);
        let export_name = format!("test_fields_storage_size_{}", export_suffix);
        module.exports.add(&export_name, test_func);

        // Execute the WASM function
        let linker = Linker::new(&Engine::default());
        let engine = linker.engine();
        let wasm_module = WasmModule::from_binary(engine, &module.emit_wasm())
            .map_err(|e| format!("Failed to create WASM module: {e:?}"))?;
        let mut store = Store::new(engine, ());
        let instance = linker
            .instantiate(&mut store, &wasm_module)
            .map_err(|e| format!("Failed to instantiate WASM module: {e:?}"))?;

        let entrypoint = instance
            .get_typed_func::<i32, i32>(&mut store, &export_name)
            .map_err(|e| format!("Failed to get entrypoint: {e:?}"))?;

        let result = entrypoint
            .call(&mut store, used_bytes_in_slot as i32)
            .map_err(|e| format!("WASM execution error: {e:?}"))?;

        Ok(result as u32)
    }

    // Helper to execute a WASM function that computes storage size
    // Creates a function that takes written_bytes_in_slot as parameter and returns the computed size
    fn execute_compute_storage_size(
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        enum_: Option<&IEnum>,
        struct_: Option<&IStruct>,
        written_bytes_in_slot: u32,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let mut function = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32], // written_bytes_in_slot parameter
            &[ValType::I32], // return computed size
        );

        let mut builder = function.func_body();
        let written_bytes_local = module.locals.add(ValType::I32);

        // Call the appropriate compute function
        let size_local = if let Some(enum_) = enum_ {
            compute_enum_storage_size(
                module,
                &mut builder,
                enum_,
                written_bytes_local,
                compilation_ctx,
            )?
        } else if let Some(struct_) = struct_ {
            compute_struct_storage_size(
                module,
                &mut builder,
                struct_,
                written_bytes_local,
                compilation_ctx,
            )?
        } else {
            return Err("Either enum_ or struct_ must be provided".into());
        };

        // Return the computed size
        builder.local_get(size_local);

        // The written_bytes_local is mapped to the function parameter
        let test_func = function.finish(vec![written_bytes_local], &mut module.funcs);
        module.exports.add("test_storage_size", test_func);

        // Execute the WASM function
        let linker = Linker::new(&Engine::default());
        let engine = linker.engine();
        let wasm_module = WasmModule::from_binary(engine, &module.emit_wasm())
            .map_err(|e| format!("Failed to create WASM module: {e:?}"))?;
        let mut store = Store::new(engine, ());
        let instance = linker
            .instantiate(&mut store, &wasm_module)
            .map_err(|e| format!("Failed to instantiate WASM module: {e:?}"))?;

        let entrypoint = instance
            .get_typed_func::<i32, i32>(&mut store, "test_storage_size")
            .map_err(|e| format!("Failed to get entrypoint: {e:?}"))?;

        let result = entrypoint
            .call(&mut store, written_bytes_in_slot as i32)
            .map_err(|e| format!("WASM execution error: {e:?}"))?;

        Ok(result as u32)
    }

    #[rstest]
    #[case(0, 13)]
    #[case(4, 13)]
    #[case(10, 13)]
    #[case(25, 15)]
    #[case(28, 16)]
    #[case(32, 13)]
    fn enum_primitive_fields(#[case] written_bytes_in_slot: u32, #[case] expected_size: u32) {
        let (mut module, _, _) = build_module(None);
        let enum_ = IEnum::new(
            0,
            vec![
                IEnumVariant::new(0, 0, vec![IntermediateType::IU8, IntermediateType::IU16]),
                IEnumVariant::new(1, 0, vec![IntermediateType::IU32, IntermediateType::IU64]),
            ],
        )
        .unwrap();

        let ctx = create_wasm_test_ctx(vec![], vec![enum_.clone()]);
        let compilation_ctx = &ctx;

        let result = execute_compute_storage_size(
            &mut module,
            compilation_ctx,
            Some(&enum_),
            None,
            written_bytes_in_slot,
        )
        .unwrap();

        assert_eq!(
            result, expected_size,
            "WASM enum storage size mismatch at written_bytes_in_slot={}: got {}, expected {}",
            written_bytes_in_slot, result, expected_size
        );
    }

    #[rstest]
    #[case(0, vec![71, 91], 92)]
    #[case(10, vec![61, 81], 82)]
    #[case(20, vec![83, 71], 84)]
    #[case(30, vec![73, 61], 74)]
    #[case(31, vec![72, 60], 73)]
    #[case(32, vec![71, 91], 92)]
    fn enum_with_vectors(
        #[case] initial_used_bytes: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
    ) {
        let (mut module, _, _) = build_module(None);
        let enum_ = IEnum::new(
            0,
            vec![
                IEnumVariant::new(
                    0,
                    0,
                    vec![
                        IntermediateType::IAddress,
                        IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                        IntermediateType::IU64,
                    ],
                ),
                IEnumVariant::new(
                    1,
                    0,
                    vec![
                        IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                        IntermediateType::IU64,
                        IntermediateType::IAddress,
                    ],
                ),
            ],
        )
        .unwrap();

        let ctx = create_wasm_test_ctx(vec![], vec![enum_.clone()]);
        let compilation_ctx = &ctx;

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = execute_compute_fields_storage_size(
                &mut module,
                compilation_ctx,
                &variant.fields,
                used_bytes_plus_variant_index,
                j as u32,
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = execute_compute_storage_size(
            &mut module,
            compilation_ctx,
            Some(&enum_),
            None,
            initial_used_bytes,
        )
        .unwrap();
        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at initial_used_bytes={}: got {}, expected {}",
            initial_used_bytes, total, expected_total_size
        );
    }

    #[rstest]
    #[case(0, vec![1, 91], 92)]
    #[case(10, vec![1, 81], 82)]
    #[case(20, vec![1, 83], 84)]
    #[case(30, vec![1, 73], 74)]
    #[case(31, vec![1, 92], 93)]
    #[case(32, vec![1, 91], 92)]
    fn enum_with_nested_enum(
        #[case] initial_used_bytes: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
    ) {
        let (mut module, _, _) = build_module(None);
        let module_id = ModuleId::default();

        // Create a simple enum (no fields)
        let nested_enum_1 = IEnum::new(
            0,
            vec![
                IEnumVariant::new(0, 0, vec![]),
                IEnumVariant::new(1, 0, vec![]),
                IEnumVariant::new(2, 0, vec![]),
            ],
        )
        .unwrap();

        // More complex enum
        let nested_enum_2 = IEnum::new(
            0,
            vec![
                IEnumVariant::new(
                    0,
                    0,
                    vec![
                        IntermediateType::IAddress,
                        IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                        IntermediateType::IU64,
                    ],
                ),
                IEnumVariant::new(
                    1,
                    0,
                    vec![
                        IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                        IntermediateType::IU64,
                        IntermediateType::IAddress,
                    ],
                ),
            ],
        )
        .unwrap();

        // Create enum that contains the simple enum
        let enum_ = IEnum::new(
            2,
            vec![
                IEnumVariant::new(
                    0,
                    2,
                    vec![IntermediateType::IEnum {
                        module_id: module_id.clone(),
                        index: 0,
                    }],
                ),
                IEnumVariant::new(
                    1,
                    2,
                    vec![IntermediateType::IEnum {
                        module_id: module_id.clone(),
                        index: 1,
                    }],
                ),
            ],
        )
        .unwrap();

        let ctx = create_wasm_test_ctx(
            vec![],
            vec![nested_enum_1.clone(), nested_enum_2.clone(), enum_.clone()],
        );
        let compilation_ctx = &ctx;

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = execute_compute_fields_storage_size(
                &mut module,
                compilation_ctx,
                &variant.fields,
                used_bytes_plus_variant_index,
                j as u32,
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = execute_compute_storage_size(
            &mut module,
            compilation_ctx,
            Some(&enum_),
            None,
            initial_used_bytes,
        )
        .unwrap();
        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at initial_used_bytes={}: got {}, expected {}",
            initial_used_bytes, total, expected_total_size
        );
    }

    #[rstest]
    #[case(0, vec![63, 12, 63], 64)]
    #[case(10, vec![53, 12, 53], 54)]
    #[case(17, vec![46, 12, 46], 47)]
    #[case(20, vec![75, 19, 43], 76)]
    #[case(28, vec![67, 15, 35], 68)]
    #[case(30, vec![65, 13, 33], 66)]
    #[case(31, vec![64, 12, 32], 65)]
    #[case(32, vec![63, 12, 63], 64)]
    fn enum_with_structs(
        #[case] initial_used_bytes: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
    ) {
        let (mut module, _, _) = build_module(None);
        let module_id = ModuleId::default();

        // Create a struct without key
        let child_struct_1 = IStruct::new(
            StructDefinitionIndex::new(0),
            "ChildStruct1".to_string(),
            vec![
                (None, IntermediateType::IU32), // 4
                (None, IntermediateType::IU64), // 8
                (
                    None,
                    IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                ), // 32
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let child_struct_2 = IStruct::new(
            StructDefinitionIndex::new(1),
            "ChildStruct2".to_string(),
            vec![
                (None, IntermediateType::IU32), // 4
                (None, IntermediateType::IU64), // 8
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        // Struct with key
        let child_struct_3 = IStruct::new(
            StructDefinitionIndex::new(2),
            "ChildStruct3".to_string(),
            vec![
                (
                    None,
                    IntermediateType::IStruct {
                        module_id: module_id.clone(),
                        index: 0,
                        vm_handled_struct: VmHandledStruct::None,
                    },
                ), // 8
                (
                    None,
                    IntermediateType::IVector(Box::new(IntermediateType::IU8)),
                ), // 32
            ],
            HashMap::new(),
            true,
            IStructType::Common,
        );

        let enum_ = IEnum::new(
            0,
            vec![
                IEnumVariant::new(
                    0,
                    0,
                    vec![IntermediateType::IStruct {
                        module_id: module_id.clone(),
                        index: 0,
                        vm_handled_struct: VmHandledStruct::None,
                    }],
                ),
                IEnumVariant::new(
                    1,
                    0,
                    vec![IntermediateType::IStruct {
                        module_id: module_id.clone(),
                        index: 1,
                        vm_handled_struct: VmHandledStruct::None,
                    }],
                ),
                IEnumVariant::new(
                    2,
                    0,
                    vec![IntermediateType::IStruct {
                        module_id: module_id.clone(),
                        index: 2,
                        vm_handled_struct: VmHandledStruct::None,
                    }],
                ),
            ],
        )
        .unwrap();

        let ctx = create_wasm_test_ctx(
            vec![child_struct_1, child_struct_2, child_struct_3],
            vec![enum_.clone()],
        );
        let compilation_ctx = &ctx;

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = execute_compute_fields_storage_size(
                &mut module,
                compilation_ctx,
                &variant.fields,
                used_bytes_plus_variant_index,
                j as u32,
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = execute_compute_storage_size(
            &mut module,
            compilation_ctx,
            Some(&enum_),
            None,
            initial_used_bytes,
        )
        .unwrap();
        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at initial_used_bytes={}: got {}, expected {}",
            initial_used_bytes, total, expected_total_size
        );
    }
}
