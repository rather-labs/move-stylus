use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_32",
    "tests/operations_cast/move_sources/uint_32.move"
);

sol!(
    #[allow(missing_docs)]
    function castDown(uint64 x) external returns (uint32);
    function castUp(uint16 x) external returns (uint32);
    function castFromU128(uint128 x) external returns (uint32);
    function castFromU256(uint256 x) external returns (uint32);
);

#[rstest]
#[case(castDownCall::new((6464,)), 6464)]
#[case(castDownCall::new((u32::MAX as u64,)), u32::MAX)]
#[case(castUpCall::new((1616,)), 1616)]
#[case(castUpCall::new((u16::MAX,)), u16::MAX as u32)]
#[case(castFromU128Call::new((3232,)), 3232)]
#[case(castFromU128Call::new((u32::MAX as u128,)), u32::MAX)]
#[case(castFromU256Call::new((U256::from(3232),)), 3232)]
#[case(castFromU256Call::new((U256::from(u32::MAX),)), u32::MAX)]
fn test_uint_32_cast<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(castDownCall::new((u32::MAX as u64 + 1,)))]
#[case(castFromU128Call::new((u32::MAX as u128 + 1,)))]
#[case(castFromU256Call::new((U256::from(u32::MAX) + U256::from(1),)))]
fn test_uint_32_cast_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    run_test(runtime, call_data.abi_encode(), vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
}
