use crate::types::Type;
use move_compiler::{
    parser::ast::{Attribute_, FunctionSignature},
    shared::Identifier,
};

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub modifiers: Vec<FunctionModifier>,
    // pub is_entry: bool,
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
    pub name: String,
    pub type_: Type,
}

#[derive(Debug)]
pub struct Signature {
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionModifier {
    Pure,
    View,
    Payable,
    ExternalCall,
    Abi,
}

impl Function {
    pub fn parse_signature(signature: &FunctionSignature) -> Signature {
        let parameters = signature
            .parameters
            .iter()
            .map(|(_, n, t)| Parameter {
                name: n.value().as_str().to_string(),
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pure => "pure",
            Self::View => "view",
            Self::Payable => "payable",
            Self::ExternalCall => "external_call",
            Self::Abi => "abi",
        }
    }

    pub fn parse_modifiers(attribute: &Attribute_) -> Vec<Self> {
        match attribute {
            Attribute_::Parameterized(_, spanned1) => spanned1
                .value
                .iter()
                .flat_map(|s| Self::parse_modifiers(&s.value))
                .collect::<Vec<FunctionModifier>>(),
            Attribute_::Name(name) => match name.value.as_str() {
                "pure" => vec![Self::Pure],
                "view" => vec![Self::View],
                "payable" => vec![Self::Payable],
                "external_call" => vec![Self::ExternalCall],
                "abi" => vec![Self::Abi],
                _ => vec![],
            },
            _ => vec![],
        }
    }
}
