use crate::common::run_test;
use crate::common::runtime;
use alloy_primitives::fixed_bytes;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function testBytes4AsVec(bytes4 b) external returns (uint8[]);
    function testBytes1AsVec(bytes1 b) external returns (uint8[]);
    function testBytes2AsVec(bytes2 b) external returns (uint8[]);
    function testBytes8AsVec(bytes8 b) external returns (uint8[]);
    function testBytes16AsVec(bytes16 b) external returns (uint8[]);
    function testBytes32AsVec(bytes32 b) external returns (uint8[]);
    function testMixedBytesAsVec(bytes4 a, bytes4 b, bytes8 c, bytes16 d) external returns (bool);
);

#[rstest]
#[case(testBytes1AsVecCall::new((fixed_bytes!("01"),)), vec![0x01])]
#[case(testBytes2AsVecCall::new((fixed_bytes!("1122"),)), vec![0x11, 0x22])]
#[case(testBytes4AsVecCall::new((fixed_bytes!("01020304"),)), vec![0x01, 0x02, 0x03, 0x04])]
#[case(testBytes8AsVecCall::new((fixed_bytes!("0102030405060708"),)), vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])]
#[case(testBytes16AsVecCall::new((fixed_bytes!("0102030405060708090A0B0C0D0E0F10"),)), vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10])]
#[case(testBytes32AsVecCall::new((fixed_bytes!("0102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F20"),)), vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20])]
#[case(testMixedBytesAsVecCall::new((fixed_bytes!("01020304"), fixed_bytes!("05060708"), fixed_bytes!("090A0B0C0D0E0F10"), fixed_bytes!("0102030405060708090A0B0C0D0E0F10"))), (true,))]
fn test_bytes4_as_vec<T: SolCall, V: SolValue>(
    #[by_ref]
    #[with("bytes", "tests/primitives/move_sources/bytes.move")]
    runtime: &RuntimeSandbox,
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
