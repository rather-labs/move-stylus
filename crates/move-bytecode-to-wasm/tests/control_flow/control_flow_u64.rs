use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "control_flow_u64",
    "tests/control_flow/move_sources/control_flow_u64.move"
);

sol!(
    #[allow(missing_docs)]
    function collatz(uint64 x) external returns (uint64);
    function fibonacci(uint64 n) external returns (uint64);
    function isPrime(uint64 i) external returns (bool);
    function sumSpecial(uint64 n) external returns (uint64);
);

#[rstest]
#[case(collatzCall::new((1u64,)), 0u64)]
#[case(collatzCall::new((2u64,)), 1u64)]
#[case(collatzCall::new((3u64,)), 7u64)]
#[case(collatzCall::new((4u64,)), 2u64)]
#[case(collatzCall::new((5u64,)), 5u64)]
#[case(collatzCall::new((6u64,)), 8u64)]
#[case(collatzCall::new((7u64,)), 16u64)]
#[case(collatzCall::new((8u64,)), 3u64)]
#[case(collatzCall::new((9u64,)), 19u64)]
#[case(collatzCall::new((10u64,)), 6u64)]
#[case(fibonacciCall::new((0u64,)), 0u64)]
#[case(fibonacciCall::new((1u64,)), 1u64)]
#[case(fibonacciCall::new((2u64,)), 1u64)]
#[case(fibonacciCall::new((3u64,)), 2u64)]
#[case(fibonacciCall::new((4u64,)), 3u64)]
#[case(fibonacciCall::new((5u64,)), 5u64)]
#[case(isPrimeCall::new((1u64,)), 0)]
#[case(isPrimeCall::new((2u64,)), 1)]
#[case(isPrimeCall::new((3u64,)), 1)]
#[case(isPrimeCall::new((4u64,)), 0)]
#[case(isPrimeCall::new((5u64,)), 1)]
#[case(isPrimeCall::new((7u64,)), 1)]
#[case(isPrimeCall::new((13u64,)), 1)]
#[case(isPrimeCall::new((53u64,)), 1)]
#[case(isPrimeCall::new((54u64,)), 0)]
#[case(sumSpecialCall::new((0u64,)), 0u64)]
#[case(sumSpecialCall::new((1u64,)), 0u64)]
#[case(sumSpecialCall::new((2u64,)), 7u64)]
#[case(sumSpecialCall::new((3u64,)), 14u64)]
#[case(sumSpecialCall::new((4u64,)), 14u64)]
fn test_control_flow_u64<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        <sol!((uint64,))>::abi_encode(&(expected_result,)),
    )
    .unwrap();
}
