use super::types_stack::TypesStackError;

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Types stack error: {0}")]
    TypesStackError(#[from] TypesStackError),
}
