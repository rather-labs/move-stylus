use std::collections::HashMap;

use move_binary_format::file_format::FunctionDefinition;

use crate::{
    compilation_context::CompilationContextError,
    translation::{
        functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionId,
    },
};

use super::error::ModuleDataError;

#[derive(Debug, Default)]
pub struct FunctionData<'move_compiled_unit> {
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

    /// The receive function of the module.
    pub receive: Option<FunctionId>,

    /// The fallback function of the module.
    pub fallback: Option<FunctionId>,

    /// Function definition from Move bytecode.
    pub move_definitions: HashMap<FunctionId, &'move_compiled_unit FunctionDefinition>,
}

impl FunctionData<'_> {
    pub fn get_information_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<&MappedFunction, CompilationContextError> {
        Ok(self
            .information
            .iter()
            .find(|f| *f.function_id.identifier == *identifier)
            .ok_or(ModuleDataError::FunctionByIdentifierNotFound(
                identifier.to_string(),
            ))?)
    }

    pub fn get_move_definition_by_id(
        &self,
        function_id: &FunctionId,
    ) -> Result<&FunctionDefinition, CompilationContextError> {
        Ok(self.move_definitions.get(function_id).ok_or(
            ModuleDataError::FunctionDefinitionByIdNotFound(function_id.clone()),
        )?)
    }
}
