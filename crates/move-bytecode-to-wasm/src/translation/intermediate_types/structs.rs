use std::collections::BTreeMap;

use super::IntermediateType;
use move_binary_format::file_format::{FieldHandleIndex, StructDefinitionIndex};

#[derive(Debug)]
pub struct IStruct {
    // Name that identifies the struct
    // name: String,
    /// Field's types ordered by index
    pub fields: BTreeMap<FieldHandleIndex, IntermediateType>,

    /// Move's struct index
    pub struct_definition_index: StructDefinitionIndex,

    /// How much memory this struct occupies (in bytes)
    pub heap_size: u32,
}

impl IStruct {
    pub fn new(
        index: StructDefinitionIndex,
        fields: BTreeMap<FieldHandleIndex, IntermediateType>,
    ) -> Self {
        let heap_size = Self::calculate_heap_size(fields.values());
        Self {
            struct_definition_index: index,
            heap_size,
            fields,
        }
    }

    /// Calculates how much space we need to save the struct in heap.
    ///
    /// We use the stack data size because for complex or heap types we just save the pointer. In
    /// the case of simple types, the stack size matches the heap size.
    fn calculate_heap_size<'a>(fields: impl Iterator<Item = &'a IntermediateType>) -> u32 {
        fields.fold(0, |mut acc, f| {
            acc += f.stack_data_size();
            acc
        })
    }
}
