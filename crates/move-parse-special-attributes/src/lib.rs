pub mod event;

pub use event::Event;

#[derive(Default)]
pub struct SpecialAttributes {
    pub events: HashMap<String, Event>,
}

use move_compiler::{
    Compiler, PASS_PARSER,
    parser::ast::{Definition, ModuleMember},
    shared::NumericalAddress,
};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

pub fn process_special_attributes(path: &Path) -> SpecialAttributes {
    let (_, program_res) = Compiler::from_files(
        None,
        vec![path.to_str().unwrap()],
        Vec::new(),
        BTreeMap::<String, NumericalAddress>::new(),
    )
    .run::<PASS_PARSER>()
    .unwrap();

    let mut result = SpecialAttributes::default();

    let ast = program_res.unwrap().into_ast().1;

    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            // println!("{:#?}", module);
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(f) => {
                        // println!("found function!");
                    }
                    ModuleMember::Struct(ref s) => {
                        println!("Processing struct {}", s.name);
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

    result
}
