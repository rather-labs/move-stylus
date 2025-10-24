use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    translation::{
        TranslationError,
        intermediate_types::{IntermediateType, enums::IEnum},
        types_stack::TypesStack,
    },
};

use super::structs::unpack_fields;

/// Packs an enum variant.
///
/// This function is used with PackVariant and PackVariantGeneric bytecodes to allocate memory for
/// a struct and save its fields into the allocated memory.
pub fn pack_variant(
    enum_: &IEnum,
    variant_index: u16,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    // Pointer to the enum
    let pointer = module.locals.add(ValType::I32);

    // Pointer for simple types
    let ptr_to_data = module.locals.add(ValType::I32);

    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

    // This accounts for the variant that occupies most space in memory (plus the 4 bytes for the variant index)
    let heap_size = enum_
        .heap_size
        .ok_or(TranslationError::PackingGenericEnumVariant {
            enum_index: enum_.index,
        })?;

    // Allocate memory for the enum variant
    builder
        .i32_const(heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    // Save the variant index in the first 4 bytes
    builder
        .local_get(pointer)
        .i32_const(variant_index as i32)
        .store(
            compilation_ctx.memory_id,
            walrus::ir::StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    let variant = &enum_.variants[variant_index as usize];
    let variant_fields_count = variant.fields.len();

    // Start packing the fields in reverse order to match the types stack order
    let mut offset = variant_fields_count as u32 * 4;

    for pack_type in variant.fields.iter().rev() {
        match types_stack.pop()? {
            t if &t == pack_type => {
                match pack_type {
                    // Stack values: create a middle pointer to save the actual value
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        let data_size = pack_type.stack_data_size();
                        let (val, store_kind) = if data_size == 8 {
                            (val_64, StoreKind::I64 { atomic: false })
                        } else {
                            (val_32, StoreKind::I32 { atomic: false })
                        };

                        // Save the actual value
                        builder.local_set(val);

                        // Create a pointer for the value
                        builder
                            .i32_const(data_size as i32)
                            .call(compilation_ctx.allocator)
                            .local_tee(ptr_to_data);

                        // Store the actual value behind the middle_ptr
                        builder.local_get(val).store(
                            compilation_ctx.memory_id,
                            store_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );
                    }
                    // Heap types
                    IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress
                    | IntermediateType::ISigner
                    | IntermediateType::IVector(_)
                    | IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. }
                    | IntermediateType::IEnum(_) => {
                        builder.local_set(ptr_to_data);
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(TranslationError::FoundReferenceInsideEnum {
                            enum_index: enum_.index,
                        });
                    }
                    IntermediateType::ITypeParameter(_) => {
                        return Err(TranslationError::FoundTypeParameterInsideEnumVariant {
                            enum_index: enum_.index,
                            variant_index,
                        });
                    }
                };

                // Store the ptr to the value in the enum variant memory
                builder.local_get(pointer).local_get(ptr_to_data).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );

                offset -= 4;
            }

            t => Err(TranslationError::TypeMismatch {
                expected: pack_type.clone(),
                found: t,
            })?,
        }
    }

    builder.local_get(pointer);

    Ok(())
}

/// Unpacks an enum variant
///
/// It expects a pointer to the enum variant on top of the stack, and it pushes the unpacked variant fields to the stack.
pub fn unpack_variant(
    enum_: &IEnum,
    variant_index: u16,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    let pointer = module.locals.add(ValType::I32);

    // Skit the first 4 bytes which is the variant index, and unpack the fields
    builder
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(pointer);

    // Use the common field unpacking logic
    unpack_fields(
        &enum_.variants[variant_index as usize].fields,
        builder,
        compilation_ctx,
        pointer,
        &IntermediateType::IEnum(enum_.index),
        types_stack,
    )?;

    Ok(())
}

/// Unpacks a variant reference
///
/// It expects a reference to the enum variant on top of the stack, and it pushes (mut or imm) references to the unpacked variant fields.
pub fn unpack_variant_ref(
    enum_: &IEnum,
    variant_index: u16,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
    is_mut_ref: bool,
) -> Result<(), TranslationError> {
    // Pointer to the enum variant
    let pointer = module.locals.add(ValType::I32);

    // Load the reference to the enum variant
    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(pointer);

    // Skip the first 4 bytes which is the variant index
    let mut offset = 4;

    for field in &enum_.variants[variant_index as usize].fields {
        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum(_) => {
                // Add the offset to the pointer
                builder
                    .local_get(pointer)
                    .i32_const(offset)
                    .binop(BinaryOp::I32Add);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(TranslationError::FoundReferenceInsideEnum {
                    enum_index: enum_.index,
                });
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(TranslationError::FoundTypeParameterInsideEnumVariant {
                    enum_index: enum_.index,
                    variant_index,
                });
            }
        }

        // Push the reference to the unpacked field
        if is_mut_ref {
            types_stack.push(IntermediateType::IMutRef(Box::new(field.clone())));
        } else {
            types_stack.push(IntermediateType::IRef(Box::new(field.clone())));
        }

        offset += 4;
    }

    Ok(())
}
