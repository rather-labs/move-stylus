use std::rc::Rc;

pub enum Type {
    Address,
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
    Unit,
    Array(Rc<Type>),
    Bytes32,
    UserDefined(String),
    Tuple(Vec<Type>),
}

impl From<&move_parse_special_attributes::types::Type> for Type {
    fn from(value: &move_parse_special_attributes::types::Type) -> Self {
        match value {
            move_parse_special_attributes::types::Type::Address => Self::Address,
            move_parse_special_attributes::types::Type::Bool => Self::Bool,
            move_parse_special_attributes::types::Type::UserDataType(d) => {
                Self::UserDefined(d.clone())
            }
            move_parse_special_attributes::types::Type::Signer => Self::Address, // TODO: This is
            // not correct
            move_parse_special_attributes::types::Type::Vector(t) => {
                Self::Array(Rc::new(Self::from(t.as_ref())))
            }
            move_parse_special_attributes::types::Type::U8 => Self::Uint8,
            move_parse_special_attributes::types::Type::U16 => Self::Uint16,
            move_parse_special_attributes::types::Type::U32 => Self::Uint32,
            move_parse_special_attributes::types::Type::U64 => Self::Uint64,
            move_parse_special_attributes::types::Type::U128 => Self::Uint128,
            move_parse_special_attributes::types::Type::U256 => Self::Uint256,
            move_parse_special_attributes::types::Type::Unit => Self::Unit,
            move_parse_special_attributes::types::Type::Tuple(items) => {
                Self::Tuple(items.iter().map(Self::from).collect())
            }
        }
    }
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Address => "address".to_owned(),
            Type::Bool => "bool".to_owned(),
            Type::Uint8 => "uint8".to_owned(),
            Type::Uint16 => "uint16".to_owned(),
            Type::Uint32 => "uint32".to_owned(),
            Type::Uint64 => "uint64".to_owned(),
            Type::Uint128 => "uint128".to_owned(),
            Type::Uint256 => "uint256".to_owned(),
            Type::Unit => "".to_owned(),
            Type::Array(inner) => format!("{}[]", inner.name()),
            Type::Bytes32 => "bytes32".to_owned(),
            Type::UserDefined(name) => name.clone(),
            Type::Tuple(items) => {
                format!(
                    "({})",
                    items
                        .iter()
                        .map(|i| i.name())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}
