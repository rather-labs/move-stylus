use crate::common::run_test;
use crate::common::runtime;
use alloy_primitives::{Address, U256, address, hex, keccak256};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::{
    constants::{
        BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, GAS_PRICE,
        MSG_SENDER_ADDRESS, MSG_VALUE,
    },
    wasm_runner::{CrossContractCallType, ExecutionData, RuntimeSandbox},
};
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function getSender() external returns (address);
    function getMsgValue() external returns (uint256);
    function getBlockNumber() external returns (uint64);
    function getBlockBasefee() external returns (uint256);
    function getBlockGasLimit() external returns (uint64);
    function getBlockTimestamp() external returns (uint64);
    function getGasPrice() external returns (uint256);
    function getFreshObjectAddress() external returns (address, address, address);
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
    #[by_ref]
    #[with("tx_context", "tests/framework/move_sources/tx_context.move")]
    runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[rstest]
#[case(
        getFreshObjectAddressCall::new(()),
        (
            hex::decode("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap(),
            hex::decode("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap(),
            hex::decode("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap()
        )
    )]
fn test_tx_fresh_id<T: SolCall>(
    #[by_ref]
    #[with("tx_context", "tests/framework/move_sources/tx_context.move")]
    runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: ([u8; 32], [u8; 32], [u8; 32]),
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}
