use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_256",
    "tests/operations_bitwise/move_sources/uint_256.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint256 x, uint256 y) external returns (uint256);
    function xor(uint256 x, uint256 y) external returns (uint256);
    function and(uint256 x, uint256 y) external returns (uint256);
    function shiftLeft(uint256 x, uint8 slots) external returns (uint256);
    function shiftRight(uint256 x, uint8 slots) external returns (uint256);
);

#[rstest]
#[case(orCall::new((U256::from(256256), U256::from(256256))), U256::from(256256))]
#[case(orCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(u128::MAX) + U256::from(1) + U256::from(256256))]
#[case(orCall::new((U256::from(256256), U256::from(0))), U256::from(256256))]
#[case(orCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::MAX)]
#[case(orCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::MAX)]
#[case(orCall::new((U256::from(0), U256::from(0))), U256::from(0))]
#[case(orCall::new((U256::MAX, U256::MAX)), U256::MAX)]
fn test_uint_256_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((U256::from(256256), U256::from(256256))), U256::from(0))]
#[case(xorCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(u128::MAX) + U256::from(1) + U256::from(256256))]
#[case(xorCall::new((U256::from(256256), U256::from(0))), U256::from(256256))]
#[case(xorCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::MAX)]
#[case(xorCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::MAX)]
#[case(xorCall::new((U256::from(0), U256::from(0))), U256::from(0))]
#[case(xorCall::new((U256::MAX, U256::MAX)), U256::from(0))]
fn test_uint_256_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((U256::from(256256), U256::from(256256))), U256::from(256256))]
#[case(andCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(0))]
#[case(andCall::new((U256::from(256256), U256::from(0))), U256::from(0))]
#[case(andCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::from(0))]
#[case(andCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::from(0))]
#[case(andCall::new((U256::from(0), U256::from(0))), U256::from(0))]
#[case(andCall::new((U256::MAX, U256::MAX)), U256::MAX)]
fn test_uint_256_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((U256::from(256256), 0)), U256::from(256256))]
#[case(shiftLeftCall::new((U256::from(256256), 7)), U256::from(256256) << 7)]
#[case(shiftLeftCall::new((U256::from(u128::MAX), 35)), U256::from(u128::MAX) << 35)]
#[case(shiftLeftCall::new((U256::from(u128::MAX), 68)), U256::from(u128::MAX) << 68)]
#[case(shiftLeftCall::new((U256::from(u128::MAX), 100)), U256::from(u128::MAX) << 100)]
#[case(shiftLeftCall::new((U256::MAX, 150)), U256::MAX << 150)]
#[case(shiftLeftCall::new((U256::MAX, 210)), U256::MAX << 210)]
#[case(shiftRightCall::new((U256::from(256256), 0)), U256::from(256256))]
#[case(shiftRightCall::new((U256::from(256256), 7)), U256::from(256256) >> 7)]
#[case(shiftRightCall::new((U256::from(256256), 35)), U256::from(256256) >> 35)]
#[case(shiftRightCall::new((U256::from(u128::MAX), 68)), U256::from(u128::MAX) >> 68)]
#[case(shiftRightCall::new((U256::from(u128::MAX), 100)), U256::from(u128::MAX) >> 100)]
#[case(shiftRightCall::new((U256::MAX, 150)), U256::MAX >> 150)]
#[case(shiftRightCall::new((U256::MAX, 210)), U256::MAX >> 210)]
fn test_uint_256_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
