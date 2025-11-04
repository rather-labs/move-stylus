pub mod error;
pub mod event;
mod external_call;
pub mod function_modifiers;
mod shared;
pub mod struct_modifiers;

pub use error::SpecialAttributeError;
use error::SpecialAttributeErrorKind;
pub use event::Event;
use event::EventParseError;
pub use external_call::error::{ExternalCallFunctionError, ExternalCallStructError};
// TODO: Create error struct with LOC and error info

#[derive(Default, Debug)]
pub struct SpecialAttributes {
    pub events: HashMap<String, Event>,
    pub functions: Vec<Function>,
    pub external_calls: HashMap<String, Function>,
    pub external_struct: HashMap<String, ExternalStruct>,
    pub external_call_structs: HashSet<String>,
}

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
            for module_member in &module.members {
                match module_member {
                    ModuleMember::Struct(s) => {
                        if let Some(attributes) = s.attributes.first() {
                            let first_modifier = attributes.value.first().and_then(|s| {
                                let sm = StructModifier::parse_modifiers(&s.value);
                                sm.first().cloned()
                            });

                            match first_modifier {
                                Some(StructModifier::ExternalCall) => {
                                    match validate_external_call_struct(s) {
                                        Ok(_)
                                            if !result
                                                .external_call_structs
                                                .contains(s.name.value().as_str()) =>
                                        {
                                            result
                                                .external_call_structs
                                                .insert(s.name.value().to_string());
                                        }
                                        Ok(_) => continue,
                                        Err(e) => {
                                            found_error = true;
                                            module_errors.extend(e);
                                        }
                                    }
                                }
                                Some(StructModifier::ExternalStruct) => {
                                    match ExternalStruct::try_from(s) {
                                        Ok(external_struct) => {
                                            result
                                                .external_struct
                                                .insert(s.name.to_string(), external_struct);
                                        }
                                        Err(SpecialAttributeError {
                                            kind:
                                                SpecialAttributeErrorKind::ExternalStruct(
                                                    ExternalStructError::NotAnExternalStruct,
                                                ),
                                            ..
                                        }) => continue,
                                        Err(e) => {
                                            found_error = true;
                                            module_errors.push(e);
                                        }
                                    }
                                }
                                Some(StructModifier::Event) => match Event::try_from(s) {
                                    Ok(event) => {
                                        result.events.insert(s.name.to_string(), event);
                                    }
                                    Err(SpecialAttributeError {
                                        kind:
                                            SpecialAttributeErrorKind::Event(
                                                EventParseError::NotAnEvent,
                                            ),
                                        ..
                                    }) => continue,
                                    Err(e) => {
                                        found_error = true;
                                        module_errors.push(e);
                                    }
                                },
                                None => continue,
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

    // println!("{:#?}", ast.source_definitions);
    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(ref f) => {
                        // println!("{f:#?}");
                        let is_entry = f.entry.is_some();
                        let visibility: Visibility = (&f.visibility).into();

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
                                                is_entry,
                                                visibility,
                                            },
                                        );
                                    }
                                }
                                Some(FunctionModifier::Abi) => {
                                    println!("ACA? {modifiers:?}");
                                    let modifiers: Vec<FunctionModifier> =
                                        modifiers.into_iter().collect();

                                    result.functions.push(Function {
                                        name: f.name.to_owned().to_string(),
                                        modifiers,
                                        is_entry,
                                        visibility,
                                    });
                                }
                                _ => {}
                            }
                        } else {
                            result.functions.push(Function {
                                name: f.name.to_owned().to_string(),
                                modifiers: Vec::new(),
                                is_entry,
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
