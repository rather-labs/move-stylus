use crate::{
    GlobalFunctionTable,
    translation::{
        functions::MappedFunction,
        intermediate_types::{
            IntermediateType,
            enums::{IEnum, IEnumVariant},
            structs::IStruct,
        },
        table::FunctionId,
    },
};
use move_binary_format::{
    CompiledModule,
    file_format::{
        Constant, DatatypeHandleIndex, EnumDefinitionIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinitionIndex, Signature, SignatureIndex,
        SignatureToken, StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
    },
    internals::ModuleIndex,
};
use std::{collections::HashMap, fmt::Display};

use super::{CompilationContextError, Result};

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
