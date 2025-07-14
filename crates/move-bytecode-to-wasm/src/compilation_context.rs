mod error;
pub mod module_data;

use crate::translation::intermediate_types::{IntermediateType, enums::IEnum, structs::IStruct};
pub use error::CompilationContextError;
pub use module_data::{ModuleData, ModuleId, UserDefinedType, VariantData};
use move_binary_format::{
    file_format::{
        FieldHandleIndex, FieldInstantiationIndex, SignatureIndex, SignatureToken,
        StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
    },
    internals::ModuleIndex,
};
use std::collections::HashMap;
use walrus::{FunctionId, MemoryId};

/// Compilation context
///
/// Functions are processed in order. To access function information (i.e: arguments or return
/// arguments we must know the index of it)
pub struct CompilationContext {
    /// Data of the module we are currently compiling
    pub root_module_data: ModuleData,

    pub deps_data: HashMap<ModuleId, ModuleData>,

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

impl CompilationContext {
    pub fn get_struct_by_index(&self, index: u16) -> Result<&IStruct, CompilationContextError> {
        self.root_module_data
            .module_structs
            .iter()
            .find(|s| s.index() == index)
            .ok_or(CompilationContextError::StructNotFound(index))
    }

    pub fn get_struct_by_field_handle_idx(
        &self,
        field_index: &FieldHandleIndex,
    ) -> Result<&IStruct, CompilationContextError> {
        let struct_id = self
            .root_module_data
            .fields_to_struct_map
            .get(field_index)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))?;

        self.root_module_data
            .module_structs
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
        self.root_module_data
            .module_structs
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
        let struct_id = self
            .root_module_data
            .generic_fields_to_struct_map
            .get(field_index)
            .ok_or(CompilationContextError::GenericStructWithFieldIdxNotFound(
                *field_index,
            ))?;

        let struct_instance = &self.root_module_data.module_generic_structs_instances[*struct_id];
        let generic_struct = &self.root_module_data.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    &self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<IStruct, CompilationContextError> {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];
        let generic_struct = &self.root_module_data.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    &self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_types_instances(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<Vec<IntermediateType>, CompilationContextError> {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    &self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(types)
    }

    pub fn get_generic_struct_idx_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> u16 {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];
        struct_instance.0.0
    }

    pub fn get_signatures_by_index(
        &self,
        index: SignatureIndex,
    ) -> Result<&Vec<SignatureToken>, CompilationContextError> {
        self.root_module_data
            .module_signatures
            .get(index.into_index())
            .map(|s| &s.0)
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }

    pub fn get_enum_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<&IEnum, CompilationContextError> {
        let VariantData { enum_index, .. } = self
            .root_module_data
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        self.root_module_data
            .module_enums
            .get(*enum_index)
            .ok_or(CompilationContextError::EnumNotFound(*enum_index as u16))
    }

    pub fn get_variant_position_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<u16, CompilationContextError> {
        let VariantData {
            index_inside_enum, ..
        } = self
            .root_module_data
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        Ok(*index_inside_enum as u16)
    }

    pub fn get_enum_by_index(&self, index: u16) -> Result<&IEnum, CompilationContextError> {
        self.root_module_data
            .module_enums
            .get(index as usize)
            .ok_or(CompilationContextError::EnumNotFound(index))
    }
}
