use std::collections::HashMap;

use crate::translation::intermediate_types::{IntermediateType, structs::IStruct};
use move_binary_format::file_format::{
    Constant, DatatypeHandleIndex, FieldHandleIndex, FieldInstantiationIndex, Signature,
    SignatureToken, StructDefInstantiationIndex, StructDefinitionIndex,
};
use walrus::{FunctionId, MemoryId};

pub enum UserDefinedType {
    Struct(u16),
    Enum(usize),
}

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CompilationContextError {
    #[error("struct with index {0} not found in compilation context")]
    StructNotFound(u16),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithFieldIdxNotFound(FieldHandleIndex),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithDefinitionIdxNotFound(StructDefinitionIndex),

    #[error("struct with generic field instance id {0:?} not found in compilation context")]
    GenericStructWithFieldIdxNotFound(FieldInstantiationIndex),

    #[error("generic struct instance with field id {0:?} not found in compilation context")]
    GenericStructWithDefinitionIdxNotFound(StructDefInstantiationIndex),
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

    /// Module's generic structs instances: contains all the user defined generic structs instances
    /// with its corresponding types
    pub module_generic_structs_instances: &'a [(StructDefinitionIndex, Vec<SignatureToken>)],

    /// Maps a field index to its corresponding struct
    pub fields_to_struct_map: &'a HashMap<FieldHandleIndex, StructDefinitionIndex>,

    /// Maps a generic field index to its corresponding struct in module_generic_structs_instances
    pub generic_fields_to_struct_map: &'a HashMap<FieldInstantiationIndex, usize>,

    /// Maps a field instantiation index to its corresponding index inside the struct.
    /// field instantiation indexes are unique per struct instantiation, so, for example if we have
    /// the following struct:
    /// ```move
    /// struct S<T> {
    ///    x: T,
    /// }
    /// ```
    /// And we instantiate it with `S<u64>`, and `S<bool>`, the we will have a
    /// FieldInstantiationIndex(0) and a FieldInstantiationIndex(1) both for the `x` field, but the
    /// index inside the struct is 0 in both cases.
    pub instantiated_fields_to_generic_fields:
        &'a HashMap<FieldInstantiationIndex, FieldHandleIndex>,

    /// This Hashmap maps the move's datatype handles to our internal representation of those
    /// types. The datatype handles are used interally by move to look for user defined data
    /// types
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

    pub fn get_generic_struct_by_field_handle_idx(
        &self,
        field_index: &FieldInstantiationIndex,
    ) -> Result<IStruct, CompilationContextError> {
        let struct_id = self.generic_fields_to_struct_map.get(field_index).ok_or(
            CompilationContextError::GenericStructWithFieldIdxNotFound(*field_index),
        )?;

        let struct_instance = &self.module_generic_structs_instances[*struct_id];
        let generic_struct = &self.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, self.datatype_handles_map))
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<IStruct, CompilationContextError> {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];
        let generic_struct = &self.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, self.datatype_handles_map))
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_types_instances(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<Vec<IntermediateType>, CompilationContextError> {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, self.datatype_handles_map))
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(types)
    }

    pub fn get_generic_struct_idx_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> u16 {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];
        struct_instance.0.0
    }
}
