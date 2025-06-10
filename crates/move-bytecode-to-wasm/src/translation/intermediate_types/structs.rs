use super::IntermediateType;
use move_binary_format::file_format::StructDefinitionIndex;

#[derive(Debug)]
pub struct IStruct {
    // Name that identifies the struct
    // name: String,
    /// Field's types ordered by index
    pub fields: Vec<IntermediateType>,

    /// Move's struct index
    pub struct_definition_index: StructDefinitionIndex,

    /// How much memory this struct occupies (in bytes)
    pub heap_size: u32,
}

impl IStruct {
    pub fn new(index: usize, fields: Vec<IntermediateType>) -> Self {
        Self {
            struct_definition_index: StructDefinitionIndex::new(index as u16),
            heap_size: Self::calculate_heap_size(&fields),
            fields,
        }
    }

    /// Calculates how much space we need to save the struct in heap.
    ///
    /// We use the stack data size because for complex or heap types we just save the pointer. In
    /// the case of simple types, the stack size matches the heap size.
    fn calculate_heap_size(fields: &[IntermediateType]) -> u32 {
        fields.iter().fold(0, |mut acc, f| {
            acc += f.stack_data_size();
            acc
        })
    }
}
