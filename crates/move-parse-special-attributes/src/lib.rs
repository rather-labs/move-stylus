pub mod event;
mod external_call;
pub mod function_modifiers;

pub use event::Event;

#[derive(Default, Debug)]
pub struct SpecialAttributes {
    pub events: HashMap<String, Event>,
    pub functions: Vec<Function>,
    pub external_calls: HashMap<String, Function>,
}

use external_call::validate_external_call_function;
use function_modifiers::{Function, FunctionModifier};
use move_compiler::{
    Compiler, PASS_PARSER,
    parser::ast::{Definition, ModuleMember},
    shared::NumericalAddress,
};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    path::Path,
};

pub fn process_special_attributes(path: &Path) -> Result<SpecialAttributes, Vec<String>> {
    let (_, program_res) = Compiler::from_files(
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

    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(f) => {
                        if let Some(attributes) = f.attributes.first() {
                            let mut modifiers = attributes
                                .value
                                .iter()
                                .flat_map(|s| FunctionModifier::parse_modifiers(&s.value))
                                .collect::<VecDeque<FunctionModifier>>();

                            if let Some(FunctionModifier::ExternalCall) = modifiers.pop_front() {
                                let modifiers: Vec<FunctionModifier> =
                                    modifiers.into_iter().collect();

                                let errors = validate_external_call_function(&f, &modifiers);

                                if let Err(errors) = errors {
                                    found_error = true;
                                    module_errors.extend(errors);
                                } else if !found_error {
                                    result.external_calls.insert(
                                        f.name.to_owned().to_string(),
                                        Function {
                                            name: f.name.to_owned().to_string(),
                                            modifiers,
                                        },
                                    );
                                }
                            } else {
                                result.functions.push(Function {
                                    name: f.name.to_owned().to_string(),
                                    modifiers: modifiers.into_iter().collect(),
                                });
                            }
                        }
                    }
                    ModuleMember::Struct(ref s) => {
                        if let Ok(event) = Event::try_from(s) {
                            result.events.insert(s.name.to_string(), event);
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
        Err(module_errors)
    } else {
        Ok(result)
    }
}
