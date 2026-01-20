use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_128", "tests/primitives/move_sources/uint_128.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint128);
    function getLocal(uint128 _z) external returns (uint128);
    function getCopiedLocal() external returns (uint128, uint128);
    function echo(uint128 x) external returns (uint128);
    function echo2(uint128 x, uint128 y) external returns (uint128);
    function sum(uint128 x, uint128 y) external returns (uint128);
    function sub(uint128 x, uint128 y) external returns (uint128);
    function mul(uint128 x, uint128 y) external returns (uint128);
    function div(uint128 x, uint128 y) external returns (uint128);
    function mod_(uint128 x, uint128 y) external returns (uint128);
);

#[rstest]
#[case(getConstantCall::new(()), (128128,))]
#[case(getLocalCall::new((111,)), (50,))]
#[case(getCopiedLocalCall::new(()), (100, 111))]
#[case(echoCall::new((222,)), (222,))]
#[case(echoCall::new((u128::MAX,)), (u128::MAX,))]
#[case(echo2Call::new((111, 222)), (222,))]
fn test_uint_128<T: SolCall, V: SolValue>(
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

// The following tests test two situations:
// 1. What happens when there is carry: we process the sum in chunks of 32 bits, so we use
//    numbers in the form 2^(n*32) where n=1,2,3,4.
//    If we add two numbers 2^(n*64) - 1, wthe first 64 bits will overflow and we will have to
//    take care of the carry.
//
//    For example
//    2^64 - 1 = [0, ..., 0, 0, 255, 255, 255, 255]
//
// 2. What happens if there is not carry :
//    If we add two numbers 2^(n*64), the first 64 bits will of both numbers will be zero, so,
//    when we add them there will be no carry at the beginning.
//
//    For example
//    2^64     = [0, ..., 0, 0, 1, 0, 0, 0, 0]
//
// This tests are repeated for all the 32 bits chunks in the 128bits so we test a big number
// that does not overflows
#[rstest]
#[case(sumCall::new((1,1)), (2,))]
#[case(sumCall::new((4294967295,4294967295)), (8589934590_u128,))]
#[case(sumCall::new((4294967296,4294967296)), (8589934592_u128,))]
#[case(sumCall::new((18446744073709551615,18446744073709551615)), (36893488147419103230_u128,))]
#[case(sumCall::new((18446744073709551616,18446744073709551616)), (36893488147419103232_u128,))]
#[case(sumCall::new((79228162514264337593543950335,79228162514264337593543950335)), (158456325028528675187087900670_u128,))]
#[case(sumCall::new((79228162514264337593543950336,79228162514264337593543950336)), (158456325028528675187087900672_u128,))]
fn test_uint_128_sum<T: SolCall, V: SolValue>(
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
#[case(sumCall::new((u128::MAX, 42)))]
fn test_uint_128_sum_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let expected_data = RuntimeError::Overflow.encode_abi();
    assert_eq!(return_data, expected_data);
}

#[rstest]
#[case(subCall::new((2,1)), (1,))]
#[case(subCall::new((8589934590, 4294967295)), (4294967295_u128,))]
#[case(subCall::new((8589934592, 4294967296)), (4294967296_u128,))]
#[case(subCall::new((36893488147419103232, 18446744073709551616)), (18446744073709551616_u128,))]
#[case(subCall::new((158456325028528675187087900670, 79228162514264337593543950335)), (79228162514264337593543950335_u128,))]
#[case(subCall::new((158456325028528675187087900672, 79228162514264337593543950336)), (79228162514264337593543950336_u128,))]
#[case(subCall::new((36893488147419103230, 18446744073709551615)), (18446744073709551615_u128,))]
fn test_uint_128_sub<T: SolCall, V: SolValue>(
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
#[case(subCall::new((1, 2)))]
#[case(subCall::new((4294967296, 8589934592)))]
#[case(subCall::new((18446744073709551616, 36893488147419103232)))]
#[case(subCall::new((79228162514264337593543950336, 158456325028528675187087900672)))]
#[case(subCall::new((1, u128::MAX)))]
fn test_uint_128_sub_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let expected_data = RuntimeError::Overflow.encode_abi();
    assert_eq!(return_data, expected_data);
}

#[rstest]
#[case(2, 2, 4)]
#[case(0, 2, 0)]
#[case(2, 0, 0)]
#[case(1, 1, 1)]
#[case(5, 5, 25)]
#[case(u64::MAX as u128, 2, u64::MAX as u128 * 2)]
#[case(2, u64::MAX as u128, u64::MAX as u128 * 2)]
#[case(2, u64::MAX as u128 + 1, (u64::MAX as u128 + 1) * 2)]
#[case(u64::MAX as u128, u64::MAX as u128, u64::MAX as u128 * u64::MAX as u128)]
#[case::t_2pow63_x_2pow63(
    9_223_372_036_854_775_808,
    9_223_372_036_854_775_808,
    85_070_591_730_234_615_865_843_651_857_942_052_864
)]
fn test_uint_128_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u128,
    #[case] n2: u128,
    #[case] expected_result: u128,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&u128,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(u128::MAX, 2)]
#[case(u128::MAX, 5)]
#[case(u128::MAX, u64::MAX as u128)]
#[case(u64::MAX as u128 * 2, u64::MAX as u128 * 2)]
fn test_uint_128_mul_overflow(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: u128,
    #[case] n2: u128,
) {
    let (result, return_data) = runtime
        .call_entrypoint(mulCall::new((n1, n2)).abi_encode())
        .unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let expected_data = RuntimeError::Overflow.encode_abi();
    assert_eq!(return_data, expected_data);
}

#[rstest]
#[case(350, 13, 26)]
#[case(5, 2, 2)]
#[case(123456, 1, 123456)]
#[case(987654321, 123456789, 8)]
#[case(0, 2, 0)]
// 2^96 / 2^32 = [q = 2^64, r = 0]
#[case(79228162514264337593543950336, 4294967296, 18446744073709551616)]
//#[should_panic(expected = "wasm trap: integer divide by zero")]
//#[case(10, 0, 0)]
fn test_uint_128_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u128,
    #[case] divisor: u128,
    #[case] expected_result: u128,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&u128,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(350, 13, 12)]
#[case(5, 2, 1)]
#[case(123456, 1, 0)]
#[case(987654321, 123456789, 9)]
#[case(0, 2, 0)]
// 2^96 / 2^32 = [q = 2^64, r = 0]
#[case(79228162514264337593543950336, 4294967296, 0)]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(10, 0, 0)]
fn test_uint_128_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: u128,
    #[case] divisor: u128,
    #[case] expected_result: u128,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&u128,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}
