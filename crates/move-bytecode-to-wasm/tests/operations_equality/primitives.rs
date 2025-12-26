use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "equality",
    "tests/operations_equality/move_sources/primitives.move"
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
    function neqAddress(address x, address y) external returns (bool);
    function neqU256(uint256 x, uint256 y) external returns (bool);
    function neqU128(uint128 x, uint128 y) external returns (bool);
    function neqU64(uint64 x, uint64 y) external returns (bool);
    function neqU32(uint32 x, uint32 y) external returns (bool);
    function neqU16(uint16 x, uint16 y) external returns (bool);
    function neqU8(uint8 x, uint8 y) external returns (bool);
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
fn test_equality_primitive_types<T: SolCall>(
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
fn test_not_equality_primitive_types<T: SolCall>(
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
