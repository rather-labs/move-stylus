use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "equality_enums",
    "tests/operations_equality/move_sources/enums.move"
);

sol!(
    #[allow(missing_docs)]
    function eqSimpleEnumBool(bool a, bool b) external returns (bool);
    function eqSimpleEnumU8(uint8 a, uint8 b) external returns (bool);
    function eqSimpleEnumU16(uint16 a, uint16 b) external returns (bool);
    function eqSimpleEnumU32(uint32 a, uint32 b) external returns (bool);
    function eqSimpleEnumU64(uint64 a, uint64 b) external returns (bool);
    function eqSimpleEnumU128(uint128 a, uint128 b) external returns (bool);
    function eqSimpleEnumU256(uint256 a, uint256 b) external returns (bool);
    function eqSimpleEnumAddress(address a, address b) external returns (bool);
    function eqVectorEnumStack(uint32[] a, uint32[] b) external returns (bool);
    function eqVectorEnumHeap(uint128[] a, uint128[] b) external returns (bool);
    function eqVectorEnumBool(bool[] a, bool[] b) external returns (bool);
    function eqVectorEnumAddress(address[] a, address[] b) external returns (bool);
    function eqStructEnumWithStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    function eqStructEnumWithPrimitives(uint8 a, uint16 b, uint32 c, uint8 d, uint16 e, uint32 f) external returns (bool);
    function eqStructEnumMixed(uint32 a, uint128 b, uint64 c, uint32 d, uint128 e, uint64 f) external returns (bool);
    function eqComplexEnumSimple(uint32 a, uint32 b) external returns (bool);
    function eqComplexEnumVector(uint64[] a, uint64[] b) external returns (bool);
    function eqComplexEnumStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    function eqComplexEnumNested(uint32 a, uint128 b, uint32[] c, bool d, uint32 e, uint128 f, uint32[] g, bool h) external returns (bool);
    function neqSimpleEnumBool(bool a, bool b) external returns (bool);
    function neqSimpleEnumU8(uint8 a, uint8 b) external returns (bool);
    function neqSimpleEnumU16(uint16 a, uint16 b) external returns (bool);
    function neqSimpleEnumU32(uint32 a, uint32 b) external returns (bool);
    function neqSimpleEnumU64(uint64 a, uint64 b) external returns (bool);
    function neqSimpleEnumU128(uint128 a, uint128 b) external returns (bool);
    function neqSimpleEnumU256(uint256 a, uint256 b) external returns (bool);
    function neqSimpleEnumAddress(address a, address b) external returns (bool);
    function neqVectorEnumStack(uint32[] a, uint32[] b) external returns (bool);
    function neqVectorEnumHeap(uint128[] a, uint128[] b) external returns (bool);
    function neqVectorEnumBool(bool[] a, bool[] b) external returns (bool);
    function neqVectorEnumAddress(address[] a, address[] b) external returns (bool);
    function neqStructEnumWithStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    function neqStructEnumWithPrimitives(uint8 a, uint16 b, uint32 c, uint8 d, uint16 e, uint32 f) external returns (bool);
    function neqStructEnumMixed(uint32 a, uint128 b, uint64 c, uint32 d, uint128 e, uint64 f) external returns (bool);
    function neqComplexEnumSimple(uint32 a, uint32 b) external returns (bool);
    function neqComplexEnumVector(uint64[] a, uint64[] b) external returns (bool);
    function neqComplexEnumStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    function neqComplexEnumNested(uint32 a, uint128 b, uint32[] c, bool d, uint32 e, uint128 f, uint32[] g, bool h) external returns (bool);
    function eqVectorSimpleEnums(uint8[] a, uint8[] b) external returns (bool);
    function neqVectorSimpleEnums(uint8[] a, uint8[] b) external returns (bool);
    function eqVectorStructEnums(uint32[] a, uint128[] b, uint32[] c, uint128[] d) external returns (bool);
    function neqVectorStructEnums(uint32[] a, uint128[] b, uint32[] c, uint128[] d) external returns (bool);
    function eqVectorComplexEnums(uint32[] a, uint32[] b) external returns (bool);
    function neqVectorComplexEnums(uint32[] a, uint32[] b) external returns (bool);
    function eqVectorMixedEnums(uint32[] a, uint64[] b) external returns (bool);
    function neqVectorMixedEnums(uint32[] a, uint64[] b) external returns (bool);

);
#[rstest]
#[case(eqSimpleEnumBoolCall::new((true, true)), true)]
#[case(eqSimpleEnumBoolCall::new((false, true)), false)]
#[case(eqSimpleEnumU8Call::new((255, 255)), true)]
#[case(eqSimpleEnumU8Call::new((1, 255)), false)]
#[case(eqSimpleEnumU16Call::new((u16::MAX, u16::MAX)), true)]
#[case(eqSimpleEnumU16Call::new((1, u16::MAX)), false)]
#[case(eqSimpleEnumU32Call::new((u32::MAX, u32::MAX)), true)]
#[case(eqSimpleEnumU32Call::new((1, u32::MAX)), false)]
#[case(eqSimpleEnumU64Call::new((u64::MAX, u64::MAX)), true)]
#[case(eqSimpleEnumU64Call::new((1, u64::MAX)), false)]
#[case(eqSimpleEnumU128Call::new((u128::MAX, u128::MAX)), true)]
#[case(eqSimpleEnumU128Call::new((1, u128::MAX)), false)]
#[case(eqSimpleEnumU256Call::new((U256::MAX, U256::MAX)), true)]
#[case(eqSimpleEnumU256Call::new((U256::from(1), U256::MAX)), false)]
#[case(eqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), true)]
#[case(eqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), false)]
#[case(eqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), true)]
#[case(eqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), false)]
#[case(eqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,true])), true)]
#[case(eqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,false])), false)]
#[case(eqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")])), true)]
#[case(eqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007358")])), false)]
#[case(eqStructEnumWithStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
#[case(eqStructEnumWithStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
#[case(eqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 255, u16::MAX, u32::MAX)), true)]
#[case(eqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 254, u16::MAX, u32::MAX)), false)]
#[case(eqStructEnumMixedCall::new((u32::MAX, u128::MAX, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), true)]
#[case(eqStructEnumMixedCall::new((u32::MAX, u128::MAX-1, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), false)]
#[case(eqComplexEnumSimpleCall::new((u32::MAX, u32::MAX)), true)]
#[case(eqComplexEnumSimpleCall::new((u32::MAX, 1)), false)]
#[case(eqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,u64::MAX,3,4])), true)]
#[case(eqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,3,4,5])), false)]
#[case(eqComplexEnumStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
#[case(eqComplexEnumStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
#[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), true)]
#[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,3,4,5], true)), false)]
#[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false)), false)]
#[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), false)]
// Vector enum equality tests
#[case(eqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), true)]
#[case(eqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), false)]
#[case(eqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,64])), true)]
#[case(eqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,63])), false)]
#[case(eqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,3], vec![100,200,300])), true)]
#[case(eqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,4], vec![100,200,300])), false)]
#[case(eqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1])), true)]
#[case(eqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-2], vec![u128::MAX, u128::MAX-1])), false)]
#[case(eqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), true)]
#[case(eqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), false)]
#[case(eqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 0])), true)]
#[case(eqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 1])), false)]
#[case(eqVectorMixedEnumsCall::new((vec![1,2,3], vec![1,2,3])), false)] // Different variants should never be equal
#[case(eqVectorMixedEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u64::MAX, u64::MAX-1])), false)] // Different variants should never be equal
fn test_equality_enum<T: SolCall>(
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
#[case(neqSimpleEnumBoolCall::new((true, true)), false)]
#[case(neqSimpleEnumBoolCall::new((false, true)), true)]
#[case(neqSimpleEnumU8Call::new((255, 255)), false)]
#[case(neqSimpleEnumU8Call::new((1, 255)), true)]
#[case(neqSimpleEnumU16Call::new((u16::MAX, u16::MAX)), false)]
#[case(neqSimpleEnumU16Call::new((1, u16::MAX)), true)]
#[case(neqSimpleEnumU32Call::new((u32::MAX, u32::MAX)), false)]
#[case(neqSimpleEnumU32Call::new((1, u32::MAX)), true)]
#[case(neqSimpleEnumU64Call::new((u64::MAX, u64::MAX)), false)]
#[case(neqSimpleEnumU64Call::new((1, u64::MAX)), true)]
#[case(neqSimpleEnumU128Call::new((u128::MAX, u128::MAX)), false)]
#[case(neqSimpleEnumU128Call::new((1, u128::MAX)), true)]
#[case(neqSimpleEnumU256Call::new((U256::MAX, U256::MAX)), false)]
#[case(neqSimpleEnumU256Call::new((U256::from(1), U256::MAX)), true)]
#[case(neqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), false)]
#[case(neqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), true)]
#[case(neqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), false)]
#[case(neqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), true)]
#[case(neqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,true])), false)]
#[case(neqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,false])), true)]
#[case(neqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")])), false)]
#[case(neqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007358")])), true)]
#[case(neqStructEnumWithStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
#[case(neqStructEnumWithStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
#[case(neqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 255, u16::MAX, u32::MAX)), false)]
#[case(neqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 254, u16::MAX, u32::MAX)), true)]
#[case(neqStructEnumMixedCall::new((u32::MAX, u128::MAX, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), false)]
#[case(neqStructEnumMixedCall::new((u32::MAX, u128::MAX-1, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), true)]
#[case(neqComplexEnumSimpleCall::new((u32::MAX, u32::MAX)), false)]
#[case(neqComplexEnumSimpleCall::new((u32::MAX, 1)), true)]
#[case(neqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,u64::MAX,3,4])), false)]
#[case(neqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,3,4,5])), true)]
#[case(neqComplexEnumStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
#[case(neqComplexEnumStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
#[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), false)]
#[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,3,4,5], true)), true)]
#[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false)), true)]
#[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), true)]
// Vector enum inequality tests
#[case(neqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), false)]
#[case(neqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), true)]
#[case(neqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,64])), false)]
#[case(neqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,63])), true)]
#[case(neqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,3], vec![100,200,300])), false)]
#[case(neqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,4], vec![100,200,300])), true)]
#[case(neqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1])), false)]
#[case(neqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-2], vec![u128::MAX, u128::MAX-1])), true)]
#[case(neqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), false)]
#[case(neqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), true)]
#[case(neqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 0])), false)]
#[case(neqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 1])), true)]
#[case(neqVectorMixedEnumsCall::new((vec![1,2,3], vec![1,2,3])), true)] // Different variants should always be unequal
#[case(neqVectorMixedEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u64::MAX, u64::MAX-1])), true)] // Different variants should always be unequal
fn test_not_equality_enum<T: SolCall>(
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
