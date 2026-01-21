use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_64",
    "tests/operations_bitwise/move_sources/uint_64.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint64 x, uint64 y) external returns (uint64);
    function xor(uint64 x, uint64 y) external returns (uint64);
    function and(uint64 x, uint64 y) external returns (uint64);
    function shiftLeft(uint64 x, uint8 slots) external returns (uint64);
    function shiftRight(uint64 x, uint8 slots) external returns (uint64);
);

#[rstest]
#[case(orCall::new((6464, 6464)), 6464)]
#[case(orCall::new((6464, u32::MAX as u64 + 1)), u32::MAX as u64 + 1 + 6464)]
#[case(orCall::new((6464, 0)), 6464)]
#[case(orCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), u64::MAX)]
#[case(orCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), u64::MAX)]
#[case(orCall::new((0, 0)), 0)]
#[case(orCall::new((u64::MAX, u64::MAX)), u64::MAX)]
fn test_uint_64_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((6464, 6464)), 0)]
#[case(xorCall::new((6464, u32::MAX as u64 + 1)), u32::MAX as u64 + 1 + 6464)]
#[case(xorCall::new((6464, 0)), 6464)]
#[case(xorCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), u64::MAX)]
#[case(xorCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), u64::MAX)]
#[case(xorCall::new((0, 0)), 0)]
#[case(xorCall::new((u64::MAX, u64::MAX)), 0)]
fn test_uint_64_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((6464, 6464)), 6464)]
#[case(andCall::new((6464, u32::MAX as u64 + 1)), 0)]
#[case(andCall::new((6464, 0)), 0)]
#[case(andCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), 0)]
#[case(andCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), 0)]
#[case(andCall::new((0, 0)), 0)]
#[case(andCall::new((u64::MAX, u64::MAX)), u64::MAX)]
fn test_uint_64_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((6464, 7)), 6464 << 7)]
#[case(shiftLeftCall::new((6464, 1)), 6464 << 1)]
#[case(shiftLeftCall::new((6463, 7)), 6463 << 7)]
#[case(shiftLeftCall::new((6458, 0)), 6458)]
#[case(shiftLeftCall::new((6458, 4)), 6458 << 4)]
#[case(shiftRightCall::new((6464, 7)), 6464 >> 7)]
#[case(shiftRightCall::new((6464, 1)), 6464 >> 1)]
#[case(shiftRightCall::new((6463, 7)), 6463 >> 7)]
#[case(shiftRightCall::new((6458, 0)), 6458)]
#[case(shiftRightCall::new((6458, 4)), 6458 >> 4)]
fn test_uint_64_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((6400, 64)))]
#[case(shiftLeftCall::new((6400, 100)))]
#[case(shiftRightCall::new((6400, 64)))]
#[case(shiftRightCall::new((6400, 100)))]
fn test_uint_64_shift_overflow<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    // Functions should return 1 in case of overflow
    assert_eq!(result, 1_i32);
    let expected_data = RuntimeError::Overflow.encode_abi();
    assert_eq!(return_data, expected_data);
}
