pub mod abi_error;
pub mod error;
pub mod event;
mod external_call;
pub mod function_modifiers;
mod function_validation;
pub mod reserved_modules;
mod shared;
pub mod struct_modifiers;
mod struct_validation;
pub mod types;

pub use abi_error::AbiError;
pub use abi_error::AbiErrorParseError;
pub use error::SpecialAttributeError;
use error::SpecialAttributeErrorKind;
pub use event::Event;
use event::EventParseError;
pub use external_call::error::{ExternalCallFunctionError, ExternalCallStructError};
pub use function_validation::FunctionValidationError;
pub use reserved_modules::{SF_ADDRESS, SF_RESERVED_STRUCTS};
pub use struct_validation::StructValidationError;
// TODO: Create error struct with LOC and error info

use external_call::{
    external_struct::{ExternalStruct, ExternalStructError},
    validate_external_call_function, validate_external_call_struct,
};
use function_modifiers::{Function, FunctionModifier, Visibility};
use function_validation::validate_function;
use move_compiler::{
    Compiler, PASS_PARSER,
    parser::ast::{Ability_, Definition, ModuleMember, ModuleUse, Use},
    shared::{Identifier, NumericalAddress, files::MappedFiles},
};
use move_ir_types::location::Loc;
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    path::Path,
};
use struct_modifiers::StructModifier;
use struct_validation::validate_struct;
use types::Type;

#[derive(Debug, Clone)]
pub struct Struct_ {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub positional_fields: bool,
    pub loc: Loc,
    pub has_key: bool,
}

#[derive(Debug)]
pub struct TestFunction {
    pub name: String,
    pub skip: bool,
    pub expect_failure: bool,
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
    pub test_functions: Vec<TestFunction>,
}

/// ModuleId represents a unique identifier for a Move module.
/// This is a local definition to avoid circular dependencies with move-bytecode-to-wasm.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ModuleId {
    pub address: [u8; 32],
    pub module_name: String,
}

pub fn process_special_attributes(
    path: &Path,
    package_address: [u8; 32],
    deps_structs: &HashMap<ModuleId, Vec<Struct_>>,
    address_alias_instantiation: &HashMap<String, [u8; 32]>,
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
                            positional_fields: matches!(
                                s.fields,
                                move_compiler::parser::ast::StructFields::Positional(_)
                            ),
                            loc: s.loc,
                            has_key: s.abilities.iter().any(|a| a.value == Ability_::Key),
                        });

                        let mut found_modifier: bool = false;
                        'loop_att: for attributes in &s.attributes {
                            if let Some(att) = attributes.value.first() {
                                let modifier = StructModifier::parse_struct_modifier(&att.value);
                                if let Some(modifier) = modifier {
                                    if found_modifier {
                                        // Found a second match
                                        found_error = true;
                                        module_errors.push(SpecialAttributeError {
                                            kind: SpecialAttributeErrorKind::TooManyAttributes,
                                            line_of_code: s.loc,
                                        });
                                        break 'loop_att;
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
                                                    found_modifier = true;
                                                }
                                                Ok(_) => {
                                                    found_modifier = true;
                                                }
                                                Err(e) => {
                                                    found_error = true;
                                                    module_errors.extend(e);
                                                    break 'loop_att;
                                                }
                                            }
                                        }
                                        StructModifier::ExternalStruct => {
                                            match ExternalStruct::try_from(s) {
                                                Ok(external_struct) => {
                                                    result
                                                        .external_struct
                                                        .insert(struct_name.clone(), external_struct);
                                                    found_modifier = true;
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
                                                    break 'loop_att;
                                                }
                                            }
                                        }
                                        StructModifier::Event => {
                                            match Event::try_from(s) {
                                                Ok(event) => {
                                                    result.events.insert(struct_name.clone(), event);
                                                    found_modifier = true;
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
                                                    break 'loop_att;
                                                }
                                            }
                                        }
                                        StructModifier::AbiError => {
                                            match AbiError::try_from(s) {
                                                Ok(abi_error) => {
                                                    result.abi_errors.insert(struct_name.clone(), abi_error);
                                                    found_modifier = true;
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
                                                    break 'loop_att;
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

    // Validate all structs:
    // - check if the struct is reserved by the Stylus Framework
    // - validate UID/NamedId placement
    // - check for nested events/errors
    for s in &result.structs {
        let validation_errors = validate_struct(
            s,
            &result.module_name,
            package_address,
            &result.events,
            &result.abi_errors,
        );
        if !validation_errors.is_empty() {
            found_error = true;
            module_errors.extend(validation_errors);
        }
    }

    // Process use declarations.
    // Members can either be Datatypes (structs or enums) or Functions.
    // imported_members is a HashMap of module id (address + module name) to a vector of tuples,
    // where the first element is the name of the member and the second element is the alias (if any).
    let mut imported_members: HashMap<ModuleId, Vec<(String, Option<String>)>> = HashMap::new();
    for source in ast.source_definitions.clone() {
        if let Definition::Module(module) = source.def {
            for module_member in module.members {
                match module_member {
                    ModuleMember::Use(ref use_decl) => {
                        if let Use::ModuleUse(module_ident, ModuleUse::Members(members)) =
                            &use_decl.use_
                        {
                            // Extract address from module_ident
                            let module_address = match &module_ident.value.address.value {
                                move_compiler::parser::ast::LeadingNameAccess_::AnonymousAddress(addr) => {
                                    // AnonymousAddress contains NumericalAddress, convert to bytes
                                    Some(addr.into_inner().into_bytes())
                                }
                                move_compiler::parser::ast::LeadingNameAccess_::GlobalAddress(name) => {
                                    // GlobalAddress is a named address, look it up in address_alias_instantiation
                                    address_alias_instantiation
                                        .get(name.value.as_str())
                                        .copied()
                                        .or_else(|| {
                                            found_error = true;
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::NamedAddressNotFound(
                                                    name.value.as_str().to_string(),
                                                ),
                                                line_of_code: use_decl.loc,
                                            });
                                            None
                                        })
                                }
                                move_compiler::parser::ast::LeadingNameAccess_::Name(name) => {
                                    // Name is also a named address, look it up in address_alias_instantiation
                                    address_alias_instantiation
                                        .get(name.value.as_str())
                                        .copied()
                                        .or_else(|| {
                                            found_error = true;
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::NamedAddressNotFound(
                                                    name.value.as_str().to_string(),
                                                ),
                                                line_of_code: use_decl.loc,
                                            });
                                            None
                                        })
                                }
                            };

                            let Some(module_address) = module_address else {
                                continue;
                            };

                            let module_id = ModuleId {
                                address: module_address,
                                module_name: module_ident.value.module.to_string(),
                            };

                            for member in members {
                                let member_tuple = (
                                    member.0.value.as_str().to_string(),
                                    member.1.as_ref().map(|s| s.value.as_str().to_string()),
                                );
                                imported_members
                                    .entry(module_id.to_owned())
                                    .or_default()
                                    .push(member_tuple);
                            }
                        }
                    }
                    _ => continue,
                }
            }
        } else {
            continue;
        }
    }

    for source in ast.source_definitions {
        if let Definition::Module(module) = source.def {
            for module_member in module.members {
                match module_member {
                    ModuleMember::Function(ref f) => {
                        let visibility: Visibility = (&f.visibility).into();
                        let signature = Function::parse_signature(&f.signature);

                        // Function validation checks:
                        // - Entry functions must not be generic.
                        // - Functions with an Event parameter are only allowed if they are native emit functions.
                        // - Functions with an Error parameter are only allowed if they are native revert functions.
                        // - Entry functions must not return structs that have the key ability.
                        // - Functions must not accept UID or references to UID as parameters, unless they are Stylus Framework functions.
                        if let Err(error) = validate_function(
                            f,
                            &result.events,
                            &result.abi_errors,
                            &result.structs,
                            deps_structs,
                            &imported_members,
                            package_address,
                        ) {
                            found_error = true;
                            module_errors.push(error);
                        }

                        if let Some(attributes) = f.attributes.first() {
                            let mut modifiers = attributes
                                .value
                                .iter()
                                .flat_map(|s| FunctionModifier::parse_modifiers(&s.value))
                                .collect::<VecDeque<FunctionModifier>>();

                            let first_modifier = modifiers.pop_front();
                            match first_modifier {
                                // TODO: Process this only if test mode is enabled
                                Some(FunctionModifier::Test) => {
                                    let modifiers =
                                        modifiers.into_iter().collect::<Vec<FunctionModifier>>();

                                    result.test_functions.push(TestFunction {
                                        name: f.name.to_owned().to_string(),
                                        skip: modifiers.contains(&FunctionModifier::Skip),
                                        expect_failure: modifiers
                                            .contains(&FunctionModifier::ExpectedFailure),
                                    });

                                    result.functions.push(Function {
                                        name: f.name.to_owned().to_string(),
                                        modifiers: vec![],
                                        signature,
                                        visibility,
                                    });
                                }
                                Some(FunctionModifier::ExternalCall) => {
                                    let modifiers =
                                        modifiers.into_iter().collect::<Vec<FunctionModifier>>();

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
                                    let modifiers =
                                        modifiers.into_iter().collect::<Vec<FunctionModifier>>();

                                    if !found_error {
                                        result.functions.push(Function {
                                            name: f.name.to_owned().to_string(),
                                            modifiers,
                                            signature,
                                            visibility,
                                        });
                                    }
                                }
                                _ => {
                                    if !found_error {
                                        if let Some(modifier) = first_modifier {
                                            modifiers.push_front(modifier);
                                        }
                                        let modifiers = modifiers
                                            .into_iter()
                                            .collect::<Vec<FunctionModifier>>();
                                        result.functions.push(Function {
                                            name: f.name.to_owned().to_string(),
                                            modifiers,
                                            signature,
                                            visibility,
                                        });
                                    }
                                }
                            }
                        } else if !found_error {
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
