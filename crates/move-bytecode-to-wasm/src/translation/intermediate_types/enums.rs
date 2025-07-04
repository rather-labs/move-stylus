use crate::translation::TranslationError;

use super::{
    IntermediateType,
    address::IAddress,
    heap_integers::{IU128, IU256},
    signer::ISigner,
};

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
                    // For numbers and addresses, we directly save them
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32 => 4,
                    IntermediateType::IU64 => 8,
                    IntermediateType::IU128 => IU128::HEAP_SIZE,
                    IntermediateType::IU256 => IU256::HEAP_SIZE,
                    IntermediateType::IAddress => IAddress::HEAP_SIZE,
                    IntermediateType::ISigner => ISigner::HEAP_SIZE,

                    // For vectors and structs, we save a pointer to them
                    IntermediateType::IVector(_)
                    | IntermediateType::IStruct(_)
                    | IntermediateType::IGenericStructInstance(_, _) => 4,

                    IntermediateType::ITypeParameter(_) => return Ok(None),
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(TranslationError::FoundReferenceInsideEnum {
                            enum_index: variant.belongs_to,
                        });
                    }
                };

                size = std::cmp::max(size, variant_size);
            }
        }

        Ok(Some(size as u32 + 4))
    }
}
