//! This field contains the logic for common fields operations used in structs and enums variants
//! with fields.
//!
//! The memory representation is the same in both cases.

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, compilation_context::ModuleData};

use super::{IntermediateType, error::IntermediateTypeError};

/// Context for error reporting when computing storage size.
/// Determines which error variant to construct when encountering references or type parameters.
pub enum FieldsErrorContext {
    Struct { struct_index: u16 },
    Enum { enum_index: u16, variant_index: u16 },
}

pub struct UserTypeFields;

impl UserTypeFields {
    /// Common logic for copying fields from source to destination
    /// This function handles copying field values using the appropriate copy instructions
    #[allow(clippy::too_many_arguments)]
    pub fn copy_fields(
        fields: &[IntermediateType],
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
        src_ptr: LocalId,
        dst_ptr: LocalId,
        start_offset: u32,
        error_context: FieldsErrorContext,
    ) -> Result<(), IntermediateTypeError> {
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let ptr_to_data = module.locals.add(ValType::I32);

        let mut offset = start_offset;
        for field in fields {
            match field {
                // Stack values: create a middle pointer to save the actual value
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64 => {
                    let data_size = field.wasm_memory_data_size()?;
                    let val = if data_size == 8 { val_64 } else { val_32 };
                    let load_kind = field.load_kind()?;
                    let store_kind = field.store_kind()?;

                    // Load intermediate pointer and value
                    builder
                        .local_get(src_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg { align: 0, offset },
                        )
                        .load(
                            compilation_ctx.memory_id,
                            load_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(val);

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
                IntermediateType::IU128
                | IntermediateType::IU256
                | IntermediateType::IAddress
                | IntermediateType::ISigner
                | IntermediateType::IVector(_)
                | IntermediateType::IStruct { .. }
                | IntermediateType::IGenericStructInstance { .. }
                | IntermediateType::IEnum { .. }
                | IntermediateType::IGenericEnumInstance { .. } => {
                    // Load intermediate pointer
                    builder
                        .local_get(src_ptr)
                        .i32_const(offset as i32)
                        .binop(BinaryOp::I32Add)
                        .local_set(ptr_to_data);

                    field.copy_local_instructions(
                        module,
                        builder,
                        compilation_ctx,
                        module_data,
                        ptr_to_data,
                    )?;

                    builder.local_set(ptr_to_data);
                }
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => match error_context {
                    FieldsErrorContext::Struct { struct_index } => {
                        return Err(IntermediateTypeError::FoundReferenceInsideStruct(
                            struct_index,
                        ));
                    }
                    FieldsErrorContext::Enum {
                        enum_index,
                        variant_index,
                    } => {
                        return Err(IntermediateTypeError::FoundReferenceInsideEnum(
                            enum_index,
                            variant_index,
                        ));
                    }
                },
                IntermediateType::ITypeParameter(_) => {
                    return Err(IntermediateTypeError::FoundTypeParameter);
                }
            }

            // Store the middle pointer in the place of the object field
            builder.local_get(dst_ptr).local_get(ptr_to_data).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg { align: 0, offset },
            );

            offset += 4;
        }

        Ok(())
    }

    /// Common logic for comparing fields of two objects (structs or enum variants)
    /// This function handles loading field values and comparing them using the appropriate equality instructions
    /// Returns the result of the comparison as a boolean
    pub fn compare_fields(
        fields: &[IntermediateType],
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
        ptr_1: LocalId,
        ptr_2: LocalId,
    ) -> Result<(), IntermediateTypeError> {
        let result = module.locals.add(ValType::I32);
        builder.i32_const(1).local_set(result);

        let load_value_to_stack = |field: &IntermediateType, builder: &mut InstrSeqBuilder<'_>| {
            match field.load_kind() {
                Ok(load_kind) => {
                    builder.load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
                Err(e) => return Err(e),
            }

            Ok(())
        };

        let mut inner_result: Result<(), IntermediateTypeError> = Ok(());
        builder.block(None, |block| {
            let block_id = block.id();
            let mut offset = 0;
            for field in fields.iter() {
                // Load the first struct field value
                inner_result = (|| {
                    block.local_get(ptr_1).load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg { align: 0, offset },
                    );

                    if field.is_stack_type()? {
                        load_value_to_stack(field, block)?;
                    }

                    // Load the second struct field value
                    block.local_get(ptr_2).load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg { align: 0, offset },
                    );

                    if field.is_stack_type()? {
                        load_value_to_stack(field, block)?;
                    }

                    // Compare the field values
                    field.load_equality_instructions(
                        module,
                        block,
                        compilation_ctx,
                        module_data,
                    )?;

                    block.if_else(
                        None,
                        |_| {},
                        |else_| {
                            else_.i32_const(0).local_set(result).br(block_id);
                        },
                    );

                    offset += 4;

                    Ok(())
                })();
            }
        });

        inner_result?;
        builder.local_get(result);

        Ok(())
    }
}
