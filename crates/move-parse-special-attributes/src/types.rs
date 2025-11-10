use std::rc::Rc;

use move_compiler::parser::ast::{NameAccessChain_, Type_};

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Address,
    Bool,
    UserDataType(String, Option<Vec<Type>>),
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
    Function(Vec<Type>, Rc<Type>),
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
                    "vector" => {
                        if let Some(ref ty) = path_entry.tyargs {
                            assert_eq!(1, ty.value.len());
                            let inner = Self::parse_type(
                                &ty.value
                                    .first()
                                    .expect("expected a type for inner vector type")
                                    .value,
                            );
                            Self::Vector(Rc::new(inner))
                        } else {
                            panic!("found a vector without inner type")
                        }
                    }
                    datatype => {
                        let types = if let Some(ref types) = path_entry.tyargs {
                            let types = types
                                .value
                                .iter()
                                .map(|t| Self::parse_type(&t.value))
                                .collect::<Vec<Type>>();
                            Some(types)
                        } else {
                            None
                        };

                        Self::UserDataType(datatype.to_string(), types)
                    }
                },
                NameAccessChain_::Path(_name_path) => todo!(),
            },
            Type_::Ref(_, spanned) => Self::parse_type(&spanned.value),
            Type_::Unit => Self::Unit,
            Type_::Multiple(spanneds) => {
                let types = spanneds
                    .iter()
                    .map(|t| Self::parse_type(&t.value))
                    .collect();
                Self::Tuple(types)
            }
            Type_::Fun(spanneds, spanned) => {
                let arguments = spanneds
                    .iter()
                    .map(|a| Self::parse_type(&a.value))
                    .collect();
                let return_type = Self::parse_type(&spanned.value);
                Self::Function(arguments, Rc::new(return_type))
            }
            Type_::UnresolvedError => todo!(),
        }
    }
}
