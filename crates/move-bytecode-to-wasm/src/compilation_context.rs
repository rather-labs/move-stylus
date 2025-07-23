mod error;
pub mod module_data;

use crate::translation::intermediate_types::{enums::IEnum, structs::IStruct};
pub use error::CompilationContextError;
pub use module_data::{ModuleData, ModuleId, UserDefinedType};
use std::collections::HashMap;
use walrus::{FunctionId, MemoryId};

type Result<T> = std::result::Result<T, CompilationContextError>;

pub enum ExternalModuleData<'a> {
    Struct(&'a IStruct),
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
    pub fn get_external_module_data(
        &self,
        module_id: &ModuleId,
        identifier: &str,
    ) -> Result<ExternalModuleData> {
        let module = self.deps_data.get(module_id).ok_or(
            CompilationContextError::ExternalModuleNotFound(module_id.clone()),
        )?;

        if let Some(struct_) = module
            .structs
            .structs
            .iter()
            .find(|s| s.identifier == identifier)
        {
            Ok(ExternalModuleData::Struct(struct_))
        } else {
            todo!("enum case and empty case")
        }
    }
}
