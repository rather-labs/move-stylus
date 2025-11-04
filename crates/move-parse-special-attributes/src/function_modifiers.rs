use move_compiler::parser::ast::Attribute_;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub modifiers: Vec<FunctionModifier>,
    pub is_entry: bool,
    pub visibility: Visibility,
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

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionModifier {
    Pure,
    View,
    Payable,
    ExternalCall,
    Abi,
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
