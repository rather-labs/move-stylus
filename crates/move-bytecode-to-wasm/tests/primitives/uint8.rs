use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_8", "tests/primitives/move_sources/uint_8.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint8);
    function getLocal(uint8 _z) external returns (uint8);
    function getCopiedLocal() external returns (uint8, uint8);
    function echo(uint8 x) external returns (uint8);
    function echo2(uint8 x, uint8 y) external returns (uint8);
    function sum(uint8 x, uint8 y) external returns (uint8);
    function sub(uint8 x, uint8 y) external returns (uint8);
    function div(uint8 x, uint8 y) external returns (uint8);
    function mul(uint8 x, uint8 y) external returns (uint8);
    function mod_(uint8 x, uint8 y) external returns (uint8);
);

#[rstest]
#[case(getConstantCall::new(()), (88,))]
#[case(getLocalCall::new((111,)), (50,))]
#[case(getCopiedLocalCall::new(()), (100, 111))]
#[case(echoCall::new((222,)), (222,))]
#[case(echoCall::new((255,)), (255,))]
#[case(echo2Call::new((111, 222)), (222,))]
#[case(sumCall::new((42, 42)), (84,))]
#[case(subCall::new((84, 42)), (42,))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(subCall::new((42, 84)), ((),))]
fn test_uint_8<T: SolCall, V: SolValue>(
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
#[case(sumCall::new((255, 1)))]
fn test_uint_8_overflow<T: SolCall>(#[by_ref] runtime: &RuntimeSandbox, #[case] call_data: T) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let error_message = String::from_utf8_lossy(RuntimeError::Overflow.as_bytes());
    let expected_data = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&(error_message,)),
    ]
    .concat();
    assert_eq!(return_data, expected_data);
}

#[rstest]
#[case(100, 10, 10)]
#[case(0, 5, 0)]
#[case(42, 42, 1)]
#[case(3, 7, 0)]
#[case(u8::MAX, 1, u8::MAX as i32)]
#[case(u8::MAX, u8::MAX, 1)]
#[case(u8::MAX, 2, (u8::MAX / 2) as i32)]
#[case(128, 64, 2)]
#[case(127, 3, 42)]
#[case(1, u8::MAX, 0)]
#[case(0, u8::MAX, 0)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_8_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u8,
    #[case] divisor: u8,
    #[case] expected_result: i32,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&i32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, 5, 0)]
#[case(5, 10, 5)]
#[case(10, 3, 1)]
#[case(u8::MAX, 1, 0)]
#[case(u8::MAX, 2, 1)]
#[case(u8::MAX, u8::MAX, 0)]
#[case(u8::MAX, u8::MAX - 1, 1)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_8_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u8,
    #[case] divisor: u8,
    #[case] expected_result: i32,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&i32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, u8::MAX, 0)]
#[case(u8::MAX, 0, 0)]
#[case(1, u8::MAX, u8::MAX as i32)]
#[case(u8::MAX, 1, u8::MAX as i32)]
#[case(127, 2, 254)]
#[case(21, 4, 84)]
fn test_uint_8_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u8,
    #[case] n2: u8,
    #[case] expected_result: i32,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&i32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(u8::MAX, 2)]
#[case(16, 16)]
#[case(17, 17)]
fn test_uint_8_mul_overflow(#[by_ref] runtime: &RuntimeSandbox, #[case] n1: u8, #[case] n2: u8) {
    let (result, return_data) = runtime
        .call_entrypoint(mulCall::new((n1, n2)).abi_encode())
        .unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let error_message = String::from_utf8_lossy(RuntimeError::Overflow.as_bytes());
    let expected_data = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&(error_message,)),
    ]
    .concat();
    assert_eq!(return_data, expected_data);
}
