use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_32", "tests/primitives/move_sources/uint_32.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint32);
    function getLocal(uint32 _z) external returns (uint32);
    function getCopiedLocal() external returns (uint32, uint32);
    function echo(uint32 x) external returns (uint32);
    function echo2(uint32 x, uint32 y) external returns (uint32);
    function sum(uint32 x, uint32 y) external returns (uint32);
    function sub(uint32 x, uint32 y) external returns (uint32);
    function div(uint32 x, uint32 y) external returns (uint32);
    function mul(uint32 x, uint32 y) external returns (uint32);
    function mod_(uint32 x, uint32 y) external returns (uint32);
);

#[rstest]
#[case(getConstantCall::new(()), (3232,))]
#[case(getLocalCall::new((111,)), (50,))]
#[case(getCopiedLocalCall::new(()), (100, 111))]
#[case(echoCall::new((222,)), (222,))]
#[case(echoCall::new((u32::MAX,)), (u32::MAX,))]
#[case(echo2Call::new((111, 222)), (222,))]
#[case(sumCall::new((65535, 65535)), (131070,))]
#[case(sumCall::new((0, 1)), (1,))]
#[case(subCall::new((131070, 65535)), (65535,))]
fn test_uint_32<T: SolCall, V: SolValue>(
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
#[case(sumCall::new((u32::MAX, 1)))]
fn test_uint_32_sum_overflow<T: SolCall>(#[by_ref] runtime: &RuntimeSandbox, #[case] call_data: T) {
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
#[case(subCall::new((65535, 131070)))]
fn test_uint_32_sub_overflow<T: SolCall>(#[by_ref] runtime: &RuntimeSandbox, #[case] call_data: T) {
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
#[case(u32::MAX, 1, u32::MAX)]
#[case(u32::MAX, u32::MAX, 1)]
#[case(u32::MAX, 2, u32::MAX / 2)]
#[case(128, 64, 2)]
#[case(127, 3, 42)]
#[case(1, u32::MAX, 0)]
#[case(0, u32::MAX, 0)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_32_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u32,
    #[case] divisor: u32,
    #[case] expected_result: u32,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&u32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, 5, 0)]
#[case(5, 10, 5)]
#[case(10, 3, 1)]
#[case(u32::MAX, 1, 0)]
#[case(u32::MAX, u16::MAX as u32  + 1, u16::MAX as u32)]
#[case(u32::MAX, u32::MAX - 1, 1)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_32_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u32,
    #[case] divisor: u32,
    #[case] expected_result: u32,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&u32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(0, u32::MAX, 0)]
#[case(u32::MAX, 0, 0)]
#[case(1, u32::MAX, u32::MAX)]
#[case(u32::MAX, 1, u32::MAX)]
#[case(u32::MAX / 2, 2, u32::MAX - 1)]
#[case(21, 4, 84)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(u32::MAX, 2, 0)]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(u16::MAX as u32 + 1, u16::MAX as u32 + 1, 0)]
fn test_uint_32_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u32,
    #[case] n2: u32,
    #[case] expected_result: u32,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&u32,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}
