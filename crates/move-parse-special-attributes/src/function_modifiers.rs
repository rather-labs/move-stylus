// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use crate::{SpecialAttributeError, error::SpecialAttributeErrorKind, types::Type};
use move_compiler::{
    parser::ast::{
        Attribute_, AttributeValue_, FunctionSignature, LeadingNameAccess_, NameAccessChain_,
        Value_,
    },
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

/// Represents the expected abort code for an `#[expected_failure(abort_code = ...)]` attribute.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpectedAbortCode {
    /// A literal numeric abort code, e.g., `abort_code = 65540`
    Literal(u64),
    /// A constant reference like `module::CONSTANT`, e.g., `abort_code = fixed_point32::EDIVISION_BY_ZERO`
    Constant(Symbol, Symbol),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionModifier {
    ExternalCall(Vec<SolidityFunctionModifier>),
    Abi(Vec<SolidityFunctionModifier>),
    Test,
    Skip,
    ExpectedFailure(Option<ExpectedAbortCode>),
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
            Self::ExpectedFailure(_) => "expected_failure",
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
                    "expected_failure" => {
                        let abort_code = Self::parse_expected_failure_abort_code(spanned1)?;
                        result.push(Self::ExpectedFailure(abort_code));
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
                "expected_failure" => result.push(Self::ExpectedFailure(None)),
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

    /// Parses the `abort_code` parameter from an `expected_failure(abort_code = ...)` attribute.
    ///
    /// Supports two forms:
    /// - Numeric literal: `expected_failure(abort_code = 65540)`
    /// - Constant reference: `expected_failure(abort_code = fixed_point32::EDIVISION_BY_ZERO)`
    ///
    /// Note: Constants can either be declared in the same module or in a different module!
    fn parse_expected_failure_abort_code(
        attrs: &move_compiler::parser::ast::Attributes,
    ) -> Result<Option<ExpectedAbortCode>, SpecialAttributeError> {
        for attr in &attrs.value {
            if let Attribute_::Assigned(name, value) = &attr.value {
                if name.value.as_str() == "abort_code" {
                    match &value.value {
                        // Numeric literal: abort_code = 65540 or abort_code = 0x10004
                        AttributeValue_::Value(v) => match &v.value {
                            Value_::Num(n) => {
                                let s = n.as_str();
                                let code = if let Some(hex) = s.strip_prefix("0x") {
                                    u64::from_str_radix(hex, 16)
                                } else {
                                    s.parse::<u64>()
                                }.map_err(|_| {
                                    SpecialAttributeError {
                                        kind: SpecialAttributeErrorKind::InvalidExpectedFailureAbortCode,
                                        line_of_code: v.loc,
                                    }
                                })?;
                                return Ok(Some(ExpectedAbortCode::Literal(code)));
                            }
                            _ => {
                                return Err(SpecialAttributeError {
                                    kind:
                                        SpecialAttributeErrorKind::InvalidExpectedFailureAbortCode,
                                    line_of_code: v.loc,
                                });
                            }
                        },
                        // Module access: abort_code = fixed_point32::EDIVISION_BY_ZERO
                        //            or: abort_code = std::type_name::ENonModuleType
                        AttributeValue_::ModuleAccess(chain) => {
                            // Note: Here we have no information regarding the constant type or value, we need to resolve that later!
                            match &chain.value {
                                NameAccessChain_::Path(path) => {
                                    let entries = &path.entries;
                                    if entries.is_empty() {
                                        return Err(SpecialAttributeError {
                                            kind: SpecialAttributeErrorKind::InvalidExpectedFailureAbortCode,
                                            line_of_code: chain.loc,
                                        });
                                    }

                                    // The constant name is always the last entry.
                                    let constant_name = entries.last().unwrap().name.value;

                                    // The module name is:
                                    // - For 2-segment paths (module::CONST): root is the module
                                    // - For 3+ segment paths (addr::module::CONST): second-to-last entry is the module
                                    let module_name = if entries.len() == 1 {
                                        // 2-segment: root::CONST → root is the module
                                        match &path.root.name.value {
                                            LeadingNameAccess_::Name(n) => n.value,
                                            _ => {
                                                return Err(SpecialAttributeError {
                                                    kind: SpecialAttributeErrorKind::InvalidExpectedFailureAbortCode,
                                                    line_of_code: chain.loc,
                                                });
                                            }
                                        }
                                    } else {
                                        // 3+ segment: addr::module::CONST → entries[len-2] is the module
                                        entries[entries.len() - 2].name.value
                                    };

                                    return Ok(Some(ExpectedAbortCode::Constant(
                                        module_name,
                                        constant_name,
                                    )));
                                }
                                NameAccessChain_::Single(entry) => {
                                    // Single name constant from the same module
                                    return Ok(Some(ExpectedAbortCode::Constant(
                                        Symbol::from(""),
                                        entry.name.value,
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }
        // No abort_code parameter found — bare expected_failure
        Ok(None)
    }
}
