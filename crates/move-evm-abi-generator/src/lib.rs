//! Parses the AST of a package to extract the ABI
//!
//! NOTE: This is a POC and it is WIP
mod function_modifiers;

use function_modifiers::FunctionModifier;
use move_compiler::{
    Compiler, PASS_PARSER,
    parser::ast::{Definition, ModuleMember},
    shared::NumericalAddress,
};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug)]
struct Function {
    name: String,
    modifiers: Vec<FunctionModifier>,
}

pub fn generate_abi(path: Option<&Path>) {
    let sources_path = path.unwrap().join("sources");

    let (_, program_res) = Compiler::from_files(
        None,
        vec![sources_path.to_str().unwrap()],
        Vec::new(),
        BTreeMap::<String, NumericalAddress>::new(),
    )
    .run::<PASS_PARSER>()
    .unwrap();

    let ast = program_res.unwrap().into_ast().1;

    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            println!("{:#?}", module);
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(f) => {
                        let modifiers = f.attributes[0]
                            .value
                            .iter()
                            .flat_map(|s| FunctionModifier::parse_modifiers(&s.value))
                            .collect::<Vec<FunctionModifier>>();

                        let function = Function {
                            name: f.name.to_owned().to_string(),
                            modifiers,
                        };

                        println!("{function:#?}");
                    }
                    _ => continue,
                }
            }
        } else {
            continue;
        };
    }
}
