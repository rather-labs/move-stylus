use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_256",
    "tests/operations_cast/move_sources/uint_256.move"
);

sol!(
    #[allow(missing_docs)]
    function castUp(uint16 x) external returns (uint256);
    function castUpU64(uint64 x) external returns (uint256);
    function castUpU128(uint128 x) external returns (uint256);
);

#[rstest]
#[case(castUpCall::new((3232,)), U256::from(3232))]
#[case(castUpCall::new((u16::MAX,)), U256::from(u16::MAX))]
#[case(castUpU64Call::new((6464,)), U256::from(6464))]
#[case(castUpU64Call::new((u64::MAX,)), U256::from(u64::MAX))]
#[case(castUpU128Call::new((128128,)), U256::from(128128))]
#[case(castUpU128Call::new((u128::MAX,)), U256::from(u128::MAX))]
fn test_uint_128_cast<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
