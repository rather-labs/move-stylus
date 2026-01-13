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
use function_validation::check_storage_object_param;
use move_symbol_pool::Symbol;
pub use reserved_modules::{SF_ADDRESS, SF_RESERVED_STRUCTS};
pub use struct_validation::StructValidationError;
// TODO: Create error struct with LOC and error info

use external_call::{
    external_struct::ExternalStruct, validate_external_call_function, validate_external_call_struct,
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
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};
use struct_modifiers::StructModifier;
use struct_validation::validate_struct;
use types::Type;

#[derive(Debug, Clone)]
pub struct Struct_ {
    pub name: Symbol,
    pub fields: Vec<(Symbol, Type)>,
    pub positional_fields: bool,
    pub loc: Loc,
    pub has_key: bool,
}

#[derive(Debug)]
pub struct TestFunction {
    pub name: Symbol,
    pub expect_failure: bool,
}

#[derive(Debug)]
pub struct SpecialAttributes {
    pub module_name: Symbol,
    pub events: HashMap<Symbol, Event>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct_>,
    pub external_calls: HashMap<Symbol, Function>,
    pub external_struct: HashMap<Symbol, ExternalStruct>,
    pub external_call_structs: HashSet<Symbol>,
    pub abi_errors: HashMap<Symbol, AbiError>,
    pub test_functions: Vec<TestFunction>,
}

impl Default for SpecialAttributes {
    fn default() -> Self {
        Self {
            module_name: Symbol::from(""),
            events: HashMap::default(),
            functions: Vec::default(),
            structs: Vec::default(),
            external_calls: HashMap::default(),
            external_struct: HashMap::default(),
            external_call_structs: HashSet::default(),
            abi_errors: HashMap::default(),
            test_functions: Vec::default(),
        }
    }
}

/// ModuleId represents a unique identifier for a Move module.
/// This is a local definition to avoid circular dependencies with move-bytecode-to-wasm.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ModuleId {
    pub address: [u8; 32],
    pub module_name: Symbol,
}

pub fn process_special_attributes(
    path: &Path,
    package_address: [u8; 32],
    deps_structs: &HashMap<ModuleId, Vec<Struct_>>,
    address_alias_instantiation: &HashMap<Symbol, [u8; 32]>,
) -> Result<SpecialAttributes, (MappedFiles, Vec<SpecialAttributeError>)> {
    let (mapped_files, program_res) = Compiler::from_files(
        None,
        vec![path.to_str().unwrap()],
        Vec::new(),
        BTreeMap::<Symbol, NumericalAddress>::new(),
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
            result.module_name = module.name.0.value;
            for module_member in &module.members {
                match module_member {
                    ModuleMember::Struct(s) => {
                        let struct_name = s.name.value();

                        // No matter if it is a struct marked with special attributes, we collect
                        // its information.
                        let fields: Vec<(Symbol, Type)> = match &s.fields {
                            move_compiler::parser::ast::StructFields::Named(items) => items
                                .iter()
                                .map(|(_, field, type_)| {
                                    (field.value(), Type::parse_type(&type_.value))
                                })
                                .collect(),
                            move_compiler::parser::ast::StructFields::Positional(items) => items
                                .iter()
                                .enumerate()
                                .map(|(index, (_, type_))| {
                                    (
                                        Symbol::from(format!("pos{index}")),
                                        Type::parse_type(&type_.value),
                                    )
                                })
                                .collect(),
                            move_compiler::parser::ast::StructFields::Native(_) => todo!(),
                        };

                        result.structs.push(Struct_ {
                            name: struct_name,
                            fields,
                            positional_fields: matches!(
                                s.fields,
                                move_compiler::parser::ast::StructFields::Positional(_)
                            ),
                            loc: s.loc,
                            has_key: s.abilities.iter().any(|a| a.value == Ability_::Key),
                        });

                        for attributes in &s.attributes {
                            let mut modifiers: Vec<StructModifier> = Vec::new();
                            for attr in &attributes.value {
                                match StructModifier::parse_struct_modifier(&attr.value) {
                                    Ok(Some(modifier)) => modifiers.push(modifier),
                                    Ok(None) => {}
                                    Err(e) => {
                                        found_error = true;
                                        module_errors.push(e);
                                    }
                                }
                            }

                            // println!("Struct {} has modifiers: {:?}", struct_name, modifiers);
                            for modifier in modifiers {
                                match modifier {
                                    StructModifier::ExternalStruct => todo!(),
                                    StructModifier::ExternalCall => {
                                        match validate_external_call_struct(s) {
                                            Ok(_)
                                                if !result
                                                    .external_call_structs
                                                    .contains(&struct_name) =>
                                            {
                                                result.external_call_structs.insert(struct_name);
                                            }
                                            Ok(_) => {}
                                            Err(e) => {
                                                found_error = true;
                                                module_errors.extend(e);
                                            }
                                        }
                                    }
                                    StructModifier::Event {
                                        is_anonymous,
                                        indexes,
                                    } => {
                                        // Check if the event has the key ability
                                        if s.abilities.iter().any(|a| a.value == Ability_::Key) {
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::Event(
                                                    EventParseError::EventWithKey,
                                                ),
                                                line_of_code: s.loc,
                                            });
                                            found_error = true;
                                            continue;
                                        }

                                        result.events.insert(
                                            struct_name,
                                            Event {
                                                name: struct_name,
                                                is_anonymous,
                                                indexes,
                                            },
                                        );
                                    }
                                    StructModifier::AbiError => match AbiError::try_from(s) {
                                        Ok(abi_error) => {
                                            result.abi_errors.insert(struct_name, abi_error);
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
                                        }
                                    },
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
    let mut imported_members: HashMap<ModuleId, Vec<(Symbol, Option<Symbol>)>> = HashMap::new();
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
                                        .get(&name.value)
                                        .copied()
                                        .or_else(|| {
                                            found_error = true;
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::NamedAddressNotFound(
                                                    name.value,
                                                ),
                                                line_of_code: use_decl.loc,
                                            });
                                            None
                                        })
                                }
                                move_compiler::parser::ast::LeadingNameAccess_::Name(name) => {
                                    // Name is also a named address, look it up in address_alias_instantiation
                                    address_alias_instantiation
                                        .get(&name.value)
                                        .copied()
                                        .or_else(|| {
                                            found_error = true;
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::NamedAddressNotFound(
                                                    name.value,
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
                                module_name: module_ident.value.module.0.value,
                            };

                            for member in members {
                                let member_tuple =
                                    (member.0.value, member.1.as_ref().map(|s| s.value));
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

                        let mut function =
                            Function::new(f.name.0.value, signature.clone(), visibility);

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

                        for attributes in &f.attributes {
                            let modifiers = attributes
                                .value
                                .iter()
                                .map(|s| FunctionModifier::parse_modifiers(&s.value))
                                .collect::<Result<Vec<Vec<FunctionModifier>>, SpecialAttributeError>>();

                            let modifiers = match modifiers {
                                Ok(modifiers) => modifiers.concat(),
                                Err(e) => {
                                    found_error = true;
                                    module_errors.push(e);
                                    continue;
                                }
                            };

                            for modifier in modifiers {
                                match modifier {
                                    FunctionModifier::OwnedObjects(owned_objects) => {
                                        // TODO: Check declared attributes exist

                                        for owned_object_identifier in &owned_objects {
                                            if let Err(e) = check_storage_object_param(
                                                &signature,
                                                *owned_object_identifier,
                                                f.loc,
                                                &result.structs,
                                            ) {
                                                found_error = true;
                                                module_errors.push(e);
                                                continue;
                                            }
                                        }

                                        if found_error {
                                            continue;
                                        }

                                        function.owned_objects.extend(owned_objects);
                                    }
                                    FunctionModifier::SharedObjects(shared_objects) => {
                                        for shared_object_identifier in &shared_objects {
                                            if let Err(e) = check_storage_object_param(
                                                &signature,
                                                *shared_object_identifier,
                                                f.loc,
                                                &result.structs,
                                            ) {
                                                found_error = true;
                                                module_errors.push(e);
                                                continue;
                                            }
                                        }

                                        if found_error {
                                            continue;
                                        }

                                        function.shared_objects.extend(shared_objects);
                                    }
                                    FunctionModifier::FrozenObjects(frozen_objects) => {
                                        for frozen_object_identifier in &frozen_objects {
                                            if let Err(e) = check_storage_object_param(
                                                &signature,
                                                *frozen_object_identifier,
                                                f.loc,
                                                &result.structs,
                                            ) {
                                                found_error = true;
                                                module_errors.push(e);
                                                continue;
                                            }
                                        }

                                        if found_error {
                                            continue;
                                        }

                                        function.frozen_objects.extend(frozen_objects);
                                    }
                                    // TODO: Process this only if test mode is enabled
                                    FunctionModifier::Test => {
                                        result.test_functions.push(TestFunction {
                                            name: f.name.0.value,
                                            expect_failure: false,
                                        });
                                    }
                                    FunctionModifier::ExpectedFailure => {
                                        if let Some(test_function) = result
                                            .test_functions
                                            .iter_mut()
                                            .find(|tf| tf.name == f.name.0.value)
                                        {
                                            test_function.expect_failure = true;
                                        } else {
                                            found_error = true;
                                            module_errors.push(SpecialAttributeError {
                                                kind: SpecialAttributeErrorKind::ExpectedFailureWithoutTest,
                                                line_of_code: f.loc,
                                            });
                                        }
                                    }
                                    FunctionModifier::ExternalCall(solidity_modifiers) => {
                                        let errors = validate_external_call_function(
                                            f,
                                            &solidity_modifiers,
                                            &result.external_call_structs,
                                        );

                                        if let Err(errors) = errors {
                                            found_error = true;
                                            module_errors.extend(errors);
                                        } else if !found_error {
                                            result.external_calls.insert(
                                                f.name.0.value,
                                                Function {
                                                    name: f.name.0.value,
                                                    modifiers: solidity_modifiers,
                                                    owned_objects: vec![],
                                                    shared_objects: vec![],
                                                    frozen_objects: vec![],
                                                    signature: signature.clone(),
                                                    visibility,
                                                },
                                            );
                                        }
                                    }
                                    FunctionModifier::Abi(solidity_modifiers) => {
                                        if !found_error {
                                            function.modifiers.extend(solidity_modifiers);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !found_error {
                            result.functions.push(function);
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
