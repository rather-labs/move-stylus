use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "equality_external_structs",
    "tests/operations_equality/move_sources/external"
);

sol!(
    #[allow(missing_docs)]
    function eqStructBool(bool a, bool b) external returns (bool);
    function eqStructAddress(address a, address b) external returns (bool);
    function eqStructU256(uint256 a, uint256 b) external returns (bool);
    function eqStructU128(uint128 a, uint128 b) external returns (bool);
    function eqStructU64(uint64 a, uint64 b) external returns (bool);
    function eqStructU32(uint32 a, uint32 b) external returns (bool);
    function eqStructU16(uint16 a, uint16 b) external returns (bool);
    function eqStructU8(uint8 a, uint8 b) external returns (bool);
    function eqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
    function eqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
    function eqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    function neqStructBool(bool a, bool b) external returns (bool);
    function neqStructAddress(address a, address b) external returns (bool);
    function neqStructU256(uint256 a, uint256 b) external returns (bool);
    function neqStructU128(uint128 a, uint128 b) external returns (bool);
    function neqStructU64(uint64 a, uint64 b) external returns (bool);
    function neqStructU32(uint32 a, uint32 b) external returns (bool);
    function neqStructU16(uint16 a, uint16 b) external returns (bool);
    function neqStructU8(uint8 a, uint8 b) external returns (bool);
    function neqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
    function neqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
    function neqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
);

#[rstest]
#[case(eqStructBoolCall::new((true, true)), true)]
#[case(eqStructBoolCall::new((false, true)), false)]
#[case(eqStructU8Call::new((255, 255)), true)]
#[case(eqStructU8Call::new((1, 255)), false)]
#[case(eqStructU16Call::new((u16::MAX, u16::MAX)), true)]
#[case(eqStructU16Call::new((1, u16::MAX)), false)]
#[case(eqStructU32Call::new((u32::MAX, u32::MAX)), true)]
#[case(eqStructU32Call::new((1, u32::MAX)), false)]
#[case(eqStructU64Call::new((u64::MAX, u64::MAX)), true)]
#[case(eqStructU64Call::new((1, u64::MAX)), false)]
#[case(eqStructU128Call::new((u128::MAX, u128::MAX)), true)]
#[case(eqStructU128Call::new((1, u128::MAX)), false)]
#[case(eqStructU256Call::new((U256::MAX, U256::MAX)), true)]
#[case(eqStructU256Call::new((U256::from(1), U256::MAX)), false)]
#[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), true)]
#[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), false)]
#[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), true)]
#[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), false)]
#[case(eqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
#[case(eqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
#[case(eqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
#[case(eqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
fn test_equality_external_struct<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: bool,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        <sol!((bool,))>::abi_encode(&(expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(neqStructBoolCall::new((true, true)), false)]
#[case(neqStructBoolCall::new((false, true)), true)]
#[case(neqStructU8Call::new((255, 255)), false)]
#[case(neqStructU8Call::new((1, 255)), true)]
#[case(neqStructU16Call::new((u16::MAX, u16::MAX)), false)]
#[case(neqStructU16Call::new((1, u16::MAX)), true)]
#[case(neqStructU32Call::new((u32::MAX, u32::MAX)), false)]
#[case(neqStructU32Call::new((1, u32::MAX)), true)]
#[case(neqStructU64Call::new((u64::MAX, u64::MAX)), false)]
#[case(neqStructU64Call::new((1, u64::MAX)), true)]
#[case(neqStructU128Call::new((u128::MAX, u128::MAX)), false)]
#[case(neqStructU128Call::new((1, u128::MAX)), true)]
#[case(neqStructU256Call::new((U256::MAX, U256::MAX)), false)]
#[case(neqStructU256Call::new((U256::from(1), U256::MAX)), true)]
#[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), false)]
#[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), true)]
#[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), false)]
#[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), true)]
#[case(neqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
#[case(neqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
#[case(neqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
#[case(neqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
fn test_not_equality_extnernal_struct<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: bool,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        <sol!((bool,))>::abi_encode(&(expected_result,)),
    )
    .unwrap();
}
