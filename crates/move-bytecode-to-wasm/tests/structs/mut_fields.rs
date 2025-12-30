use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "struct_mut_fields",
    "tests/structs/move_sources/struct_mut_fields.move"
);

sol!(
    #[allow(missing_docs)]
    function echoMutBool(bool a) external returns (bool);
    function echoMutU8(uint8 a) external returns (uint8);
    function echoMutU16(uint16 a) external returns (uint16);
    function echoMutU32(uint32 a) external returns (uint32);
    function echoMutU64(uint64 a) external returns (uint64);
    function echoMutU128(uint128 a) external returns (uint128);
    function echoMutU256(uint256 a) external returns (uint256);
    function echoMutVecStackType(uint32[] a) external returns (uint32[]);
    function echoMutVecHeapType(uint128[] a) external returns (uint128[]);
    function echoMutAddress(address a) external returns (address);
    function echoBarStructFields(uint32 a, uint128 b) external returns (uint32, uint128);
);

#[rstest]
#[case(echoMutBoolCall::new((true,)), (true,))]
#[case(echoMutU8Call::new((255,)), (255,))]
#[case(echoMutU8Call::new((1,)), (1,))]
#[case(echoMutU16Call::new((u16::MAX,)), (u16::MAX,))]
#[case(echoMutU16Call::new((1,)), (1,))]
#[case(echoMutU32Call::new((u32::MAX,)), (u32::MAX,))]
#[case(echoMutU32Call::new((1,)), (1,))]
#[case(echoMutU64Call::new((u64::MAX,)), (u64::MAX,))]
#[case(echoMutU64Call::new((1,)), (1,))]
#[case(echoMutU128Call::new((u128::MAX,)), (u128::MAX,))]
#[case(echoMutU128Call::new((1,)), (1,))]
#[case(echoMutU256Call::new((U256::MAX,)), (U256::MAX,))]
#[case(echoMutU256Call::new((U256::from(1),)), (U256::from(1),))]
#[case(echoMutVecStackTypeCall::new((vec![1,2,u32::MAX,3,4],)), vec![1,2,u32::MAX,3,4])]
#[case(echoMutVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4],)), vec![1,2,u128::MAX,3,4])]
#[case(echoMutAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),)),
        (address!("0xcafe000000000000000000000000000000007357"),))
    ]
#[case(echoBarStructFieldsCall::new((u32::MAX, u128::MAX)), (u32::MAX, u128::MAX),)]
#[case(echoBarStructFieldsCall::new((1, u128::MAX)), (1, u128::MAX),)]
#[case(echoBarStructFieldsCall::new((u32::MAX, 1)), (u32::MAX, 1),)]
#[case(echoBarStructFieldsCall::new((1, 1)), (1, 1),)]
fn test_struct_field_mut_reference<T: SolCall, V: SolValue>(
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
