use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_16",
    "tests/operations_cast/move_sources/uint_16.move"
);

sol!(
    #[allow(missing_docs)]
    function castDown(uint32 x) external returns (uint16);
    function castUp(uint8 x) external returns (uint16);
    function castFromU128(uint128 x) external returns (uint16);
    function castFromU256(uint256 x) external returns (uint16);
);

#[rstest]
#[case(castDownCall::new((3232,)), 3232)]
#[case(castDownCall::new((u16::MAX as u32,)), u16::MAX)]
#[case(castUpCall::new((8,)), 8)]
#[case(castUpCall::new((u8::MAX,)), u8::MAX as u16)]
#[case(castFromU128Call::new((1616,)), 1616)]
#[case(castFromU128Call::new((u16::MAX as u128,)), u16::MAX)]
#[case(castFromU256Call::new((U256::from(1616),)), 1616)]
#[case(castFromU256Call::new((U256::from(u16::MAX),)), u16::MAX)]
fn test_uint_16_cast<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(castDownCall::new((u16::MAX as u32 + 1,)))]
#[case(castFromU128Call::new((u16::MAX as u128 + 1,)))]
#[case(castFromU256Call::new((U256::from(u16::MAX) + U256::from(1),)))]
fn test_uint_16_cast_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    run_test(runtime, call_data.abi_encode(), vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
}
