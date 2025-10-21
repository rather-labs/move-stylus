// TODO: Change string for symbols
#[derive(thiserror::Error, Debug)]
pub enum ExternalCallError {
    #[error(
        "An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult, found '{0}'"
    )]
    InvalidReturnType(String),

    #[error("An external call function must be declared as 'native'")]
    FunctionIsNotNative,

    #[error(
        "The 'value' argument of a payable external call function must be of type 'u256', found '{0}'"
    )]
    InvalidValueArgumentType(String),

    #[error(
        "The second argument of a payable external call function must be named 'value', found '{0}'"
    )]
    InvalidValueArgumentName(String),

    #[error("A payable external call function must have a 'value' argument of type 'u256'")]
    ValueArgumentMissing,
}
