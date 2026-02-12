// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use crate::abi::{Abi, FunctionType};
use crate::common::snake_to_upper_camel;
use crate::error::{AbiGeneratorError, AbiGeneratorErrorKind};
use crate::types::Type;
use move_parse_special_attributes::function_modifiers::SolidityFunctionModifier;
use move_symbol_pool::Symbol;
use serde::Serialize;

const EMPTY_STR: &str = "";
#[derive(Serialize)]
struct JsonAbi {
    abi: Vec<JsonAbiItem>,
}

// Todo: is it possible to get the FunctionType from within the Function variant to serialize the type as a string?
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum AbiItemType {
    Event,
    Error,
    Function(FunctionType),
}

#[derive(Serialize)]
#[serde(untagged)]
enum JsonAbiItem {
    Event {
        #[serde(rename = "type")]
        type_: AbiItemType, // Event
        name: Symbol,
        inputs: Vec<JsonIO>,
        anonymous: bool,
    },

    Error {
        #[serde(rename = "type")]
        type_: AbiItemType, // Error
        name: Symbol,
        inputs: Vec<JsonIO>,
    },

    // Unified Function-like variant
    #[serde(rename_all = "camelCase")]
    Function {
        #[serde(rename = "type")]
        type_: FunctionType,

        // For normal functions and constructors
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<Symbol>,

        #[serde(skip_serializing_if = "Option::is_none")]
        inputs: Option<Vec<JsonIO>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        outputs: Option<Vec<JsonIO>>,

        state_mutability: &'static str,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonIO {
    name: Symbol,
    #[serde(rename = "type")]
    type_: Symbol, // "uint256", "tuple", "tuple[]", "tuple[3]", ...
    internal_type: Symbol, // "uint256", "tuple", "tuple[]", "tuple[3]", ...
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed: Option<bool>, // present for event top-level inputs only
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>, // present iff type is tuple/tuple[]/tuple[k]
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonComponent {
    name: Symbol,
    #[serde(rename = "type")]
    type_: Symbol,
    internal_type: Symbol,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>,
}

pub fn process_abi(abi: &Abi) -> Result<String, AbiGeneratorError> {
    // Collect all the JSON ABI items into a single vector
    let abi_json_items = process_events(abi)?
        .into_iter()
        .chain(process_errors(abi)?)
        .chain(process_functions(abi)?)
        .collect();

    let json_abi = JsonAbi {
        abi: abi_json_items,
    };

    serde_json::to_string_pretty(&json_abi).map_err(|e| AbiGeneratorError {
        kind: AbiGeneratorErrorKind::SerializationError(e),
    })
}

fn process_errors(abi: &Abi) -> Result<Vec<JsonAbiItem>, AbiGeneratorError> {
    let mut errors: Vec<JsonAbiItem> = abi
        .abi_errors
        .iter()
        .map(|error| {
            let mut inputs = vec![];
            for field in &error.fields {
                process_io(
                    field.type_.clone(),
                    if error.positional_fields {
                        EMPTY_STR
                    } else {
                        &field.identifier
                    },
                    None,
                    &mut inputs,
                    abi,
                )?;
            }

            Ok(JsonAbiItem::Error {
                type_: AbiItemType::Error,
                name: error.identifier,
                inputs,
            })
        })
        .collect::<Result<Vec<_>, AbiGeneratorError>>()?;

    // Sort errors by name for deterministic output
    errors.sort_by_key(|item| match item {
        JsonAbiItem::Error { name, .. } => *name,
        _ => Symbol::from(""),
    });

    Ok(errors)
}

fn process_events(abi: &Abi) -> Result<Vec<JsonAbiItem>, AbiGeneratorError> {
    let mut events: Vec<JsonAbiItem> = abi
        .events
        .iter()
        .map(|event| {
            let mut inputs = vec![];
            for field in &event.fields {
                process_io(
                    field.named_type.type_.clone(),
                    if event.positional_fields {
                        EMPTY_STR
                    } else {
                        &field.named_type.identifier
                    },
                    Some(field.indexed),
                    &mut inputs,
                    abi,
                )?;
            }

            Ok(JsonAbiItem::Event {
                type_: AbiItemType::Event,
                name: event.identifier,
                inputs,
                anonymous: event.is_anonymous,
            })
        })
        .collect::<Result<Vec<_>, AbiGeneratorError>>()?;

    // Sort events by name and field signatures (name + internal_type) for deterministic output
    // This handles event overloading (same name, different fields)
    events.sort_by_key(|item| match item {
        JsonAbiItem::Event { name, inputs, .. } => {
            let field_sigs: Vec<String> = inputs
                .iter()
                .map(|input| format!("{}{}", input.name, input.internal_type))
                .collect();
            format!("{}{}", name, field_sigs.join(""))
        }
        _ => String::new(),
    });

    Ok(events)
}

fn process_functions(abi: &Abi) -> Result<Vec<JsonAbiItem>, AbiGeneratorError> {
    let mut functions: Vec<JsonAbiItem> = abi
        .functions
        .iter()
        .map(|f| {
            let (name, inputs, outputs) = match f.function_type {
                // Fallback and Receive have no name, inputs, or outputs
                FunctionType::Fallback | FunctionType::Receive => (None, None, None),
                // Constructor has no name, but has inputs
                FunctionType::Constructor => {
                    let mut inputs = vec![];
                    for param in &f.parameters {
                        process_io(
                            param.type_.clone(),
                            param.identifier,
                            None,
                            &mut inputs,
                            abi,
                        )?;
                    }
                    (None, Some(inputs), None)
                }
                FunctionType::Function => {
                    // Handle normal functions
                    let mut inputs = vec![];
                    for param in &f.parameters {
                        process_io(
                            param.type_.clone(),
                            param.identifier,
                            None,
                            &mut inputs,
                            abi,
                        )?;
                    }

                    let mut outputs = vec![];
                    match &f.return_types {
                        Type::Tuple(types_) => {
                            // For tuples, we iterate over the elements and collect them in a vector of JsonIOs
                            for t in types_ {
                                process_io(t.clone(), EMPTY_STR, None, &mut outputs, abi)?;
                            }
                        }
                        _ => {
                            process_io(f.return_types.clone(), EMPTY_STR, None, &mut outputs, abi)?;
                        }
                    };

                    (Some(f.identifier), Some(inputs), Some(outputs))
                }
            };

            let state_mutability = map_state_mutability(&f.modifiers);

            Ok(JsonAbiItem::Function {
                type_: f.function_type,
                name,
                inputs,
                outputs,
                state_mutability,
            })
        })
        .collect::<Result<Vec<_>, AbiGeneratorError>>()?;

    // Sort functions: special functions first (Constructor, Receive, Fallback), then regular functions by name
    functions.sort_by_key(|item| {
        match item {
            JsonAbiItem::Function { type_, name, .. } => {
                let priority = match type_ {
                    FunctionType::Constructor => 0,
                    FunctionType::Receive => 1,
                    FunctionType::Fallback => 2,
                    FunctionType::Function => 3,
                };
                // For regular functions, use the name; for special functions, use empty string
                (priority, *name)
            }
            _ => (4, None),
        }
    });

    Ok(functions)
}

fn map_state_mutability(mods: &[SolidityFunctionModifier]) -> &'static str {
    if mods.contains(&SolidityFunctionModifier::Pure) {
        SolidityFunctionModifier::Pure.as_str()
    } else if mods.contains(&SolidityFunctionModifier::View) {
        SolidityFunctionModifier::View.as_str()
    } else if mods.contains(&SolidityFunctionModifier::Payable) {
        SolidityFunctionModifier::Payable.as_str()
    } else {
        "nonpayable"
    }
}

/// Processes an IO (input/output) parameter and adds it to the given vector if the type is not empty.
fn process_io(
    type_: Type,
    name: impl Into<Symbol>,
    indexed: Option<bool>,
    io: &mut Vec<JsonIO>,
    abi: &Abi,
) -> Result<(), AbiGeneratorError> {
    if type_ != Type::None {
        let JsonAbiData {
            abi_type,
            abi_internal_type,
            components,
        } = encode_for_json_abi(type_.clone(), abi)?;

        io.push(JsonIO {
            name: name.into(),
            type_: abi_type,
            internal_type: abi_internal_type,
            indexed,
            components,
        });
    }
    Ok(())
}

// A struct containing the ABI type, ABI internal type, and components.
struct JsonAbiData {
    abi_type: Symbol,
    abi_internal_type: Symbol,
    components: Option<Vec<JsonComponent>>,
}

/// Encodes a Type into the JSON ABI format.
///
/// Returns a JsonAbiData struct containing the ABI type, ABI internal type, and components.
///
/// Recursively processes nested types (arrays, struct fields) to build the complete ABI representation.
fn encode_for_json_abi(type_: Type, abi: &Abi) -> Result<JsonAbiData, AbiGeneratorError> {
    match &type_ {
        Type::Address
        | Type::Bool
        | Type::Uint8
        | Type::Uint16
        | Type::Uint32
        | Type::Uint64
        | Type::Uint128
        | Type::Uint256
        | Type::Unit
        | Type::Bytes1
        | Type::Bytes2
        | Type::Bytes3
        | Type::Bytes4
        | Type::Bytes5
        | Type::Bytes6
        | Type::Bytes7
        | Type::Bytes8
        | Type::Bytes9
        | Type::Bytes10
        | Type::Bytes11
        | Type::Bytes12
        | Type::Bytes13
        | Type::Bytes14
        | Type::Bytes15
        | Type::Bytes16
        | Type::Bytes17
        | Type::Bytes18
        | Type::Bytes19
        | Type::Bytes20
        | Type::Bytes21
        | Type::Bytes22
        | Type::Bytes23
        | Type::Bytes24
        | Type::Bytes25
        | Type::Bytes26
        | Type::Bytes27
        | Type::Bytes28
        | Type::Bytes29
        | Type::Bytes30
        | Type::Bytes31
        | Type::Bytes32
        | Type::String => Ok(JsonAbiData {
            abi_type: type_.name(),
            abi_internal_type: type_.name(),
            components: None,
        }),
        Type::Enum {
            identifier,
            module_id,
        } => {
            let abi_type = Symbol::from("uint8");
            let abi_internal_type = Symbol::from(format!(
                "enum {}.{}",
                snake_to_upper_camel(&module_id.module_name),
                identifier
            ));
            Ok(JsonAbiData {
                abi_type,
                abi_internal_type,
                components: None,
            })
        }
        Type::Array(inner) => {
            let JsonAbiData {
                abi_type,
                abi_internal_type,
                components,
            } = encode_for_json_abi((**inner).clone(), abi)?;

            Ok(JsonAbiData {
                abi_type: Symbol::from(format!("{abi_type}[]")),
                abi_internal_type: Symbol::from(format!("{abi_internal_type}[]")),
                components,
            })
        }
        Type::Struct {
            module_id, has_key, ..
        } => {
            let abi_internal_type = Symbol::from(format!(
                "struct {}.{}",
                snake_to_upper_camel(&module_id.module_name),
                type_.name()
            ));
            if *has_key {
                // Struct with key: encode as bytes32 with struct internalType
                let abi_type = Symbol::from("bytes32");
                Ok(JsonAbiData {
                    abi_type,
                    abi_internal_type,
                    components: None,
                })
            } else {
                // Regular struct: encode as tuple with components
                // Find corresponding processed struct, searching by the name, which differs from the identifier in case of generic structs
                let abi_struct = abi
                    .structs
                    .iter()
                    .find(|s| s.identifier == type_.name())
                    .ok_or(AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::AbiStructNotFound(type_.name()),
                    })?;

                let components = abi_struct
                    .fields
                    .iter()
                    .map(|named_type| {
                        let JsonAbiData {
                            abi_type,
                            abi_internal_type,
                            components,
                        } = encode_for_json_abi(named_type.type_.clone(), abi)?;

                        Ok(JsonComponent {
                            name: named_type.identifier,
                            type_: abi_type,
                            internal_type: abi_internal_type,
                            components,
                        })
                    })
                    .collect::<Result<Vec<_>, AbiGeneratorError>>()?;

                let abi_type = Symbol::from("tuple");
                Ok(JsonAbiData {
                    abi_type,
                    abi_internal_type,
                    components: Some(components),
                })
            }
        }
        Type::Tuple(_) => Err(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::TupleInJsonAbi,
        }),
        Type::None => Err(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::NoneTypeInJsonAbi,
        }),
    }
}
