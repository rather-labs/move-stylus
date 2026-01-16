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
    CompilationContext, data::RuntimeErrorData, generics::replace_type_parameters,
    storage::error::StorageError,
};

use super::{
    error::IntermediateTypeError,
    user_type_fields::{FieldsErrorContext, UserTypeFields},
};

use move_symbol_pool::Symbol;
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::IntermediateType;

#[derive(Debug, Clone)]
pub struct IEnumVariant {
    /// Index inside the enum
    pub index: u16,

    /// Index to the enum this variant belongs to
    pub belongs_to: u16,

    /// Variant's fields
    pub fields: Vec<IntermediateType>,
}

#[derive(Debug, Clone)]
pub struct IEnum {
    pub identifier: Symbol,

    pub index: u16,

    pub is_simple: bool,

    pub variants: Vec<IEnumVariant>,
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
    pub fn new(
        identifier: &str,
        index: u16,
        variants: Vec<IEnumVariant>,
    ) -> Result<Self, IntermediateTypeError> {
        let is_simple = variants.iter().all(|v| v.fields.is_empty());

        Ok(Self {
            identifier: Symbol::from(identifier),
            is_simple,
            variants,
            index,
        })
    }

    /// Computes the size of the enum.
    ///
    /// This will be 4 bytes for the current variant index plus the size of the variant that
    /// occupies most space in memory.
    ///
    /// This does not take in account how much space the actual data of the variant occupies because
    /// we can't know it (if the enum variant contains dynamic data such as vector, the size can
    /// change depending on how many elements the vector has), just the pointers to them.
    ///
    /// If the enum contains a variant with a generic field, we can't know the heap size, first it
    /// must be instantiated.
    pub fn heap_size(&self) -> Result<Option<u32>, IntermediateTypeError> {
        let mut size = 0;
        for variant in &self.variants {
            let mut variant_size = 0;
            for field in &variant.fields {
                variant_size += match field {
                    IntermediateType::ITypeParameter(_) => return Ok(None),
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(IntermediateTypeError::FoundReferenceInsideEnum(
                            self.index,
                            variant.belongs_to,
                        ));
                    }
                    _ => 4,
                };
            }
            size = std::cmp::max(size, variant_size);
        }

        Ok(Some(size + 4))
    }

    /// Compares two enums for equality
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
        runtime_error_data: &mut RuntimeErrorData,
    ) -> Result<(), IntermediateTypeError> {
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

        // Set the pointers past the variant index
        builder
            .local_get(e1_ptr)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_set(e1_ptr);

        builder
            .local_get(e2_ptr)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_set(e2_ptr);

        // Compare the variant indices
        builder.binop(BinaryOp::I32Eq);

        let mut inner_result = Ok(());
        builder.if_else(
            ValType::I32,
            |then| {
                let result = module.locals.add(ValType::I32);
                then.i32_const(0).local_set(result);

                // Copy fields for the active arm, then jump to join
                self.match_on_variant(then, variant_index, |variant, arm| {
                    // Use the same logic as structs to compare the fields
                    inner_result = UserTypeFields::compare_fields(
                        &variant.fields,
                        arm,
                        module,
                        compilation_ctx,
                        runtime_error_data,
                        e1_ptr,
                        e2_ptr,
                    );
                    arm.local_set(result);
                });
                then.local_get(result);
            },
            |else_| {
                else_.i32_const(0);
            },
        );

        inner_result?;

        Ok(())
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
        runtime_error_data: &mut RuntimeErrorData,
    ) -> Result<(), IntermediateTypeError> {
        let src_ptr = module.locals.add(ValType::I32);
        let ptr = module.locals.add(ValType::I32);

        builder.local_set(src_ptr);

        let heap_size = self
            .heap_size()?
            .ok_or(IntermediateTypeError::FoundTypeParameterInEnum)?;

        // Allocate space for the new enum
        builder
            .i32_const(heap_size as i32)
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

        // Copy fields for the active arm, then jump to join
        let mut inner_result = Ok(());
        self.match_on_variant(builder, variant_index, |variant, arm| {
            inner_result = UserTypeFields::copy_fields(
                &variant.fields,
                arm,
                module,
                compilation_ctx,
                runtime_error_data,
                src_ptr,
                ptr,
                4,
                FieldsErrorContext::Enum {
                    enum_index: self.index,
                    variant_index: variant.index,
                },
            );
        });
        inner_result?;

        builder.local_get(ptr);

        Ok(())
    }

    /// Replaces all type parameters in the enum with the provided types.
    /// It also computes the enum heap and storage size, if possible.
    /// If a type parameter is found, the sizes will be None.
    pub fn instantiate(&self, types: &[IntermediateType]) -> Self {
        let variants: Vec<IEnumVariant> = self
            .variants
            .iter()
            .map(|v| {
                IEnumVariant::new(
                    v.index,
                    v.belongs_to,
                    v.fields
                        .iter()
                        .map(|f| replace_type_parameters(f, types))
                        .collect(),
                )
            })
            .collect();

        Self {
            identifier: self.identifier,
            index: self.index,
            is_simple: self.is_simple,
            variants,
        }
    }

    /// Returns the storage sizes of the enum for each possible slot offset
    pub fn storage_size_by_offset(
        &self,
        compilation_ctx: &CompilationContext,
    ) -> Result<Vec<u32>, StorageError> {
        // Compute the storage size for each possible slot offset (0-31)
        (0..32)
            .map(|offset| {
                crate::storage::storage_layout::compute_enum_storage_size(
                    self,
                    offset,
                    compilation_ctx,
                )
            })
            .collect::<Result<Vec<u32>, StorageError>>()
    }

    /// Iterates all variants and runs `on_match` only for the active one
    /// (selected by `variant_index`). Each arm is wrapped in its own block
    /// with a guard (`br_if`) that skips non-matching arms.
    ///
    /// `on_match` receives:
    ///   - &IEnumVariant: the matched variant
    ///   - &mut InstrSeqBuilder: the builder for this arm
    ///   - InstrSeqId: a `join_id` you can `br` to when you’re done
    pub fn match_on_variant<F>(
        &self,
        builder: &mut InstrSeqBuilder,
        variant_index: LocalId,
        mut on_match: F,
    ) where
        F: FnMut(&IEnumVariant, &mut InstrSeqBuilder),
    {
        builder.block(None, |join| {
            let join_id = join.id();

            for variant in &self.variants {
                join.block(None, |case| {
                    let case_id = case.id();

                    // Guard: skip if runtime index != this variant
                    case.local_get(variant_index)
                        .i32_const(variant.index as i32)
                        .binop(BinaryOp::I32Ne)
                        .br_if(case_id);

                    // Matched: emit caller-provided body
                    on_match(variant, case);

                    case.br(join_id);
                });
            }

            // Should be unreachable if the enum is well-formed
            join.unreachable();
        });
    }
}
