use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    translation::{
        TranslationError,
        intermediate_types::{IntermediateType, enums::IEnum},
        types_stack::TypesStack,
    },
};

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

    let heap_size = enum_
        .heap_size
        .ok_or(TranslationError::PackingGenericEnumVariant {
            enum_index: enum_.index,
        })?;

    builder
        .i32_const(heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    // Save the variant index
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

    let mut offset = heap_size;

    for pack_type in enum_.variants[variant_index as usize].fields.iter().rev() {
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

                        // Store the actual value behind the middle_ptr
                        builder.local_get(pointer).local_get(val).store(
                            compilation_ctx.memory_id,
                            store_kind,
                            MemArg { align: 0, offset },
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
                    | IntermediateType::IGenericStructInstance { .. } => {
                        builder.local_set(ptr_to_data);

                        // Directly write the pointer to the data
                        builder.local_get(pointer).local_get(ptr_to_data).store(
                            compilation_ctx.memory_id,
                            StoreKind::I32 { atomic: false },
                            MemArg { align: 0, offset },
                        );
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
                    IntermediateType::IEnum(_) => todo!(),
                };
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
