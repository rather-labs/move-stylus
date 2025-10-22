use move_compiler::parser::ast::Attribute_;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub modifiers: Vec<FunctionModifier>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionModifier {
    External,
    Pure,
    View,
    Payable,
    ExternalCall,
}

impl FunctionModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Pure => "pure",
            Self::View => "view",
            Self::Payable => "payable",
            Self::ExternalCall => "external_call",
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
                "external" => vec![Self::External],
                "pure" => vec![Self::Pure],
                "view" => vec![Self::View],
                "payable" => vec![Self::Payable],
                "external_call" => vec![Self::ExternalCall],
                _ => vec![],
            },
            _ => vec![],
        }
    }
}
