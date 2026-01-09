use crate::common::runtime;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[derive(Debug)]
    struct ID {
       bytes32 bytes;
    }

    #[derive(Debug)]
    struct UID {
       ID id;
    }

    #[allow(missing_docs)]
    struct Foo {
        UID id;
        uint32 secret;
    }

    #[allow(missing_docs)]
    struct Bar {
        UID id;
        uint128[] a;
        Foo b;
    }

    #[allow(missing_docs)]
    function createOwnedFoo(address owner) external;
    function ownerPeepFoo(bytes32 foo) external returns (uint32);
    function peepFoo(address owner, bytes32 id) external returns (Foo);
    function callIndirectPeepFoo(address owner, bytes32 id) external returns (uint32);
    function createOwnedBar(address owner) external;
    function peepBar(address owner, bytes32 id) external returns (Bar);
);

#[rstest]
fn test_peep(#[with("peep", "tests/framework/move_sources/peep.move")] runtime: RuntimeSandbox) {
    let owner_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xde, 0xad, 0xbe, 0xef,
    ];
    runtime.set_tx_origin(owner_address);
    runtime.set_msg_sender(owner_address);

    // Create a Foo struct, and transfer it to `owner_address` (deadbeef)
    let call_data = createOwnedFooCall::new((owner_address.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let foo_id = runtime.obtain_uid().unwrap();

    // Owner peeps into the Foo struct
    let call_data = ownerPeepFooCall::new((foo_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = ownerPeepFooCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(100, return_data);

    // Peep into the Foo struct
    let call_data = peepFooCall::new((owner_address.into(), foo_id)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = peepFooCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(100, return_data.secret);

    // Change the signer
    let peeper_address = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xca, 0xfe, 0xba, 0xbe,
    ];
    runtime.set_tx_origin(peeper_address);
    runtime.set_msg_sender(peeper_address);

    // Peep into the Foo struct
    let call_data = peepFooCall::new((owner_address.into(), foo_id)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = peepFooCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(100, return_data.secret);

    // Create a Bar struct, and transfer it to `owner_address` (deadbeef)
    let call_data = createOwnedBarCall::new((owner_address.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let bar_id = runtime.obtain_uid().unwrap();
    let _ = runtime.obtain_uid().unwrap(); // the nested foo object

    // Peep into the Bar struct
    let call_data = peepBarCall::new((owner_address.into(), bar_id)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = peepBarCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(vec![1, 2, 3], return_data.a);
    assert_eq!(100, return_data.b.secret);

    // Peep into a non-existent object
    let call_data = peepFooCall::new((owner_address.into(), [0; 32].into())).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);
    let error_message = RuntimeError::ObjectNotFound.to_string();
    let expected_data = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&(error_message,)),
    ]
    .concat();
    assert_eq!(expected_data, return_data);

    // call_peep_foo
    let call_data =
        callIndirectPeepFooCall::new((owner_address.into(), foo_id.into())).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = callIndirectPeepFooCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(100, return_data);
    assert_eq!(0, result);

    // A foo is created if the foo is found
    let _ = runtime.obtain_uid().unwrap();

    // Call peep_foo
    let call_data =
        callIndirectPeepFooCall::new((owner_address.into(), [0; 32].into())).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);
    let error_message = RuntimeError::ObjectNotFound.to_string();
    let expected_data = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&(error_message,)),
    ]
    .concat();
    assert_eq!(expected_data, return_data);

    // Here we should expect no new uid event currently available
    let err = runtime.obtain_uid().unwrap_err();
    assert_eq!(
        err.to_string(),
        "No NewUID(address) event currently available"
    );
}
