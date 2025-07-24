use alloy_sol_types::SolValue;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::runtime_sandbox::RuntimeSandbox;
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

mod tx_context {
    use alloy_primitives::Address;

    use crate::common::{
        runtime_sandbox::constants::{
            BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, GAS_PRICE,
            MSG_SENDER_ADDRESS, MSG_VALUE,
        },
        translate_test_package_with_framework,
    };

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "tx_context";
        const SOURCE_PATH: &str = "tests/framework/tx_context.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getSender() external returns (address);
        function getMsgValue() external returns (uint256);
        function getBlockNumber() external returns (uint64);
        function getBlockBasefee() external returns (uint256);
        function getBlockGasLimit() external returns (uint64);
        function getBlockTimestamp() external returns (uint64);
        function getGasPrice() external returns (uint256);
    );

    #[rstest]
    #[case(getSenderCall::new(()), (Address::new(MSG_SENDER_ADDRESS),))]
    #[case(getMsgValueCall::new(()), (MSG_VALUE,))]
    #[case(getBlockNumberCall::new(()), (BLOCK_NUMBER,))]
    #[case(getBlockBasefeeCall::new(()), (BLOCK_BASEFEE,))]
    #[case(getBlockGasLimitCall::new(()), (BLOCK_GAS_LIMIT,))]
    #[case(getBlockTimestampCall::new(()), (BLOCK_TIMESTAMP,))]
    #[case(getGasPriceCall::new(()), (GAS_PRICE,))]
    fn test_tx_context<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_params(),
        )
        .unwrap();
    }
}
