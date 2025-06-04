use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(return_data == expected_result, "return data mismatch");

    Ok(())
}

mod primitives {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality";
        const SOURCE_PATH: &str = "tests/operations-equality/primitives.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        // function eqSigner(address x) external returns (bool);
        function eqAddress(address x, address y) external returns (bool);
        function eqU256(uint256 x, uint256 y) external returns (bool);
        function eqU128(uint128 x, uint128 y) external returns (bool);
        function eqU64(uint64 x, uint64 y) external returns (bool);
        function eqU32(uint32 x, uint32 y) external returns (bool);
        function eqU16(uint16 x, uint16 y) external returns (bool);
        function eqU8(uint8 x, uint8 y) external returns (bool);
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
    // #[case(eqSignerCall::new((address!("0xcafe000000000000000000000000000000007357"),)), true)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU256Call::new((U256::from(0), U256::from(1) << 255)), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU128Call::new((u128::MAX, u128::MAX - 42)), false)]
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
            <sol!((bool,))>::abi_encode_params(&(expected_result,)),
        )
        .unwrap();
    }
}

mod vector {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality_vectors";
        const SOURCE_PATH: &str = "tests/operations-equality/vectors.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function eqVecStackType(uint16[], uint16[]) external returns (bool);
        function eqVecHeapType(uint128[], uint128[]) external returns (bool);
        // function eqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
        // function eqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
    );

    #[rstest]
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
    fn test_equality_vector<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode_params(&(expected_result,)),
        )
        .unwrap();
    }
}
