use crate::abi::{Abi, FunctionType};
use crate::common::snake_to_upper_camel;
use crate::types::Type;
use move_parse_special_attributes::function_modifiers::FunctionModifier;
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
        name: String,
        inputs: Vec<JsonIO>,
        anonymous: bool,
    },

    Error {
        #[serde(rename = "type")]
        type_: AbiItemType, // Error
        name: String,
        inputs: Vec<JsonIO>,
    },

    // Unified Function-like variant
    #[serde(rename_all = "camelCase")]
    Function {
        #[serde(rename = "type")]
        type_: FunctionType,

        // For normal functions and constructors
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        inputs: Option<Vec<JsonIO>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        outputs: Option<Vec<JsonIO>>,

        state_mutability: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonIO {
    name: String,
    #[serde(rename = "type")]
    type_: String, // "uint256", "tuple", "tuple[]", "tuple[3]", ...
    internal_type: String, // "uint256", "tuple", "tuple[]", "tuple[3]", ...
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed: Option<bool>, // present for event top-level inputs only
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>, // present iff type is tuple/tuple[]/tuple[k]
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonComponent {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    internal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>,
}

pub fn process_abi(abi: &Abi) -> String {
    // Collect all the JSON ABI items into a single vector
    let abi_json_items = process_events(abi)
        .into_iter()
        .chain(process_errors(abi))
        .chain(process_functions(abi))
        .collect();

    let json_abi = JsonAbi {
        abi: abi_json_items,
    };

    serde_json::to_string_pretty(&json_abi).unwrap()
}

fn process_errors(abi: &Abi) -> Vec<JsonAbiItem> {
    let mut errors: Vec<JsonAbiItem> = abi
        .abi_errors
        .iter()
        .map(|error| {
            let mut inputs = vec![];
            error.fields.iter().for_each(|field| {
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
                );
            });

            JsonAbiItem::Error {
                type_: AbiItemType::Error,
                name: error.identifier.clone(),
                inputs,
            }
        })
        .collect();

    // Sort errors by name for deterministic output
    errors.sort_by_key(|item| match item {
        JsonAbiItem::Error { name, .. } => name.clone(),
        _ => panic!(),
    });

    errors
}

fn process_events(abi: &Abi) -> Vec<JsonAbiItem> {
    let mut events: Vec<JsonAbiItem> = abi
        .events
        .iter()
        .map(|event| {
            let mut inputs = vec![];
            event.fields.iter().for_each(|field| {
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
                );
            });

            JsonAbiItem::Event {
                type_: AbiItemType::Event,
                name: event.identifier.clone(),
                inputs,
                anonymous: event.is_anonymous,
            }
        })
        .collect();

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
        _ => panic!(),
    });

    events
}

fn process_functions(abi: &Abi) -> Vec<JsonAbiItem> {
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
                    f.parameters.iter().for_each(|param| {
                        process_io(
                            param.type_.clone(),
                            &param.identifier,
                            None,
                            &mut inputs,
                            abi,
                        );
                    });
                    (None, Some(inputs), None)
                }
                FunctionType::Function => {
                    // Handle normal functions
                    let mut inputs = vec![];
                    f.parameters.iter().for_each(|param| {
                        process_io(
                            param.type_.clone(),
                            &param.identifier,
                            None,
                            &mut inputs,
                            abi,
                        );
                    });

                    let mut outputs = vec![];
                    match &f.return_types {
                        Type::Tuple(types_) => {
                            // For tuples, we iterate over the elements and collect them in a vector of JsonIOs
                            types_.iter().for_each(|t| {
                                process_io(t.clone(), EMPTY_STR, None, &mut outputs, abi);
                            });
                        }
                        _ => {
                            process_io(f.return_types.clone(), EMPTY_STR, None, &mut outputs, abi);
                        }
                    };

                    (Some(f.identifier.clone()), Some(inputs), Some(outputs))
                }
            };

            let state_mutability = map_state_mutability(&f.modifiers).to_string();

            JsonAbiItem::Function {
                type_: f.function_type,
                name,
                inputs,
                outputs,
                state_mutability,
            }
        })
        .collect();

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
                let name_key = name.clone().unwrap_or_default();
                (priority, name_key)
            }
            _ => panic!("Expected Function variant"),
        }
    });

    functions
}

fn map_state_mutability(mods: &[FunctionModifier]) -> &'static str {
    if mods.contains(&FunctionModifier::Pure) {
        "pure"
    } else if mods.contains(&FunctionModifier::View) {
        "view"
    } else if mods.contains(&FunctionModifier::Payable) {
        "payable"
    } else {
        "nonpayable"
    }
}

/// Processes an IO (input/output) parameter and adds it to the given vector if the type is not empty.
fn process_io(
    type_: Type,
    name: impl Into<String>,
    indexed: Option<bool>,
    io: &mut Vec<JsonIO>,
    abi: &Abi,
) {
    if type_ != Type::None {
        let JsonAbiData {
            abi_type,
            abi_internal_type,
            components,
        } = encode_for_json_abi(type_.clone(), abi);

        io.push(JsonIO {
            name: name.into(),
            type_: abi_type,
            internal_type: abi_internal_type,
            indexed,
            components,
        });
    }
}

// A struct containing the ABI type, ABI internal type, and components.
struct JsonAbiData {
    abi_type: String,
    abi_internal_type: String,
    components: Option<Vec<JsonComponent>>,
}

/// Encodes a Type into the JSON ABI format.
///
/// Returns a JsonAbiData struct containing the ABI type, ABI internal type, and components.
///
/// Recursively processes nested types (arrays, struct fields) to build the complete ABI representation.
fn encode_for_json_abi(type_: Type, abi: &Abi) -> JsonAbiData {
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
        | Type::Bytes32
        | Type::String => JsonAbiData {
            abi_type: type_.name(),
            abi_internal_type: type_.name(),
            components: None,
        },
        Type::Enum {
            identifier,
            module_id,
        } => {
            let abi_type = "uint8".to_string();
            let abi_internal_type = format!(
                "enum {}.{}",
                snake_to_upper_camel(&module_id.module_name),
                identifier
            );
            JsonAbiData {
                abi_type,
                abi_internal_type,
                components: None,
            }
        }
        Type::Array(inner) => {
            let JsonAbiData {
                abi_type,
                abi_internal_type,
                components,
            } = encode_for_json_abi((**inner).clone(), abi);

            JsonAbiData {
                abi_type: format!("{abi_type}[]"),
                abi_internal_type: format!("{abi_internal_type}[]"),
                components,
            }
        }
        Type::Struct { module_id, .. } => {
            // Find corresponding processed struct, searching by the name, which differs from the identifier in case of generic structs
            let abi_struct = abi
                .structs
                .iter()
                .find(|s| s.identifier == type_.name())
                .unwrap();

            let components = abi_struct
                .fields
                .iter()
                .map(|named_type| {
                    let JsonAbiData {
                        abi_type,
                        abi_internal_type,
                        components,
                    } = encode_for_json_abi(named_type.type_.clone(), abi);

                    JsonComponent {
                        name: named_type.identifier.clone(),
                        type_: abi_type,
                        internal_type: abi_internal_type,
                        components,
                    }
                })
                .collect();

            let abi_type = "tuple".to_string();
            let abi_internal_type = format!(
                "struct {}.{}",
                snake_to_upper_camel(&module_id.module_name),
                type_.name()
            );
            JsonAbiData {
                abi_type,
                abi_internal_type,
                components: Some(components),
            }
        }
        Type::Tuple(_) => {
            panic!("Found a Tuple type in the JSON ABI generation");
        }
        Type::None => {
            panic!("Found a None type in the JSON ABI generation");
        }
    }
}
