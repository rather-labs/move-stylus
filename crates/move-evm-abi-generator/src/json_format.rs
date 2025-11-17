use crate::abi::{Abi, Event, Function, Struct_};
use crate::types::Type;
use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use move_parse_special_attributes::function_modifiers::FunctionModifier;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct JsonAbi {
    abi: Vec<JsonAbiItem>,
}

#[derive(Clone, Copy, Debug, Serialize)]
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

pub fn process_abi(abi: &Abi, modules_data: &HashMap<ModuleId, ModuleData>) -> String {
    // Collect all the JSON ABI items into a single vector
    let abi_json_items = process_events(&abi.events, modules_data)
        .into_iter()
        .chain(process_errors(&abi.abi_errors, modules_data))
        .chain(process_functions(&abi.functions, modules_data))
        .collect();

    let json_abi = JsonAbi {
        abi: abi_json_items,
    };

    serde_json::to_string_pretty(&json_abi).unwrap()
}

fn process_errors(
    errors: &[Struct_],
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    errors
        .iter()
        .map(|error| {
            let mut inputs = vec![];
            error.fields.iter().for_each(|field| {
                process_io(
                    field.type_.clone(),
                    field.identifier.clone(),
                    None,
                    &mut inputs,
                    modules_data,
                );
            });

            JsonAbiItem::Error {
                type_: AbiItemType::Error,
                name: error.identifier.clone(),
                inputs,
            }
        })
        .collect()
}

fn process_events(
    events: &[Event],
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    events
        .iter()
        .map(|event| {
            let mut inputs = vec![];
            event.fields.iter().for_each(|field| {
                process_io(
                    field.named_type.type_.clone(),
                    field.named_type.identifier.clone(),
                    Some(field.indexed),
                    &mut inputs,
                    modules_data,
                );
            });

            JsonAbiItem::Event {
                type_: AbiItemType::Event,
                name: event.identifier.clone(),
                inputs,
                anonymous: event.is_anonymous,
            }
        })
        .collect()
}

fn process_functions(
    functions: &[Function],
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> Vec<JsonAbiItem> {
    functions
        .iter()
        .map(|f| {
            let fn_type = FunctionType::from_identifier(&f.identifier);

            let (name, inputs, outputs) = match fn_type {
                // Fallback and Receive have no name, inputs, or outputs
                FunctionType::Fallback | FunctionType::Receive => (None, None, None),
                // Constructor has no name, but has inputs
                FunctionType::Constructor => {
                    let mut inputs = vec![];
                    f.parameters.iter().for_each(|param| {
                        process_io(
                            param.type_.clone(),
                            param.identifier.clone(),
                            None,
                            &mut inputs,
                            modules_data,
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
                            param.identifier.clone(),
                            None,
                            &mut inputs,
                            modules_data,
                        );
                    });

                    let mut outputs = vec![];
                    match &f.return_types {
                        Type::Tuple(types_) => {
                            // For tuples, we iterate over the elements and collect them in a vector of JsonIOs
                            types_.iter().for_each(|t| {
                                process_io(
                                    t.clone(),
                                    "".to_string(),
                                    None,
                                    &mut outputs,
                                    modules_data,
                                );
                            });
                        }
                        _ => {
                            process_io(
                                f.return_types.clone(),
                                "".to_string(),
                                None,
                                &mut outputs,
                                modules_data,
                            );
                        }
                    };

                    (Some(f.identifier.clone()), Some(inputs), Some(outputs))
                }
            };

            let state_mutability = map_state_mutability(&f.modifiers).to_string();

            JsonAbiItem::Function {
                type_: fn_type,
                name,
                inputs,
                outputs,
                state_mutability,
            }
        })
        .collect()
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
    name: String,
    indexed: Option<bool>,
    io: &mut Vec<JsonIO>,
    modules_data: &HashMap<ModuleId, ModuleData>,
) {
    let (abi_type, abi_internal_type, components) = encode_for_json_abi(type_, modules_data);
    if !abi_type.is_empty() {
        io.push(JsonIO {
            name,
            type_: abi_type,
            internal_type: abi_internal_type,
            indexed,
            components,
        });
    }
}

/// Encodes a Type into the JSON ABI format.
///
/// Returns a tuple of `(type_name, components)` where:
/// - `type_name`: The ABI type string (e.g., "uint256", "tuple", "tuple[]")
/// - `components`: `Some(Vec<JsonComponent>)` for struct types (tuples), `None` for primitive types
///
/// Recursively processes nested types (arrays, struct fields) to build the complete ABI representation.
fn encode_for_json_abi(
    type_: Type,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> (String, String, Option<Vec<JsonComponent>>) {
    match type_ {
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
        | Type::None => {
            let abi_type = type_.name();
            (abi_type.clone(), abi_type, None)
        }
        Type::String => {
            let abi_type = type_.name();
            (abi_type.clone(), abi_type, None)
        }
        Type::Enum {
            identifier,
            module_id,
        } => {
            let abi_type = "uint8".to_string();
            let abi_internal_type = format!("enum {}.{}", module_id.module_name, identifier);
            (abi_type, abi_internal_type, None)
        }
        Type::Array(inner) => {
            let (inner_abi_type, inner_internal_type, inner_components) =
                encode_for_json_abi((*inner).clone(), modules_data);

            (
                format!("{inner_abi_type}[]"),
                format!("{inner_internal_type}[]"),
                inner_components,
            )
        }
        Type::Struct {
            identifier,
            module_id,
            ..
        } => {
            let struct_module = modules_data.get(&module_id).unwrap();
            // We use the IStruct to get the Type of the fields, which differs from the Type defined in special_attributes
            let struct_ = struct_module
                .structs
                .get_by_identifier(&identifier)
                .unwrap();

            // Get field names from the Struct_ defined in special_attributes
            let struct_sa = struct_module
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
                    let (field_abi_type, field_abi_internal_type, field_comps) =
                        encode_for_json_abi(field_type, modules_data);
                    JsonComponent {
                        // positional fields do not have names in the abi
                        name: if struct_sa.positional_fields {
                            "".to_string()
                        } else {
                            field_name.clone()
                        },
                        type_: field_abi_type,
                        internal_type: field_abi_internal_type,
                        components: field_comps,
                    }
                })
                .collect();

            let abi_type = "tuple".to_string();
            let abi_internal_type = format!("struct {}.{}", module_id.module_name, identifier);
            (abi_type, abi_internal_type, Some(components))
        }
        Type::Tuple(_) => {
            panic!("Tuple types should be destructered by the caller");
        }
    }
}
