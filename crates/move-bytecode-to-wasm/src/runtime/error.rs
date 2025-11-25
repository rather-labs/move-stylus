use std::rc::Rc;

use crate::{
    storage::error::StorageError, translation::intermediate_types::error::IntermediateTypeError,
};

#[derive(thiserror::Error, Debug)]
pub enum RuntimeFunctionError {
    #[error("there was an error processing the storage")]
    Storage(#[source] Rc<StorageError>),

    #[error("an error ocurred while processing an intermediate type")]
    IntermediateType(#[source] Rc<IntermediateTypeError>),

    #[error(r#"there was an error linking "{0}" runtime function, missing compilation context?"#)]
    CouldNotLink(String),

    #[error(r#"there was an error linking "{0}" runtime function, is this function generic?"#)]
    CouldNotLinkGeneric(String),

    #[error("generic_function_name called with no generics")]
    GenericFunctionNameNoGenerics,

    #[error(
        "there was an error linking {function_name} expected {expected} type parameter(s), found {found}"
    )]
    WrongNumberOfTypeParameters {
        function_name: String,
        expected: usize,
        found: usize,
    },
}

impl From<IntermediateTypeError> for RuntimeFunctionError {
    fn from(err: IntermediateTypeError) -> Self {
        RuntimeFunctionError::IntermediateType(Rc::new(err))
    }
}

impl From<StorageError> for RuntimeFunctionError {
    fn from(err: StorageError) -> Self {
        RuntimeFunctionError::Storage(Rc::new(err))
    }
}
