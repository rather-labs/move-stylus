use move_compiler::diagnostics::codes::{DiagnosticInfo, Severity, custom};

// TODO: Change string for symbols
#[derive(thiserror::Error, Debug)]
pub enum ExternalCallFunctionError {
    #[error(
        "An external call function must return either stylus::contract_calls::ContractCallResult<T> or stylus::contract_calls::ContractCallEmptyResult, found '{0}'"
    )]
    InvalidReturnType(String),

    #[error("An external call function must be declared as 'native'")]
    FunctionIsNotNative,

    #[error(
        "A function marked with #[ext(external_call, ..)] must have as first argument a reference to a struct marked with #[ext(external_struct)]"
    )]
    InvalidFirstArgument,
}

impl From<&ExternalCallFunctionError> for DiagnosticInfo {
    fn from(value: &ExternalCallFunctionError) -> Self {
        custom(
            "",
            Severity::BlockingError,
            3,
            1,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
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

impl From<&ExternalCallStructError> for DiagnosticInfo {
    fn from(value: &ExternalCallStructError) -> Self {
        custom(
            "External call struct error",
            Severity::BlockingError,
            2,
            2,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}
