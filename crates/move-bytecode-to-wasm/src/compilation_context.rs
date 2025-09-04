mod error;
pub mod module_data;
pub mod reserved_modules;

use crate::translation::intermediate_types::{IntermediateType, structs::IStruct};
pub use error::CompilationContextError;
pub use module_data::{ModuleData, ModuleId, UserDefinedType};
use std::{borrow::Cow, collections::HashMap};
use walrus::{FunctionId, MemoryId};

type Result<T> = std::result::Result<T, CompilationContextError>;

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

    pub fn get_module_data_by_id(&self, module_id: &ModuleId) -> Result<&ModuleData> {
        if let Some(m) = self.deps_data.get(module_id) {
            Ok(m)
        } else if &self.root_module_data.id == module_id {
            Ok(self.root_module_data)
        } else {
            Err(CompilationContextError::ModuleNotFound(module_id.clone()))
        }
    }

    /// Looks for a struct with index `index` within the module with id `module_id`
    pub fn get_struct_by_index(&self, module_id: &ModuleId, index: u16) -> Result<&IStruct> {
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
                let struct_ = self.get_struct_by_index(module_id, *index)?;
                Ok(Cow::Borrowed(struct_))
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = self.get_struct_by_index(module_id, *index)?;
                let instance = struct_.instantiate(types);
                Ok(Cow::Owned(instance))
            }
            _ => Err(CompilationContextError::ExpectedStruct),
        }
    }
}
