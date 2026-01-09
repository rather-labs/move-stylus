use crate::types::Type;
use move_compiler::{
    parser::ast::{Attribute_, FunctionSignature},
    shared::Identifier,
};
use move_symbol_pool::Symbol;

#[derive(Debug)]
pub struct Function {
    pub name: Symbol,
    pub modifiers: Vec<FunctionModifier>,
    pub visibility: Visibility,
    pub signature: Signature,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

impl From<&move_compiler::parser::ast::Visibility> for Visibility {
    fn from(value: &move_compiler::parser::ast::Visibility) -> Self {
        match value {
            move_compiler::parser::ast::Visibility::Public(_) => Self::Public,
            _ => Self::Private,
        }
    }
}

#[derive(Debug)]
pub struct Parameter {
    pub name: Symbol,
    pub type_: Type,
}

#[derive(Debug)]
pub struct Signature {
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionModifier {
    Pure,
    View,
    Payable,
    ExternalCall,
    Abi,
    Test,
    Skip,
    ExpectedFailure,
    OwnedObjects,
    SharedObjects,
    FrozenObjects,
    Identifier(Symbol),
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
            Self::Pure => "pure",
            Self::View => "view",
            Self::Payable => "payable",
            Self::ExternalCall => "external_call",
            Self::Abi => "abi",
            Self::Test => "test",
            Self::Skip => "skip",
            Self::ExpectedFailure => "expected_failure",
            Self::OwnedObjects => "owned_objects",
            Self::SharedObjects => "shared_objects",
            Self::FrozenObjects => "frozen_objects",
            Self::Identifier(id) => id.as_str(),
        }
    }

    pub fn parse_modifiers(attribute: &Attribute_) -> Vec<Self> {
        let mut result = Vec::new();

        match attribute {
            Attribute_::Parameterized(name, spanned1) => {
                match name.value.as_str() {
                    "owned_objects" => {
                        result.push(Self::SharedObjects);
                    }
                    "shared_objects" => {
                        result.push(Self::SharedObjects);
                    }
                    "frozen_objects" => {
                        result.push(Self::SharedObjects);
                    }
                    _ => (),
                }

                result.extend(
                    spanned1
                        .value
                        .iter()
                        .flat_map(|s| Self::parse_modifiers(&s.value))
                        .collect::<Vec<FunctionModifier>>(),
                );
            }
            Attribute_::Name(name) => match name.value.as_str() {
                "pure" => result.push(Self::Pure),
                "view" => result.push(Self::View),
                "payable" => result.push(Self::Payable),
                "external_call" => result.push(Self::ExternalCall),
                "abi" => result.push(Self::Abi),
                "test" => result.push(Self::Test),
                "skip" => result.push(Self::Skip),
                "expected_failure" => result.push(Self::ExpectedFailure),
                _ => result.push(Self::Identifier(name.value)),
            },
            _ => (),
        }

        result
    }
}
