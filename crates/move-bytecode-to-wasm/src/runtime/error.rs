use std::rc::Rc;

use crate::storage::error::StorageError;

#[derive(thiserror::Error, Debug)]
pub enum RuntimeFunctionError {
    #[error("there was an error processing the storage")]
    Storage(#[source] Rc<StorageError>),

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
