use crate::{
    CompilationContext,
    translation::TranslationError,
    translation::intermediate_types::{IntermediateType, enums::IEnum, structs::IStruct},
};

const SLOT_SIZE: u32 = 32;

/// Computes the total storage size for an enum (maximum of all variants).
///
/// Arguments:
/// - `enum_`: the enum whose size to compute
/// - `used_bytes`: bytes already used in the storage slot (affects layout)
/// - `compilation_ctx`: context for type resolution
///
/// Returns `Ok(Some(u32))` with the size if possible, `Ok(None)` if generic type encountered, or an error.
pub fn compute_enum_storage_layout(
    enum_: &IEnum,
    used_bytes: u32,
    compilation_ctx: &CompilationContext,
) -> Result<Option<u32>, TranslationError> {
    // Increment the used bytes by 1 to account for the variant index
    let used_bytes = (used_bytes + 1) % SLOT_SIZE;

    // Compute the size of each variant and pick the largest one
    let mut max_size: u32 = 0u32;
    for variant in &enum_.variants {
        match compute_fields_storage_size(&variant.fields, used_bytes, compilation_ctx, || {
            TranslationError::FoundReferenceInsideEnum {
                enum_index: variant.belongs_to,
            }
        })? {
            Some(size) => {
                max_size = std::cmp::max(max_size, size);
            }
            None => {
                // Generic type encountered, can't compute size
                return Ok(None);
            }
        }
    }

    Ok(Some(max_size + 1)) // Add 1 to account for the variant index
}

/// Computes the total storage size for a struct.
///
/// Arguments:
/// - `struct_`: the struct whose storage size is being calculated
/// - `used_bytes`: number of bytes already used in the current slot (affects alignment/layout)
/// - `compilation_ctx`: context used to resolve types and perform lookups
///
/// Filters out UID or NamedId fields (which are not stored) and computes the total size of the remaining fields.
/// Returns `Ok(Some(u32))` with the computed size, `Ok(None)` if any field contains a generic type parameter, or an error if a reference is found.
pub fn compute_struct_storage_layout(
    struct_: &IStruct,
    used_bytes: u32,
    compilation_ctx: &CompilationContext,
) -> Result<Option<u32>, TranslationError> {
    // Filter out UIDs (not stored)
    let filtered_fields: Vec<IntermediateType> = struct_
        .fields
        .iter()
        .filter(|f| !f.is_uid_or_named_id(compilation_ctx))
        .cloned()
        .collect();

    let used_bytes = used_bytes % SLOT_SIZE;
    compute_fields_storage_size(&filtered_fields, used_bytes, compilation_ctx, || {
        TranslationError::FoundReferenceInsideStruct {
            struct_index: struct_.index(),
        }
    })
}

/// Internal helper to compute total storage size for a sequence of fields.
/// The `on_ref` closure defines the error to emit when a reference is found
/// (differs for enums vs structs).
fn compute_fields_storage_size<F>(
    fields: &[IntermediateType],
    used_bytes: u32,
    compilation_ctx: &CompilationContext,
    on_ref: F,
) -> Result<Option<u32>, TranslationError>
where
    F: Fn() -> TranslationError,
{
    let mut total_bytes = 0u32;
    let mut used_bytes = used_bytes;

    for field in fields {
        match field {
            IntermediateType::ITypeParameter(_) => {
                return Ok(None);
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
                    total_bytes += SLOT_SIZE + (SLOT_SIZE - used_bytes) % SLOT_SIZE;
                    used_bytes = SLOT_SIZE;
                } else {
                    let struct_size =
                        compute_struct_storage_layout(&struct_, used_bytes, compilation_ctx)?;
                    match struct_size {
                        Some(size) => {
                            total_bytes += size;
                            used_bytes = (used_bytes + size) % SLOT_SIZE;
                        }
                        None => return Ok(None),
                    }
                }
            }
            // Enums
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx
                    .get_enum_by_intermediate_type(field)
                    .expect("enum not found");

                let enum_size = compute_enum_storage_layout(&enum_, used_bytes, compilation_ctx)?;
                match enum_size {
                    Some(size) => {
                        total_bytes += size;
                        used_bytes = (used_bytes + size) % SLOT_SIZE;
                    }
                    None => return Ok(None),
                }
            }
            // All other types
            _ => {
                let free_bytes = SLOT_SIZE - used_bytes;
                let field_size_bytes = field_size(field, compilation_ctx);
                if field_size_bytes > free_bytes {
                    total_bytes += field_size_bytes + (free_bytes % SLOT_SIZE);
                    used_bytes = field_size_bytes;
                } else {
                    total_bytes += field_size_bytes;
                    used_bytes += field_size_bytes;
                }
            }
        }
    }

    Ok(Some(total_bytes))
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
mod tests {
    use super::*;
    use crate::compilation_context::{ModuleData, ModuleId};
    use crate::translation::intermediate_types::VmHandledStruct;
    use crate::translation::intermediate_types::enums::IEnumVariant;
    use crate::translation::intermediate_types::structs::IStructType;
    use move_binary_format::file_format::StructDefinitionIndex;
    use rstest::rstest;
    use std::collections::HashMap;

    // Helper to create a compilation context with registered structs and enums
    fn create_test_ctx(
        structs: Vec<IStruct>,
        enums: Vec<IEnum>,
    ) -> &'static CompilationContext<'static> {
        use std::ptr::addr_of_mut;

        let memory_id: walrus::MemoryId = unsafe { std::mem::zeroed() };
        let allocator: walrus::FunctionId = unsafe { std::mem::zeroed() };

        let module_id = ModuleId::default();
        let mut module_data = ModuleData::default();
        module_data.id = module_id;

        unsafe {
            // Get a raw pointer to the structs Vec field inside StructData
            let structs_data_ptr = addr_of_mut!(module_data.structs) as *mut u8;
            // The structs Vec<IStruct> is the first field in StructData (offset 0)
            let structs_vec_ptr = structs_data_ptr as *mut Vec<IStruct>;
            std::ptr::write(structs_vec_ptr, structs);
        }
        module_data.enums.enums = enums;

        let root = Box::leak(Box::new(module_data));
        let deps = Box::leak(Box::new(HashMap::new()));
        Box::leak(Box::new(CompilationContext::new(
            root, deps, memory_id, allocator,
        )))
    }

    // Minimal inert compilation context for tests (no derefs on primitive/vector paths)
    fn dummy_ctx<'a>() -> CompilationContext<'a> {
        let memory_id: walrus::MemoryId = unsafe { std::mem::zeroed() };
        let allocator: walrus::FunctionId = unsafe { std::mem::zeroed() };
        let root = Box::leak(Box::new(ModuleData::default()));
        let deps = Box::leak(Box::new(HashMap::new()));
        CompilationContext::new(root, deps, memory_id, allocator)
    }

    #[rstest]
    #[case(0, vec![3, 12], 13)]
    #[case(25, vec![3, 14], 15)]
    #[case(28, vec![3, 15], 16)]
    #[case(32, vec![3, 12], 13)]
    fn enum_primitive_fields(
        #[case] initial_used_bytes: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
    ) {
        let ctx = dummy_ctx();
        let enum_ = IEnum::new(
            0,
            vec![
                IEnumVariant::new(0, 0, vec![IntermediateType::IU8, IntermediateType::IU16]),
                IEnumVariant::new(1, 0, vec![IntermediateType::IU32, IntermediateType::IU64]),
            ],
        )
        .unwrap();

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                used_bytes_plus_variant_index,
                &ctx,
                || TranslationError::FoundReferenceInsideEnum { enum_index: 0 },
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_layout(&enum_, initial_used_bytes, &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at initial_used_bytes={}: got {}, expected {}",
            initial_used_bytes, total, expected_total_size
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
        let ctx = dummy_ctx();
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

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                used_bytes_plus_variant_index,
                &ctx,
                || TranslationError::FoundReferenceInsideEnum { enum_index: 0 },
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_layout(&enum_, initial_used_bytes, &ctx)
            .unwrap()
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

        let ctx = create_test_ctx(vec![], vec![nested_enum_1.clone(), nested_enum_2.clone()]);

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

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                used_bytes_plus_variant_index,
                ctx,
                || TranslationError::FoundReferenceInsideEnum { enum_index: 0 },
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_layout(&enum_, initial_used_bytes, ctx)
            .unwrap()
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

        let ctx = create_test_ctx(vec![child_struct_1, child_struct_2, child_struct_3], vec![]);

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

        let used_bytes_plus_variant_index = (initial_used_bytes + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                used_bytes_plus_variant_index,
                ctx,
                || TranslationError::FoundReferenceInsideEnum { enum_index: 0 },
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at initial_used_bytes={}: got {}, expected {}",
                j, initial_used_bytes, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_layout(&enum_, initial_used_bytes, ctx)
            .unwrap()
            .unwrap();
        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at initial_used_bytes={}: got {}, expected {}",
            initial_used_bytes, total, expected_total_size
        );
    }
}
