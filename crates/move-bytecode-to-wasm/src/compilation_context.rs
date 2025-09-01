mod error;
pub mod module_data;
pub mod reserved_modules;

use crate::translation::intermediate_types::{IntermediateType, enums::IEnum, structs::IStruct};
pub use error::CompilationContextError;
pub use module_data::{ModuleData, ModuleId, UserDefinedType};
use std::{borrow::Cow, collections::HashMap};
use walrus::{FunctionId, MemoryId};

type Result<T> = std::result::Result<T, CompilationContextError>;

pub enum ExternalModuleData<'a> {
    Struct(Cow<'a, IStruct>),
    Enum(&'a IEnum),
}

/// Compilation context
///
/// Functions are processed in order. To access function information (i.e: arguments or return
/// arguments we must know the index of it)
pub struct CompilationContext<'a> {
    /// Data of the module we are currently compiling
    pub root_module_data: &'a ModuleData,

    pub deps_data: &'a HashMap<ModuleId, ModuleData>,

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

impl CompilationContext<'_> {
    /// Creates a new compilation context
    pub fn new<'a>(
        root_module_data: &'a ModuleData,
        deps_data: &'a HashMap<ModuleId, ModuleData>,
        memory_id: MemoryId,
        allocator: FunctionId,
    ) -> CompilationContext<'a> {
        CompilationContext::<'a> {
            root_module_data,
            deps_data,
            memory_id,
            allocator,
        }
    }

    pub fn get_external_module_data(
        &self,
        module_id: &ModuleId,
        identifier: &str,
        types: &Option<Vec<IntermediateType>>,
    ) -> Result<ExternalModuleData> {
        let module = self
            .deps_data
            .get(module_id)
            .ok_or(CompilationContextError::ModuleNotFound(module_id.clone()))?;

        if let Some(struct_) = module
            .structs
            .structs
            .iter()
            .find(|s| s.identifier == identifier)
        {
            if let Some(instantiation_types) = types {
                let struct_instance = struct_.instantiate(instantiation_types);
                Ok(ExternalModuleData::Struct(Cow::Owned(struct_instance)))
            } else {
                Ok(ExternalModuleData::Struct(Cow::Borrowed(struct_)))
            }
        } else {
            todo!("enum case and empty case")
        }
    }

    pub fn get_module_data_by_id(&self, module_id: &ModuleId) -> Result<&ModuleData> {
        if let Some(m) = self.deps_data.get(module_id) {
            Ok(m)
        } else if &self.root_module_data.id == module_id {
            Ok(self.root_module_data)
        } else {
            Err(CompilationContextError::ModuleNotFound(module_id.clone()))
        }
    }

    pub fn get_user_data_type_by_index(
        &self,
        module_id: &ModuleId,
        index: u16,
    ) -> Result<&IStruct> {
        let module = self
            .deps_data
            .get(module_id)
            .unwrap_or(self.root_module_data);

        module.structs.get_by_index(index)
    }

    /// This function tries to get an struct from the `IntermediateType` enum. In the named enum we
    /// can have three variants of the struct:
    ///
    /// - IStruct: a concrete struct defined in the root module or immediate dependency.
    /// - IGenericStructInstance: a generic struct insantiation defined in the root module or immediate
    ///   dependency.
    /// - ExternalModuleData: can contain either an enum or a struct defined in a dependency.
    ///
    /// The information to reconstruct the `IStruct` object is in different places within the
    /// compilation contect. With this macro we can easily avoid all the boilerplate and obtain
    /// a reference to the `IStruct` directly.
    pub fn get_struct_by_intermediate_type(
        &self,
        itype: &IntermediateType,
    ) -> Result<Cow<IStruct>> {
        match itype {
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = self.get_user_data_type_by_index(module_id, *index)?;
                Ok(Cow::Borrowed(struct_))
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = self.get_user_data_type_by_index(module_id, *index)?;
                let instance = struct_.instantiate(types);
                Ok(Cow::Owned(instance))
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                types,
            } => {
                let external_data = self.get_external_module_data(module_id, identifier, types)?;

                match external_data {
                    ExternalModuleData::Struct(external_struct) => Ok(external_struct),
                    ExternalModuleData::Enum(_) => Err(CompilationContextError::ExpectedStruct),
                }
            }
            _ => Err(CompilationContextError::ExpectedStruct),
        }
    }
}
