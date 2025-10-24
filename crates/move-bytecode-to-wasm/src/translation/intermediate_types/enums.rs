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

use super::structs::IStruct;

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

    /// Compares two enums for equality
    /// Ar
    /// # Arguments
    ///    - pointer to the first enum
    ///    - pointer to the second enum
    /// # Returns
    ///    - true if the enums are equal, false otherwise
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
            ValType::I32,
            |then| {
                let result = module.locals.add(ValType::I32);
                then.i32_const(0).local_set(result);
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

                        block
                            .local_get(e1_ptr)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(e1_ptr);
                        block
                            .local_get(e2_ptr)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(e2_ptr);

                        // Use the same logic as structs to compare the fields
                        IStruct::compare_fields(
                            &variant.fields,
                            block,
                            module,
                            compilation_ctx,
                            module_data,
                            e1_ptr,
                            e2_ptr,
                        );
                        block.local_set(result);
                    });
                }
                then.local_get(result);
            },
            |else_| {
                else_.i32_const(0);
            },
        );
    }

    /// Copies the local instructions for an enum
    ///
    /// # Arguments
    ///    - pointer to the source enum
    /// # Returns
    ///    - pointer to the new enum
    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
        let src_ptr = module.locals.add(ValType::I32);
        let ptr = module.locals.add(ValType::I32);

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

                // Use the common field copying logic
                IStruct::copy_fields(
                    &variant.fields,
                    block,
                    module,
                    compilation_ctx,
                    module_data,
                    src_ptr,
                    ptr,
                    4, // start_offset for enums is 4 because we already wrote the variant index
                );
            });
        }

        builder.local_get(ptr);
    }
}
