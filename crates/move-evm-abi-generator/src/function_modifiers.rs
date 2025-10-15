use move_compiler::parser::ast::Attribute_;

#[derive(Debug)]
pub enum FunctionModifier {
    External,
    Pure,
    View,
    Payable,
}

impl FunctionModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Pure => "pure",
            Self::View => "view",
            Self::Payable => "payable",
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
                _ => panic!("unssuported attribute {name:?}"),
            },
            // TODO: we must just ignore it
            _ => panic!("unssuported attribute {attribute:?}"),
        }
    }
}
