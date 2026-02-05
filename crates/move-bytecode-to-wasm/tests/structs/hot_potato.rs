use crate::common::runtime;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol! {
    struct OptionFoo {
        Foo[] val;
    }

    struct Foo {
        bytes32 id;
        uint32 value;
    }

    struct ContainerFoo {
        bytes32 id;
        OptionFoo value;
    }

    struct Promise {
        bytes32 id;
        bytes32 container_id;
    }

    function borrowValFoo(bytes32 container) external returns (Promise);
    function returnValFoo(bytes32 container, bytes32 foo, Promise promise) external;
    function createContainerFoo(bytes32 foo) external;
    function createFoo(uint32 value) external;
    function inspectContainerFoo(bytes32 container) external returns (ContainerFoo);
}

#[rstest]
fn test_hot_potato(
    #[with("hot_potato", "tests/structs/move_sources/hot_potato.move")] runtime: RuntimeSandbox,
) {
    let owner = address!("0x00000000000000000000000000000000aaaaaaaa").0.0;
    runtime.set_tx_origin(owner);
    runtime.set_msg_sender(owner);

    // Create a Foo struct
    let value = 100u32;
    let call_data = createFooCall::new((value,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let foo_uid = runtime.obtain_uid().unwrap();

    // Create a ContainerFoo struct
    let call_data = createContainerFooCall::new((foo_uid,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let container_uid = runtime.obtain_uid().unwrap();

    // Re-borrow the Foo struct
    let call_data = borrowValFooCall::new((container_uid,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let promise = borrowValFooCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(0, result);

    let call_data = returnValFooCall::new((container_uid, foo_uid, promise)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}
