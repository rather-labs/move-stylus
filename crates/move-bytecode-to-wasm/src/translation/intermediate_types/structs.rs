use move_binary_format::file_format::{
    DatatypeHandleIndex, FieldDefinition, StructFieldInformation,
};

use super::IntermediateType;

#[derive(Debug)]
pub struct IStruct {
    // Name that identifies the struct
    // name: String,
    /// Field's types ordered by index
    pub fields: Vec<IntermediateType>,

    /// Move's handle index
    pub datatype_handle_index: DatatypeHandleIndex,
}

impl IStruct {
    pub fn new(datatype_handle_index: DatatypeHandleIndex, fields: Vec<IntermediateType>) -> Self {
        Self {
            datatype_handle_index,
            fields,
        }
    }
}
