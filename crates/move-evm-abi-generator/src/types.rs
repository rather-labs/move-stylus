// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

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
use crate::error::{AbiGeneratorError, AbiGeneratorErrorKind};

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
        has_key: bool,
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
    ) -> Result<Self, AbiGeneratorError> {
        match itype {
            IntermediateType::IBool => Ok(Self::Bool),
            IntermediateType::IU8 => Ok(Self::Uint8),
            IntermediateType::IU16 => Ok(Self::Uint16),
            IntermediateType::IU32 => Ok(Self::Uint32),
            IntermediateType::IU64 => Ok(Self::Uint64),
            IntermediateType::IU128 => Ok(Self::Uint128),
            IntermediateType::IU256 => Ok(Self::Uint256),
            IntermediateType::IAddress => Ok(Self::Address),
            IntermediateType::ISigner | IntermediateType::ITypeParameter(_) => Ok(Self::None),
            IntermediateType::IVector(intermediate_type) => {
                let inner = Self::from_intermediate_type(intermediate_type, modules_data)?;
                Ok(Self::Array(Rc::new(inner)))
            }
            IntermediateType::IRef(intermediate_type)
            | IntermediateType::IMutRef(intermediate_type) => {
                Self::from_intermediate_type(intermediate_type, modules_data)
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound,
                })?;

                let struct_ =
                    struct_module
                        .structs
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::StructNotFoundByIndex,
                        })?;

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("ID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Ok(Self::Bytes32),
                    ("UID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => Ok(Self::Bytes32),
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_STRING) => Ok(Self::String),
                    ("String", STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII) => Ok(Self::String),
                    (identifier, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_SOL_TYPES) => {
                        if let Some(bytes_num) = identifier.strip_prefix("Bytes") {
                            match bytes_num {
                                "1" => Ok(Self::Bytes1),
                                "2" => Ok(Self::Bytes2),
                                "3" => Ok(Self::Bytes3),
                                "4" => Ok(Self::Bytes4),
                                "5" => Ok(Self::Bytes5),
                                "6" => Ok(Self::Bytes6),
                                "7" => Ok(Self::Bytes7),
                                "8" => Ok(Self::Bytes8),
                                "9" => Ok(Self::Bytes9),
                                "10" => Ok(Self::Bytes10),
                                "11" => Ok(Self::Bytes11),
                                "12" => Ok(Self::Bytes12),
                                "13" => Ok(Self::Bytes13),
                                "14" => Ok(Self::Bytes14),
                                "15" => Ok(Self::Bytes15),
                                "16" => Ok(Self::Bytes16),
                                "17" => Ok(Self::Bytes17),
                                "18" => Ok(Self::Bytes18),
                                "19" => Ok(Self::Bytes19),
                                "20" => Ok(Self::Bytes20),
                                "21" => Ok(Self::Bytes21),
                                "22" => Ok(Self::Bytes22),
                                "23" => Ok(Self::Bytes23),
                                "24" => Ok(Self::Bytes24),
                                "25" => Ok(Self::Bytes25),
                                "26" => Ok(Self::Bytes26),
                                "27" => Ok(Self::Bytes27),
                                "28" => Ok(Self::Bytes28),
                                "29" => Ok(Self::Bytes29),
                                "30" => Ok(Self::Bytes30),
                                "31" => Ok(Self::Bytes31),
                                "32" => Ok(Self::Bytes32),
                                _ => Err(AbiGeneratorError {
                                    kind: AbiGeneratorErrorKind::UnknownBytesNType(
                                        bytes_num.to_string(),
                                    ),
                                }),
                            }
                        } else {
                            Err(AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::UnknownSolTypesStruct(
                                    identifier.to_string(),
                                ),
                            })
                        }
                    }
                    _ => Ok(Self::Struct {
                        identifier: struct_.identifier,
                        type_instances: None,
                        module_id: *module_id,
                        has_key: struct_.has_key,
                    }),
                }
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound,
                })?;
                let struct_ =
                    struct_module
                        .structs
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::StructNotFoundByIndex,
                        })?;
                let types = types
                    .iter()
                    .map(|t| Self::from_intermediate_type(t, modules_data))
                    .collect::<Result<Vec<_>, _>>()?;

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("NamedId", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {
                        Ok(Self::Bytes32)
                    }
                    _ => Ok(Self::Struct {
                        identifier: struct_.identifier,
                        type_instances: Some(types),
                        module_id: *module_id,
                        has_key: struct_.has_key,
                    }),
                }
            }
            IntermediateType::IEnum { module_id, index } => {
                let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound,
                })?;
                let enum_ =
                    enum_module
                        .enums
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::EnumNotFoundByIndex,
                        })?;
                if enum_.is_simple {
                    Ok(Type::Enum {
                        identifier: enum_.identifier,
                        module_id: *module_id,
                    })
                } else {
                    Ok(Type::None)
                }
            }
            IntermediateType::IGenericEnumInstance {
                module_id,
                index,
                types,
            } => {
                let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound,
                })?;
                let enum_ = enum_module
                    .enums
                    .get_by_index(*index)
                    .map_err(|_| AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::EnumNotFoundByIndex,
                    })?
                    .instantiate(types);

                if enum_.is_simple {
                    Ok(Type::Enum {
                        identifier: enum_.identifier,
                        module_id: *module_id,
                    })
                } else {
                    Ok(Type::None)
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
