pub mod abi_error;
pub mod error;
pub mod event;
mod external_call;
pub mod function_modifiers;
mod shared;
pub mod struct_modifiers;
pub mod types;

pub use abi_error::AbiError;
pub use abi_error::AbiErrorParseError;
pub use error::SpecialAttributeError;
use error::SpecialAttributeErrorKind;
pub use event::Event;
use event::EventParseError;
pub use external_call::error::{ExternalCallFunctionError, ExternalCallStructError};
// TODO: Create error struct with LOC and error info

use external_call::{
    external_struct::{ExternalStruct, ExternalStructError},
    validate_external_call_function, validate_external_call_struct,
};
use function_modifiers::{Function, FunctionModifier, Visibility};
use move_compiler::{
    Compiler, PASS_PARSER,
    parser::ast::{Definition, ModuleMember},
    shared::{Identifier, NumericalAddress, files::MappedFiles},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    path::Path,
};
use struct_modifiers::StructModifier;
use types::Type;

#[derive(Debug)]
pub struct Struct_ {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Default, Debug)]
pub struct SpecialAttributes {
    pub module_name: String,
    pub events: HashMap<String, Event>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct_>,
    pub external_calls: HashMap<String, Function>,
    pub external_struct: HashMap<String, ExternalStruct>,
    pub external_call_structs: HashSet<String>,
    pub abi_errors: HashMap<String, AbiError>,
}

pub fn process_special_attributes(
    path: &Path,
) -> Result<SpecialAttributes, (MappedFiles, Vec<SpecialAttributeError>)> {
    let (mapped_files, program_res) = Compiler::from_files(
        None,
        vec![path.to_str().unwrap()],
        Vec::new(),
        BTreeMap::<String, NumericalAddress>::new(),
    )
    .run::<PASS_PARSER>()
    .unwrap();

    let mut result = SpecialAttributes::default();
    let mut module_errors = Vec::new();

    let mut found_error = false;

    let ast = program_res.unwrap().into_ast().1;
    // First we need to process the structs, since there are functions (like the external call
    // ones) that should have as first argument structs marked with a modifier.
    for source in &ast.source_definitions {
        if let Definition::Module(ref module) = source.def {
            result.module_name = module.name.0.to_string();
            for module_member in &module.members {
                match module_member {
                    ModuleMember::Struct(s) => {
                        let struct_name = s.name.value().as_str().to_string();

                        // No matter if it is a struct marked with special attributes, we collect
                        // its information.
                        let fields: Vec<(String, Type)> = match &s.fields {
                            move_compiler::parser::ast::StructFields::Named(items) => items
                                .iter()
                                .map(|(_, field, type_)| {
                                    let name = field.value();
                                    (name.to_string(), Type::parse_type(&type_.value))
                                })
                                .collect(),
                            move_compiler::parser::ast::StructFields::Positional(items) => items
                                .iter()
                                .enumerate()
                                .map(|(index, (_, type_))| {
                                    (format!("pos{index}"), Type::parse_type(&type_.value))
                                })
                                .collect(),
                            move_compiler::parser::ast::StructFields::Native(_) => todo!(),
                        };

                        result.structs.push(Struct_ {
                            name: struct_name.clone(),
                            fields,
                        });
                        
                        let mut found_match: Option<StructModifier> = None;
                        'outer: for attributes in &s.attributes {
                            if let Some(att) = attributes.value.first() {
                                let modifiers = StructModifier::parse_modifiers(&att.value);
                                for modifier in modifiers {
                                    if found_match.is_some() {
                                        // Found a second match
                                        found_error = true;
                                        module_errors.push(SpecialAttributeError {
                                            kind: SpecialAttributeErrorKind::TooManyAttributes,
                                            line_of_code: s.loc,
                                        });
                                        break 'outer;
                                    }

                                    match modifier {
                                        StructModifier::ExternalCall => {
                                            match validate_external_call_struct(s) {
                                                Ok(_)
                                                    if !result
                                                        .external_call_structs
                                                        .contains(&struct_name) =>
                                                {
                                                    result.external_call_structs.insert(struct_name.clone());
                                                    found_match = Some(StructModifier::ExternalCall);
                                                }
                                                Ok(_) => {
                                                    found_match = Some(StructModifier::ExternalCall);
                                                }
                                                Err(e) => {
                                                    found_error = true;
                                                    module_errors.extend(e);
                                                    break 'outer;
                                                }
                                            }
                                        }
                                        StructModifier::ExternalStruct => {
                                            match ExternalStruct::try_from(s) {
                                                Ok(external_struct) => {
                                                    result
                                                        .external_struct
                                                        .insert(struct_name.clone(), external_struct);
                                                    found_match = Some(StructModifier::ExternalStruct);
                                                }
                                                Err(SpecialAttributeError {
                                                    kind:
                                                        SpecialAttributeErrorKind::ExternalStruct(
                                                            ExternalStructError::NotAnExternalStruct,
                                                        ),
                                                    ..
                                                }) => {}
                                                Err(e) => {
                                                    found_error = true;
                                                    module_errors.push(e);
                                                    break 'outer;
                                                }
                                            }
                                        }
                                        StructModifier::Event => {
                                            match Event::try_from(s) {
                                                Ok(event) => {
                                                    result.events.insert(struct_name.clone(), event);
                                                    found_match = Some(StructModifier::Event);
                                                }
                                                Err(SpecialAttributeError {
                                                    kind:
                                                        SpecialAttributeErrorKind::Event(
                                                            EventParseError::NotAnEvent,
                                                        ),
                                                    ..
                                                }) => {}
                                                Err(e) => {
                                                    found_error = true;
                                                    module_errors.push(e);
                                                    break 'outer;
                                                }
                                            }
                                        }
                                        StructModifier::AbiError => {
                                            match AbiError::try_from(s) {
                                                Ok(abi_error) => {
                                                    result.abi_errors.insert(struct_name.clone(), abi_error);
                                                    found_match = Some(StructModifier::AbiError);
                                                }
                                                Err(SpecialAttributeError {
                                                    kind:
                                                        SpecialAttributeErrorKind::AbiError(
                                                            AbiErrorParseError::NotAnAbiError,
                                                        ),
                                                    ..
                                                }) => {}
                                                Err(e) => {
                                                    found_error = true;
                                                    module_errors.push(e);
                                                    break 'outer;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => continue,
                }
            }
        } else {
            continue;
        };
    }

    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(ref f) => {
                        let visibility: Visibility = (&f.visibility).into();
                        let signature = Function::parse_signature(&f.signature);

                        if let Some(attributes) = f.attributes.first() {
                            let mut modifiers = attributes
                                .value
                                .iter()
                                .flat_map(|s| FunctionModifier::parse_modifiers(&s.value))
                                .collect::<VecDeque<FunctionModifier>>();

                            match modifiers.pop_front() {
                                Some(FunctionModifier::ExternalCall) => {
                                    let modifiers: Vec<FunctionModifier> =
                                        modifiers.into_iter().collect();

                                    let errors = validate_external_call_function(
                                        f,
                                        &modifiers,
                                        &result.external_call_structs,
                                    );

                                    if let Err(errors) = errors {
                                        found_error = true;
                                        module_errors.extend(errors);
                                    } else if !found_error {
                                        result.external_calls.insert(
                                            f.name.to_owned().to_string(),
                                            Function {
                                                name: f.name.to_owned().to_string(),
                                                modifiers,
                                                signature,
                                                visibility,
                                            },
                                        );
                                    }
                                }
                                Some(FunctionModifier::Abi) => {
                                    let modifiers: Vec<FunctionModifier> =
                                        modifiers.into_iter().collect();

                                    result.functions.push(Function {
                                        name: f.name.to_owned().to_string(),
                                        modifiers,
                                        signature,
                                        visibility,
                                    });
                                }
                                _ => {}
                            }
                        } else {
                            result.functions.push(Function {
                                name: f.name.to_owned().to_string(),
                                modifiers: Vec::new(),
                                signature,
                                visibility,
                            });
                        }
                    }
                    _ => continue,
                }
            }
        } else {
            continue;
        };
    }

    if found_error {
        Err((mapped_files, module_errors))
    } else {
        Ok(result)
    }
}
