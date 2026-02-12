// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use std::collections::{HashMap, HashSet};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId,
    module_data::struct_data::{IStruct, IntermediateType},
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, SF_MODULE_NAME_TX_CONTEXT, STYLUS_FRAMEWORK_ADDRESS,
    },
};
use move_parse_special_attributes::function_modifiers::{Parameter, SolidityFunctionModifier};
use move_symbol_pool::Symbol;

use crate::{
    ErrorStruct, EventStruct,
    common::snake_to_camel,
    error::{AbiGeneratorError, AbiGeneratorErrorKind},
    special_types::{is_bytes_n, is_id, is_named_id, is_string, is_uid},
    types::Type,
};

use serde::Serialize;

const STYLUS_FW_NAMED_ID: &str = "NamedId";
const STYLUS_FW_UID: &str = "UID";

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FunctionType {
    Constructor,
    Fallback,
    Receive,
    Function,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub struct Function {
    pub(crate) function_type: FunctionType,
    pub(crate) identifier: Symbol,
    pub(crate) parameters: Vec<NamedType>,
    pub(crate) return_types: Type,
    pub(crate) visibility: Visibility,
    pub(crate) modifiers: Vec<SolidityFunctionModifier>,
    pub(crate) is_entry: bool,
}

#[derive(Debug)]
pub struct Struct_ {
    pub(crate) identifier: Symbol,
    pub(crate) fields: Vec<NamedType>,
    pub(crate) positional_fields: bool,
}

#[derive(Debug)]
pub struct Enum_ {
    pub(crate) identifier: Symbol,
    pub(crate) variants: Vec<Symbol>,
}

#[derive(Debug)]
pub struct Event {
    pub(crate) identifier: Symbol,
    pub(crate) fields: Vec<EventField>,
    pub(crate) is_anonymous: bool,
    pub(crate) positional_fields: bool,
}

/// A unified struct representing a typed field used in functions, structs, and events.
#[derive(Debug)]
pub struct NamedType {
    pub(crate) identifier: Symbol,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub struct EventField {
    pub(crate) named_type: NamedType,
    pub(crate) indexed: bool,
}

#[derive(Debug)]
pub struct Abi {
    pub(crate) contract_name: Symbol,
    pub(crate) functions: Vec<Function>,
    pub(crate) structs: Vec<Struct_>,
    pub(crate) enums: Vec<Enum_>,
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
    ) -> Result<Abi, AbiGeneratorError> {
        // Create a single HashSet to collect all structs that need to be processed
        // This includes structs from events, errors, and functions
        let mut structs_to_process = HashSet::new();
        let mut enums_to_process = HashSet::new();

        let events = Self::process_events(event_structs, modules_data, &mut structs_to_process)?;

        let abi_errors =
            Self::process_abi_errors(error_structs, modules_data, &mut structs_to_process)?;

        let functions = Self::process_functions(
            processing_module,
            modules_data,
            &mut structs_to_process,
            &mut enums_to_process,
        )?;

        let mut processed_structs = HashSet::new();
        let structs =
            Self::process_structs(structs_to_process, modules_data, &mut processed_structs)?;

        let enums = Self::process_enums(enums_to_process, modules_data)?;

        Ok(Abi {
            contract_name: processing_module.special_attributes.module_name,
            functions,
            structs,
            enums,
            events,
            abi_errors,
        })
    }

    /// This contains all the structs that appear as argument or return of functions. Once we
    /// process the functions this will be the structs appearing in the ABI
    fn process_functions(
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
        enums_to_process: &mut HashSet<IntermediateType>,
    ) -> Result<Vec<Function>, AbiGeneratorError> {
        let mut result = Vec::new();

        // First we filter the functions we are going to process
        let functions: Vec<_> = processing_module
            .functions
            .information
            .iter()
            .filter(|f| f.is_entry)
            .collect();

        'functions_loop: for function in functions {
            let parsed_function = processing_module
                .special_attributes
                .functions
                .iter()
                .find(|f| *f.name == *function.function_id.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::FunctionNotFound(function.function_id.identifier),
                })?;

            // Determine the function type based on the function ID
            let function_type = if processing_module.functions.init.as_ref()
                == Some(&function.function_id)
            {
                continue;
            } else if processing_module.functions.receive.as_ref() == Some(&function.function_id) {
                FunctionType::Receive
            } else if processing_module.functions.fallback.as_ref() == Some(&function.function_id) {
                FunctionType::Fallback
            } else {
                FunctionType::Function
            };

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
                        let struct_module =
                            modules_data.get(module_id).ok_or(AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                            })?;
                        let struct_ = struct_module.structs.get_by_index(*index).map_err(|_| {
                            AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::StructNotFoundByIndex(
                                    *index, *module_id,
                                ),
                            }
                        })?;

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
                                    Self::process_storage_struct(
                                        struct_,
                                        itype,
                                        *module_id,
                                        modules_data,
                                        &mut function_parameters,
                                        param,
                                    )?;
                                } else {
                                    function_parameters.push(NamedType {
                                        identifier: param.name,
                                        type_: Type::from_intermediate_type(itype, modules_data)?,
                                    });
                                    if Self::should_process_struct(itype, modules_data)? {
                                        structs_to_process.insert(itype.clone());
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
                        let struct_module =
                            modules_data.get(module_id).ok_or(AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                            })?;
                        let struct_ = struct_module
                            .structs
                            .get_by_index(*index)
                            .map_err(|_| AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::StructNotFoundByIndex(
                                    *index, *module_id,
                                ),
                            })?
                            .instantiate(types);

                        if struct_.has_key {
                            Self::process_storage_struct(
                                &struct_,
                                itype,
                                *module_id,
                                modules_data,
                                &mut function_parameters,
                                param,
                            )?;
                        } else {
                            {
                                function_parameters.push(NamedType {
                                    identifier: param.name,
                                    type_: Type::from_intermediate_type(itype, modules_data)?,
                                });
                                if Self::should_process_struct(itype, modules_data)? {
                                    structs_to_process.insert(itype.clone());
                                }
                            }
                        }
                    }
                    IntermediateType::IEnum { module_id, index } => {
                        let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                        })?;
                        let enum_ = enum_module.enums.get_by_index(*index).map_err(|_| {
                            AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::EnumNotFoundByIndex(
                                    *index, *module_id,
                                ),
                            }
                        })?;
                        if !enum_.is_simple {
                            return Err(AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::NonSimpleEnumInSignature(
                                    enum_.identifier,
                                    function.function_id.identifier.to_string(),
                                ),
                            });
                        } else {
                            function_parameters.push(NamedType {
                                identifier: param.name,
                                type_: Type::from_intermediate_type(itype, modules_data)?,
                            });

                            enums_to_process.insert(itype.clone());
                        }
                    }
                    IntermediateType::IGenericEnumInstance {
                        module_id,
                        index,
                        types,
                    } => {
                        let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                        })?;
                        let enum_ = enum_module
                            .enums
                            .get_by_index(*index)
                            .map_err(|_| AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::EnumNotFoundByIndex(
                                    *index, *module_id,
                                ),
                            })?
                            .instantiate(types);

                        if !enum_.is_simple {
                            return Err(AbiGeneratorError {
                                kind: AbiGeneratorErrorKind::NonSimpleEnumInSignature(
                                    enum_.identifier,
                                    function.function_id.identifier.to_string(),
                                ),
                            });
                        } else {
                            function_parameters.push(NamedType {
                                identifier: param.name,
                                type_: Type::from_intermediate_type(itype, modules_data)?,
                            });

                            enums_to_process.insert(itype.clone());
                        }
                    }
                    _ => {
                        function_parameters.push(NamedType {
                            identifier: param.name,
                            type_: Type::from_intermediate_type(itype, modules_data)?,
                        });
                    }
                }
            }

            let return_type = if function.signature.returns.is_empty() {
                Type::None
            } else if function.signature.returns.len() == 1 {
                Self::process_return_type(
                    &function.signature.returns[0],
                    modules_data,
                    structs_to_process,
                    enums_to_process,
                )?;

                Type::from_intermediate_type(&function.signature.returns[0], modules_data)?
            } else {
                for t in &function.signature.returns {
                    Self::process_return_type(
                        t,
                        modules_data,
                        structs_to_process,
                        enums_to_process,
                    )?;
                }
                let tuple_types: Vec<Type> = function
                    .signature
                    .returns
                    .iter()
                    .map(|t| Type::from_intermediate_type(t, modules_data))
                    .collect::<Result<Vec<_>, AbiGeneratorError>>()?;
                Type::Tuple(tuple_types)
            };

            let visibility = if parsed_function.visibility
                == move_parse_special_attributes::function_modifiers::Visibility::Public
            {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Function name
            let function_name = if function_type == FunctionType::Constructor {
                Symbol::from("constructor")
            } else {
                Symbol::from(snake_to_camel(&function.function_id.identifier))
            };

            result.push(Function {
                function_type,
                identifier: function_name,
                parameters: function_parameters,
                return_types: return_type,
                is_entry: function.is_entry,
                modifiers: parsed_function.modifiers.clone(),
                visibility,
            });
        }
        Ok(result)
    }

    fn process_storage_struct(
        struct_: &IStruct,
        struct_itype: &IntermediateType,
        module_id: ModuleId,
        modules_data: &HashMap<ModuleId, ModuleData>,
        function_parameters: &mut Vec<NamedType>,
        param: &Parameter,
    ) -> Result<(), AbiGeneratorError> {
        assert!(struct_.has_key);

        // 1. Identify the first field and its metadata
        let first_field = struct_.fields.first().ok_or(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::StorageStructNoFields(struct_.identifier, module_id),
        })?;

        let (m_id, index, is_generic) = match first_field {
            IntermediateType::IStruct {
                module_id, index, ..
            } => (module_id, index, false),
            IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } => (module_id, index, true),
            _ => {
                return Err(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StorageStructInvalidFirstField(
                        struct_.identifier,
                        module_id,
                        format!("{STYLUS_FW_UID} or {STYLUS_FW_NAMED_ID}"),
                        format!("{first_field:?}"),
                    ),
                });
            }
        };

        // 2. Resolve the struct definition
        let struct_module = modules_data.get(m_id).ok_or(AbiGeneratorError {
            kind: AbiGeneratorErrorKind::ModuleDataNotFound(*m_id),
        })?;
        let field_struct =
            struct_module
                .structs
                .get_by_index(*index)
                .map_err(|_| AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StructNotFoundByIndex(*index, *m_id),
                })?;

        let is_stylus_fw = m_id.address == STYLUS_FRAMEWORK_ADDRESS
            && m_id.module_name.as_str() == SF_MODULE_NAME_OBJECT;
        let ident = field_struct.identifier.as_str();

        // 3. Strict requirement validation
        match (ident, is_generic, is_stylus_fw) {
            (STYLUS_FW_UID, false, true) => {
                // Success case for UID
                function_parameters.push(NamedType {
                    identifier: param.name,
                    type_: Type::from_intermediate_type(struct_itype, modules_data)?,
                });
            }
            (STYLUS_FW_UID, true, true) => {
                // Expected NamedId, but got UID
                return Err(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StorageStructInvalidFirstField(
                        struct_.identifier,
                        module_id,
                        STYLUS_FW_NAMED_ID.to_string(),
                        STYLUS_FW_UID.to_string(),
                    ),
                });
            }
            (STYLUS_FW_NAMED_ID, true, true) => {
                // Success case for NamedId (ignored)
            }
            (STYLUS_FW_NAMED_ID, false, true) => {
                // Expected UID, but got NamedId
                return Err(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StorageStructInvalidFirstField(
                        struct_.identifier,
                        module_id,
                        STYLUS_FW_UID.to_string(),
                        STYLUS_FW_NAMED_ID.to_string(),
                    ),
                });
            }
            _ => {
                return Err(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StorageStructInvalidFirstField(
                        struct_.identifier,
                        module_id,
                        format!("{STYLUS_FW_UID} or {STYLUS_FW_NAMED_ID}"),
                        ident.to_string(),
                    ),
                });
            }
        }

        Ok(())
    }

    pub fn process_structs(
        structs: HashSet<IntermediateType>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        processed_structs: &mut HashSet<IntermediateType>,
    ) -> Result<Vec<Struct_>, AbiGeneratorError> {
        let mut result = Vec::new();

        for itype in structs {
            // Atomic check-and-insert to prevent infinite recursion and duplicates
            if !processed_structs.insert(itype.clone()) {
                continue;
            }

            // 1. Resolve Location and Generics
            let (module_id, index, types) = match &itype {
                IntermediateType::IStruct {
                    module_id, index, ..
                } => (module_id, index, None),
                IntermediateType::IGenericStructInstance {
                    module_id,
                    index,
                    types,
                    ..
                } => (module_id, index, Some(types)),
                t => {
                    return Err(AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::ExpectedStructType(format!("{t:?}")),
                    });
                }
            };

            // 2. Resolve Module and Base Struct
            let module_data = modules_data.get(module_id).ok_or(AbiGeneratorError {
                kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
            })?;

            let base_struct =
                module_data
                    .structs
                    .get_by_index(*index)
                    .map_err(|_| AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::StructNotFoundByIndex(*index, *module_id),
                    })?;

            // 3. Instantiate if needed and get parsed metadata from the special attributes
            let struct_ = types.map_or_else(|| base_struct.clone(), |t| base_struct.instantiate(t));

            let parsed_struct = module_data
                .special_attributes
                .structs
                .iter()
                .find(|s| *s.name == *struct_.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ParsedStructNotFound(
                        struct_.identifier,
                        *module_id,
                    ),
                })?;

            // 4. Process fields and discover children
            let mut child_structs = HashSet::new();
            let fields: Vec<NamedType> = struct_
                .fields
                .iter()
                .zip(&parsed_struct.fields)
                .map(|(field_itype, (name, _))| {
                    if Self::should_process_struct(field_itype, modules_data)? {
                        child_structs.insert(field_itype.clone());
                    }
                    Ok(NamedType {
                        identifier: *name,
                        type_: Type::from_intermediate_type(field_itype, modules_data)?,
                    })
                })
                .collect::<Result<Vec<_>, AbiGeneratorError>>()?;

            // 5. Build ABI Struct and Recurse
            result.push(Struct_ {
                identifier: Type::from_intermediate_type(&itype, modules_data)?.name(),
                fields,
                positional_fields: parsed_struct.positional_fields,
            });

            // Recurse for nested structs found in fields
            if !child_structs.is_empty() {
                result.extend(Self::process_structs(
                    child_structs,
                    modules_data,
                    processed_structs,
                )?);
            }
        }

        Ok(result)
    }

    pub fn process_enums(
        enums: HashSet<IntermediateType>,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> Result<Vec<Enum_>, AbiGeneratorError> {
        let mut result = Vec::new();

        for itype in enums {
            // 1. Resolve Location and Generics
            let (module_id, index, types) = match &itype {
                IntermediateType::IEnum {
                    module_id, index, ..
                } => (module_id, index, None),
                IntermediateType::IGenericEnumInstance {
                    module_id,
                    index,
                    types,
                    ..
                } => (module_id, index, Some(types)),
                t => {
                    return Err(AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::ExpectedEnumType(format!("{t:?}")),
                    });
                }
            };

            // 2. Resolve Module and Base Enum safely
            let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
            })?;

            let base_enum =
                enum_module
                    .enums
                    .get_by_index(*index)
                    .map_err(|_| AbiGeneratorError {
                        kind: AbiGeneratorErrorKind::EnumNotFoundByIndex(*index, *module_id),
                    })?;

            // 3. Instantiate if needed and get parsed metadata from the special attributes
            let enum_ = types.map_or_else(|| base_enum.clone(), |t| base_enum.instantiate(t));

            let parsed_enum = enum_module
                .special_attributes
                .enums
                .iter()
                .find(|e| *e.name == *enum_.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ParsedEnumNotFound(enum_.identifier, *module_id),
                })?;

            // 4. Build ABI Enum
            result.push(Enum_ {
                identifier: enum_.identifier,
                variants: parsed_enum.variants.iter().map(|v| v.0).collect(),
            });
        }

        Ok(result)
    }

    pub fn process_events(
        events: &HashSet<EventStruct>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) -> Result<Vec<Event>, AbiGeneratorError> {
        let mut result = Vec::new();

        for event in events {
            // 1. Resolve Module Data
            let event_module_id = ModuleId::new(
                event.module_id.address().into_bytes().into(),
                event.module_id.name().as_str(),
            );
            let event_module = modules_data
                .get(&event_module_id)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound(event_module_id),
                })?;

            // 2. Resolve and Instantiate the underlying Struct
            let base_struct = event_module
                .structs
                .get_by_identifier(&event.identifier)
                .map_err(|_| AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StructNotFoundByIdentifier(
                        event.identifier,
                        event_module_id,
                    ),
                })?;

            let event_struct = match &event.type_parameters {
                Some(params) => base_struct.instantiate(params),
                None => base_struct.clone(),
            };

            // 3. Get parsed event and struct from the special attributes
            let parsed_event = event_module
                .special_attributes
                .events
                .get(&event_struct.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ParsedEventNotFound(
                        event_struct.identifier,
                        event_module_id,
                    ),
                })?;

            let parsed_struct = event_module
                .special_attributes
                .structs
                .iter()
                .find(|s| *s.name == *event_struct.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ParsedStructNotFound(
                        event_struct.identifier,
                        event_module_id,
                    ),
                })?;

            // 4. Collect nested structs for later processing
            for field_itype in &event_struct.fields {
                if Self::should_process_struct(field_itype, modules_data)? {
                    structs_to_process.insert(field_itype.clone());
                }
            }

            // 5. Build the Event ABI
            result.push(Event {
                identifier: event.identifier,
                fields: event_struct
                    .fields
                    .iter()
                    .zip(&parsed_struct.fields)
                    .enumerate()
                    .map(|(i, (itype, (name, _)))| {
                        Ok(EventField {
                            named_type: NamedType {
                                identifier: *name,
                                type_: Type::from_intermediate_type(itype, modules_data)?,
                            },
                            // Stylus/Move events usually index the first N fields
                            indexed: i < parsed_event.indexes as usize,
                        })
                    })
                    .collect::<Result<Vec<_>, AbiGeneratorError>>()?,
                is_anonymous: parsed_event.is_anonymous,
                positional_fields: parsed_struct.positional_fields,
            });
        }

        Ok(result)
    }

    pub fn process_abi_errors(
        error_structs: &HashSet<ErrorStruct>,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
    ) -> Result<Vec<Struct_>, AbiGeneratorError> {
        let mut result = Vec::new();

        for error in error_structs {
            // 1. Resolve Module Data
            let error_module_id = ModuleId::new(
                error.module_id.address().into_bytes().into(),
                error.module_id.name().as_str(),
            );
            let error_module = modules_data
                .get(&error_module_id)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound(error_module_id),
                })?;

            // 2. Resolve Bytecode Struct
            let struct_def = error_module
                .structs
                .get_by_identifier(&error.identifier)
                .map_err(|_| AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::StructNotFoundByIdentifier(
                        error.identifier,
                        error_module_id,
                    ),
                })?;

            // 3. Get parsed struct from the special attributes
            let parsed_struct = error_module
                .special_attributes
                .structs
                .iter()
                .find(|s| *s.name == *struct_def.identifier)
                .ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ParsedStructNotFound(
                        struct_def.identifier,
                        error_module_id,
                    ),
                })?;

            // 4. Update the "Structs to Process" set for deep resolution
            for field_itype in &struct_def.fields {
                if Self::should_process_struct(field_itype, modules_data)? {
                    structs_to_process.insert(field_itype.clone());
                }
            }

            // 5. Transform into ABI Struct
            result.push(Struct_ {
                identifier: struct_def.identifier,
                fields: struct_def
                    .fields
                    .iter()
                    .zip(&parsed_struct.fields)
                    .map(|(itype, (name, _))| {
                        Ok(NamedType {
                            identifier: *name,
                            type_: Type::from_intermediate_type(itype, modules_data)?,
                        })
                    })
                    .collect::<Result<Vec<_>, AbiGeneratorError>>()?,
                positional_fields: parsed_struct.positional_fields,
            });
        }

        Ok(result)
    }

    /// Helper function to check if a struct type should be added to the process HashSet.
    /// Returns true if the struct is not a named_id, uid or string, false otherwise.
    fn should_process_struct(
        itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> Result<bool, AbiGeneratorError> {
        match itype {
            IntermediateType::IStruct {
                module_id, index, ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } => {
                let struct_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                })?;
                let struct_ =
                    struct_module
                        .structs
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::StructNotFoundByIndex(*index, *module_id),
                        })?;

                // True if the struct is not a named_id, uid or string
                Ok(!is_named_id(&struct_.identifier, module_id)
                    && !is_uid(&struct_.identifier, module_id)
                    && !is_id(&struct_.identifier, module_id)
                    && !is_string(&struct_.identifier, module_id)
                    && !is_bytes_n(&struct_.identifier, module_id))
            }

            _ => Ok(false),
        }
    }

    fn process_return_type(
        itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
        structs_to_process: &mut HashSet<IntermediateType>,
        enums_to_process: &mut HashSet<IntermediateType>,
    ) -> Result<(), AbiGeneratorError> {
        match itype {
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                Self::process_return_type(
                    inner.as_ref(),
                    modules_data,
                    structs_to_process,
                    enums_to_process,
                )?;
            }
            IntermediateType::IEnum { module_id, index } => {
                let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                })?;
                let enum_ =
                    enum_module
                        .enums
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::EnumNotFoundByIndex(*index, *module_id),
                        })?;
                if enum_.is_simple {
                    enums_to_process.insert(itype.clone());
                }
            }
            IntermediateType::IGenericEnumInstance {
                module_id,
                index,
                types,
            } => {
                let enum_module = modules_data.get(module_id).ok_or(AbiGeneratorError {
                    kind: AbiGeneratorErrorKind::ModuleDataNotFound(*module_id),
                })?;
                let enum_ =
                    enum_module
                        .enums
                        .get_by_index(*index)
                        .map_err(|_| AbiGeneratorError {
                            kind: AbiGeneratorErrorKind::EnumNotFoundByIndex(*index, *module_id),
                        })?;
                let enum_ = enum_.instantiate(types);
                if enum_.is_simple {
                    enums_to_process.insert(itype.clone());
                }
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. }
                if Self::should_process_struct(itype, modules_data)? =>
            {
                structs_to_process.insert(itype.clone());
            }
            _ => {}
        }
        Ok(())
    }
}
