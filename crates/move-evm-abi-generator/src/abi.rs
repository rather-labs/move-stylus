use std::collections::{HashMap, HashSet};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId,
    module_data::struct_data::{IStruct, IntermediateType},
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, SF_MODULE_NAME_TX_CONTEXT, STYLUS_FRAMEWORK_ADDRESS,
    },
};
use move_parse_special_attributes::function_modifiers::{FunctionModifier, Parameter};

use crate::{
    ErrorStruct, EventStruct,
    common::{snake_to_camel, snake_to_upper_camel},
    special_types::{is_named_id, is_uid},
    types::Type,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub struct Function {
    pub(crate) identifier: String,
    pub(crate) parameters: Vec<NamedType>,
    pub(crate) return_types: Type,
    pub(crate) visibility: Visibility,
    pub(crate) modifiers: Vec<FunctionModifier>,
    pub(crate) is_entry: bool,
}

#[derive(Debug)]
pub struct Struct_ {
    pub(crate) identifier: String,
    pub(crate) fields: Vec<NamedType>,
    pub(crate) positional_fields: bool,
}

#[derive(Debug)]
pub struct Event {
    pub(crate) identifier: String,
    pub(crate) fields: Vec<EventField>,
    pub(crate) is_anonymous: bool,
    pub(crate) positional_fields: bool,
}

/// A unified struct representing a typed field used in functions, structs, and events.
#[derive(Debug)]
pub struct NamedType {
    pub(crate) identifier: String,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub struct EventField {
    pub(crate) named_type: NamedType,
    pub(crate) indexed: bool,
}

#[derive(Debug)]
pub struct Abi {
    pub(crate) contract_name: String,
    pub(crate) functions: Vec<Function>,
    pub(crate) structs: Vec<Struct_>,
    pub(crate) events: Vec<Event>,
    pub(crate) abi_errors: Vec<Struct_>,
}

impl Abi {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.structs.is_empty()
    }

    pub(crate) fn new(
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
        event_structs: &HashSet<EventStruct>,
        error_structs: &HashSet<ErrorStruct>,
    ) -> Abi {
        // Create a single HashSet to collect all structs that need to be processed
        // This includes structs from events, errors, and functions
        let mut structs_to_process = HashSet::new();

        let events = Self::process_events(event_structs, modules_data, &mut structs_to_process);

        let abi_errors =
            Self::process_abi_errors(error_structs, modules_data, &mut structs_to_process);

        let functions =
            Self::process_functions(processing_module, modules_data, &mut structs_to_process);

        let mut processed_structs = HashSet::new();
        let structs =
            Self::process_structs(structs_to_process, modules_data, &mut processed_structs);

        Abi {
            contract_name: processing_module.special_attributes.module_name.clone(),
            functions,
            structs,
            events,
            abi_errors,
        }
    }

    /// This contains all the structs that appear as argument or return of functions. Once we
    /// process the functions this will be the structs appearing in the ABI
    fn process_functions(
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) -> Vec<Function> {
        let mut result = Vec::new();

        // First we filter the functions we are going to process
        let functions = processing_module
            .functions
            .information
            .iter()
            .filter(|f| f.is_entry);

        'functions_loop: for function in functions {
            let parsed_function = processing_module
                .special_attributes
                .functions
                .iter()
                .find(|f| f.name == function.function_id.identifier)
                .expect("function not found");

            // Function name
            let function_name = snake_to_camel(&function.function_id.identifier);

            // Process fuction arguments
            let mut function_parameters = Vec::new();
            for (param, itype) in parsed_function
                .signature
                .parameters
                .iter()
                .zip(&function.signature.arguments)
            {
                // Remove the references if any
                let itype = match itype {
                    IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                        inner.as_ref()
                    }
                    _ => itype,
                };

                // Check if the type is hidden in the signature (TxContext, signer, etc...)
                // If the type should not be in the signature, we continue the loop
                match itype {
                    IntermediateType::ISigner => continue,
                    // If we find a type parameter, this function is a generic one and can't be part of
                    // the ABI
                    IntermediateType::ITypeParameter(_) => continue 'functions_loop,
                    IntermediateType::IStruct {
                        module_id, index, ..
                    } => {
                        let struct_module = modules_data.get(module_id).unwrap();
                        let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                        match (
                            struct_.identifier.as_str(),
                            module_id.address,
                            module_id.module_name.as_str(),
                        ) {
                            ("TxContext", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                                continue;
                            }
                            _ => {
                                if struct_.has_key {
                                    // TODO: can an error/event have a key? if so, we need to resolve conflicts here too!
                                    Self::process_storage_struct(
                                        struct_,
                                        itype,
                                        modules_data,
                                        &mut function_parameters,
                                        param,
                                        structs_to_process,
                                    );
                                } else {
                                    {
                                        function_parameters.push(NamedType {
                                            identifier: param.name.clone(),
                                            type_: Type::from_intermediate_type(
                                                itype,
                                                modules_data,
                                            ),
                                        });
                                        if let Some(struct_itype) =
                                            Self::should_process_struct(itype, modules_data)
                                        {
                                            structs_to_process.insert(struct_itype);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    IntermediateType::IGenericStructInstance {
                        module_id,
                        index,
                        types,
                        ..
                    } => {
                        let struct_module = modules_data.get(module_id).unwrap();
                        let struct_ = struct_module
                            .structs
                            .get_by_index(*index)
                            .unwrap()
                            .instantiate(types);

                        if struct_.has_key {
                            Self::process_storage_struct(
                                &struct_,
                                itype,
                                modules_data,
                                &mut function_parameters,
                                param,
                                structs_to_process,
                            );
                        } else {
                            {
                                function_parameters.push(NamedType {
                                    identifier: param.name.clone(),
                                    type_: Type::from_intermediate_type(itype, modules_data),
                                });
                                if let Some(struct_itype) =
                                    Self::should_process_struct(itype, modules_data)
                                {
                                    structs_to_process.insert(struct_itype);
                                }
                            }
                        }
                    }
                    IntermediateType::IEnum { module_id, index } => {
                        let enum_module = modules_data.get(module_id).unwrap();
                        let enum_ = enum_module.enums.get_by_index(*index).unwrap();
                        if !enum_.is_simple {
                            panic!("found not simple enum in function signature");
                        } else {
                            function_parameters.push(NamedType {
                                identifier: param.name.clone(),
                                type_: Type::from_intermediate_type(itype, modules_data),
                            });
                        }
                    }
                    IntermediateType::IGenericEnumInstance {
                        module_id,
                        index,
                        types,
                    } => {
                        let enum_module = modules_data.get(module_id).unwrap();
                        let enum_ = enum_module
                            .enums
                            .get_by_index(*index)
                            .unwrap()
                            .instantiate(types);

                        if !enum_.is_simple {
                            panic!("found not simple enum in function signature");
                        } else {
                            function_parameters.push(NamedType {
                                identifier: param.name.clone(),
                                type_: Type::from_intermediate_type(itype, modules_data),
                            });
                        }
                    }
                    _ => {
                        function_parameters.push(NamedType {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(itype, modules_data),
                        });
                    }
                }
            }

            let return_type = if function.signature.returns.is_empty() {
                Type::None
            } else if function.signature.returns.len() == 1 {
                if let Some(struct_itype) =
                    Self::should_process_struct(&function.signature.returns[0], modules_data)
                {
                    structs_to_process.insert(struct_itype);
                }

                Type::from_intermediate_type(&function.signature.returns[0], modules_data)
            } else {
                let tuple_types: Vec<Type> = function
                    .signature
                    .returns
                    .iter()
                    .map(|t| {
                        if let Some(struct_itype) = Self::should_process_struct(t, modules_data) {
                            structs_to_process.insert(struct_itype);
                        }
                        Type::from_intermediate_type(t, modules_data)
                    })
                    .collect();
                Type::Tuple(tuple_types)
            };

            let visibility = if parsed_function.visibility
                == move_parse_special_attributes::function_modifiers::Visibility::Public
            {
                Visibility::Public
            } else {
                Visibility::Private
            };

            result.push(Function {
                identifier: function_name,
                parameters: function_parameters,
                return_types: return_type,
                is_entry: function.is_entry,
                modifiers: parsed_function.modifiers.clone(),
                visibility,
            });
        }
        result
    }

    fn process_storage_struct(
        struct_: &IStruct,
        struct_itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
        function_parameters: &mut Vec<NamedType>,
        param: &Parameter,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) {
        assert!(struct_.has_key);
        let first_parameter = struct_.fields.first();
        // If the first parameter:
        // - is a UID, then the signature type is bytes32
        // - is a NamedId<>, then the signature type ignored

        match first_parameter {
            Some(IntermediateType::IStruct {
                module_id, index, ..
            }) => {
                let struct_module = modules_data.get(module_id).unwrap();
                let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("UID", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {
                        function_parameters.push(NamedType {
                            identifier: param.name.clone(),
                            type_: Type::Bytes32,
                        });
                    }
                    _ => {
                        function_parameters.push(NamedType {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(struct_itype, modules_data),
                        });
                        if let Some(struct_itype_to_add) =
                            Self::should_process_struct(struct_itype, modules_data)
                        {
                            structs_to_process.insert(struct_itype_to_add);
                        }
                    }
                }
            }
            Some(IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            }) => {
                let struct_module = modules_data.get(module_id).unwrap();
                let struct_ = struct_module
                    .structs
                    .get_by_index(*index)
                    .unwrap()
                    .instantiate(types);

                match (
                    struct_.identifier.as_str(),
                    module_id.address,
                    module_id.module_name.as_str(),
                ) {
                    ("NamedId", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {}
                    _ => {
                        function_parameters.push(NamedType {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(struct_itype, modules_data),
                        });
                        if let Some(struct_itype_to_add) =
                            Self::should_process_struct(struct_itype, modules_data)
                        {
                            structs_to_process.insert(struct_itype_to_add);
                        }
                    }
                }
            }
            _ => panic!("processing a storager struct that has no id as first parameter"),
        }
    }

    pub fn process_structs(
        structs: HashSet<IntermediateType>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        processed_structs: &mut HashSet<IntermediateType>,
    ) -> Vec<Struct_> {
        let mut result = Vec::new();
        for struct_itype in structs {
            if processed_structs.contains(&struct_itype) {
                continue;
            }

            let (struct_, parsed_struct) = {
                let (module_id, index, types) = match &struct_itype {
                    IntermediateType::IStruct {
                        module_id, index, ..
                    } => (module_id, index, None),
                    IntermediateType::IGenericStructInstance {
                        module_id,
                        index,
                        types,
                        ..
                    } => (module_id, index, Some(types)),
                    t => panic!("found {t:?} instead of struct"),
                };

                let struct_module = modules_data.get(module_id).unwrap();
                let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                let struct_ = match types {
                    Some(types) => struct_.instantiate(types),
                    None => struct_.clone(),
                };
                let parsed_struct = struct_module
                    .special_attributes
                    .structs
                    .iter()
                    .find(|s| s.name == struct_.identifier)
                    .unwrap();

                (struct_, parsed_struct)
            };

            let mut child_structs_to_process = HashSet::new();
            let fields = struct_
                .fields
                .iter()
                .zip(&parsed_struct.fields)
                .map(|(field_itype, (name, _))| {
                    match field_itype {
                        IntermediateType::IStruct { .. }
                        | IntermediateType::IGenericStructInstance { .. } => {
                            child_structs_to_process.insert(field_itype.clone());
                        }
                        _ => {}
                    }
                    NamedType {
                        identifier: name.clone(),
                        type_: Type::from_intermediate_type(field_itype, modules_data),
                    }
                })
                .collect();

            let struct_abi_type = Type::from_intermediate_type(&struct_itype, modules_data);

            result.push(Struct_ {
                // Resolve struct identifier conflicts with events or errors
                identifier: struct_abi_type.name(),
                fields,
                positional_fields: parsed_struct.positional_fields,
            });

            processed_structs.insert(struct_itype);

            // Process child structs
            let child_structs =
                Self::process_structs(child_structs_to_process, modules_data, processed_structs);

            result.extend(child_structs);
        }

        result
    }

    pub fn process_events(
        event_structs: &HashSet<EventStruct>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) -> Vec<Event> {
        let mut result = Vec::new();

        for event_struct in event_structs {
            let event_module = modules_data
                .get(&ModuleId {
                    address: event_struct.module_id.address().into_bytes().into(),
                    module_name: event_struct.module_id.name().to_string(),
                })
                .unwrap();

            let (event_struct, event_identifier) = if let Some(struct_def_instantiation_index) =
                &event_struct.struct_def_instantiation_index
            {
                let event_struct = event_module
                    .structs
                    .get_struct_instance_by_struct_definition_idx(struct_def_instantiation_index)
                    .unwrap();

                let types = event_module
                    .structs
                    .get_generic_struct_types_instances(struct_def_instantiation_index)
                    .unwrap();

                let concrete_type_parameters_names = types
                    .iter()
                    .map(|t| Type::from_intermediate_type(t, modules_data).name())
                    .collect::<Vec<String>>()
                    .join("_");

                let event_identifier = snake_to_upper_camel(&format!(
                    "{}_{}",
                    event_struct.identifier, concrete_type_parameters_names
                ));
                (event_struct, event_identifier)
            } else {
                (
                    event_module
                        .structs
                        .get_by_identifier(&event_struct.identifier)
                        .unwrap()
                        .clone(),
                    event_struct.identifier.clone(),
                )
            };

            let event_special_attributes = event_module
                .special_attributes
                .events
                .get(&event_struct.identifier)
                .unwrap();

            let event_struct_parsed = event_module
                .special_attributes
                .structs
                .iter()
                .find(|s| s.name.as_str() == event_struct.identifier)
                .unwrap();

            // Collect structs from event fields
            for field_itype in &event_struct.fields {
                // println!("field_itype: {:#?}", field_itype);
                if let Some(struct_itype) = Self::should_process_struct(field_itype, modules_data) {
                    structs_to_process.insert(struct_itype);
                }
            }

            result.push(Event {
                identifier: event_identifier,
                fields: event_struct
                    .fields
                    .iter()
                    .zip(&event_struct_parsed.fields)
                    .enumerate()
                    .map(|(index, (f, (identifier, _)))| EventField {
                        named_type: NamedType {
                            identifier: identifier.clone(),
                            type_: Type::from_intermediate_type(f, modules_data),
                        },
                        indexed: index < event_special_attributes.indexes as usize,
                    })
                    .collect(),
                is_anonymous: event_special_attributes.is_anonymous,
                positional_fields: event_struct_parsed.positional_fields,
            });
        }

        result
    }

    pub fn process_abi_errors(
        error_structs: &HashSet<ErrorStruct>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) -> Vec<Struct_> {
        let mut result = Vec::new();

        for error_struct in error_structs {
            let error_module = modules_data
                .get(&ModuleId {
                    address: error_struct.module_id.address().into_bytes().into(),
                    module_name: error_struct.module_id.name().to_string(),
                })
                .unwrap();

            let error_struct = error_module
                .structs
                .get_by_identifier(&error_struct.identifier)
                .unwrap();

            let error_struct_parsed = error_module
                .special_attributes
                .structs
                .iter()
                .find(|s| s.name.as_str() == error_struct.identifier)
                .unwrap();

            // Collect structs from error fields
            for field_itype in &error_struct.fields {
                if let Some(struct_itype) = Self::should_process_struct(field_itype, modules_data) {
                    structs_to_process.insert(struct_itype);
                }
            }

            result.push(Struct_ {
                identifier: error_struct.identifier.to_string(),
                fields: error_struct
                    .fields
                    .iter()
                    .zip(&error_struct_parsed.fields)
                    .map(|(f, (identifier, _))| NamedType {
                        identifier: identifier.clone(),
                        type_: Type::from_intermediate_type(f, modules_data),
                    })
                    .collect(),
                positional_fields: error_struct_parsed.positional_fields,
            });
        }

        result
    }

    /// Helper function to check if a struct type should be added to the process HashSet.
    /// Returns Some(IntermediateType) if it should be added, None otherwise.
    fn should_process_struct(
        itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> Option<IntermediateType> {
        match itype {
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_module = modules_data.get(module_id).unwrap();
                let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                // Only add if it's not a named_id or uid
                if !is_named_id(&struct_.identifier, module_id)
                    && !is_uid(&struct_.identifier, module_id)
                {
                    Some(itype.clone())
                } else {
                    None
                }
            }
            IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } => {
                let struct_module = modules_data.get(module_id).unwrap();
                let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                // Only add if it's not a named_id or uid
                if !is_named_id(&struct_.identifier, module_id)
                    && !is_uid(&struct_.identifier, module_id)
                {
                    Some(itype.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
