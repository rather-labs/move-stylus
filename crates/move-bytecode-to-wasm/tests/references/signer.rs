use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("ref_signer", "tests/references/move_sources/signer.move");

sol!(
    #[allow(missing_docs)]
    function useDummy() external returns (address);  // Returns the signer
);

#[rstest]
#[should_panic]
#[case(useDummyCall::new(()), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7])]
fn test_signer_immutable_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: [u8; 20],
) {
    let expected_result = <sol!((address,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
