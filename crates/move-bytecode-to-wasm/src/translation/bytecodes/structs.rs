use move_binary_format::{file_format::FieldHandleIndex, internals::ModuleIndex};
use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    translation::{
        TranslationError,
        intermediate_types::{IntermediateType, VmHandledStruct, structs::IStruct},
        types_stack::TypesStack,
    },
    vm_handled_types::{VmHandledType, is_uid_or_named_id, named_id::NamedId, uid::Uid},
};

/// (Mutably) Borrows a field of a struct.
///
/// Leaves the value pointer in the stack.
pub fn borrow_field(
    struct_: &IStruct,
    field_id: &FieldHandleIndex,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
) -> Result<IntermediateType, TranslationError> {
    let Some(field_type) = struct_.fields_types.get(field_id) else {
        return Err(TranslationError::StructFieldNotFound {
            field_id: field_id.into_index(),
            struct_identifier: struct_.identifier.clone(),
        });
    };

    let Some(field_offset) = struct_.field_offsets.get(field_id) else {
        return Err(TranslationError::StructFieldOffsetNotFound {
            field_id: field_id.into_index(),
            struct_identifier: struct_.identifier.clone(),
        });
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

    Ok(field_type.clone())
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

    let mut offset = struct_.heap_size;

    // If the struct is saved in storage (has key ability), the owner's id must be prepended to the
    // struct memory representation. Since we are packing it, means it is a new structure, so it
    // has no owner (all zeroes). We just allocate the space
    if struct_.has_key {
        builder.i32_const(32).call(compilation_ctx.allocator).drop();
    }

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
                        let data_size = pack_type.wasm_memory_data_size()?;
                        let val = module.locals.add(ValType::try_from(pack_type)?);
                        let store_kind = pack_type.store_kind()?;

                        // Save the actual value
                        builder.local_set(val);

                        // Create a pointer for the value
                        builder
                            .i32_const(data_size)
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

                    // If we find an UID or NamedId struct, in the 4 bytes before its pointer, we
                    // write the address of the struct holding it
                    pack_type if is_uid_or_named_id(pack_type, compilation_ctx)? => {
                        builder.local_set(ptr_to_data);

                        builder
                            .local_get(ptr_to_data)
                            .i32_const(4)
                            .binop(BinaryOp::I32Sub)
                            .local_get(pointer)
                            .store(
                                compilation_ctx.memory_id,
                                StoreKind::I32 { atomic: false },
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
                    | IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. }
                    | IntermediateType::IEnum { .. }
                    | IntermediateType::IGenericEnumInstance { .. } => {
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
/// This function is used with Unpack and UnpackGeneric bytecodes
pub fn unpack(
    struct_: &IStruct,
    itype: &IntermediateType,
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    // Pointer to the struct
    let pointer = module.locals.add(ValType::I32);

    builder.local_set(pointer);

    // Use the common field unpacking logic
    unpack_fields(
        &struct_.fields,
        builder,
        compilation_ctx,
        pointer,
        itype,
        types_stack,
    )?;

    Ok(())
}

/// Common logic for unpacking fields from memory
/// This function handles loading field values and managing the types stack
#[allow(clippy::too_many_arguments)]
pub fn unpack_fields(
    fields: &[IntermediateType],
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    pointer: walrus::LocalId,
    parent_type: &IntermediateType,
    types_stack: &mut TypesStack,
) -> Result<(), TranslationError> {
    let mut offset = 0;

    for field in fields {
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
                    field.load_kind()?,
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
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {}
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(TranslationError::FoundReferenceInsideStruct {
                    struct_index: 0, // This will be overridden by the caller
                });
            }
            IntermediateType::ITypeParameter(index) => {
                return Err(TranslationError::FoundTypeParameterInsideStruct {
                    struct_index: 0, // This will be overridden by the caller
                    type_parameter_index: *index,
                });
            }
        }

        // When unpacking a struct, at the moment of unpacking its UID or NamedId (if some found)
        // we also push to the types stack the wrapping struct information.
        //
        // The wrapping struct information is needed for some UID operations such as delete.
        match field {
            IntermediateType::IStruct {
                module_id, index, ..
            } if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
                let (instance_types, parent_module_id, parent_index) = match parent_type {
                    IntermediateType::IStruct {
                        module_id: parent_module_id,
                        index: parent_index,
                        ..
                    } => (None, parent_module_id.clone(), *parent_index),
                    IntermediateType::IGenericStructInstance {
                        module_id: parent_module_id,
                        index: parent_index,
                        types,
                        ..
                    } => (Some(types.clone()), parent_module_id.clone(), *parent_index),
                    _ => {
                        return Err(TranslationError::InvalidTypeInUnpackFunction(
                            parent_type.clone(),
                        ));
                    }
                };

                types_stack.push(IntermediateType::IStruct {
                    module_id: module_id.clone(),
                    index: *index,
                    vm_handled_struct: VmHandledStruct::StorageId {
                        parent_module_id,
                        parent_index,
                        instance_types,
                    },
                })
            }

            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
                let (instance_types, parent_module_id, parent_index) = match parent_type {
                    IntermediateType::IStruct {
                        module_id: parent_module_id,
                        index: parent_index,
                        ..
                    } => (None, parent_module_id.clone(), *parent_index),
                    IntermediateType::IGenericStructInstance {
                        module_id: parent_module_id,
                        index: parent_index,
                        types,
                        ..
                    } => (Some(types.clone()), parent_module_id.clone(), *parent_index),
                    _ => {
                        return Err(TranslationError::InvalidTypeInUnpackFunction(
                            parent_type.clone(),
                        ));
                    }
                };

                types_stack.push(IntermediateType::IGenericStructInstance {
                    module_id: module_id.clone(),
                    index: *index,
                    types: types.clone(),
                    vm_handled_struct: VmHandledStruct::StorageId {
                        parent_module_id,
                        parent_index,
                        instance_types,
                    },
                })
            }
            _ => types_stack.push(field.clone()),
        }

        offset += 4;
    }

    Ok(())
}
