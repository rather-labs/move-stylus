use std::collections::HashMap;

use super::Result;
use move_binary_format::file_format::{
    EnumDefInstantiationIndex, EnumDefinitionIndex, VariantHandleIndex,
    VariantInstantiationHandleIndex,
};

use crate::{
    compilation_context::CompilationContextError,
    translation::intermediate_types::{IntermediateType, enums::IEnum},
};

#[derive(Debug)]
pub struct VariantData {
    pub enum_index: usize,
    pub index_inside_enum: usize,
}

#[derive(Debug)]
pub struct VariantInstantiationData {
    pub enum_index: usize,
    pub enum_def_instantiation_index: EnumDefInstantiationIndex,
    pub index_inside_enum: usize,
    pub types: Vec<IntermediateType>,
}
#[derive(Debug, Default)]
pub struct EnumData {
    /// Module's enums: contains all the user defined enums
    pub enums: Vec<IEnum>,

    /// Module's generic enums instances: contains all the user defined generic enums instances
    /// with its corresponding types
    pub generic_enum_instantiations: Vec<(EnumDefinitionIndex, Vec<IntermediateType>)>,

    /// Maps a enum's variant index to its corresponding enum and position inside the enum
    pub variants_to_enum: HashMap<VariantHandleIndex, VariantData>,

    /// Maps a enum's variant instantiation index to its corresponding enum and position inside the enum
    pub variants_instantiation_to_enum:
        HashMap<VariantInstantiationHandleIndex, VariantInstantiationData>,
}

impl EnumData {
    pub fn get_enum_by_variant_handle_idx(&self, idx: &VariantHandleIndex) -> Result<&IEnum> {
        let VariantData { enum_index, .. } = self
            .variants_to_enum
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        self.enums
            .get(*enum_index)
            .ok_or(CompilationContextError::EnumNotFound(*enum_index as u16))
    }

    pub fn get_enum_by_variant_instantiation_handle_idx(
        &self,
        idx: &VariantInstantiationHandleIndex,
    ) -> Result<&IEnum> {
        let VariantInstantiationData { enum_index, .. } = self
            .variants_instantiation_to_enum
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        self.enums
            .get(*enum_index)
            .ok_or(CompilationContextError::EnumNotFound(*enum_index as u16))
    }

    pub fn get_variant_position_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<u16> {
        let VariantData {
            index_inside_enum, ..
        } = self
            .variants_to_enum
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        Ok(*index_inside_enum as u16)
    }

    pub fn get_variant_position_by_variant_instantiation_handle_idx(
        &self,
        idx: &VariantInstantiationHandleIndex,
    ) -> Result<u16> {
        let VariantInstantiationData {
            index_inside_enum, ..
        } = self
            .variants_instantiation_to_enum
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;
        Ok(*index_inside_enum as u16)
    }

    pub fn get_enum_instance_by_variant_instantiation_handle_idx(
        &self,
        variant_instantiation_handle_index: &VariantInstantiationHandleIndex,
    ) -> Result<IEnum> {
        let VariantInstantiationData {
            enum_def_instantiation_index,
            ..
        } = self
            .variants_instantiation_to_enum
            .get(variant_instantiation_handle_index)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(
                variant_instantiation_handle_index.0,
            ))?;

        // We can index the generic_enums_intances vector with the enum_def_index because we created it in the same order as the enum_def_instantiations vector
        let (idx, types) =
            &self.generic_enum_instantiations[enum_def_instantiation_index.0 as usize];
        let generic_enum = &self.enums[idx.0 as usize];

        Ok(generic_enum.instantiate(types))
    }

    pub fn get_enum_instance_types(
        &self,
        variant_instantiation_handle_index: &VariantInstantiationHandleIndex,
    ) -> Result<&[IntermediateType]> {
        let VariantInstantiationData {
            enum_def_instantiation_index,
            ..
        } = self
            .variants_instantiation_to_enum
            .get(variant_instantiation_handle_index)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(
                variant_instantiation_handle_index.0,
            ))?;

        let (_, types) = &self.generic_enum_instantiations[enum_def_instantiation_index.0 as usize];
        Ok(types)
    }

    pub fn get_enum_by_index(&self, index: u16) -> Result<&IEnum> {
        self.enums
            .get(index as usize)
            .ok_or(CompilationContextError::EnumNotFound(index))
    }
}
