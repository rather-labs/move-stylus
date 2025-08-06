use std::ops::Deref;

use crate::translation::intermediate_types::IntermediateType;
use move_binary_format::file_format::Bytecode;

#[derive(Debug, Clone)]
pub struct TypesStack(pub Vec<IntermediateType>);

type Result<T> = std::result::Result<T, TypesStackError>;

impl TypesStack {
    pub fn new() -> Self {
        TypesStack(Vec::new())
    }

    pub fn push(&mut self, item: IntermediateType) {
        self.0.push(item)
    }

    pub fn pop(&mut self) -> Result<IntermediateType> {
        self.0.pop().ok_or(TypesStackError::EmptyStack)
    }

    pub fn append(&mut self, items: &[IntermediateType]) {
        self.0.extend_from_slice(items);
    }

    pub fn pop_expecting(&mut self, expected_type: &IntermediateType) -> Result<()> {
        let Ok(ty) = self.pop() else {
            return Err(TypesStackError::EmptyStackExpecting {
                expected: expected_type.clone(),
            });
        };

        if ty != *expected_type {
            return Err(TypesStackError::TypeMismatch {
                expected: expected_type.clone(),
                found: ty,
            });
        }

        Ok(())
    }

    pub fn pop_n_from_stack<const N: usize>(&mut self) -> Result<[IntermediateType; N]> {
        // We use IU8 as placeholder, it gets replaced on the for loop
        let mut res = [const { IntermediateType::IU8 }; N];
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            if let Ok(t) = self.pop() {
                res[i] = t;
            } else {
                return Err(TypesStackError::ExpectedNElements(N));
            }
        }

        Ok(res)
    }
}

impl Deref for TypesStack {
    type Target = [IntermediateType];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TypesStackError {
    #[error("expected {expected:?} but types stack is empty")]
    EmptyStackExpecting { expected: IntermediateType },

    #[error("types stack is empty")]
    EmptyStack,

    #[error("expected {0} but types stack is empty")]
    ExpectedNElements(usize),

    #[error("expected {expected:?} but found {found:?}")]
    TypeMismatch {
        expected: IntermediateType,
        found: IntermediateType,
    },

    #[error("expected {expected:?} but found {found:?}")]
    MatchError {
        expected: &'static str,
        found: IntermediateType,
    },

    #[error(
        "unable to perform \"{operation:?}\" on types {operand1:?} and {operand2:?}, expected the same type on types stack"
    )]
    OperationTypeMismatch {
        operand1: IntermediateType,
        operand2: IntermediateType,
        operation: Bytecode,
    },
}

macro_rules! match_types {
    ($(($expected_pattern: pat, $expected_type: expr, $variable: expr)),*) => {
        $(
            let $expected_pattern = $variable else {
                return Err($crate::translation::types_stack::TypesStackError::MatchError {
                    expected: $expected_type,
                    found: $variable,
                })?;
            };
        )*
    };
}

pub(crate) use match_types;
