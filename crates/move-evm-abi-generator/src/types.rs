use std::{collections::HashMap, rc::Rc};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId, module_data::struct_data::IntermediateType,
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
    UserDefined(String, Option<Vec<Type>>),
    Tuple(Vec<Type>),
    // This type represents a type that appears in Move but not in the ABI signature
    None,
}

impl From<&move_parse_special_attributes::types::Type> for Type {
    fn from(value: &move_parse_special_attributes::types::Type) -> Self {
        match value {
            move_parse_special_attributes::types::Type::Address => Self::Address,
            move_parse_special_attributes::types::Type::Bool => Self::Bool,

            move_parse_special_attributes::types::Type::UserDataType(name, None)
                if TYPES_WITH_NO_SIGNATURE.contains(&name.as_str()) =>
            {
                Self::None
            }
            move_parse_special_attributes::types::Type::UserDataType(name, None)
                if name == "UID" =>
            {
                Self::Bytes32
            }
            move_parse_special_attributes::types::Type::UserDataType(name, types) => {
                println!("----------> {types:?}");
                Self::UserDefined(
                    name.clone(),
                    types.as_ref().map(|t| t.iter().map(Self::from).collect()),
                )
            }
            move_parse_special_attributes::types::Type::Signer => Self::Address, // TODO: This is
            // not correct
            move_parse_special_attributes::types::Type::Vector(t) => {
                Self::Array(Rc::new(Self::from(t.as_ref())))
            }
            move_parse_special_attributes::types::Type::U8 => Self::Uint8,
            move_parse_special_attributes::types::Type::U16 => Self::Uint16,
            move_parse_special_attributes::types::Type::U32 => Self::Uint32,
            move_parse_special_attributes::types::Type::U64 => Self::Uint64,
            move_parse_special_attributes::types::Type::U128 => Self::Uint128,
            move_parse_special_attributes::types::Type::U256 => Self::Uint256,
            move_parse_special_attributes::types::Type::Unit => Self::Unit,
            move_parse_special_attributes::types::Type::Tuple(items) => {
                Self::Tuple(items.iter().map(Self::from).collect())
            }
            move_parse_special_attributes::types::Type::Function(_, _) => Self::None,
        }
    }
}

impl From<move_parse_special_attributes::types::Type> for Type {
    fn from(value: move_parse_special_attributes::types::Type) -> Self {
        Self::from(&value)
    }
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
            Type::Array(inner) => format!("{}[]", inner.name()),
            Type::Bytes32 => "bytes32".to_owned(),
            Type::UserDefined(name, None) if TYPES_WITH_NO_SIGNATURE.contains(&name.as_str()) => {
                "".to_owned()
            }
            Type::UserDefined(name, types) => {
                if let Some(types) = types {
                    println!("----------> 2 {types:?}");
                    let concrete_type_parameters_names = types
                        .iter()
                        .map(|t| t.name())
                        .collect::<Vec<String>>()
                        .join("_");

                    snake_to_upper_camel(&format!("{}_{}", name, concrete_type_parameters_names))
                } else {
                    name.clone()
                }
            }
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
            IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => {
                panic!("Should never happen")
            }
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
                Self::UserDefined(struct_.identifier.clone(), None)
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
                Self::UserDefined(struct_.identifier.clone(), Some(types))
            }
            IntermediateType::IEnum { module_id, index } => todo!(),
            IntermediateType::IGenericEnumInstance {
                module_id,
                index,
                types,
            } => todo!(),
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
