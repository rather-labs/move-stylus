use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_64", "tests/primitives/move_sources/uint_64.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint64);
    function getLocal(uint64 _z) external returns (uint64);
    function getCopiedLocal() external returns (uint64, uint64);
    function echo(uint64 x) external returns (uint64);
    function echo2(uint64 x, uint64 y) external returns (uint64);
    function sum(uint64 x, uint64 y) external returns (uint64);
    function sub(uint64 x, uint64 y) external returns (uint64);
    function div(uint64 x, uint64 y) external returns (uint64);
    function mul(uint64 x, uint64 y) external returns (uint64);
    function mod_(uint64 x, uint64 y) external returns (uint64);
);

#[rstest]
#[case(getConstantCall::new(()), (6464,))]
#[case(getLocalCall::new((111,)), (50,))]
#[case(getCopiedLocalCall::new(()), (100, 111))]
#[case(echoCall::new((222,)), (222,))]
#[case(echoCall::new((u64::MAX,)), (u64::MAX,))]
#[case(echo2Call::new((111, 222)), (222,))]
#[case(sumCall::new((4294967295, 4294967295)), (8589934590_u64,))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(sumCall::new((u64::MAX, 1)), ())]
#[case(subCall::new((8589934590, 4294967295)), (4294967295_u64,))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(subCall::new((4294967295, 8589934590)), ())]
fn test_uint_64<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
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
#[case(100, 10, 10)]
#[case(0, 5, 0)]
#[case(42, 42, 1)]
#[case(3, 7, 0)]
#[case(u64::MAX, 1, u64::MAX)]
#[case(u64::MAX, u64::MAX, 1)]
#[case(u64::MAX, 2, u64::MAX / 2)]
#[case(128, 64, 2)]
#[case(127, 3, 42)]
#[case(1, u64::MAX, 0)]
#[case(0, u64::MAX, 0)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_64_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u64,
    #[case] divisor: u64,
    #[case] expected_result: u64,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&u64,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, 5, 0)]
#[case(5, 10, 5)]
#[case(10, 3, 1)]
#[case(u64::MAX, 1, 0)]
#[case(u64::MAX, u32::MAX as u64 + 1, u32::MAX as u64)]
#[case(u64::MAX, u64::MAX - 1, 1)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_32_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u64,
    #[case] divisor: u64,
    #[case] expected_result: u64,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&u64,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, u64::MAX, 0)]
#[case(u64::MAX, 0, 0)]
#[case(1, u64::MAX, u64::MAX)]
#[case(u64::MAX, 1, u64::MAX)]
#[case(u64::MAX / 2, 2, u64::MAX - 1)]
#[case(21, 4, 84)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(u64::MAX, 2, 0)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(u32::MAX as u64 + 1, u32::MAX as u64 + 1, 0)]
fn test_uint_64_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u64,
    #[case] n2: u64,
    #[case] expected_result: u64,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&u64,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}
