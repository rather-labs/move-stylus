use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_16", "tests/primitives/move_sources/uint_16.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint16);
    function getLocal(uint16 _z) external returns (uint16);
    function getCopiedLocal() external returns (uint16, uint16);
    function echo(uint16 x) external returns (uint16);
    function echo2(uint16 x, uint16 y) external returns (uint16);
    function sum(uint16 x, uint16 y) external returns (uint16);
    function sub(uint16 x, uint16 y) external returns (uint16);
    function div(uint16 x, uint16 y) external returns (uint16);
    function mul(uint16 x, uint16 y) external returns (uint16);
    function mod_(uint16 x, uint16 y) external returns (uint16);
);

#[rstest]
#[case(getConstantCall::new(()), (1616,))]
#[case(getLocalCall::new((111,)), (50,))]
#[case(getCopiedLocalCall::new(()), (100, 111))]
#[case(echoCall::new((222,)), (222,))]
#[case(echoCall::new((u16::MAX,)), (u16::MAX,))]
#[case(echo2Call::new((111, 222)), (222,))]
#[case(sumCall::new((255, 255)), (510,))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(sumCall::new((u16::MAX, 1)), ((),))]
#[case(subCall::new((510, 255)), (255,))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(subCall::new((255, 510)), ((),))]
fn test_uint_16<T: SolCall, V: SolValue>(
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
#[case(u16::MAX, 1, u16::MAX)]
#[case(u16::MAX, u16::MAX, 1)]
#[case(u16::MAX, 2, u16::MAX / 2)]
#[case(128, 64, 2)]
#[case(127, 3, 42)]
#[case(1, u16::MAX, 0)]
#[case(0, u16::MAX, 0)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_16_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u16,
    #[case] divisor: u16,
    #[case] expected_result: u16,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&u16,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, 5, 0)]
#[case(5, 10, 5)]
#[case(10, 3, 1)]
#[case(u16::MAX, 1, 0)]
#[case(u16::MAX, u8::MAX as u16 + 1, u8::MAX as u16)]
#[case(u16::MAX, u16::MAX - 1, 1)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_16_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u16,
    #[case] divisor: u16,
    #[case] expected_result: u16,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&u16,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, u16::MAX, 0)]
#[case(u16::MAX, 0, 0)]
#[case(1, u16::MAX, u16::MAX)]
#[case(u16::MAX, 1, u16::MAX)]
#[case(32767, 2, 65534)]
#[case(21, 4, 84)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(u16::MAX, 2, 0)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(256, 256, 0)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(256, 257, 0)]
fn test_uint_16_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u16,
    #[case] n2: u16,
    #[case] expected_result: u16,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&u16,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}
