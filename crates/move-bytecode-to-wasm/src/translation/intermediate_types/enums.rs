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
use crate::translation::TranslationError;

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
                    _ => field.stack_data_size(),
                };

                size = std::cmp::max(size, variant_size);
            }
        }

        Ok(Some(size + 4))
    }
}
