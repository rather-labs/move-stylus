use crate::{
    CompilationContext,
    runtime::RuntimeFunction,
    translation::TranslationError,
    translation::intermediate_types::{IntermediateType, enums::IEnum},
};
use walrus::ir::{BinaryOp, MemArg, StoreKind};
use walrus::{InstrSeqBuilder, LocalId, Module, ValType};

const SLOT_SIZE: u32 = 32;

/// Context for error reporting when computing storage size.
/// Determines which error variant to construct when encountering references or type parameters.
enum FieldsErrorContext {
    Struct { struct_index: u16 },
    Enum { enum_index: u16, variant_index: u16 },
}

/// Calculates the total number of bytes needed to store an enum in storage.
///  Note: The storage size depends on the starting offset within the storage slot.
///
/// # Arguments
/// - `enum_`: The enum type whose storage size we want to compute.
/// - `slot_offset`: The byte offset (within a slot) where the enum starts.
/// - `compilation_ctx`: Context with type and layout information.
///
/// # Returns
/// - `Ok(size)`: The total size in bytes needed to store the enum (including one byte for the tag/discriminant).
/// - `Err(e)`: Returns an error if the calculation failed (e.g., due to a type parameter).
///
/// The returned size is the space for the tag plus enough bytes to store the largest variant of the enum.
pub fn compute_enum_storage_size(
    enum_: &IEnum,
    slot_offset: u32,
    compilation_ctx: &CompilationContext,
) -> Result<u32, TranslationError> {
    // Increment the offset by 1 to account for the variant index
    let slot_offset = (slot_offset + 1) % SLOT_SIZE;

    // Compute the size of each variant and pick the largest one
    let mut enum_size = 0u32;

    for variant in &enum_.variants {
        let variant_size = compute_fields_storage_size(
            &variant.fields,
            slot_offset,
            compilation_ctx,
            FieldsErrorContext::Enum {
                enum_index: enum_.index,
                variant_index: variant.index,
            },
        )?;

        // Take the maximum across all variants
        enum_size = enum_size.max(variant_size);
    }

    // Add 1 to account for the variant index
    Ok(enum_size + 1)
}

/// Compute the storage size for a sequence of fields.
///  Note: The storage size depends on the starting offset within the slot.
///
/// # Arguments:
/// - `fields`: the fields to compute the size for
/// - `slot_offset`: the offset within the slot where the fields start
/// - `compilation_ctx`: the compilation context
/// - `error_context`: the error context
///
/// # Returns
/// - `Ok(size)`: the size in bytes needed to store the fields
fn compute_fields_storage_size(
    fields: &[IntermediateType],
    slot_offset: u32,
    compilation_ctx: &CompilationContext,
    error_context: FieldsErrorContext,
) -> Result<u32, TranslationError> {
    let mut size = 0u32;
    let mut offset = slot_offset;

    for field in fields {
        match field {
            IntermediateType::ITypeParameter(type_parameter_index) => {
                return Err(match error_context {
                    FieldsErrorContext::Struct { struct_index } => {
                        TranslationError::FoundTypeParameterInsideStruct {
                            struct_index,
                            type_parameter_index: *type_parameter_index,
                        }
                    }
                    FieldsErrorContext::Enum {
                        enum_index,
                        variant_index,
                    } => TranslationError::FoundTypeParameterInsideEnumVariant {
                        enum_index,
                        variant_index,
                    },
                });
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(match error_context {
                    FieldsErrorContext::Struct { struct_index } => {
                        TranslationError::FoundReferenceInsideStruct { struct_index }
                    }
                    FieldsErrorContext::Enum { enum_index, .. } => {
                        TranslationError::FoundReferenceInsideEnum { enum_index }
                    }
                });
            }
            IntermediateType::IGenericStructInstance { .. } | IntermediateType::IStruct { .. } => {
                let struct_ = compilation_ctx
                    .get_struct_by_intermediate_type(field)
                    .expect("struct not found");

                if struct_.has_key {
                    // Parent stores 32-byte UID when child has key
                    // size += SLOT_SIZE + (SLOT_SIZE - used_bytes) % SLOT_SIZE
                    size += SLOT_SIZE + (SLOT_SIZE - offset) % SLOT_SIZE;

                    // offset = SLOT_SIZE
                    offset = SLOT_SIZE;
                } else {
                    // Filter out UIDs (not stored)
                    let filtered_fields: Vec<IntermediateType> = struct_
                        .fields
                        .iter()
                        .filter(|f| !f.is_uid_or_named_id(compilation_ctx))
                        .cloned()
                        .collect();

                    // Compute the size of the struct
                    let struct_size = compute_fields_storage_size(
                        &filtered_fields,
                        offset,
                        compilation_ctx,
                        FieldsErrorContext::Struct {
                            struct_index: struct_.index(),
                        },
                    )?;

                    // total_bytes += size
                    size += struct_size;

                    // offset = (offset + size) % SLOT_SIZE
                    offset = (offset + struct_size) % SLOT_SIZE;
                }
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx
                    .get_enum_by_intermediate_type(field)
                    .expect("enum not found");

                // Compute the size of the enum
                let enum_size = compute_enum_storage_size(&enum_, offset, compilation_ctx)?;

                // total_bytes += size
                size += enum_size;

                // offset = (offset + size) % SLOT_SIZE
                offset = (offset + enum_size) % SLOT_SIZE;
            }
            _ => {
                let field_size = field_size(field, compilation_ctx);

                // free_bytes = SLOT_SIZE - used_bytes
                let free_bytes = SLOT_SIZE - offset;

                // if field_size > free_bytes
                if field_size > free_bytes {
                    // total_bytes += field_size + (free_bytes % SLOT_SIZE)
                    size += field_size + (free_bytes % SLOT_SIZE);

                    // offset = field_size
                    offset = field_size;
                } else {
                    // total_bytes += field_size
                    size += field_size;

                    // offset += field_size
                    offset += field_size;
                }
            }
        }
    }

    Ok(size)
}

/// Compute where an enum's storage ends as a tuple (tail_slot_ptr, tail_slot_offset)
///
/// - `head_slot_ptr`: start slot (LocalId, U256 big-endian)
/// - `head_slot_offset`: start offset (LocalId, 0-31)
/// - `itype`: enum type
/// - `compilation_ctx`: context
///
/// Returns (tail_slot_ptr, tail_slot_offset):
/// - If enum fits in current slot: (head_slot_ptr, head_slot_offset + enum_size)
/// - If not: advances slots as needed so offset wraps to final position.
pub fn compute_enum_storage_tail_position(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    itype: &IntermediateType,
    head_slot_ptr: LocalId,
    head_slot_offset: LocalId,
    compilation_ctx: &CompilationContext,
) -> Result<(LocalId, LocalId), TranslationError> {
    let match_on_offset_fn =
        RuntimeFunction::MatchOnOffset.get_generic(module, compilation_ctx, &[itype]);

    let enum_size = module.locals.add(ValType::I32);

    builder
        .local_get(head_slot_offset)
        .call(match_on_offset_fn)
        .local_set(enum_size);

    let tail_slot_offset = module.locals.add(ValType::I32);
    let tail_slot_ptr = module.locals.add(ValType::I32);

    // *tail_slot_ptr = *head_slot_ptr (start by copying the head slot)
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(tail_slot_ptr)
        .local_get(head_slot_ptr)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // free_bytes = 32 - head_slot_offset
    let free_bytes = module.locals.add(ValType::I32);
    builder
        .i32_const(SLOT_SIZE as i32)
        .local_get(head_slot_offset)
        .binop(BinaryOp::I32Sub)
        .local_set(free_bytes);

    builder
        .local_get(enum_size)
        .local_get(free_bytes)
        .binop(BinaryOp::I32GeU)
        .if_else(
            None,
            |then| {
                // Case: enum_size >= free_bytes, so it will span multiple slots
                // 1) *tail_slot_ptr = *head_slot_ptr + ((enum_size - free_bytes) / 32) as u256 LE

                // delta_slot_ptr = (enum_size - free_bytes) / 32 as u256 LE (how many slots to add to the current slot)
                let delta_slot_ptr = module.locals.add(ValType::I32);
                // Allocate 32 bytes for the slot offset
                then.i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(delta_slot_ptr);

                // (enum_size - free_bytes) / 32 as u32
                then.local_get(enum_size)
                    .local_get(free_bytes)
                    .binop(BinaryOp::I32Sub)
                    .i32_const(SLOT_SIZE as i32)
                    .binop(BinaryOp::I32DivS)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add);

                // Store the offset in the first 4 bytes to make it a u256 LE
                then.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // Swap the end slot from BE to LE for addition
                let swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
                then.local_get(tail_slot_ptr)
                    .local_get(tail_slot_ptr)
                    .call(swap_256_fn);

                // Add the offset to the end slot (right now equal to the current slot)
                let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));
                then.local_get(delta_slot_ptr)
                    .local_get(tail_slot_ptr)
                    .local_get(tail_slot_ptr)
                    .i32_const(32)
                    .call(add_u256_fn)
                    .local_set(tail_slot_ptr); // Why do we need to set the end_slot_ptr again? TODO: check this

                // Swap back to BE
                then.local_get(tail_slot_ptr)
                    .local_get(tail_slot_ptr)
                    .call(swap_256_fn);

                // 2) tail_slot_offset = (enum_size - free_bytes) % 32
                then.local_get(enum_size)
                    .local_get(free_bytes)
                    .binop(BinaryOp::I32Sub)
                    .i32_const(SLOT_SIZE as i32)
                    .binop(BinaryOp::I32RemS)
                    .local_set(tail_slot_offset);
            },
            |else_| {
                // Case: enum_size < free_bytes, so it fits entirely in the current slot
                // 1) end_slot = start_slot (already set by the copy above)
                // 2) tail_slot_offset = head_slot_offset + enum_size
                else_
                    .local_get(head_slot_offset)
                    .local_get(enum_size)
                    .binop(BinaryOp::I32Add)
                    .local_set(tail_slot_offset);
            },
        );

    Ok((tail_slot_ptr, tail_slot_offset))
}

/// Returns the storage-encoded size in bytes for a given intermediate type.
///
/// Note:
/// - For structs without `key`, size is 0 because their inline size depends on fields;
///   callers compute layout using field-by-field accumulation.
/// - For structs with `key`, at least 32 bytes are used to store the UID reference.
pub fn field_size(field: &IntermediateType, compilation_ctx: &CompilationContext) -> u32 {
    match field {
        IntermediateType::IBool | IntermediateType::IU8 => 1,
        // Enums have variable size depending on its fields. This is computed via compute_enum_storage_size
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => 0,
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
    use alloy_primitives::U256;
    use move_binary_format::file_format::StructDefinitionIndex;
    use rstest::rstest;
    use std::collections::HashMap;

    use crate::compilation_context::{ModuleData, ModuleId};
    use crate::test_tools::build_module;
    use crate::translation::intermediate_types::{
        VmHandledStruct,
        enums::IEnumVariant,
        structs::{IStruct, IStructType},
    };

    // Helper to create a compilation context for tests
    // Returns both the context and the module so they share the same memory_id and allocator
    fn create_test_ctx(
        structs: Vec<IStruct>,
        enums: Vec<IEnum>,
    ) -> (CompilationContext<'static>, Module) {
        use std::ptr::addr_of_mut;

        let module_id = ModuleId::default();
        let mut module_data = ModuleData {
            id: module_id,
            ..Default::default()
        };

        unsafe {
            let structs_data_ptr = addr_of_mut!(module_data.structs) as *mut u8;
            let structs_vec_ptr = structs_data_ptr as *mut Vec<IStruct>;
            std::ptr::write(structs_vec_ptr, structs);
        }
        module_data.enums.enums = enums;

        let root = Box::leak(Box::new(module_data));
        let deps = Box::leak(Box::new(HashMap::new()));
        // Create module with memory_id and allocator that will be shared
        let (module, allocator, memory_id) = build_module(None);
        let ctx = CompilationContext::new(root, deps, memory_id, allocator);
        (ctx, module)
    }

    fn execute_compute_enum_storage_tail_position(
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        itype: &IntermediateType,
        start_slot: [u8; 32],
        start_written_bytes: u32,
    ) -> Result<([u8; 32], u32), Box<dyn std::error::Error>> {
        // Ensure the enum's storage_size is computed before creating runtime functions
        let enum_ = compilation_ctx
            .get_enum_by_intermediate_type(itype)
            .map_err(|e| format!("Failed to get enum: {e:?}"))?;
        // This will compute and cache the storage_size if needed
        let _ = enum_
            .get_storage_size(compilation_ctx)
            .map_err(|e| format!("Failed to compute enum storage size: {e:?}"))?;

        // Pre-create all runtime functions that will be needed by compute_enum_storage_tail_position
        // This ensures they're properly registered in the module before we try to use them
        let _match_on_offset_fn =
            RuntimeFunction::MatchOnOffset.get_generic(module, compilation_ctx, &[itype]);
        let _swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
        let _add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));

        // Test function setup
        let mut function = walrus::FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32, ValType::I32],
        );

        let mut builder = function.func_body();
        let start_slot_ptr = module.locals.add(ValType::I32);
        let start_written_bytes_in_slot = module.locals.add(ValType::I32);

        // Call compute_enum_end_slot
        let (end_slot_ptr, end_written_bytes_in_slot) = compute_enum_storage_tail_position(
            module,
            &mut builder,
            itype,
            start_slot_ptr,
            start_written_bytes_in_slot,
            compilation_ctx,
        )?;

        builder.local_get(end_slot_ptr);
        builder.local_get(end_written_bytes_in_slot);

        let test_func = function.finish(
            vec![start_slot_ptr, start_written_bytes_in_slot],
            &mut module.funcs,
        );
        module.exports.add("test_compute_enum_end_slot", test_func);

        // Execute the WASM function - returns (end_slot_ptr, end_written_bytes)
        let (_, instance, mut store, entrypoint) =
            crate::test_tools::setup_wasmtime_module::<(i32, i32), (i32, i32)>(
                module,
                start_slot.into(),
                "test_compute_enum_end_slot",
                None,
            );

        // Wrap start_written_bytes to 0-31 range since head_slot_offset represents bytes within a slot
        let (end_slot_ptr, end_written_bytes) = entrypoint
            .call(&mut store, (0_i32, start_written_bytes as i32))
            .map_err(|e| format!("WASM execution error: {e:?}"))?;

        // Read end_slot from the returned pointer
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or("Failed to get memory")?;
        let mut end_slot = [0u8; 32];
        memory
            .read(&store, end_slot_ptr as usize, &mut end_slot)
            .map_err(|e| format!("Failed to read end slot: {e:?}"))?;

        Ok((end_slot, end_written_bytes as u32))
    }

    #[rstest]
    #[case(
        0,
        [0u8; 32],
        13,
        [0u8; 32],
        13
    )]
    #[case(
        4,
        [0u8; 32],
        13,
        [0u8; 32],
        17
    )]
    #[case(
        10,
        [0u8; 32],
        13,
        [0u8; 32],
        23
    )]
    #[case(
        25,
        [0u8; 32],
        15,
        U256::from(1).to_be_bytes(),
        8
    )]
    #[case(
        28,
        [0u8; 32],
        16,
        U256::from(1).to_be_bytes(),
        12
    )]
    #[case(
        32,
        [0u8; 32],
        13,
        U256::from(1).to_be_bytes(),
        13
    )]
    fn enum_primitive_fields(
        #[case] head_offset: u32,
        #[case] head_slot: [u8; 32],
        #[case] expected_size: u32,
        #[case] expected_tail_slot: [u8; 32],
        #[case] expected_end_written_bytes: u32,
    ) {
        let enum_ = IEnum::new(
            0,
            vec![
                IEnumVariant::new(0, 0, vec![IntermediateType::IU8, IntermediateType::IU16]),
                IEnumVariant::new(1, 0, vec![IntermediateType::IU32, IntermediateType::IU64]),
            ],
        )
        .unwrap();

        let (ctx, mut module) = create_test_ctx(vec![], vec![enum_.clone()]);
        let compilation_ctx = &ctx;

        let result = compute_enum_storage_size(&enum_, head_offset, compilation_ctx).unwrap();

        assert_eq!(
            result, expected_size,
            "Enum storage size mismatch at head_offset={}: got {}, expected {}",
            head_offset, result, expected_size
        );

        // Create IntermediateType for the enum
        let itype = IntermediateType::IEnum {
            module_id: ModuleId::default(),
            index: enum_.index,
        };

        // Test compute_enum_end_slot
        let (tail_slot_bytes, tail_offset) = execute_compute_enum_storage_tail_position(
            &mut module,
            compilation_ctx,
            &itype,
            head_slot,
            head_offset,
        )
        .unwrap();

        assert_eq!(
            tail_slot_bytes, expected_tail_slot,
            "Tail slot mismatch at head_offset={}: got {:?}, expected {:?}",
            head_offset, tail_slot_bytes, expected_tail_slot
        );

        assert_eq!(
            tail_offset, expected_end_written_bytes,
            "Tail offset mismatch at head_offset={}: got {}, expected {}",
            head_offset, tail_offset, expected_end_written_bytes
        );
    }

    #[rstest]
    #[case(
        0,
        vec![71, 91],
        92,
        [0u8; 32],
        U256::from(2).to_be_bytes(),
        28
    )]
    #[case(
        10,
        vec![61, 81],
        82,
        [0u8; 32],
        U256::from(2).to_be_bytes(),
        28
    )]
    #[case(
        20,
        vec![83, 71],
        84,
        [0u8; 32],
        U256::from(3).to_be_bytes(),
        8
    )]
    #[case(
        30,
        vec![73, 61],
        74,
        [0u8; 32],
        U256::from(3).to_be_bytes(),
        8
    )]
    #[case(
        31,
        vec![72, 60],
        73,
        U256::from(1000230001).to_be_bytes(),
        U256::from(1000230004).to_be_bytes(),
        8
    )]
    #[case(
        32,
        vec![71, 91],
        92,
        U256::from(12345).to_be_bytes(),
        U256::from(12348).to_be_bytes(),
        28
    )]
    fn enum_with_vectors(
        #[case] head_offset: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
        #[case] head_slot: [u8; 32],
        #[case] expected_tail_slot: [u8; 32],
        #[case] expected_tail_offset: u32,
    ) {
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

        let (ctx, mut module) = create_test_ctx(vec![], vec![enum_.clone()]);
        let compilation_ctx = &ctx;

        let head_offset_plus_variant = (head_offset + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                head_offset_plus_variant,
                compilation_ctx,
                FieldsErrorContext::Enum {
                    enum_index: enum_.index,
                    variant_index: variant.index,
                },
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at head_offset={}: got {}, expected {}",
                j, head_offset, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_size(&enum_, head_offset, compilation_ctx).unwrap();

        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at head_offset={}: got {}, expected {}",
            head_offset, total, expected_total_size
        );

        // Create IntermediateType for the enum
        let itype = IntermediateType::IEnum {
            module_id: ModuleId::default(),
            index: enum_.index,
        };

        // Test compute_enum_end_slot
        let (tail_slot, tail_offset) = execute_compute_enum_storage_tail_position(
            &mut module,
            compilation_ctx,
            &itype,
            head_slot,
            head_offset,
        )
        .unwrap();

        assert_eq!(
            tail_slot, expected_tail_slot,
            "Tail slot mismatch at head_offset={}: got {:?}, expected {:?}",
            head_offset, tail_slot, expected_tail_slot
        );

        assert_eq!(
            tail_offset, expected_tail_offset,
            "Tail offset mismatch at head_offset={}: got {}, expected {}",
            head_offset, tail_offset, expected_tail_offset
        );
    }

    #[rstest]
    #[case(0, vec![1, 91], 92, [0u8; 32], U256::from(2).to_be_bytes(), 28)]
    #[case(10, vec![1, 81], 82, [0u8; 32], U256::from(2).to_be_bytes(), 28)]
    #[case(20, vec![1, 83], 84, [0u8; 32], U256::from(3).to_be_bytes(), 8)]
    #[case(30, vec![1, 73], 74, [0u8; 32], U256::from(3).to_be_bytes(), 8)]
    #[case(31, vec![1, 92], 93, [0u8; 32], U256::from(3).to_be_bytes(), 28)]
    #[case(32, vec![1, 91], 92, [0u8; 32], U256::from(3).to_be_bytes(), 28)]
    fn enum_with_nested_enum(
        #[case] head_offset: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
        #[case] head_slot: [u8; 32],
        #[case] expected_tail_slot: [u8; 32],
        #[case] expected_tail_offset: u32,
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

        let (ctx, mut module) = create_test_ctx(
            vec![],
            vec![nested_enum_1.clone(), nested_enum_2.clone(), enum_.clone()],
        );
        let compilation_ctx = &ctx;

        let head_offset_plus_variant = (head_offset + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                head_offset_plus_variant,
                compilation_ctx,
                FieldsErrorContext::Enum {
                    enum_index: enum_.index,
                    variant_index: variant.index,
                },
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at head_offset={}: got {}, expected {}",
                j, head_offset, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_size(&enum_, head_offset, compilation_ctx).unwrap();

        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at head_offset={}: got {}, expected {}",
            head_offset, total, expected_total_size
        );

        // Create IntermediateType for the enum
        let itype = IntermediateType::IEnum {
            module_id: ModuleId::default(),
            index: enum_.index,
        };

        // Test compute_enum_end_slot
        let (tail_slot, tail_offset) = execute_compute_enum_storage_tail_position(
            &mut module,
            compilation_ctx,
            &itype,
            head_slot,
            head_offset,
        )
        .unwrap();

        assert_eq!(
            tail_slot, expected_tail_slot,
            "Tail slot mismatch at head_offset={}: got {:?}, expected {:?}",
            head_offset, tail_slot, expected_tail_slot
        );

        assert_eq!(
            tail_offset, expected_tail_offset,
            "Tail offset mismatch at head_offset={}: got {}, expected {}",
            head_offset, tail_offset, expected_tail_offset
        );
    }

    #[rstest]
    #[case(0, vec![63, 12, 63], 64, [0u8; 32], U256::from(2).to_be_bytes(), 0)]
    #[case(10, vec![53, 12, 53], 54, [0u8; 32], U256::from(2).to_be_bytes(), 0)]
    #[case(17, vec![46, 12, 46], 47, [0u8; 32], U256::from(2).to_be_bytes(), 0)]
    #[case(20, vec![75, 19, 43], 76, [0u8; 32], U256::from(3).to_be_bytes(), 0)]
    #[case(28, vec![67, 15, 35], 68, [0u8; 32], U256::from(3).to_be_bytes(), 0)]
    #[case(30, vec![65, 13, 33], 66, [0u8; 32], U256::from(3).to_be_bytes(), 0)]
    #[case(31, vec![64, 12, 32], 65, [0u8; 32], U256::from(3).to_be_bytes(), 0)]
    #[case(32, vec![63, 12, 63], 64, [0u8; 32], U256::from(3).to_be_bytes(), 0)]
    fn enum_with_structs(
        #[case] head_offset: u32,
        #[case] expected_variant_sizes: Vec<u32>,
        #[case] expected_total_size: u32,
        #[case] head_slot: [u8; 32],
        #[case] expected_tail_slot: [u8; 32],
        #[case] expected_tail_offset: u32,
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

        let (ctx, mut module) = create_test_ctx(
            vec![child_struct_1, child_struct_2, child_struct_3],
            vec![enum_.clone()],
        );
        let compilation_ctx = &ctx;

        let head_offset_plus_variant = (head_offset + 1) % SLOT_SIZE;

        // Test each variant size
        for (j, variant) in enum_.variants.iter().enumerate() {
            let variant_size = compute_fields_storage_size(
                &variant.fields,
                head_offset_plus_variant,
                compilation_ctx,
                FieldsErrorContext::Enum {
                    enum_index: enum_.index,
                    variant_index: variant.index,
                },
            )
            .unwrap();
            assert_eq!(
                variant_size, expected_variant_sizes[j],
                "variant{} size mismatch at head_offset={}: got {}, expected {}",
                j, head_offset, variant_size, expected_variant_sizes[j]
            );
        }

        // Test enum picks the max
        let total = compute_enum_storage_size(&enum_, head_offset, compilation_ctx).unwrap();

        assert_eq!(
            total, expected_total_size,
            "enum max size mismatch at head_offset={}: got {}, expected {}",
            head_offset, total, expected_total_size
        );

        // Create IntermediateType for the enum
        let itype = IntermediateType::IEnum {
            module_id: ModuleId::default(),
            index: enum_.index,
        };

        // Test compute_enum_end_slot
        let (tail_slot, tail_offset) = execute_compute_enum_storage_tail_position(
            &mut module,
            compilation_ctx,
            &itype,
            head_slot,
            head_offset,
        )
        .unwrap();

        assert_eq!(
            tail_slot, expected_tail_slot,
            "Tail slot mismatch at head_offset={}: got {:?}, expected {:?}",
            head_offset, tail_slot, expected_tail_slot
        );

        assert_eq!(
            tail_offset, expected_tail_offset,
            "Tail offset mismatch at head_offset={}: got {}, expected {}",
            head_offset, tail_offset, expected_tail_offset
        );
    }
}
