use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind, types::Type};
use move_compiler::{
    parser::ast::{Attribute_, FunctionSignature},
    shared::Identifier,
};
use move_ir_types::location::Loc;
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
    OwnedObjects(Vec<(Symbol, Loc)>),
    SharedObjects(Vec<(Symbol, Loc)>),
    FrozenObjects(Vec<(Symbol, Loc)>),
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

    pub fn parse_modifiers(attribute: &Attribute_) -> Result<Vec<Self>, SpecialAttributeError> {
        let mut result = Vec::new();

        match attribute {
            Attribute_::Parameterized(name, spanned1) => {
                match name.value.as_str() {
                    "owned_objects" => {
                        let ids = spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value, s.loc))
                            .collect::<Result<Vec<(Symbol, Loc)>, SpecialAttributeError>>()?;
                        result.push(Self::OwnedObjects(ids));
                    }
                    "shared_objects" => {
                        let ids = spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value, s.loc))
                            .collect::<Result<Vec<(Symbol, Loc)>, SpecialAttributeError>>()?;
                        result.push(Self::SharedObjects(ids));
                    }
                    "frozen_objects" => {
                        let ids = spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_identifiers(&s.value, s.loc))
                            .collect::<Result<Vec<(Symbol, Loc)>, SpecialAttributeError>>()?;
                        result.push(Self::FrozenObjects(ids));
                    }
                    "abi" => {
                        let modifiers = spanned1
                        .value
                        .iter()
                        .map(|s| Self::parse_solidity_modifier(&s.value, s.loc))
                        .collect::<Result<Vec<SolidityFunctionModifier>, SpecialAttributeError>>()?;
                        result.push(Self::Abi(modifiers));
                    }
                    "external_call" => {
                        let modifiers = spanned1
                        .value
                        .iter()
                        .map(|s| Self::parse_solidity_modifier(&s.value, s.loc))
                        .collect::<Result<Vec<SolidityFunctionModifier>, SpecialAttributeError>>()?;
                        result.push(Self::ExternalCall(modifiers));
                    }
                    _ => result.extend(
                        spanned1
                            .value
                            .iter()
                            .map(|s| Self::parse_modifiers(&s.value))
                            .collect::<Result<Vec<Vec<FunctionModifier>>, SpecialAttributeError>>()?
                            .concat(),
                    ),
                }
            }
            Attribute_::Name(name) => match name.value.as_str() {
                "external_call" => result.push(Self::ExternalCall(Vec::new())),
                "test" => result.push(Self::Test),
                "skip" => result.push(Self::Skip),
                "expected_failure" => result.push(Self::ExpectedFailure),
                _ => (),
            },
            _ => (),
        }

        Ok(result)
    }

    fn parse_identifiers(
        attribute: &Attribute_,
        loc: Loc,
    ) -> Result<(Symbol, Loc), SpecialAttributeError> {
        match attribute {
            Attribute_::Name(name) => Ok((name.value, loc)),
            a => Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::UnsupportedAttributeForIdentifiers(
                    a.attribute_name().value,
                ),
                line_of_code: loc,
            }),
        }
    }

    fn parse_solidity_modifier(
        attribute: &Attribute_,
        loc: Loc,
    ) -> Result<SolidityFunctionModifier, SpecialAttributeError> {
        match attribute {
            Attribute_::Name(name) => match name.value.as_str() {
                "pure" => Ok(SolidityFunctionModifier::Pure),
                "view" => Ok(SolidityFunctionModifier::View),
                "payable" => Ok(SolidityFunctionModifier::Payable),
                _ => Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::UnsupportedSolidityFunctionModifier(
                        name.value,
                    ),
                    line_of_code: loc,
                }),
            },
            a => Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::UnsupportedAttributeForSolidityFunctionModifier(
                    a.attribute_name().value,
                ),
                line_of_code: loc,
            }),
        }
    }
}
