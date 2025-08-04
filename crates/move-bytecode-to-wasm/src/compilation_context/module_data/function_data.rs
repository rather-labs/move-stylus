use crate::translation::{
    functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionId,
};
use move_binary_format::file_format::FunctionInstantiationIndex;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct FunctionData {
    /// Module's functions arguments.
    pub arguments: Vec<Vec<IntermediateType>>,

    /// Module's functions Returns.
    pub returns: Vec<Vec<IntermediateType>>,

    /// Functions called inside this module. The functions on this list can be defined inside the
    /// current module or in an immediate dependency
    pub calls: Vec<FunctionId>,

    /// Function information about this module's defined functions
    pub information: Vec<MappedFunction>,

    /// Maps a function instantiation index to its corresponding function id
    pub generic_function_instance: HashMap<usize, FunctionId>,

    /// The init function of the module
    pub init: Option<FunctionId>,
}
