use move_binary_format::file_format::FieldHandleIndex;
use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    translation::{
        TranslationError,
        intermediate_types::{IntermediateType, structs::IStruct},
        types_stack::TypesStack,
    },
};

/// Borrows a field of a struct.
///
/// Leaves the value pointer in the stack.
pub fn borrow_field(
    struct_: &IStruct,
    field_id: &FieldHandleIndex,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) {
    let Some(field_type) = struct_.fields_types.get(field_id) else {
        panic!(
            "{field_id} not found in {}",
            struct_.struct_definition_index
        )
    };

    let Some(field_offset) = struct_.field_offsets.get(field_id) else {
        panic!(
            "{field_id} offset not found in {}",
            struct_.struct_definition_index
        )
    };

    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(*field_offset as i32)
        .binop(BinaryOp::I32Add);

    types_stack.push(IntermediateType::IRef(Box::new(field_type.clone())));
}

/// Mutably borrows a field of a struct.
///
/// Leaves the value pointer in the stack.
pub fn mut_borrow_field(
    struct_: &IStruct,
    field_id: &FieldHandleIndex,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) {
    let Some(field_type) = struct_.fields_types.get(field_id) else {
        panic!(
            "{field_id:?} not found in {}",
            struct_.struct_definition_index
        )
    };

    let Some(field_offset) = struct_.field_offsets.get(field_id) else {
        panic!(
            "{field_id:?} offset not found in {}",
            struct_.struct_definition_index
        )
    };

    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(*field_offset as i32)
        .binop(BinaryOp::I32Add);

    types_stack.push(IntermediateType::IMutRef(Box::new(field_type.clone())));
}

/// Packs an struct.
///
/// This function is used with Pack and PackGeneric bytecodes to allocate memory for a struct and
/// save its fields into the allocated memory.
pub fn pack(
    struct_: &IStruct,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    // Pointer to the struct
    let pointer = module.locals.add(ValType::I32);
    // Pointer for simple types
    let ptr_to_data = module.locals.add(ValType::I32);

    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let mut offset = struct_.heap_size;

    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    for pack_type in struct_.fields.iter().rev() {
        offset -= 4;
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
                    // Heap types: The stack data is a pointer to the value, store directly
                    // that pointer in the struct
                    IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress
                    | IntermediateType::ISigner
                    | IntermediateType::IVector(_)
                    | IntermediateType::IStruct(_)
                    | IntermediateType::IGenericStructInstance(_, _) => {
                        builder.local_set(ptr_to_data);
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(TranslationError::FoundReferenceInsideStruct {
                            struct_index: struct_.index(),
                        });
                    }
                    IntermediateType::ITypeParameter(index) => {
                        return Err(TranslationError::FoundTypeParameterInsideStruct {
                            struct_index: struct_.index(),
                            type_parameter_index: *index,
                        });
                    }
                    IntermediateType::IEnum(_) => todo!(),
                    IntermediateType::IExternalUserData { .. } => todo!(),
                };

                builder.local_get(pointer).local_get(ptr_to_data).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );
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

/// Unpack an struct.
///
/// This function is used with Pack and PackGeneric bytecodes to allocate memory for a struct and
/// save its fields into the allocated memory.
pub fn unpack(
    struct_: &IStruct,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    // Pointer to the struct
    let pointer = module.locals.add(ValType::I32);
    let mut offset = 0;

    builder.local_set(pointer);

    for field in &struct_.fields {
        // Load the middle pointer
        builder.local_get(pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg { align: 0, offset },
        );

        match field {
            // Stack values: load in stack the actual value from the middle pointer
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                // Load the actual value
                builder.load(
                    compilation_ctx.memory_id,
                    if field.stack_data_size() == 8 {
                        LoadKind::I64 { atomic: false }
                    } else {
                        LoadKind::I32 { atomic: false }
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            // Heap types: The stack data is a pointer to the value is loaded at the beginning of
            // the loop
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _) => {}
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(TranslationError::FoundReferenceInsideStruct {
                    struct_index: struct_.index(),
                });
            }
            IntermediateType::ITypeParameter(index) => {
                return Err(TranslationError::FoundTypeParameterInsideStruct {
                    struct_index: struct_.index(),
                    type_parameter_index: *index,
                });
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IExternalUserData { .. } => todo!(),
        }

        types_stack.push(field.clone());
        offset += 4;
    }

    Ok(())
}
