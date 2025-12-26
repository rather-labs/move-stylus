use super::*;
use crate::common::runtime;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    #[derive(Debug)]
    struct ID {
       bytes32 bytes;
    }

    #[derive(Debug)]
    struct UID {
       ID id;
    }

    struct Object {
        UID id;
        uint8 scarcity;
        uint8 style;
    }

    struct SwapRequest {
        UID id;
        address owner;
        Object object;
        uint64 fee;
    }

    function createObject(uint8 scarcity, uint8 style) public;
    function readObject(bytes32 id) public view returns (Object);
    function requestSwap(bytes32 id, address service, uint64 fee) public;
    function executeSwap(bytes32 id1, bytes32 id2) public returns (uint64);
);

const OWNER_A: [u8; 20] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
const OWNER_B: [u8; 20] = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
const SERVICE: [u8; 20] = [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];

#[rstest]
fn test_successful_swap(
    #[with("trusted_swap", "tests/storage/move_sources/trusted_swap.move")] runtime: RuntimeSandbox,
) {
    ////// First owner creates an object //////
    runtime.set_msg_sender(OWNER_A);
    runtime.set_tx_origin(OWNER_A);
    let fee_a = 1000;

    let call_data = createObjectCall::new((7, 2)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_a_id = runtime.obtain_uid();

    let obj_a_slot = derive_object_slot(&OWNER_A, &obj_a_id.0);

    let call_data = readObjectCall::new((obj_a_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
    let obj_a_expected = Object::abi_encode(&Object {
        id: UID {
            id: ID { bytes: obj_a_id },
        },
        scarcity: 7,
        style: 2,
    });
    assert_eq!(Object::abi_encode(&return_data), obj_a_expected);
    assert_eq!(0, result);

    let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the swap request id emmited from the contract's events
    let swap_request_a_id = runtime.obtain_uid();
    println!("Swap Request A ID: {swap_request_a_id:#x}");

    // Assert that the slot is empty
    assert_eq!(
        runtime.get_storage_at_slot(obj_a_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    ////// Second owner requests a swap //////
    runtime.set_msg_sender(OWNER_B);
    runtime.set_tx_origin(OWNER_B);
    let fee_b = 1250;

    let call_data = createObjectCall::new((7, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_b_id = runtime.obtain_uid();

    let obj_b_slot = derive_object_slot(&OWNER_B, &obj_b_id.0);

    let call_data = readObjectCall::new((obj_b_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
    let obj_b_expected = Object::abi_encode(&Object {
        id: UID {
            id: ID { bytes: obj_b_id },
        },
        scarcity: 7,
        style: 3,
    });
    assert_eq!(Object::abi_encode(&return_data), obj_b_expected);
    assert_eq!(0, result);

    let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let swap_request_b_id = runtime.obtain_uid();

    // Assert that the slot is empty
    assert_eq!(
        runtime.get_storage_at_slot(obj_b_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    ////// Execute the swap //////
    runtime.set_msg_sender(SERVICE);
    runtime.set_tx_origin(SERVICE);

    let storage_before_delete = runtime.get_storage();

    let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = executeSwapCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(fee_a + fee_b, return_data);
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    assert_empty_storage(&storage_before_delete, &storage_after_delete);

    for (key, value) in storage_after_delete.iter() {
        if *key != COUNTER_KEY && value != &[0u8; 32] {
            println!("{key:?} \n {value:?} \n");
        }
    }

    ////// Read the objects //////
    // Now owner A should have the object B, and owner B should have the object A.
    runtime.set_msg_sender(OWNER_A);
    runtime.set_tx_origin(OWNER_A);

    let call_data = readObjectCall::new((obj_b_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(Object::abi_encode(&return_data), obj_b_expected);
    assert_eq!(0, result);

    runtime.set_msg_sender(OWNER_B);
    runtime.set_tx_origin(OWNER_B);

    let call_data = readObjectCall::new((obj_a_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(Object::abi_encode(&return_data), obj_a_expected);
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_swap_too_cheap(
    #[with("trusted_swap", "tests/storage/move_sources/trusted_swap.move")] runtime: RuntimeSandbox,
) {
    // Create an object
    runtime.set_msg_sender(OWNER_A);
    runtime.set_tx_origin(OWNER_A);

    let call_data = createObjectCall::new((7, 2)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_a_id = runtime.obtain_uid();

    // Request a swap with a fee too low
    let fee_a = 999;
    let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_swap_different_scarcity(
    #[with("trusted_swap", "tests/storage/move_sources/trusted_swap.move")] runtime: RuntimeSandbox,
) {
    ////// First owner creates an object //////
    runtime.set_msg_sender(OWNER_A);
    runtime.set_tx_origin(OWNER_A);
    let fee_a = 1000;

    let call_data = createObjectCall::new((7, 2)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_a_id = runtime.obtain_uid();

    let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the swap request id emmited from the contract's events
    let swap_request_a_id = runtime.obtain_uid();

    ////// Second owner requests a swap //////
    runtime.set_msg_sender(OWNER_B);
    runtime.set_tx_origin(OWNER_B);
    let fee_b = 1250;

    let call_data = createObjectCall::new((8, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_b_id = runtime.obtain_uid();

    let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let swap_request_b_id = runtime.obtain_uid();

    ////// Execute the swap //////
    runtime.set_msg_sender(SERVICE);
    runtime.set_tx_origin(SERVICE);

    let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_swap_same_style(
    #[with("trusted_swap", "tests/storage/move_sources/trusted_swap.move")] runtime: RuntimeSandbox,
) {
    ////// First owner creates an object //////
    runtime.set_msg_sender(OWNER_A);
    runtime.set_tx_origin(OWNER_A);
    let fee_a = 1000;

    let call_data = createObjectCall::new((7, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_a_id = runtime.obtain_uid();

    let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the swap request id emmited from the contract's events
    let swap_request_a_id = runtime.obtain_uid();

    ////// Second owner requests a swap //////
    runtime.set_msg_sender(OWNER_B);
    runtime.set_tx_origin(OWNER_B);
    let fee_b = 1250;

    let call_data = createObjectCall::new((7, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let obj_b_id = runtime.obtain_uid();

    let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let swap_request_b_id = runtime.obtain_uid();

    ////// Execute the swap //////
    runtime.set_msg_sender(SERVICE);
    runtime.set_tx_origin(SERVICE);

    let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}
