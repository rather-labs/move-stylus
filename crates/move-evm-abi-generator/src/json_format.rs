use crate::abi::{Abi, Event, Function, Struct_};
use crate::types::Type;
use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use move_parse_special_attributes::function_modifiers::FunctionModifier;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct JsonAbi {
    abi: Vec<JsonAbiItem>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FunctionType {
    Constructor,
    Fallback,
    Receive,
    Function,
}

impl FunctionType {
    fn from_identifier(identifier: &str) -> Self {
        match identifier {
            "constructor" => Self::Constructor,
            "fallback" => Self::Fallback,
            "receive" => Self::Receive,
            _ => Self::Function,
        }
    }
}

// Todo: is it possible to get the FunctionType from within the Function variant to serialize the type as a string?
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AbiItemType {
    Event,
    Error,
    Function(FunctionType),
}

#[derive(Serialize, Deserialize)]
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

        #[serde(rename = "stateMutability")]
        state_mutability: String,
    },
}

#[derive(Serialize, Deserialize)]
struct JsonIO {
    name: String,
    #[serde(rename = "type")]
    type_: String, // "uint256", "tuple", "tuple[]", "tuple[3]", ...
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed: Option<bool>, // present for event top-level inputs only
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>, // present iff type is tuple/tuple[]/tuple[k]
}

#[derive(Serialize, Deserialize)]
struct JsonComponent {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<JsonComponent>>,
}

pub fn process_abi(
    abi: &Abi,
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> String {
    let mut json_abi = JsonAbi { abi: Vec::new() };

    let json_events = process_events(&abi.events, processing_module, modules_data);
    let json_errors = process_errors(&abi.abi_errors, processing_module, modules_data);
    let json_functions = process_functions(&abi.functions, processing_module, modules_data);

    json_abi.abi.extend(json_events);
    json_abi.abi.extend(json_errors);
    json_abi.abi.extend(json_functions);

    serde_json::to_string_pretty(&json_abi).unwrap()
}

fn process_errors(
    errors: &Vec<Struct_>,
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    let mut json_errors: Vec<JsonAbiItem> = Vec::new();
    for error in errors {
        let inputs = error
            .fields
            .iter()
            .map(|field| {
                process_io(
                    field.identifier.clone(),
                    &field.type_,
                    None,
                    processing_module,
                    modules_data,
                )
                // let (field_type_name, components) =
                //     field.type_.encode_for_abi(processing_module, modules_data);

                // JsonIO {
                //     name: field.identifier.clone(),
                //     type_: field_type_name,
                //     indexed: None, // Only for events
                //     components,
                // }
            })
            .collect();

        json_errors.push(JsonAbiItem::Error {
            type_: AbiItemType::Error,
            name: error.identifier.clone(),
            inputs,
        });
    }
    json_errors
}

fn process_events(
    events: &Vec<Event>,
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    let mut json_events: Vec<JsonAbiItem> = Vec::new();

    for event in events {
        let inputs = event
            .fields
            .iter()
            .map(|field| {
                process_io(
                    field.identifier.clone(),
                    &field.type_,
                    Some(field.indexed),
                    processing_module,
                    modules_data,
                )
                // let (field_type_name, components) =
                //     field.type_.encode_for_abi(processing_module, modules_data);

                // JsonIO {
                //     name: field.identifier.clone(),
                //     type_: field_type_name,
                //     indexed: Some(field.indexed), // allowed only here at the input level
                //     components,                   // present if tuple/tuple[]
                // }
            })
            .collect();

        json_events.push(JsonAbiItem::Event {
            type_: AbiItemType::Event,
            name: event.identifier.clone(),
            inputs,
            anonymous: event.is_anonymous,
        });
    }

    json_events
}

fn process_functions(
    functions: &[Function],
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    functions
        .iter()
        .map(|f| {
            let type_ = FunctionType::from_identifier(&f.identifier);

            let (name, inputs, outputs) = match type_ {
                // Fallback and Receive have no name, inputs, or outputs
                FunctionType::Fallback | FunctionType::Receive => (None, None, None),
                // Constructor has no name, but has inputs
                FunctionType::Constructor => {
                    let inputs = Some(
                        f.parameters
                            .iter()
                            .map(|p| {
                                process_io(
                                    p.identifier.clone(),
                                    &p.type_,
                                    None,
                                    processing_module,
                                    modules_data,
                                )
                            })
                            .collect(),
                    );
                    (None, inputs, None)
                }
                FunctionType::Function => {
                    // Handle normal functions
                    let inputs = f
                        .parameters
                        .iter()
                        .map(|p| {
                            process_io(
                                p.identifier.clone(),
                                &p.type_,
                                None,
                                processing_module,
                                modules_data,
                            )
                        })
                        .collect();

                    let outputs = match &f.return_types {
                        crate::types::Type::Tuple(types) => types
                            .iter()
                            .map(|t| JsonIO {
                                name: "".into(),
                                type_: t.name(),
                                indexed: None,
                                components: None,
                            })
                            .collect(),
                        crate::types::Type::None => Vec::new(),
                        _ => vec![JsonIO {
                            name: "".into(),
                            type_: f.return_types.name(),
                            indexed: None,
                            components: None,
                        }],
                    };

                    (Some(f.identifier.clone()), Some(inputs), Some(outputs))
                }
            };

            // TODO: what happens if there are multiple modifiers?
            let state_mutability = if let Some(m) = f.modifiers.first() {
                match m {
                    FunctionModifier::Pure => m.as_str().to_string(),
                    FunctionModifier::View => m.as_str().to_string(),
                    FunctionModifier::Payable => m.as_str().to_string(),
                    _ => "nonpayable".to_string(),
                }
            } else {
                // Default is "nonpayable" unless explicitly specified as pure, view, or payable
                "nonpayable".to_string()
            };

            JsonAbiItem::Function {
                type_,
                name,
                inputs,
                outputs,
                state_mutability,
            }
        })
        .collect()
}

fn process_io(
    name: String,
    type_: &Type,
    indexed: Option<bool>,
    processing_module: &ModuleData,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> JsonIO {
    let (type_name, components) = type_.encode_for_abi(processing_module, modules_data);

    JsonIO {
        name,
        type_: type_name,
        indexed,
        components,
    }
}

impl Type {
    fn encode_for_abi(
        &self,
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> (String, Option<Vec<JsonComponent>>) {
        match self {
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
            | Type::None => (self.name(), None),
            Type::String => (self.name(), None),
            Type::Array(inner) => {
                let (inner_type_name, inner_components) =
                    inner.encode_for_abi(processing_module, modules_data);
                (format!("{}[]", inner_type_name), inner_components)
            }
            Type::Struct { identifier, .. } => {
                // We use the IStruct to get the Type of the fields, which differs from the Type defined in special_attributes
                let struct_ = processing_module
                    .structs
                    .get_by_identifier(identifier)
                    .unwrap();

                // Get field names from the Struct_ defined inspecial_attributes
                let struct_sa = processing_module
                    .special_attributes
                    .structs
                    .iter()
                    .find(|s| s.name.as_str() == identifier)
                    .unwrap();

                let components = struct_
                    .fields
                    .iter()
                    .zip(&struct_sa.fields)
                    .map(|(field_itype, (field_name, _))| {
                        let field_type = Type::from_intermediate_type(field_itype, modules_data);
                        let (field_type_name, field_comps) =
                            field_type.encode_for_abi(processing_module, modules_data);
                        JsonComponent {
                            // positional fields do not have names in the abi
                            name: if struct_sa.positional_fields {
                                "".to_string()
                            } else {
                                field_name.clone()
                            },
                            type_: field_type_name,
                            components: field_comps,
                        }
                    })
                    .collect();

                ("tuple".to_string(), Some(components))
            }
            Type::Enum { .. } => ("enum".to_string(), None),
            Type::Tuple(_types) => ("tuple".to_string(), None),
        }
    }
}
