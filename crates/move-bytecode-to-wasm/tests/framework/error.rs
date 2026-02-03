use crate::common::runtime;
use alloy_primitives::{U256, address, keccak256};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    struct SimpleError {
        string e;
    }

    struct CustomError {
        string error_message;
        uint64 error_code;
    }

    struct CustomError2 {
        bool a;
        uint8 b;
        uint16 c;
        uint32 d;
        uint64 e;
        uint128 f;
        uint256 g;
        address h;
    }

    struct CustomError3 {
        uint32[] a;
        uint128[] b;
        uint64[][] c;
    }

    struct CustomError4 {
        SimpleError a;
        CustomError b;
    }

    struct NestedStruct1 {
        string e;
    }
    struct NestedStruct2 {
        string a;
        uint64 b;
    }

    function revertStandardError(string s) external;
    function revertCustomError(string s, uint64 code) external;
    function revertCustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h) external;
    function revertCustomError3(uint32[] a, uint128[] b, uint64[][] c) external;
    function revertCustomError4(string a, string b, uint64 c) external;
    function abortWithErrorMacro() external;
);

#[rstest]
#[case(
        revertStandardErrorCall::new((String::from("Not enough Ether provided."),)),
        [
            keccak256(b"SimpleError(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&("Not enough Ether provided.",)),
        ].concat()
    )]
#[case(
        revertCustomErrorCall::new((
            String::from("Custom error message"),
            42,
        )),
        [
            keccak256(b"CustomError(string,uint64)")[..4].to_vec(),
            <sol!((string, uint64))>::abi_encode_params(&(
                "Custom error message",
                42,
            )),
        ].concat()
    )]
#[case(
        revertCustomError2Call::new((true, 2u8, 3u16, 4u32, 5u64, 5u128, U256::from(5), address!("0xffffffffffffffffffffffffffffffffffffffff"))),
        [
            keccak256(b"CustomError2(bool,uint8,uint16,uint32,uint64,uint128,uint256,address)")[..4].to_vec(),
            <sol!((bool, uint8, uint16, uint32, uint64, uint128, uint256, address))>::abi_encode_params(&(true, 2u8, 3u16, 4u32, 5u64, 5u128, U256::from(5), address!("0xffffffffffffffffffffffffffffffffffffffff"))),
        ].concat()
    )]
#[case(
        revertCustomError3Call::new((vec![1, 2, 3], vec![4, 5], vec![vec![6, 7, 8], vec![9, 10, 11]])),
        [
            keccak256(b"CustomError3(uint32[],uint128[],uint64[][])")[..4].to_vec(),
            <sol!((uint32[], uint128[], uint64[][]))>::abi_encode_params(&(vec![1, 2, 3], vec![4, 5], vec![vec![6, 7, 8], vec![9, 10, 11]])),
        ].concat()
    )]
#[case(
        revertCustomError4Call::new((
            String::from("Custom error message"),
            String::from("Custom error message 2"),
            42,
        )),
        [
            keccak256(b"CustomError4((string),(string,uint64))")[..4].to_vec(),
            {
                let params = (
                    NestedStruct1 { e: String::from("Custom error message") },
                    NestedStruct2 { a: String::from("Custom error message 2"), b: 42 },
                );
                <sol!((NestedStruct1, NestedStruct2)) as alloy_sol_types::SolValue>::abi_encode_params(&params)
            },
        ].concat()
    )]
#[case(
        abortWithErrorMacroCall::new(()),
        [
            keccak256(b"Error(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&("Error for testing #[error] macro",)),
        ].concat()
    )]
fn test_revert<T: SolCall>(
    #[with("error", "tests/framework/move_sources/error.move")] runtime: RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_data: Vec<u8>,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(1, result);

    assert_eq!(return_data, expected_data);
}
