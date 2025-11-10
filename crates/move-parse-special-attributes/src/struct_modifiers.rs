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

    pub fn parse_struct_modifier(attribute: &Attribute_) -> Option<Self> {
        match attribute {
            Attribute_::Parameterized(name, spanned1) => {
                if name.value.as_str() == "ext" {
                    spanned1
                        .value
                        .iter()
                        .flat_map(|s| Self::parse_struct_modifier(&s.value))
                        .next()
                } else {
                    None
                }
            }

            Attribute_::Name(name) => match name.value.as_str() {
                "external_struct" => Some(Self::ExternalStruct),
                "external_call" => Some(Self::ExternalCall),
                "event" => Some(Self::Event),
                "abi_error" => Some(Self::AbiError),
                _ => None,
            },
            _ => None,
        }
    }
}
