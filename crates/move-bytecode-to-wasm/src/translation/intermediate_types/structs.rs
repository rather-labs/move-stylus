use std::collections::BTreeMap;

use super::IntermediateType;
use move_binary_format::file_format::{FieldHandleIndex, StructDefinitionIndex};

#[derive(Debug)]
pub struct IStruct {
    /// Field's types ordered by index
    pub fields: Vec<IntermediateType>,

    /// Map between handles and fields types
    pub fields_types: BTreeMap<FieldHandleIndex, IntermediateType>,

    /// Map between handles and fields offset
    pub field_offsets: BTreeMap<FieldHandleIndex, u32>,

    /// Move's struct index
    pub struct_definition_index: StructDefinitionIndex,

    /// How much memory this struct occupies (in bytes)
    pub heap_size: u32,
}

impl IStruct {
    pub fn new(
        index: StructDefinitionIndex,
        fields: Vec<(Option<FieldHandleIndex>, IntermediateType)>,
        fields_types: BTreeMap<FieldHandleIndex, IntermediateType>,
    ) -> Self {
        // To compute the heap size, we use the stack data size because for complex or heap types
        // we just save the pointer. In the case of simple types, the stack size matches the heap
        // size.
        let mut heap_size = 0;
        let mut field_offsets = BTreeMap::new();
        for (index, field) in fields.iter().rev() {
            if let Some(idx) = index {
                field_offsets.insert(*idx, heap_size);
            }
            heap_size += field.stack_data_size();
        }

        let fields = fields.into_iter().map(|(_, t)| t).collect();

        Self {
            struct_definition_index: index,
            heap_size,
            field_offsets,
            fields_types,
            fields,
        }
    }

    pub fn index(&self) -> u16 {
        self.struct_definition_index.0
    }
}
