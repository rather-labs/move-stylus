// TODO: Change string for symbols
#[derive(thiserror::Error, Debug)]
pub enum ExternalCallFunctionError {
    #[error(
        "An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult, found '{0}'"
    )]
    InvalidReturnType(String),

    #[error("An external call function must be declared as 'native'")]
    FunctionIsNotNative,

    #[error("An external call function have as first argument a reference to an external struct")]
    InvalidFirstArgument,
}

#[derive(thiserror::Error, Debug)]
pub enum ExternalCallStructError {
    #[error("Should have the 'drop' ability")]
    MissingAbilityDrop,

    #[error(
        "Should wrap the cross contract call configuration struct stylus::contract_calls::CrossContractCall"
    )]
    MissingConfiguration,

    #[error(
        "Too many fields, should contain only the cross contract call configuration struct stylus::contract_calls::CrossContractCall"
    )]
    TooManyFields,

    #[error(
        "Invalid configuration field, expectedc cross contract call configuration struct stylus::contract_calls::CrossContractCall"
    )]
    InvalidConfigurationField,
}
