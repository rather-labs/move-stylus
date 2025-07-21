use crate::translation::{
    functions::MappedFunction, intermediate_types::IntermediateType, table::FunctionId,
};

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
}
