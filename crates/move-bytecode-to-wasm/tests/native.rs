mod common;

use alloy_primitives::{FixedBytes, U256, keccak256};
use common::runtime_sandbox::constants::SIGNER_ADDRESS;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

mod hash_type_and_key {
    use alloy_primitives::{FixedBytes, address};
    use alloy_sol_types::{SolCall, sol};

    use crate::common::{runtime_sandbox::ExecutionData, translate_test_package};

    use super::*;

    const ADDRESS: &[u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca,
        0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe,
    ];

    const ADDRESS_2: &[u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe,
        0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef,
    ];

    fn merge_arrays<T: Clone>(arrays: &[&[T]]) -> Vec<T> {
        arrays
            .iter()
            .flat_map(|slice| slice.iter().cloned())
            .collect()
    }

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "hash_type_and_key";
        const SOURCE_PATH: &str = "tests/native/hash_type_and_key.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function hashU8(uint8 a) public view;
        function hashU16(uint16 a) public view;
        function hashU32(uint32 a) public view;
        function hashU64(uint64 a) public view;
        function hashU128(uint128 a) public view;
        function hashU256(uint256 a) public view;
        function hashBool(bool a) public view;
        function hashAddress(address a) public view;
        function hashVectorU8(uint8[] a) public view;
        function hashVectorU16(uint16[] a) public view;
        function hashVectorU32(uint32[] a) public view;
        function hashVectorU64(uint64[] a) public view;
        function hashVectorU128(uint128[] a) public view;
        function hashVectorU256(uint256[] a) public view;
        function hashVectorBool(bool[] a) public view;
        function hashVectorAddress(address[] a) public view;
    );

    #[rstest]
    #[case(
        hashU8Call::new((42,)),
        merge_arrays(&[ADDRESS, &[42], b"u8".as_slice()])
    )]
    #[case(
        hashU16Call::new((4242,)),
        merge_arrays(&[ADDRESS, &4242u16.to_le_bytes(), b"u16".as_slice()])
    )]
    #[case(
        hashU32Call::new((42424242,)),
        merge_arrays(&[ADDRESS, &42424242u32.to_le_bytes(), b"u32".as_slice()])
    )]
    #[case(
        hashU64Call::new((42424242424242,)),
        merge_arrays(&[ADDRESS, &42424242424242u64.to_le_bytes(), b"u64".as_slice()])
    )]
    #[case(
        hashU128Call::new((42424242424242_u128,)),
        merge_arrays(&[ADDRESS, &42424242424242_u128.to_le_bytes(), b"u128".as_slice()])
    )]
    #[case(
        hashU256Call::new((U256::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007913129639935", 10).unwrap(),)),
        merge_arrays(&[ADDRESS, &U256::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007913129639935", 10).unwrap().to_le_bytes::<32>(), b"u256".as_slice()])
    )]
    #[case(
        hashBoolCall::new((true,)),
        merge_arrays(&[ADDRESS, &[1], b"bool".as_slice()])
    )]
    #[case(
        hashBoolCall::new((false,)),
        merge_arrays(&[ADDRESS, &[0], b"bool".as_slice()])
    )]
    #[case(
        hashAddressCall::new((address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),)),
        merge_arrays(&[ADDRESS, ADDRESS_2, b"address".as_slice()])
    )]
    #[case(
        hashVectorU8Call::new((vec![1u8, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &[1u8, 2, 3, 4, 5], b"vector<u8>".as_slice()])
    )]
    #[case(
        hashVectorU16Call::new((vec![1u16, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u16.to_le_bytes(), &2u16.to_le_bytes(), &3u16.to_le_bytes(), &4u16.to_le_bytes(), &5u16.to_le_bytes(), b"vector<u16>".as_slice()])
    )]
    #[case(
        hashVectorU32Call::new((vec![1u32, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u32.to_le_bytes(), &2u32.to_le_bytes(), &3u32.to_le_bytes(), &4u32.to_le_bytes(), &5u32.to_le_bytes(), b"vector<u32>".as_slice()])
    )]
    #[case(
        hashVectorU64Call::new((vec![1u64, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u64.to_le_bytes(), &2u64.to_le_bytes(), &3u64.to_le_bytes(), &4u64.to_le_bytes(), &5u64.to_le_bytes(), b"vector<u64>".as_slice()])
    )]
    #[case(
        hashVectorU128Call::new((vec![1u128, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u128.to_le_bytes(), &2u128.to_le_bytes(), &3u128.to_le_bytes(), &4u128.to_le_bytes(), &5u128.to_le_bytes(), b"vector<u128>".as_slice()])
    )]
    #[case(
        hashVectorU256Call::new((vec![
            U256::from(1u64),
            U256::from(2u64),
            U256::from(3u64),
            U256::from(4u64),
            U256::from(5u64)
        ],)),
        merge_arrays(&[ADDRESS, &U256::from(1u64).to_le_bytes::<32>(), &U256::from(2u64).to_le_bytes::<32>(), &U256::from(3u64).to_le_bytes::<32>(), &U256::from(4u64).to_le_bytes::<32>(), &U256::from(5u64).to_le_bytes::<32>(), b"vector<u256>".as_slice()])
    )]
    #[case(
        hashVectorBoolCall::new((vec![true, false, true, false],)),
        merge_arrays(&[ADDRESS, &[1, 0, 1, 0], b"vector<bool>".as_slice()])
    )]
    #[case(
        hashVectorAddressCall::new((
            vec![
                address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
                address!("0xcafecafecafecafecafecafecafecafecafecafe")
            ]
        ,)),
        merge_arrays(&[ADDRESS, ADDRESS_2, ADDRESS, b"vector<address>".as_slice()])
    )]
    fn test_hash_type_and_key_primitives<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u8>,
    ) {
        let ExecutionData {
            result,
            return_data,
            instance,
            mut store,
        } = runtime
            .call_entrypoint_with_data(call_data.abi_encode())
            .unwrap();
        let read_from = i32::from_be_bytes(
            return_data[return_data.len() - 4..return_data.len()]
                .try_into()
                .unwrap(),
        );
        // The last allocated position belongs to the 32 bytes allocated for the keccak function,
        // so, to read what we are really hashing we need to extract that, and the expected result
        // length
        let read_from = read_from as usize - 32 - expected_result.len();

        let read_memory = RuntimeSandbox::read_memory_from(
            &instance,
            &mut store,
            read_from,
            expected_result.len(),
        )
        .unwrap();
        /*
        println!(
            "Return data: {} {:?} {:?}",
            read_from - 32 - expected_result.len() - 4,
            return_data,
            RuntimeSandbox::read_memory_from(
                &instance,
                &mut store,
                312,
                expected_result.len() + 32
            )
            .unwrap()
        );
        */

        assert_eq!(expected_result, read_memory);
    }
}
