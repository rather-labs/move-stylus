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
    pub is_simple: bool,

    pub variants: Vec<IEnumVariant>,

    pub index: u16,
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
    pub fn new(index: u16, variants: Vec<IEnumVariant>) -> Self {
        let is_simple = variants.iter().all(|v| v.fields.is_empty());
        Self {
            is_simple,
            variants,
            index,
        }
    }
}
