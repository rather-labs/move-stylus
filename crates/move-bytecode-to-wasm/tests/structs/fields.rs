use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "struct_fields",
    "tests/structs/move_sources/struct_fields.move"
);

sol!(
    #[allow(missing_docs)]
    function echoBool(bool a) external returns (bool);
    function echoU8(uint8 a) external returns (uint8);
    function echoU16(uint16 a) external returns (uint16);
    function echoU32(uint32 a) external returns (uint32);
    function echoU64(uint64 a) external returns (uint64);
    function echoU128(uint128 a) external returns (uint128);
    function echoU256(uint256 a) external returns (uint256);
    function echoVecStackType(uint32[] a) external returns (uint32[]);
    function echoVecHeapType(uint128[] a) external returns (uint128[]);
    function echoAddress(address a) external returns (address);
    function echoBarStructFields(uint32 a, uint128 b) external returns (uint32, uint128);
);

#[rstest]
#[case(echoBoolCall::new((true,)), (true,))]
#[case(echoBoolCall::new((false,)), (false,))]
#[case(echoU8Call::new((255,)), (255,))]
#[case(echoU8Call::new((1,)), (1,))]
#[case(echoU16Call::new((u16::MAX,)), (u16::MAX,))]
#[case(echoU16Call::new((1,)), (1,))]
#[case(echoU32Call::new((u32::MAX,)), (u32::MAX,))]
#[case(echoU32Call::new((1,)), (1,))]
#[case(echoU64Call::new((u64::MAX,)), (u64::MAX,))]
#[case(echoU64Call::new((1,)), (1,))]
#[case(echoU128Call::new((u128::MAX,)), (u128::MAX,))]
#[case(echoU128Call::new((1,)), (1,))]
#[case(echoU256Call::new((U256::MAX,)), (U256::MAX,))]
#[case(echoU256Call::new((U256::from(1),)), (U256::from(1),))]
#[case(echoVecStackTypeCall::new((vec![1,2,u32::MAX,3,4],)), vec![1,2,u32::MAX,3,4])]
#[case(echoVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4],)), vec![1,2,u128::MAX,3,4])]
#[case(echoAddressCall::new(
    (address!("0xcafe000000000000000000000000000000007357"),)),
    (address!("0xcafe000000000000000000000000000000007357"),))
    ]
#[case(echoBarStructFieldsCall::new((u32::MAX, u128::MAX)), (u32::MAX, u128::MAX),)]
#[case(echoBarStructFieldsCall::new((1, u128::MAX)), (1, u128::MAX),)]
#[case(echoBarStructFieldsCall::new((u32::MAX, 1)), (u32::MAX, 1),)]
#[case(echoBarStructFieldsCall::new((1, 1)), (1, 1),)]
fn test_struct_field_reference<T: SolCall, V: SolValue>(
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
