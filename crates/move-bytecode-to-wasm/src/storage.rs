pub mod encoding;
pub mod read;

pub const READ_STRUCT_FROM_STORAGE_FN_NAME: &str = "read_struct_from_storage";

/// Macro used to obtain an IStruct for storage
#[macro_export]
macro_rules! get_struct {
    ($intermediate_type: ident, $compilation_ctx: ident) => {
        match $intermediate_type {
            $crate::translation::intermediate_types::IntermediateType::IStruct {
                module_id,
                index,
            } => $compilation_ctx
                .get_user_data_type_by_index(module_id, *index)
                .unwrap(),
            $crate::translation::intermediate_types::IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = $compilation_ctx
                    .get_user_data_type_by_index(module_id, *index)
                    .unwrap();
                &struct_.instantiate(types)
            }
            $crate::translation::intermediate_types::IntermediateType::IExternalUserData {
                module_id,
                identifier,
            } => {
                let external_data = $compilation_ctx
                    .get_external_module_data(module_id, identifier)
                    .unwrap();

                match external_data {
                    crate::compilation_context::ExternalModuleData::Struct(external_struct) => {
                        external_struct
                    }
                    crate::compilation_context::ExternalModuleData::Enum(_) => {
                        panic!("expected struct, found {:?}", $intermediate_type)
                    }
                }
            }
            _ => panic!("expected struct, found {:?}", $intermediate_type),
        }
    };
}

pub use get_struct;
