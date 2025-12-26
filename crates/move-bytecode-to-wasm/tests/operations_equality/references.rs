use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "equality_references",
    "tests/operations_equality/move_sources/references.move"
);

sol!(
    #[allow(missing_docs)]
    function eqAddress(address x, address y) external returns (bool);
    function eqU256(uint256 x, uint256 y) external returns (bool);
    function eqU128(uint128 x, uint128 y) external returns (bool);
    function eqU64(uint64 x, uint64 y) external returns (bool);
    function eqU32(uint32 x, uint32 y) external returns (bool);
    function eqU16(uint16 x, uint16 y) external returns (bool);
    function eqU8(uint8 x, uint8 y) external returns (bool);
    function eqVecStackType(uint16[], uint16[]) external returns (bool);
    function eqVecHeapType(uint128[], uint128[]) external returns (bool);
    function eqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
    function eqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
    function neqAddress(address x, address y) external returns (bool);
    function neqU256(uint256 x, uint256 y) external returns (bool);
    function neqU128(uint128 x, uint128 y) external returns (bool);
    function neqU64(uint64 x, uint64 y) external returns (bool);
    function neqU32(uint32 x, uint32 y) external returns (bool);
    function neqU16(uint16 x, uint16 y) external returns (bool);
    function neqU8(uint8 x, uint8 y) external returns (bool);
    function neqVecStackType(uint16[], uint16[]) external returns (bool);
    function neqVecHeapType(uint128[], uint128[]) external returns (bool);
    function neqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
    function neqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
);

#[rstest]
#[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        true
    )]
#[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        false
    )]
#[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
#[case(eqU256Call::new((U256::from(0), U256::from(1) << 255)), false)]
#[case(eqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), false)]
#[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
#[case(eqU128Call::new((u128::MAX, u128::MAX - 42)), false)]
#[case(eqU128Call::new((0, 1 << 127)), false)]
#[case(eqU128Call::new((u128::MAX, u128::MAX)), true)]
#[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
#[case(eqU64Call::new((u64::MAX, u64::MAX)), true)]
#[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
#[case(eqU32Call::new((u32::MAX, u32::MAX)), true)]
#[case(eqU32Call::new((u32::MAX, u32::MAX - 42)), false)]
#[case(eqU16Call::new((u16::MAX, u16::MAX)), true)]
#[case(eqU16Call::new((u16::MAX, u16::MAX - 42)), false)]
#[case(eqU8Call::new((u8::MAX, u8::MAX)), true)]
#[case(eqU8Call::new((u8::MAX, u8::MAX - 42)), false)]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        true
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 9, 8, 7, 6, u16::MAX])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, 4])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],)),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        true
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 9, 8, 7, 6, u128::MAX])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, 4])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 2], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, 4]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![50], vec![61], vec![70]],
        vec![vec![50], vec![62], vec![70]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, 1], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, 4]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX - 1]])),
        false
    )]
fn test_equality_references<T: SolCall>(
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
#[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        false
    )]
#[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        true
    )]
#[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
#[case(neqU256Call::new((U256::from(0), U256::from(1) << 255)), true)]
#[case(neqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), true)]
#[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
#[case(neqU128Call::new((u128::MAX, u128::MAX - 42)), true)]
#[case(neqU128Call::new((0, 1 << 127)), true)]
#[case(neqU128Call::new((u128::MAX, u128::MAX)), false)]
#[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
#[case(neqU64Call::new((u64::MAX, u64::MAX)), false)]
#[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
#[case(neqU32Call::new((u32::MAX, u32::MAX)), false)]
#[case(neqU32Call::new((u32::MAX, u32::MAX - 42)), true)]
#[case(neqU16Call::new((u16::MAX, u16::MAX)), false)]
#[case(neqU16Call::new((u16::MAX, u16::MAX - 42)), true)]
#[case(neqU8Call::new((u8::MAX, u8::MAX)), false)]
#[case(neqU8Call::new((u8::MAX, u8::MAX - 42)), true)]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        false
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 9, 8, 7, 6, u16::MAX])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, 4])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],)),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        false
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 9, 8, 7, 6, u128::MAX])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, 4])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 2], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, 4]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![50], vec![61], vec![70]],
        vec![vec![50], vec![62], vec![70]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, 1], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, 4]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX - 1]])),
        true
    )]
fn test_not_equality_references<T: SolCall>(
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
