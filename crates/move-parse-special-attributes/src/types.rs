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
            Type_::Unit => todo!(),
            Type_::Multiple(spanneds) => todo!(),
            Type_::UnresolvedError => todo!(),
            /*
            Type_::Address => Self::Address,
            ast::Type_::Signer => Self::Signer,
            ast::Type_::U8 => Self::U8,
            ast::Type_::U16 => Self::U16,
            ast::Type_::U32 => Self::U32,
            ast::Type_::U64 => Self::U64,
            ast::Type_::U128 => Self::U128,
            ast::Type_::U256 => Self::U256,
            ast::Type_::Bool => Self::Bool,
            ast::Type_::Vector(spanned) => {
                let inner = Self::parse_type(&spanned.value);
                Self::Vector(Rc::new(inner))
            }
            ast::Type_::Datatype(qualified_datatype_ident, _) => {
                Self::UserDataType(qualified_datatype_ident.name.to_string())
            }
            ast::Type_::Reference(_, spanned) => Self::parse_type(&spanned.value),
            ast::Type_::TypeParameter(_) => panic!("Found data type"), // TODO: Change this
                                                                       */
        }
    }
}
