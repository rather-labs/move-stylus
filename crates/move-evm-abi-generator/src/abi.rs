use std::collections::{HashMap, HashSet};

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId,
    module_data::struct_data::IntermediateType,
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, SF_MODULE_NAME_TX_CONTEXT, STYLUS_FRAMEWORK_ADDRESS,
    },
};
use move_parse_special_attributes::function_modifiers::FunctionModifier;

use crate::{common::snake_to_camel, types::Type};

#[derive(Debug)]
pub struct Abi {
    pub(crate) functions: Vec<Function>,
}

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

/// This contains all the structs that appear as argument o return of functions. Once we
/// process the functions this will be the structs appearing in the ABi
pub(crate) fn process_functions(
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
                IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => inner.as_ref(),
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
                            function_parameters.push(FunctionParameters {
                                identifier: param.name.clone(),
                                type_: Type::from_intermediate_type(itype, modules_data),
                            });
                            struct_to_process.insert(itype.clone());
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
                    let struct_ = struct_module.structs.get_by_index(*index).unwrap();
                    match (
                        struct_.identifier.as_str(),
                        module_id.address,
                        module_id.module_name.as_str(),
                    ) {
                        ("NamedId", STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {
                            continue;
                        }
                        _ => {
                            function_parameters.push(FunctionParameters {
                                identifier: param.name.clone(),
                                type_: Type::from_intermediate_type(itype, modules_data),
                            });
                            struct_to_process.insert(itype.clone());
                        }
                    }
                }
                IntermediateType::IEnum { module_id, index } => todo!(),
                IntermediateType::IGenericEnumInstance {
                    module_id,
                    index,
                    types,
                } => todo!(),
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
            Type::from_intermediate_type(&function.signature.returns[0], modules_data)
        } else {
            Type::Tuple(
                function
                    .signature
                    .returns
                    .iter()
                    .map(|t| Type::from_intermediate_type(t, modules_data))
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
