use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_8",
    "tests/operations_bitwise/move_sources/uint_8.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint8 x, uint8 y) external returns (uint8);
    function xor(uint8 x, uint8 y) external returns (uint8);
    function and(uint8 x, uint8 y) external returns (uint8);
    function shiftLeft(uint8 x, uint8 slots) external returns (uint8);
    function shiftRight(uint8 x, uint8 slots) external returns (uint8);
);

#[rstest]
#[case(orCall::new((250, 250)), 250)]
#[case(orCall::new((250, 50)), 250)]
#[case(orCall::new((250, 0)), 250)]
#[case(orCall::new((15, 240)), 255)]
#[case(orCall::new((240, 15)), 255)]
#[case(orCall::new((0, 0)), 0)]
#[case(orCall::new((u8::MAX, u8::MAX)), u8::MAX)]
fn test_uint_8_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u8,
) {
    let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((250, 250)), 0)]
#[case(xorCall::new((250, 50)), 200)]
#[case(xorCall::new((250, 0)), 250)]
#[case(xorCall::new((15, 240)), 255)]
#[case(xorCall::new((240, 15)), 255)]
#[case(xorCall::new((u8::MAX, u8::MAX)), 0)]
#[case(xorCall::new((0, 0)), 0)]
#[case(xorCall::new((u8::MAX, 0)), u8::MAX)]
fn test_uint_8_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u8,
) {
    let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((250, 250)), 250)]
#[case(andCall::new((250, 50)), 50)]
#[case(andCall::new((250, 0)), 0)]
#[case(andCall::new((15, 240)), 0)]
#[case(andCall::new((240, 15)), 0)]
#[case(andCall::new((u8::MAX, u8::MAX)), u8::MAX)]
#[case(andCall::new((0, 0)), 0)]
#[case(andCall::new((u8::MAX, 0)), 0)]
fn test_uint_8_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u8,
) {
    let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((255, 7)), 255 << 7)]
#[case(shiftLeftCall::new((255, 1)), 255 << 1)]
#[case(shiftLeftCall::new((254, 7)), 254 << 7)]
#[case(shiftLeftCall::new((250, 0)), 250)]
#[case(shiftLeftCall::new((250, 4)), 250 << 4)]
#[case(shiftRightCall::new((255, 7)), 255 >> 7)]
#[case(shiftRightCall::new((255, 1)), 255 >> 1)]
#[case(shiftRightCall::new((254, 7)), 254 >> 7)]
#[case(shiftRightCall::new((250, 0)), 250)]
#[case(shiftRightCall::new((250, 4)), 250 >> 4)]
fn test_uint_8_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u8,
) {
    let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((240, 8)))]
#[case(shiftLeftCall::new((240, 10)))]
#[case(shiftLeftCall::new((255, 8)))]
#[case(shiftLeftCall::new((255, 10)))]
#[case(shiftRightCall::new((255, 8)))]
#[case(shiftRightCall::new((255, 10)))]
#[case(shiftRightCall::new((240, 8)))]
#[case(shiftRightCall::new((240, 10)))]
fn test_uint_8_shift_overflow<T: SolCall>(
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
