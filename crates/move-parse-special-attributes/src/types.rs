use std::sync::Arc;

use move_compiler::parser::ast::{NameAccessChain_, Type_};
use move_symbol_pool::Symbol;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Address,
    Bool,
    UserDataType(Symbol, Option<Vec<Type>>),
    Signer,
    Vector(Arc<Type>),
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Unit,
    Tuple(Vec<Type>),
    Function(Vec<Type>, Arc<Type>),
    Ref(Arc<Type>),
    MutRef(Arc<Type>),
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
                            Self::Vector(Arc::new(inner))
                        } else {
                            panic!("found a vector without inner type")
                        }
                    }
                    _ => {
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

                        Self::UserDataType(path_entry.name.value, types)
                    }
                },
                NameAccessChain_::Path(name_path) => {
                    // The last entry is the one that contains the datatype name
                    let last_entry = name_path.entries.last().unwrap();
                    let types = if let Some(ref types) = last_entry.tyargs {
                        let types = types
                            .value
                            .iter()
                            .map(|t| Self::parse_type(&t.value))
                            .collect::<Vec<Type>>();
                        Some(types)
                    } else {
                        None
                    };

                    Self::UserDataType(last_entry.name.value, types)
                }
            },
            Type_::Ref(is_mut, spanned) => {
                if *is_mut {
                    Self::MutRef(Arc::new(Self::parse_type(&spanned.value)))
                } else {
                    Self::Ref(Arc::new(Self::parse_type(&spanned.value)))
                }
            }
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
                Self::Function(arguments, Arc::new(return_type))
            }
            Type_::UnresolvedError => todo!(),
        }
    }
}
