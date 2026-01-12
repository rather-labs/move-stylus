use crate::types::Type;
use move_compiler::{
    parser::ast::{Attribute_, FunctionSignature},
    shared::Identifier,
};
use move_symbol_pool::Symbol;

#[derive(Debug)]
pub struct Function {
    pub name: Symbol,
    pub modifiers: Vec<SolidityFunctionModifier>,
    pub owned_objects: Vec<Symbol>,
    pub shared_objects: Vec<Symbol>,
    pub frozen_objects: Vec<Symbol>,
    pub visibility: Visibility,
    pub signature: Signature,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Visibility {
    Private,
    Public,
}

impl Function {
    pub fn new(name: Symbol, signature: Signature, visibility: Visibility) -> Self {
        Self {
            name,
            modifiers: Vec::new(),
            owned_objects: Vec::new(),
            shared_objects: Vec::new(),
            frozen_objects: Vec::new(),
            visibility,
            signature,
        }
    }
}

impl From<&move_compiler::parser::ast::Visibility> for Visibility {
    fn from(value: &move_compiler::parser::ast::Visibility) -> Self {
        match value {
            move_compiler::parser::ast::Visibility::Public(_) => Self::Public,
            _ => Self::Private,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Symbol,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SolidityFunctionModifier {
    Pure,
    View,
    Payable,
}

impl SolidityFunctionModifier {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pure => "pure",
            Self::View => "view",
            Self::Payable => "payable",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionModifier {
    ExternalCall(Vec<SolidityFunctionModifier>),
    Abi(Vec<SolidityFunctionModifier>),
    Test,
    Skip,
    ExpectedFailure,
    OwnedObjects(Vec<Symbol>),
    SharedObjects(Vec<Symbol>),
    FrozenObjects(Vec<Symbol>),
}

impl Function {
    pub fn parse_signature(signature: &FunctionSignature) -> Signature {
        let parameters = signature
            .parameters
            .iter()
            .map(|(_, n, t)| Parameter {
                name: n.value(),
                type_: Type::parse_type(&t.value),
            })
            .collect();

        let return_type = Type::parse_type(&signature.return_type.value);

        Signature {
            parameters,
            return_type,
        }
    }
}

impl FunctionModifier {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ExternalCall(_) => "external_call",
            Self::Abi(_) => "abi",
            Self::Test => "test",
            Self::Skip => "skip",
            Self::ExpectedFailure => "expected_failure",
            Self::OwnedObjects(_) => "owned_objects",
            Self::SharedObjects(_) => "shared_objects",
            Self::FrozenObjects(_) => "frozen_objects",
        }
    }

    pub fn parse_modifiers(attribute: &Attribute_) -> Vec<Self> {
        let mut result = Vec::new();

        println!("Parsing attribute: {:?}", attribute);

        match attribute {
            Attribute_::Parameterized(name, spanned1) => match name.value.as_str() {
                "owned_objects" => {
                    result.push(Self::OwnedObjects(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value))
                            .collect::<Vec<Symbol>>(),
                    ));
                }
                "shared_objects" => {
                    result.push(Self::SharedObjects(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value))
                            .collect::<Vec<Symbol>>(),
                    ));
                }
                "frozen_objects" => {
                    result.push(Self::FrozenObjects(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value))
                            .collect::<Vec<Symbol>>(),
                    ));
                }
                "abi" => {
                    result.push(Self::Abi(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_solidity_modifier(&s.value))
                            .collect::<Vec<SolidityFunctionModifier>>(),
                    ));
                }
                "external_call" => {
                    result.push(Self::ExternalCall(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_solidity_modifier(&s.value))
                            .collect::<Vec<SolidityFunctionModifier>>(),
                    ));
                }
                _ => result.extend(
                    spanned1
                        .value
                        .iter()
                        .flat_map(|s| Self::parse_modifiers(&s.value))
                        .collect::<Vec<FunctionModifier>>(),
                ),
            },
            Attribute_::Name(name) => match name.value.as_str() {
                "external_call" => result.push(Self::ExternalCall(Vec::new())),
                "test" => result.push(Self::Test),
                "skip" => result.push(Self::Skip),
                "expected_failure" => result.push(Self::ExpectedFailure),
                _ => panic!(
                    "Unsupported attribute name for function modifier: {:?}",
                    name
                ),
            },
            _ => (),
        }

        result
    }

    fn parse_identifiers(attribute: &Attribute_) -> Symbol {
        match attribute {
            Attribute_::Name(name) => name.value,
            a => panic!("Unsupported attribute for identifiers: {:?}", a),
        }
    }

    fn parse_solidity_modifier(attribute: &Attribute_) -> SolidityFunctionModifier {
        match attribute {
            Attribute_::Name(name) => match name.value.as_str() {
                "pure" => SolidityFunctionModifier::Pure,
                "view" => SolidityFunctionModifier::View,
                "payable" => SolidityFunctionModifier::Payable,
                _ => panic!("Unsupported solidity function modifier: {:?}", name),
            },
            _ => panic!(
                "Unsupported attribute for solidity function modifier: {:?}",
                attribute
            ),
        }
    }
}
