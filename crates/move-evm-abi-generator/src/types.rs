use std::{collections::HashMap, rc::Rc};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId,
    module_data::struct_data::IntermediateType,
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII,
        STDLIB_MODULE_NAME_STRING, STYLUS_FRAMEWORK_ADDRESS,
    },
};

use crate::common::snake_to_upper_camel;

const TYPES_WITH_NO_SIGNATURE: &[&str] = &["TxContext"];

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Type {
    Address,
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
    Unit,
    Array(Rc<Type>),
    Bytes32,
    String,
    Struct {
        identifier: String,
        type_instances: Option<Vec<Type>>,
        module_id: ModuleId,
    },
    Enum {
        identifier: String,
        module_id: ModuleId,
    },
    Tuple(Vec<Type>),
    // This type represents a type that appears in Move but not in the ABI signature
    None,
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Address => "address".to_owned(),
            Type::Bool => "bool".to_owned(),
            Type::Uint8 => "uint8".to_owned(),
            Type::Uint16 => "uint16".to_owned(),
            Type::Uint32 => "uint32".to_owned(),
            Type::Uint64 => "uint64".to_owned(),
            Type::Uint128 => "uint128".to_owned(),
            Type::Uint256 => "uint256".to_owned(),
            Type::Unit | Type::None => "".to_owned(),
            Type::Bytes32 => "bytes32".to_owned(),
            Type::String => "string".to_owned(),
            Type::Array(inner) => format!("{}[]", inner.name()),
            Type::Struct {
                identifier,
                type_instances,
                ..
            } => {
                if let Some(types) = type_instances {
                    let concrete_type_parameters_names = types
                        .iter()
                        .map(|t| t.name())
                        .collect::<Vec<String>>()
                        .join("_");

                    snake_to_upper_camel(&format!("{identifier}_{concrete_type_parameters_names}"))
                } else {
                    identifier.clone()
                }
            }
            Type::Enum { identifier, .. } => identifier.clone(),
            Type::Tuple(items) => {
                format!(
                    "({})",
                    items
                        .iter()
                        .map(|i| i.name())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }

    pub fn from_intermediate_type(
        itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> Self {
        match itype {
            IntermediateType::IBool => Self::Bool,
            IntermediateType::IU8 => Self::Uint8,
            IntermediateType::IU16 => Self::Uint16,
            IntermediateType::IU32 => Self::Uint32,
            IntermediateType::IU64 => Self::Uint64,
            IntermediateType::IU128 => Self::Uint128,
            IntermediateType::IU256 => Self::Uint256,
            IntermediateType::IAddress => Self::Address,
            IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => Self::None,
            IntermediateType::IVector(intermediate_type) => {
                let inner = Self::from_intermediate_type(intermediate_type, modules_data);
                Self::Array(Rc::new(inner))
            }
            IntermediateType::IRef(intermediate_type)
            | IntermediateType::IMutRef(intermediate_type) => {
                Self::from_intermediate_type(intermediate_type, modules_data)
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_module = modules_data
                    .get(module_id)
                    .expect("struct module not found");

                let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("UID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Self::Bytes32,
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_STRING) => Self::String,
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII) => Self::String,
                    _ => Self::Struct {
                        identifier: struct_.identifier.to_string(),
                        type_instances: None,
                        module_id: *module_id,
                    },
                }
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_module = modules_data
                    .get(module_id)
                    .expect("struct module not found");
                let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                let types = types
                    .iter()
                    .map(|t| Self::from_intermediate_type(t, modules_data))
                    .collect();

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("NamedId", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Self::Bytes32,
                    _ => Self::Struct {
                        identifier: struct_.identifier.to_string(),
                        type_instances: Some(types),
                        module_id: *module_id,
                    },
                }
            }
            IntermediateType::IEnum { module_id, index } => {
                let enum_module = modules_data.get(module_id).expect("enum module not found");
                let enum_ = enum_module.enums.get_by_index(*index).unwrap();
                if enum_.is_simple {
                    Type::Enum {
                        identifier: enum_.identifier.to_string(),
                        module_id: *module_id,
                    }
                } else {
                    Type::None
                }
            }
            IntermediateType::IGenericEnumInstance {
                module_id,
                index,
                types,
            } => {
                let enum_module = modules_data.get(module_id).expect("enum module not found");
                let enum_ = enum_module
                    .enums
                    .get_by_index(*index)
                    .unwrap()
                    .instantiate(types);

                if enum_.is_simple {
                    Type::Enum {
                        identifier: enum_.identifier.to_string(),
                        module_id: *module_id,
                    }
                } else {
                    Type::None
                }
            }
        }
    }
}

/// This function returns true if there is a type parameter in some of the intermediate types and
/// `false` otherwise.
pub fn type_contains_generics(itype: &IntermediateType) -> bool {
    match itype {
        IntermediateType::IRef(intermediate_type)
        | IntermediateType::IMutRef(intermediate_type) => {
            type_contains_generics(intermediate_type.as_ref())
        }
        IntermediateType::ITypeParameter(_) => true,
        IntermediateType::IGenericStructInstance { types, .. }
        | IntermediateType::IGenericEnumInstance { types, .. } => {
            types.iter().any(type_contains_generics)
        }
        IntermediateType::IVector(inner) => type_contains_generics(inner),
        _ => false,
    }
}
