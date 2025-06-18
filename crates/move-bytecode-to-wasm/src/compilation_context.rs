use std::collections::HashMap;

use crate::translation::intermediate_types::{IntermediateType, structs::IStruct};
use move_binary_format::file_format::{
    Constant, DatatypeHandleIndex, FieldHandleIndex, Signature, StructDefinitionIndex,
};
use walrus::{FunctionId, MemoryId};

pub enum UserDefinedType {
    Struct(u16),
    Enum(usize),
}

#[derive(Debug, thiserror::Error)]
pub enum CompilationContextError {
    #[error("struct with index {0} not found in compilation context")]
    StructNotFound(u16),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithFieldIdxNotFound(FieldHandleIndex),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithDefinitionIdxNotFound(StructDefinitionIndex),
}

/// Compilation context
///
/// Functions are processed in order. To access function information (i.e: arguments or return
/// arguments we must know the index of it)
pub struct CompilationContext<'a> {
    /// Move's connstant pool
    pub constants: &'a [Constant],

    /// Module's functions arguments.
    pub functions_arguments: &'a [Vec<IntermediateType>],

    /// Module's functions Returns.
    pub functions_returns: &'a [Vec<IntermediateType>],

    /// Module's signatures
    pub module_signatures: &'a [Signature],

    /// Module's structs: contains all the user defined structs
    pub module_structs: &'a [IStruct],

    /// Maps a field index to its corresponding struct
    pub fields_to_struct_map: &'a HashMap<FieldHandleIndex, StructDefinitionIndex>,

    // This Hashmap maps the move's datatype handles to our internal representation of those
    // types. The datatype handles are used interally by move to look for user defined data
    // types
    pub datatype_handles_map: &'a HashMap<DatatypeHandleIndex, UserDefinedType>,

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

impl CompilationContext<'_> {
    pub fn get_struct_by_index(&self, index: u16) -> Result<&IStruct, CompilationContextError> {
        self.module_structs
            .iter()
            .find(|s| s.index() == index)
            .ok_or(CompilationContextError::StructNotFound(index))
    }

    pub fn get_struct_by_field_handle_idx(
        &self,
        field_index: &FieldHandleIndex,
    ) -> Result<&IStruct, CompilationContextError> {
        let struct_id = self.fields_to_struct_map.get(field_index).ok_or(
            CompilationContextError::StructWithFieldIdxNotFound(*field_index),
        )?;

        self.module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_id)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))
    }

    pub fn get_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefinitionIndex,
    ) -> Result<&IStruct, CompilationContextError> {
        self.module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_index)
            .ok_or(CompilationContextError::StructWithDefinitionIdxNotFound(
                *struct_index,
            ))
    }
}
