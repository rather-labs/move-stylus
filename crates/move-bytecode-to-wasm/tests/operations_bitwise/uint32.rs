use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "bitwise_uint_32",
    "tests/operations_bitwise/move_sources/uint_32.move"
);

sol!(
    #[allow(missing_docs)]
    function or(uint32 x, uint32 y) external returns (uint32);
    function xor(uint32 x, uint32 y) external returns (uint32);
    function and(uint32 x, uint32 y) external returns (uint32);
    function shiftLeft(uint32 x, uint8 slots) external returns (uint32);
    function shiftRight(uint32 x, uint8 slots) external returns (uint32);
);

#[rstest]
#[case(orCall::new((3232, 3232)), 3232)]
#[case(orCall::new((3232, u16::MAX as u32 + 1)), u16::MAX as u32 + 1 + 3232)]
#[case(orCall::new((3232, 0)), 3232)]
#[case(orCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), u32::MAX)]
#[case(orCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), u32::MAX)]
#[case(orCall::new((0, 0)), 0)]
#[case(orCall::new((u32::MAX, u32::MAX)), u32::MAX)]
fn test_uint_32_or<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    println!("expected_result: {expected_result}");
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(xorCall::new((3232, 3232)), 0)]
#[case(xorCall::new((3232, u16::MAX as u32 + 1)), u16::MAX as u32 + 1 + 3232)]
#[case(xorCall::new((3232, 0)), 3232)]
#[case(xorCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), u32::MAX)]
#[case(xorCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), u32::MAX)]
#[case(xorCall::new((0, 0)), 0)]
#[case(xorCall::new((u32::MAX, u32::MAX)), 0)]
fn test_uint_32_xor<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(andCall::new((3232, 3232)), 3232)]
#[case(andCall::new((3232, u16::MAX as u32 + 1)), 0)]
#[case(andCall::new((3232, 0)), 0)]
#[case(andCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), 0)]
#[case(andCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), 0)]
#[case(andCall::new((0, 0)), 0)]
#[case(andCall::new((u32::MAX, u32::MAX)), u32::MAX)]
fn test_uint_32_and<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((3232, 7)), 3232 << 7)]
#[case(shiftLeftCall::new((3232, 1)), 3232 << 1)]
#[case(shiftLeftCall::new((3231, 7)), 3231 << 7)]
#[case(shiftLeftCall::new((3226, 0)), 3226)]
#[case(shiftLeftCall::new((3226, 4)), 3226 << 4)]
#[case(shiftRightCall::new((3232, 7)), 3232 >> 7)]
#[case(shiftRightCall::new((3232, 1)), 3232 >> 1)]
#[case(shiftRightCall::new((3231, 7)), 3231 >> 7)]
#[case(shiftRightCall::new((3226, 0)), 3226)]
#[case(shiftRightCall::new((3226, 4)), 3226 >> 4)]
fn test_uint_32_shift<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(shiftLeftCall::new((3200, 32)))]
#[case(shiftLeftCall::new((3200, 50)))]
#[case(shiftRightCall::new((3200, 32)))]
#[case(shiftRightCall::new((3200, 50)))]
fn test_uint_32_shift_overflow<T: SolCall>(
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
