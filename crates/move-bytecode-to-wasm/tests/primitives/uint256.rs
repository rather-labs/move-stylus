use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, keccak256};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("uint_256", "tests/primitives/move_sources/uint_256.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint256);
    function getLocal(uint256 _z) external returns (uint256);
    function getCopiedLocal() external returns (uint256, uint256);
    function echo(uint256 x) external returns (uint256);
    function echo2(uint256 x, uint256 y) external returns (uint256);
    function sum(uint256 x, uint256 y) external returns (uint256);
    function sub(uint256 x, uint256 y) external returns (uint256);
    function mul(uint256 x, uint256 y) external returns (uint256);
    function div(uint256 x, uint256 y) external returns (uint256);
    function mod_(uint256 x, uint256 y) external returns (uint256);
);

#[rstest]
#[case(getConstantCall::new(()), (256256,))]
#[case(getLocalCall::new((U256::from(111),)), (U256::from(50),))]
#[case(getCopiedLocalCall::new(()), (U256::from(100), U256::from(111)))]
#[case(echoCall::new((U256::from(222),)), (U256::from(222),))]
#[case(echoCall::new((U256::MAX,)), (U256::MAX,))]
#[case(echo2Call::new((U256::from(111),U256::from(222))), (U256::from(222),))]
fn test_uint_256<T: SolCall, V: SolValue>(
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
//    numbers in the form 2^(n*32) where n=1,2,3,4,5,6,7,8.
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
// This tests are repeated for all the 32 bits chunks in the 256bits so we test a big number
// that does not overflows
#[rstest]
#[case(sumCall::new((U256::from(1), U256::from(1))), (U256::from(2),))]
#[case(
        sumCall::new((
            U256::from(4294967295_u128),
            U256::from(4294967295_u128)
        )),
        (U256::from(8589934590_u128),))
    ]
#[case(
        sumCall::new((
            U256::from(4294967296_u128),
            U256::from(4294967296_u128)
        )),
        (U256::from(8589934592_u128),))
    ]
#[case(
        sumCall::new((
            U256::from(18446744073709551615_u128),
            U256::from(18446744073709551615_u128)
        )),
        (U256::from(36893488147419103230_u128),))
    ]
#[case(
        sumCall::new((
            U256::from(18446744073709551616_u128),
            U256::from(18446744073709551616_u128)
        )),
        (U256::from(36893488147419103232_u128),))
    ]
#[case(
        sumCall::new(
            (U256::from(79228162514264337593543950335_u128),
            U256::from(79228162514264337593543950335_u128)
        )),
        (U256::from(158456325028528675187087900670_u128),))
    ]
#[case(
        sumCall::new((
            U256::from(79228162514264337593543950336_u128),
            U256::from(79228162514264337593543950336_u128)
        )),
        (U256::from(158456325028528675187087900672_u128),))
    ]
#[case(
        sumCall::new((
           U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
           U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        )),
        (U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),)
    )]
#[case(
        sumCall::new((
           U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
           U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        )),
        (U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),)
    )]
#[case(
        sumCall::new((
           U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
           U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
        )),
        (U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),)
    )]
fn test_uint_256_sum<T: SolCall, V: SolValue>(
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
#[case(sumCall::new((U256::MAX, U256::from(42))))]
fn test_uint_256_sum_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
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
#[case(subCall::new((U256::from(2), U256::from(1))), (1,))]
#[case(subCall::new((U256::from(8589934590_u128), U256::from(4294967295_u128))), (4294967295_u128,))]
#[case(subCall::new((U256::from(8589934592_u128), U256::from(4294967296_u128))), (4294967296_u128,))]
#[case(subCall::new((U256::from(36893488147419103230_u128), U256::from(18446744073709551615_u128))), (18446744073709551615_u128,))]
#[case(subCall::new((U256::from(36893488147419103232_u128), U256::from(18446744073709551616_u128))), (18446744073709551616_u128,))]
#[case(subCall::new((U256::from(158456325028528675187087900670_u128), U256::from(79228162514264337593543950335_u128))), (79228162514264337593543950335_u128,))]
#[case(subCall::new((U256::from(158456325028528675187087900672_u128), U256::from(79228162514264337593543950336_u128))), (79228162514264337593543950336_u128,))]
fn test_uint_256_sub<T: SolCall, V: SolValue>(
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
#[case(subCall::new((U256::from(1), U256::from(2))))]
#[case(subCall::new((U256::from(4294967296_u128), U256::from(8589934592_u128))))]
#[case(subCall::new((U256::from(18446744073709551616_u128), U256::from(36893488147419103232_u128))))]
#[case(subCall::new((U256::from(79228162514264337593543950336_u128), U256::from(158456325028528675187087900672_u128))))]
#[case(
    subCall::new((
        U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),
    ))
)]
#[case(
    subCall::new((
        U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),
    ))
)]
#[case(
    subCall::new((
        U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
        U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),
    ))
)]
#[case(subCall::new((U256::from(1), U256::from(u128::MAX))))]
#[case(subCall::new((U256::from(1), U256::MAX)))]
fn test_uint_256_sub_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
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
#[case(U256::from(2), U256::from(2), U256::from(4))]
#[case(U256::from(0), U256::from(2), U256::from(0))]
#[case(U256::from(2), U256::from(0), U256::from(0))]
#[case(U256::from(1), U256::from(1), U256::from(1))]
#[case(U256::from(5), U256::from(5), U256::from(25))]
#[case(U256::from(u64::MAX), U256::from(2), U256::from(u64::MAX as u128 * 2))]
#[case(U256::from(2), U256::from(u64::MAX), U256::from(u64::MAX as u128 * 2))]
#[case(
        U256::from(2),
        U256::from(u64::MAX as u128 + 1),
        U256::from((u64::MAX as u128 + 1) * 2)
    )]
#[case(
        U256::from(u64::MAX),
        U256::from(u64::MAX),
        U256::from(u64::MAX as u128 * u64::MAX as u128)
    )]
#[case::t_2pow63_x_2pow63(
    U256::from(9_223_372_036_854_775_808_u128),
    U256::from(9_223_372_036_854_775_808_u128),
    U256::from(85_070_591_730_234_615_865_843_651_857_942_052_864_u128)
)]
#[case(
        U256::from(u128::MAX),
        U256::from(2),
        U256::from(u128::MAX) * U256::from(2)
    )]
#[case(
        U256::from(u128::MAX),
        U256::from(5),
        U256::from(u128::MAX) * U256::from(5)
    )]
#[case(
        U256::from(u128::MAX),
        U256::from(u128::MAX),
        U256::from(u128::MAX) * U256::from(u128::MAX)
    )]
#[case(
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2) * U256::from(u64::MAX as u128 * 2),
    )]
#[case(
        U256::from(2),
        U256::from(u128::MAX) + U256::from(1),
        (U256::from(u128::MAX) + U256::from(1)) * U256::from(2)
    )]
// asd
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(U256::MAX, U256::from(2), U256::from(0))]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(U256::MAX, U256::from(5), U256::from(0))]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(U256::MAX, U256::MAX, U256::from(0))]
#[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
#[case(
        U256::from(u128::MAX) * U256::from(2),
        U256::from(u128::MAX) * U256::from(2),
        U256::from(0),
    )]
fn test_uint_256_mul(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] n1: U256,
    #[case] n2: U256,
    #[case] expected_result: U256,
) {
    run_test(
        runtime,
        mulCall::new((n1, n2)).abi_encode(),
        <(&U256,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(U256::from(350), U256::from(13), U256::from(26))]
#[case(U256::from(5), U256::from(2), U256::from(2))]
#[case(U256::from(123456), U256::from(1), U256::from(123456))]
#[case(U256::from(987654321), U256::from(123456789), U256::from(8))]
#[case(U256::from(0), U256::from(2), U256::from(0))]
// 2^96 / 2^32 = [q = 2^64, r = 0]
#[case(
    U256::from(79228162514264337593543950336_u128),
    U256::from(4294967296_u128),
    U256::from(18446744073709551616_u128)
)]
// 2^192 / 2^64 = [q = 2^128, r = 0]
#[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(u128::MAX) + U256::from(1),
    )]
//#[should_panic(expected = "wasm trap: integer divide by zero")]
//#[case(10, 0, 0)]
fn test_uint_256_div(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: U256,
    #[case] divisor: U256,
    #[case] expected_result: U256,
) {
    run_test(
        runtime,
        divCall::new((dividend, divisor)).abi_encode(),
        <(&U256,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(U256::from(350), U256::from(13), U256::from(12))]
#[case(U256::from(5), U256::from(2), U256::from(1))]
#[case(U256::from(123456), U256::from(1), U256::from(0))]
#[case(U256::from(987654321), U256::from(123456789), U256::from(9))]
#[case(U256::from(0), U256::from(2), U256::from(0))]
// 2^96 / 2^32 = [q = 2^64, r = 0]
#[case(
    U256::from(79228162514264337593543950336_u128),
    U256::from(4294967296_u128),
    U256::from(0)
)]
// 2^192 / 2^64 = [q = 2^128, r = 0]
#[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(0)
    )]
#[should_panic(expected = "wasm trap: integer divide by zero")]
#[case(U256::from(10), U256::from(0), U256::from(0))]
fn test_uint_256_mod(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] dividend: U256,
    #[case] divisor: U256,
    #[case] expected_result: U256,
) {
    run_test(
        runtime,
        mod_Call::new((dividend, divisor)).abi_encode(),
        <(&U256,)>::abi_encode(&(&expected_result,)),
    )
    .unwrap();
}
