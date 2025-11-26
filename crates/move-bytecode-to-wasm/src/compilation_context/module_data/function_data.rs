use crate::{
    compilation_context::CompilationContextError,
    translation::{
        functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionId,
    },
};

use super::error::ModuleDataError;

#[derive(Debug, Default)]
pub struct FunctionData {
    /// Module's functions arguments.
    pub arguments: Vec<Vec<IntermediateType>>,

    /// Module's functions Returns.
    pub returns: Vec<Vec<IntermediateType>>,

    /// Functions called inside this module. The functions on this list can be defined inside the
    /// current module or in an immediate dependency.
    pub calls: Vec<FunctionId>,

    /// Generic function calls. They can be from this module or from an immediate dependency.
    pub generic_calls: Vec<FunctionId>,

    /// Function information about this module's defined functions.
    pub information: Vec<MappedFunction>,

    /// The init function of the module.
    pub init: Option<FunctionId>,
}

impl FunctionData {
    pub fn get_information_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<&MappedFunction, CompilationContextError> {
        Ok(self
            .information
            .iter()
            .find(|f| f.function_id.identifier == identifier)
            .ok_or(ModuleDataError::FunctionByIdentifierNotFound(
                identifier.to_string(),
            ))?)
    }
}
