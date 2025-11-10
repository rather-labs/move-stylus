use move_compiler::parser::ast::Attribute_;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StructModifier {
    ExternalStruct,
    ExternalCall,
    Event,
    AbiError,
}

impl StructModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExternalStruct => "external_struct",
            Self::ExternalCall => "external_call",
            Self::Event => "event",
            Self::AbiError => "abi_error",
        }
    }

    pub fn parse_modifiers(attribute: &Attribute_) -> Vec<Self> {
        match attribute {
            Attribute_::Parameterized(name, spanned1) => {
                if name.value.as_str() == "ext" {
                    spanned1
                        .value
                        .iter()
                        .flat_map(|s| Self::parse_modifiers(&s.value))
                        .collect::<Vec<StructModifier>>()
                } else {
                    vec![]
                }
            }

            Attribute_::Name(name) => match name.value.as_str() {
                "external_struct" => vec![Self::ExternalStruct],
                "external_call" => vec![Self::ExternalCall],
                "event" => vec![Self::Event],
                "abi_error" => vec![Self::AbiError],
                _ => vec![],
            },
            _ => vec![],
        }
    }
}
