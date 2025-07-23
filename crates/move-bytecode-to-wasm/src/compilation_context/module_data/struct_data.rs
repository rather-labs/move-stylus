pub use crate::translation::intermediate_types::{IntermediateType, structs::IStruct};
use move_binary_format::{
    file_format::{
        FieldHandleIndex, FieldInstantiationIndex, StructDefInstantiationIndex,
        StructDefinitionIndex,
    },
    internals::ModuleIndex,
};
use std::collections::HashMap;

use super::{CompilationContextError, Result};

#[derive(Debug, Default)]
pub struct StructData {
    /// Module's structs: contains all the user defined structs
    pub(crate) structs: Vec<IStruct>,

    /// Module's generic structs instances: contains all the user defined generic structs instances
    /// with its corresponding types
    pub(crate) generic_structs_instances: Vec<(StructDefinitionIndex, Vec<IntermediateType>)>,

    /// Maps a field index to its corresponding struct
    pub(crate) fields_to_struct: HashMap<FieldHandleIndex, StructDefinitionIndex>,

    /// Maps a generic field index to its corresponding struct in module_generic_structs_instances
    pub(crate) generic_fields_to_struct: HashMap<FieldInstantiationIndex, usize>,

    /// Maps a field instantiation index to its corresponding index inside the struct.
    /// Field instantiation indexes are unique per struct instantiation, so, for example if we have
    /// the following struct:
    /// ```move
    /// struct S<T> {
    ///    x: T,
    /// }
    /// ```
    /// And we instantiate it with `S<u64>`, and `S<bool>`, the we will have a
    /// FieldInstantiationIndex(0) and a FieldInstantiationIndex(1) both for the `x` field, but the
    /// index inside the struct is 0 in both cases.
    ///
    /// We also map the concrete types of the instantiated generic struct where this field
    /// instantiuation belongs to. This is needed because there are situations where we need to
    /// intantiate the struct only with the field instantiation index and no other information.
    pub(crate) instantiated_fields_to_generic_fields:
        HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<IntermediateType>)>,
}

impl StructData {
    pub fn get_by_index(&self, index: u16) -> Result<&IStruct> {
        self.structs
            .iter()
            .find(|s| s.index() == index)
            .ok_or(CompilationContextError::StructNotFound(index))
    }

    pub fn get_by_struct_definition_idx(
        &self,
        struct_index: &StructDefinitionIndex,
    ) -> Result<&IStruct> {
        self.structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_index)
            .ok_or(CompilationContextError::StructWithDefinitionIdxNotFound(
                *struct_index,
            ))
    }

    pub fn get_by_field_handle_idx(&self, field_index: &FieldHandleIndex) -> Result<&IStruct> {
        let struct_id = self.fields_to_struct.get(field_index).ok_or(
            CompilationContextError::StructWithFieldIdxNotFound(*field_index),
        )?;

        self.structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_id)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))
    }

    pub fn get_struct_instance_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<IStruct> {
        let (idx, concrete_types) = &self.generic_structs_instances[struct_index.into_index()];
        let generic_struct = &self.structs[idx.into_index()];

        Ok(generic_struct.instantiate(concrete_types))
    }

    pub fn get_struct_instance_by_field_handle_idx(
        &self,
        field_index: &FieldInstantiationIndex,
    ) -> Result<IStruct> {
        let struct_id = self.generic_fields_to_struct.get(field_index).ok_or(
            CompilationContextError::GenericStructWithFieldIdxNotFound(*field_index),
        )?;

        let (idx, types) = &self.generic_structs_instances[*struct_id];
        let generic_struct = &self.structs[idx.into_index()];

        Ok(generic_struct.instantiate(types))
    }

    pub fn get_generic_struct_types_instances(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<&[IntermediateType]> {
        let (_, types) = &self.generic_structs_instances[struct_index.into_index()];

        Ok(types)
    }

    pub fn get_generic_struct_idx_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> u16 {
        let struct_instance = &self.generic_structs_instances[struct_index.0 as usize];
        struct_instance.0.0
    }
}
