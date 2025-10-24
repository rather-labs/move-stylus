use move_compiler::parser::ast::Attribute_;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StructModifier {
    ExternalStruct,
    Event,
}

impl StructModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExternalStruct => "external_struct",
            Self::Event => "event",
        }
    }

    pub fn parse_modifiers(attribute: &Attribute_) -> Vec<Self> {
        match attribute {
            Attribute_::Parameterized(_, spanned1) => spanned1
                .value
                .iter()
                .flat_map(|s| Self::parse_modifiers(&s.value))
                .collect::<Vec<StructModifier>>(),

            Attribute_::Name(name) => match name.value.as_str() {
                "external_struct" => vec![Self::ExternalStruct],
                "event" => vec![Self::Event],
                _ => vec![],
            },
            _ => vec![],
        }
    }
}
