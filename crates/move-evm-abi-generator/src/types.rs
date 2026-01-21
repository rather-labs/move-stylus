use std::{collections::HashMap, rc::Rc};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId,
    module_data::struct_data::IntermediateType,
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, SF_MODULE_NAME_SOL_TYPES, STANDARD_LIB_ADDRESS,
        STDLIB_MODULE_NAME_ASCII, STDLIB_MODULE_NAME_STRING, STYLUS_FRAMEWORK_ADDRESS,
    },
};
use move_symbol_pool::Symbol;

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
    Bytes1,
    Bytes2,
    Bytes3,
    Bytes4,
    Bytes5,
    Bytes6,
    Bytes7,
    Bytes8,
    Bytes9,
    Bytes10,
    Bytes11,
    Bytes12,
    Bytes13,
    Bytes14,
    Bytes15,
    Bytes16,
    Bytes17,
    Bytes18,
    Bytes19,
    Bytes20,
    Bytes21,
    Bytes22,
    Bytes23,
    Bytes24,
    Bytes25,
    Bytes26,
    Bytes27,
    Bytes28,
    Bytes29,
    Bytes30,
    Bytes31,
    Bytes32,
    String,
    Struct {
        identifier: Symbol,
        type_instances: Option<Vec<Type>>,
        module_id: ModuleId,
    },
    Enum {
        identifier: Symbol,
        module_id: ModuleId,
    },
    Tuple(Vec<Type>),
    // This type represents a type that appears in Move but not in the ABI signature
    None,
}

impl Type {
    pub fn name(&self) -> Symbol {
        match self {
            Type::Address => Symbol::from("address"),
            Type::Bool => Symbol::from("bool"),
            Type::Uint8 => Symbol::from("uint8"),
            Type::Uint16 => Symbol::from("uint16"),
            Type::Uint32 => Symbol::from("uint32"),
            Type::Uint64 => Symbol::from("uint64"),
            Type::Uint128 => Symbol::from("uint128"),
            Type::Uint256 => Symbol::from("uint256"),
            Type::Unit | Type::None => Symbol::from(""),
            Type::Bytes1 => Symbol::from("bytes1"),
            Type::Bytes2 => Symbol::from("bytes2"),
            Type::Bytes3 => Symbol::from("bytes3"),
            Type::Bytes4 => Symbol::from("bytes4"),
            Type::Bytes5 => Symbol::from("bytes5"),
            Type::Bytes6 => Symbol::from("bytes6"),
            Type::Bytes7 => Symbol::from("bytes7"),
            Type::Bytes8 => Symbol::from("bytes8"),
            Type::Bytes9 => Symbol::from("bytes9"),
            Type::Bytes10 => Symbol::from("bytes10"),
            Type::Bytes11 => Symbol::from("bytes11"),
            Type::Bytes12 => Symbol::from("bytes12"),
            Type::Bytes13 => Symbol::from("bytes13"),
            Type::Bytes14 => Symbol::from("bytes14"),
            Type::Bytes15 => Symbol::from("bytes15"),
            Type::Bytes16 => Symbol::from("bytes16"),
            Type::Bytes17 => Symbol::from("bytes17"),
            Type::Bytes18 => Symbol::from("bytes18"),
            Type::Bytes19 => Symbol::from("bytes19"),
            Type::Bytes20 => Symbol::from("bytes20"),
            Type::Bytes21 => Symbol::from("bytes21"),
            Type::Bytes22 => Symbol::from("bytes22"),
            Type::Bytes23 => Symbol::from("bytes23"),
            Type::Bytes24 => Symbol::from("bytes24"),
            Type::Bytes25 => Symbol::from("bytes25"),
            Type::Bytes26 => Symbol::from("bytes26"),
            Type::Bytes27 => Symbol::from("bytes27"),
            Type::Bytes28 => Symbol::from("bytes28"),
            Type::Bytes29 => Symbol::from("bytes29"),
            Type::Bytes30 => Symbol::from("bytes30"),
            Type::Bytes31 => Symbol::from("bytes31"),
            Type::Bytes32 => Symbol::from("bytes32"),
            Type::String => Symbol::from("string"),
            Type::Array(inner) => Symbol::from(format!("{}[]", inner.name())),
            Type::Struct {
                identifier,
                type_instances,
                ..
            } => {
                if let Some(types) = type_instances {
                    let concrete_type_parameters_names = types
                        .iter()
                        .map(|t| t.name().to_string())
                        .collect::<Vec<String>>()
                        .join("_");

                    Symbol::from(snake_to_upper_camel(&format!(
                        "{identifier}_{concrete_type_parameters_names}"
                    )))
                } else {
                    *identifier
                }
            }
            Type::Enum { identifier, .. } => *identifier,
            Type::Tuple(items) => Symbol::from(format!(
                "({})",
                items
                    .iter()
                    .map(|i| i.name().to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
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
                    ("ID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Self::Bytes32,
                    ("UID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Self::Bytes32,
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_STRING) => Self::String,
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII) => Self::String,
                    (identifier, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_SOL_TYPES) => {
                        if let Some(bytes_num) = identifier.strip_prefix("Bytes") {
                            match bytes_num {
                                "1" => Self::Bytes1,
                                "2" => Self::Bytes2,
                                "3" => Self::Bytes3,
                                "4" => Self::Bytes4,
                                "5" => Self::Bytes5,
                                "6" => Self::Bytes6,
                                "7" => Self::Bytes7,
                                "8" => Self::Bytes8,
                                "9" => Self::Bytes9,
                                "10" => Self::Bytes10,
                                "11" => Self::Bytes11,
                                "12" => Self::Bytes12,
                                "13" => Self::Bytes13,
                                "14" => Self::Bytes14,
                                "15" => Self::Bytes15,
                                "16" => Self::Bytes16,
                                "17" => Self::Bytes17,
                                "18" => Self::Bytes18,
                                "19" => Self::Bytes19,
                                "20" => Self::Bytes20,
                                "21" => Self::Bytes21,
                                "22" => Self::Bytes22,
                                "23" => Self::Bytes23,
                                "24" => Self::Bytes24,
                                "25" => Self::Bytes25,
                                "26" => Self::Bytes26,
                                "27" => Self::Bytes27,
                                "28" => Self::Bytes28,
                                "29" => Self::Bytes29,
                                "30" => Self::Bytes30,
                                "31" => Self::Bytes31,
                                "32" => Self::Bytes32,
                                _ => panic!("unknown BytesN type"),
                            }
                        } else {
                            panic!("unknown sol types struct")
                        }
                    }
                    _ => Self::Struct {
                        identifier: struct_.identifier,
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
                        identifier: struct_.identifier,
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
                        identifier: enum_.identifier,
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
                        identifier: enum_.identifier,
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
