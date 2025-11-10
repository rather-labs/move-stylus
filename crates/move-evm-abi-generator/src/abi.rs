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
    common::snake_to_camel,
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
    pub(crate) parameters: Vec<FunctionParameters>,
    pub(crate) return_types: Type,
    pub(crate) visibility: Visibility,
    pub(crate) modifiers: Vec<FunctionModifier>,
    pub(crate) is_entry: bool,
}

#[derive(Debug)]
pub struct FunctionParameters {
    pub(crate) identifier: String,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub struct Struct_ {
    pub(crate) identifier: String,
    pub(crate) fields: Vec<StructField>,
}

#[derive(Debug)]
pub struct StructField {
    pub(crate) identifier: String,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub struct Abi {
    pub(crate) contract_name: String,
    pub(crate) functions: Vec<Function>,
    pub(crate) structs: Vec<Struct_>,
}

impl Abi {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.structs.is_empty()
    }

    pub(crate) fn new(
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> Abi {
        let (functions, structs_to_process) =
            Self::process_functions(processing_module, modules_data);

        let mut processed_structs = HashSet::new();
        let structs =
            Self::process_structs(structs_to_process, modules_data, &mut processed_structs);

        Abi {
            contract_name: processing_module.special_attributes.module_name.clone(),
            functions,
            structs,
        }
    }

    /// This contains all the structs that appear as argument o return of functions. Once we
    /// process the functions this will be the structs appearing in the ABi
    fn process_functions(
        processing_module: &ModuleData,
        modules_data: &HashMap<ModuleId, ModuleData>,
    ) -> (Vec<Function>, HashSet<IntermediateType>) {
        let mut result = Vec::new();
        let mut struct_to_process = HashSet::new();

        // First we filter the functions we are ging to process
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
                                    Self::process_storage_struct(
                                        struct_,
                                        itype,
                                        modules_data,
                                        &mut function_parameters,
                                        param,
                                        &mut struct_to_process,
                                    );
                                } else {
                                    {
                                        function_parameters.push(FunctionParameters {
                                            identifier: param.name.clone(),
                                            type_: Type::from_intermediate_type(
                                                itype,
                                                modules_data,
                                            ),
                                        });
                                        struct_to_process.insert(itype.clone());
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
                                &mut struct_to_process,
                            );
                        } else {
                            {
                                function_parameters.push(FunctionParameters {
                                    identifier: param.name.clone(),
                                    type_: Type::from_intermediate_type(itype, modules_data),
                                });
                                struct_to_process.insert(itype.clone());
                            }
                        }
                    }
                    IntermediateType::IEnum { module_id, index } => {
                        let enum_module = modules_data.get(module_id).unwrap();
                        let enum_ = enum_module.enums.get_by_index(*index).unwrap();
                        if !enum_.is_simple {
                            panic!("found not simple enum in function signature");
                        } else {
                            function_parameters.push(FunctionParameters {
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
                            function_parameters.push(FunctionParameters {
                                identifier: param.name.clone(),
                                type_: Type::from_intermediate_type(itype, modules_data),
                            });
                        }
                    }
                    _ => {
                        function_parameters.push(FunctionParameters {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(itype, modules_data),
                        });
                    }
                }
            }

            let return_type = if function.signature.returns.is_empty() {
                Type::None
            } else if function.signature.returns.len() == 1 {
                match &function.signature.returns[0] {
                    IntermediateType::IGenericStructInstance {
                        module_id, index, ..
                    }
                    | IntermediateType::IStruct {
                        module_id, index, ..
                    } => {
                        let struct_module = modules_data.get(module_id).unwrap();
                        let struct_ = struct_module.structs.get_by_index(*index).unwrap();

                        if !is_named_id(&struct_.identifier, module_id)
                            && !is_uid(&struct_.identifier, module_id)
                        {
                            struct_to_process.insert(function.signature.returns[0].clone());
                        }
                    }
                    _ => {}
                }

                Type::from_intermediate_type(&function.signature.returns[0], modules_data)
            } else {
                Type::Tuple(
                    function
                        .signature
                        .returns
                        .iter()
                        .map(|t| {
                            match &function.signature.returns[0] {
                                IntermediateType::IGenericStructInstance {
                                    module_id,
                                    index,
                                    ..
                                }
                                | IntermediateType::IStruct {
                                    module_id, index, ..
                                } => {
                                    let struct_module = modules_data.get(module_id).unwrap();
                                    let struct_ =
                                        struct_module.structs.get_by_index(*index).unwrap();

                                    if !is_named_id(&struct_.identifier, module_id)
                                        && !is_uid(&struct_.identifier, module_id)
                                    {
                                        struct_to_process
                                            .insert(function.signature.returns[0].clone());
                                    }
                                }
                                _ => {}
                            }
                            Type::from_intermediate_type(t, modules_data)
                        })
                        .collect(),
                )
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
        (result, struct_to_process)
    }

    fn process_storage_struct(
        struct_: &IStruct,
        struct_itype: &IntermediateType,
        modules_data: &HashMap<ModuleId, ModuleData>,
        function_parameters: &mut Vec<FunctionParameters>,
        param: &Parameter,
        struct_to_process: &mut HashSet<IntermediateType>,
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
                        function_parameters.push(FunctionParameters {
                            identifier: param.name.clone(),
                            type_: Type::Bytes32,
                        });
                    }
                    _ => {
                        function_parameters.push(FunctionParameters {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(struct_itype, modules_data),
                        });
                        struct_to_process.insert(struct_itype.clone());
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
                        function_parameters.push(FunctionParameters {
                            identifier: param.name.clone(),
                            type_: Type::from_intermediate_type(struct_itype, modules_data),
                        });
                        struct_to_process.insert(struct_itype.clone());
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

            let (struct_, parsed_struct) = match &struct_itype {
                IntermediateType::IStruct {
                    module_id, index, ..
                } => {
                    let struct_module = modules_data.get(module_id).unwrap();
                    let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                    let parsed_struct = struct_module
                        .special_attributes
                        .structs
                        .iter()
                        .find(|s| s.name == struct_.identifier)
                        .unwrap();

                    (struct_.clone(), parsed_struct)
                }
                IntermediateType::IGenericStructInstance {
                    module_id,
                    index,
                    types,
                    ..
                } => {
                    let struct_module = modules_data.get(module_id).unwrap();
                    let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                    let struct_ = struct_.instantiate(types);
                    let parsed_struct = struct_module
                        .special_attributes
                        .structs
                        .iter()
                        .find(|s| s.name == struct_.identifier)
                        .unwrap();

                    (struct_, parsed_struct)
                }
                t => panic!("found {t:?} instead of struct"),
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
                    StructField {
                        identifier: name.clone(),
                        type_: Type::from_intermediate_type(field_itype, modules_data),
                    }
                })
                .collect();

            let struct_abi_type = Type::from_intermediate_type(&struct_itype, modules_data);
            result.push(Struct_ {
                identifier: struct_abi_type.name(),
                fields,
            });

            processed_structs.insert(struct_itype);

            // Process child structs
            let child_structs =
                Self::process_structs(child_structs_to_process, modules_data, processed_structs);

            result.extend(child_structs);
        }

        result
    }
}
