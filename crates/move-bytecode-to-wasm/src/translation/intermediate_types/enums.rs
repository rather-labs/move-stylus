//! Represents an enum type in Move
//!
//! The struct memory layout is composed of two parts:
//! - The first 4 bytes are the enum variant value.
//! - The rest bytes can vary depending on which variant is currently saved in memory. After the
//!   first 4 bytes the vairant's fields will be encoded contigously.
//!
//! The size that the enum will occupy in memory depends on its variants. This will be 4 bytes for
//! the variant index plus the size of the variant that occupies most space in memory.
//!
//! If a variant contains a dynamic type, it does not take in account how much space the actual data of
//! the variant occupies because we can't know it (such as vector, the size can change depending on how
//! many elements the vector has), in that case we save just the pointers to them.
//!
//! For stack types the data is saved in-place, for heap-types we just save the pointer to the
//! data.
use crate::{
    CompilationContext, compilation_context::module_data::ModuleData, translation::TranslationError,
};

use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::IntermediateType;

#[derive(Debug)]
pub struct IEnumVariant {
    /// Index inside the enum
    pub index: u16,

    /// Index to the enum this variant belongs to
    pub belongs_to: u16,

    /// Variant's fields
    pub fields: Vec<IntermediateType>,
}

#[derive(Debug)]
pub struct IEnum {
    pub index: u16,

    pub is_simple: bool,

    pub variants: Vec<IEnumVariant>,

    /// How much memory occupies (in bytes).
    ///
    /// This will be 4 bytes for the variant index plus the size of the variant that occupies most
    /// space in memory.
    ///
    /// This does not take in account how much space the actual data of the variant occupies because
    /// we can't know it (if the enum variant contains dynamic data such as vector, the size can
    /// change depending on how many elements the vector has), just the pointers to them.
    ///
    /// If the enum contains a variant with a generic field, we can't know the heap size, first it
    /// must be instantiated.
    pub heap_size: Option<u32>,
}

impl IEnumVariant {
    pub fn new(index: u16, belongs_to: u16, fields: Vec<IntermediateType>) -> Self {
        Self {
            index,
            belongs_to,
            fields,
        }
    }
}

impl IEnum {
    pub fn new(index: u16, variants: Vec<IEnumVariant>) -> Result<Self, TranslationError> {
        let is_simple = variants.iter().all(|v| v.fields.is_empty());
        let heap_size = Self::compute_heap_size(&variants)?;
        Ok(Self {
            is_simple,
            variants,
            index,
            heap_size,
        })
    }

    /// Computes the size of the enum.
    ///
    /// This will be 4 bytes for the current variant index plus the size of the variant that
    /// occupies most space in memory.
    ///
    /// If the enum contains a variant with a generic type parameter, returns None, since we can't
    /// know it.
    fn compute_heap_size(variants: &[IEnumVariant]) -> Result<Option<u32>, TranslationError> {
        let mut size = 0;
        for variant in variants {
            let mut variant_size = 0;
            for field in &variant.fields {
                variant_size += match field {
                    IntermediateType::ITypeParameter(_) => return Ok(None),
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(TranslationError::FoundReferenceInsideEnum {
                            enum_index: variant.belongs_to,
                        });
                    }
                    _ => 4,
                };
            }
            size = std::cmp::max(size, variant_size);
        }

        Ok(Some(size + 4))
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
        let src_ptr = module.locals.add(ValType::I32);
        let ptr = module.locals.add(ValType::I32);

        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let ptr_to_data = module.locals.add(ValType::I32);

        builder.local_set(src_ptr);

        // Allocate space for the new enum
        builder
            .i32_const(self.heap_size.unwrap() as i32)
            .call(compilation_ctx.allocator)
            .local_set(ptr);

        // Read the variant index
        let variant_index = module.locals.add(ValType::I32);
        builder
            .local_get(src_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(variant_index);

        // Write the variant index to the new enum memory
        builder.local_get(ptr).local_get(variant_index).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Find the corresponding IEnumVariant for the variant index local
        for variant in self.variants.iter() {
            builder.block(None, |block| {
                let block_id = block.id();
                // If the variant index does not match, branch to the end of the block
                block
                    .local_get(variant_index)
                    .i32_const(variant.index as i32)
                    .binop(BinaryOp::I32Ne)
                    .br_if(block_id);

                // Set offset past the index
                let mut offset: u32 = 4;
                // Iterate over the fields and copy them
                for field in &variant.fields {
                    match field {
                        IntermediateType::IBool
                        | IntermediateType::IU8
                        | IntermediateType::IU16
                        | IntermediateType::IU32
                        | IntermediateType::IU64 => {
                            let data_size = field.stack_data_size();
                            let (val, load_kind, store_kind) = if data_size == 8 {
                                (
                                    val_64,
                                    LoadKind::I64 { atomic: false },
                                    StoreKind::I64 { atomic: false },
                                )
                            } else {
                                (
                                    val_32,
                                    LoadKind::I32 { atomic: false },
                                    StoreKind::I32 { atomic: false },
                                )
                            };

                            // Load intermediate pointer and value
                            block.local_get(src_ptr).load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset,
                                },
                            ).load(
                                compilation_ctx.memory_id,
                                load_kind,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            ).local_set(val);

                            // Create a pointer for the value
                            block
                                .i32_const(data_size as i32)
                                .call(compilation_ctx.allocator)
                                .local_tee(ptr_to_data);

                            // Store the actual value behind the middle_ptr
                            block.local_get(val).store(
                                compilation_ctx.memory_id,
                                store_kind,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            );
                        }
                        IntermediateType::IStruct { .. }
                        | IntermediateType::IGenericStructInstance { .. }
                        | IntermediateType::IAddress
                        | IntermediateType::ISigner
                        | IntermediateType::IU128
                        | IntermediateType::IU256
                        | IntermediateType::IVector(_)
                        | IntermediateType::IEnum(_) => {
                            // Load intermediate pointer
                            block
                                .local_get(src_ptr)
                                .i32_const(offset as i32)
                                .binop(BinaryOp::I32Add)
                                .local_set(ptr_to_data);

                            field.copy_local_instructions(
                                module,
                                block,
                                compilation_ctx,
                                module_data,
                                ptr_to_data,
                            );

                            block.local_set(ptr_to_data);
                        }
                        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                            panic!("references inside enums not allowed")
                        }
                        IntermediateType::ITypeParameter(_) => {
                            panic!(
                                "Trying to copy a type parameter inside an enum, expected a concrete type"
                            );
                        }
                    }
                    // Store the middle pointer at ptr + offset
                    block.local_get(ptr).local_get(ptr_to_data).store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg { align: 0, offset },
                    );
                    offset += 4;
                }
            });
        }

        builder.local_get(ptr);
    }

    pub fn equality(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
        let e1_ptr = module.locals.add(ValType::I32);
        let e2_ptr = module.locals.add(ValType::I32);
        builder.local_set(e1_ptr).local_set(e2_ptr);

        let result = module.locals.add(ValType::I32);
        builder.i32_const(1).local_set(result);

        let variant_index = module.locals.add(ValType::I32);

        // Read the variant index from the first enum
        builder
            .local_get(e1_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_tee(variant_index);

        // Read the variant index from the second enum
        builder.local_get(e2_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Compare the variant indices
        builder.binop(BinaryOp::I32Eq);

        builder.if_else(
            None,
            |then| {
                // Proceed to compare the fields of the variant
                // Find the corresponding IEnumVariant for the variant index local
                for variant in self.variants.iter() {
                    then.block(None, |block| {
                        let block_id = block.id();
                        // If the variant index does not match, branch to the end of the block
                        block
                            .local_get(variant_index)
                            .i32_const(variant.index as i32)
                            .binop(BinaryOp::I32Ne)
                            .br_if(block_id);

                        let load_value_to_stack =
                            |field: &IntermediateType, builder: &mut InstrSeqBuilder<'_>| {
                                if field.stack_data_size() == 8 {
                                    builder.load(
                                        compilation_ctx.memory_id,
                                        LoadKind::I64 { atomic: false },
                                        MemArg {
                                            align: 0,
                                            offset: 0,
                                        },
                                    );
                                } else {
                                    builder.load(
                                        compilation_ctx.memory_id,
                                        LoadKind::I32 { atomic: false },
                                        MemArg {
                                            align: 0,
                                            offset: 0,
                                        },
                                    );
                                }
                            };

                        // Set offset past the index
                        let mut offset: u32 = 4;
                        // Iterate over the fields and compare them
                        for field in &variant.fields {
                            // Load the first enum field value
                            block.local_get(e1_ptr).load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg { align: 0, offset },
                            );

                            if field.is_stack_type() {
                                load_value_to_stack(field, block);
                            }

                            // Load the second enum field value
                            block.local_get(e2_ptr).load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg { align: 0, offset },
                            );

                            if field.is_stack_type() {
                                load_value_to_stack(field, block);
                            }

                            // Compare the field values
                            field.load_equality_instructions(
                                module,
                                block,
                                compilation_ctx,
                                module_data,
                            );
                            block.if_else(
                                None,
                                |_| {},
                                |else_| {
                                    else_.i32_const(0).local_set(result).br(block_id);
                                },
                            );
                            offset += 4;
                        }
                        block.i32_const(1).local_set(result);
                    });
                }
            },
            |else_| {
                else_.i32_const(0).local_set(result);
            },
        );

        builder.local_get(result);
    }
}
