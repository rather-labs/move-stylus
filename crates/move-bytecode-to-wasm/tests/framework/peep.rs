use crate::common::runtime;
use alloy_sol_types::{SolCall, sol};
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
    function createOwnedFoo(address owner) external;
    function ownerPeepFoo(bytes32 foo) external returns (uint32);
    function peepFoo(address owner, bytes32 id) external returns (Foo);
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
    let foo_id = runtime.obtain_uid();

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
}
