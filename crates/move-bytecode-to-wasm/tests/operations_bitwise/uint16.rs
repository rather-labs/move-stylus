use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_16",
    "tests/operations_bitwise/move_sources/uint_16.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint16 x, uint16 y) external returns (uint16);
    function xor(uint16 x, uint16 y) external returns (uint16);
    function and(uint16 x, uint16 y) external returns (uint16);
    function shiftLeft(uint16 x, uint8 slots) external returns (uint16);
    function shiftRight(uint16 x, uint8 slots) external returns (uint16);
);

#[rstest]
#[case(orCall::new((1616, 1616)), 1616)]
#[case(orCall::new((1616, u8::MAX as u16 + 1)), u8::MAX as u16 + 1 + 1616)]
#[case(orCall::new((1616, 0)), 1616)]
#[case(orCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), u16::MAX)]
#[case(orCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), u16::MAX)]
#[case(orCall::new((0, 0)), 0)]
#[case(orCall::new((u16::MAX, u16::MAX)), u16::MAX)]
fn test_uint_16_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((1616, 1616)), 0)]
#[case(xorCall::new((1616, u8::MAX as u16 + 1)), u8::MAX as u16 + 1 + 1616)]
#[case(xorCall::new((1616, 0)), 1616)]
#[case(xorCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), u16::MAX)]
#[case(xorCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), u16::MAX)]
#[case(xorCall::new((0, 0)), 0)]
#[case(xorCall::new((u16::MAX, u16::MAX)), 0)]
fn test_uint_16_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((1616, 1616)), 1616)]
#[case(andCall::new((1616, u8::MAX as u16 + 1)), 0)]
#[case(andCall::new((1616, 0)), 0)]
#[case(andCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), 0)]
#[case(andCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), 0)]
#[case(andCall::new((0, 0)), 0)]
#[case(andCall::new((u16::MAX, u16::MAX)), u16::MAX)]
fn test_uint_16_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((1616, 7)), 1616 << 7)]
#[case(shiftLeftCall::new((1616, 1)), 1616 << 1)]
#[case(shiftLeftCall::new((1615, 7)), 1615 << 7)]
#[case(shiftLeftCall::new((1610, 0)), 1610)]
#[case(shiftLeftCall::new((1610, 4)), 1610 << 4)]
#[case(shiftRightCall::new((1616, 7)), 1616 >> 7)]
#[case(shiftRightCall::new((1616, 1)), 1616 >> 1)]
#[case(shiftRightCall::new((1615, 7)), 1615 >> 7)]
#[case(shiftRightCall::new((1610, 0)), 1610)]
#[case(shiftRightCall::new((1610, 4)), 1610 >> 4)]
fn test_uint_16_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftRightCall::new((1600, 16)))]
#[case(shiftRightCall::new((1600, 30)))]
#[case(shiftLeftCall::new((1600, 16)))]
#[case(shiftLeftCall::new((1600, 30)))]
fn test_uint_16_shift_overflow<T: SolCall>(
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
