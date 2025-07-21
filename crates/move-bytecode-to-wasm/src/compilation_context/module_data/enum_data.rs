use std::collections::HashMap;

use super::Result;
use move_binary_format::file_format::VariantHandleIndex;

use crate::{
    compilation_context::CompilationContextError, translation::intermediate_types::enums::IEnum,
};

#[derive(Debug)]
pub struct VariantData {
    pub enum_index: usize,
    pub index_inside_enum: usize,
}

#[derive(Debug, Default)]
pub struct EnumData {
    /// Module's enums: contains all the user defined enums
    pub enums: Vec<IEnum>,

    /// Maps a enum's variant index to its corresponding enum and position inside the enum
    pub variants_to_enum: HashMap<VariantHandleIndex, VariantData>,
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

    pub fn get_enum_by_index(&self, index: u16) -> Result<&IEnum> {
        self.enums
            .get(index as usize)
            .ok_or(CompilationContextError::EnumNotFound(index))
    }
}
