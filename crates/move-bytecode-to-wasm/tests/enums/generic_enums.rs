use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "generic_enums",
    "tests/enums/move_sources/generic_enums.move"
);

sol! {
    function packUnpackFoo(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
    function packUnpackFooViaWrapper(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
    function packUnpackFooViaWrapper2(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint32[]);
    function packUnpackBaz(uint8 variant_index, uint32 value32) external returns (uint32);
    function packMutateUnpackFu(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
}

#[rstest]
#[case(packUnpackFooCall::new((0u8, 42u64, 32u32)), (42u64, 32u32))]
#[case(packUnpackFooCall::new((1u8, u64::MAX, u32::MAX)), (u64::MAX, u32::MAX))]
#[case(packUnpackFooCall::new((2u8, 0u64, 0u32)), (0u64, 0u32))]
#[case(packUnpackFooViaWrapperCall::new((0u8, 42u64, 32u32)), (42u64, 32u32))]
#[case(packUnpackFooViaWrapperCall::new((1u8, u64::MAX, u32::MAX)), (u64::MAX, u32::MAX))]
#[case(packUnpackFooViaWrapperCall::new((2u8, 0u64, 0u32)), (0u64, 0u32))]
#[case(packUnpackBazCall::new((0u8, 42u32)), 42u32)]
#[case(packUnpackBazCall::new((1u8, 8u32)), 24u32)]
#[case(packUnpackBazCall::new((2u8, 33u32)), 66u32)]
#[case(packMutateUnpackFuCall::new((0u8, 42u64, 32u32)), (43u64, 33u32))]
#[case(packMutateUnpackFuCall::new((1u8, u64::MAX-1, u32::MAX-1)), (u64::MAX, u32::MAX))]
#[case(packMutateUnpackFuCall::new((2u8, 0u64, 0u32)), (1u64, 1u32))]
fn test_generic_enums<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[rstest]
#[case(packUnpackFooViaWrapper2Call::new((0u8, 42u64, 32u32)), vec![0u32; 0])]
#[case(packUnpackFooViaWrapper2Call::new((1u8, u64::MAX, u32::MAX)), vec![u32::MAX, u32::MAX, u32::MAX])]
#[case(packUnpackFooViaWrapper2Call::new((2u8, 0u64, 0u32)), vec![0u32, 0u32, 0u32])]
fn test_generic_enums_vectors<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u32>,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}
