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
    );

    #[rstest]
    #[case(
        hashU8Call::new((42,)),
        merge_arrays(&[ADDRESS, &[42], b"u8".as_slice()])
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
        let read_from = read_from as usize - 32 - expected_result.len() - 3;

        let read_memory = RuntimeSandbox::read_memory_from(
            &instance,
            &mut store,
            read_from,
            expected_result.len(),
        )
        .unwrap();
        println!(
            "Return data: {} {:?} {:?}",
            read_from - 32 - expected_result.len() - 4,
            return_data,
            RuntimeSandbox::read_memory_from(&instance, &mut store, 312, expected_result.len())
                .unwrap()
        );

        assert_eq!(expected_result, read_memory);
    }
}
