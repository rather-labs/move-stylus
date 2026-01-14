use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_128",
    "tests/operations_bitwise/move_sources/uint_128.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint128 x, uint128 y) external returns (uint128);
    function xor(uint128 x, uint128 y) external returns (uint128);
    function and(uint128 x, uint128 y) external returns (uint128);
    function shiftLeft(uint128 x, uint8 slots) external returns (uint128);
    function shiftRight(uint128 x, uint8 slots) external returns (uint128);
);

#[rstest]
#[case(orCall::new((128128, 128128)), 128128)]
#[case(orCall::new((128128, u64::MAX as u128 + 1)), u64::MAX as u128 + 1 + 128128)]
#[case(orCall::new((128128, 0)), 128128)]
#[case(orCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), u128::MAX)]
#[case(orCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), u128::MAX)]
#[case(orCall::new((0, 0)), 0)]
#[case(orCall::new((u128::MAX, u128::MAX)), u128::MAX)]
fn test_uint_128_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u128,
) {
    let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((128128, 128128)), 0)]
#[case(xorCall::new((128128, u64::MAX as u128 + 1)), u64::MAX as u128 + 1 + 128128)]
#[case(xorCall::new((128128, 0)), 128128)]
#[case(xorCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), u128::MAX)]
#[case(xorCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), u128::MAX)]
#[case(xorCall::new((0, 0)), 0)]
#[case(xorCall::new((u128::MAX, u128::MAX)), 0)]
fn test_uint_128_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u128,
) {
    let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((128128, 128128)), 128128)]
#[case(andCall::new((128128, u64::MAX as u128 + 1)), 0)]
#[case(andCall::new((128128, 0)), 0)]
#[case(andCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), 0)]
#[case(andCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), 0)]
#[case(andCall::new((0, 0)), 0)]
#[case(andCall::new((u128::MAX, u128::MAX)), u128::MAX)]
fn test_uint_128_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u128,
) {
    let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((128128, 7)), 128128 << 7)]
#[case(shiftLeftCall::new((128128, 35)), 128128 << 35)]
#[case(shiftLeftCall::new((128127, 68)), 128127 << 68)]
#[case(shiftLeftCall::new((128122, 0)), 128122)]
#[case(shiftLeftCall::new((128122, 100)), 128122 << 100)]
#[case(shiftRightCall::new((128128, 7)), 128128 >> 7)]
#[case(shiftRightCall::new((128128, 35)), 128128 >> 35)]
#[case(shiftRightCall::new((128127, 68)), 128127 >> 68)]
#[case(shiftRightCall::new((128122, 0)), 128122)]
#[case(shiftRightCall::new((128122, 100)), 128122 >> 100)]
fn test_uint_128_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u128,
) {
    let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftRightCall::new((128000, 128)))]
#[case(shiftRightCall::new((128000, 240)))]
#[case(shiftLeftCall::new((128000, 128)))]
#[case(shiftLeftCall::new((128000, 250)))]
fn test_uint_128_shift_overflow<T: SolCall>(
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
