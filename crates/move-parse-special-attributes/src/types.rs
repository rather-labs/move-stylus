use std::rc::Rc;

use move_compiler::parser::ast::{NameAccessChain_, Type_};

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Address,
    Bool,
    UserDataType(String),
    Signer,
    Vector(Rc<Type>),
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Unit,
    Tuple(Vec<Type>),
}

impl Type {
    pub fn parse_type(type_: &Type_) -> Self {
        match type_ {
            Type_::Apply(named) => match &named.value {
                NameAccessChain_::Single(path_entry) => match path_entry.name.value.as_str() {
                    "address" => Self::Address,
                    "bool" => Self::Bool,
                    "signer" => Self::Signer,
                    "u8" => Self::U8,
                    "u16" => Self::U16,
                    "u32" => Self::U32,
                    "u64" => Self::U64,
                    "u128" => Self::U128,
                    "u256" => Self::U256,
                    d => Self::UserDataType(d.to_string()),
                },
                NameAccessChain_::Path(name_path) => todo!(),
            },
            Type_::Ref(_, spanned) => Self::parse_type(&spanned.value),
            Type_::Fun(spanneds, spanned) => todo!(),
            Type_::Unit => Self::Unit,
            Type_::Multiple(spanneds) => {
                let types = spanneds
                    .iter()
                    .map(|t| Self::parse_type(&t.value))
                    .collect();
                Self::Tuple(types)
            }
            Type_::UnresolvedError => todo!(),
        }
    }
}
