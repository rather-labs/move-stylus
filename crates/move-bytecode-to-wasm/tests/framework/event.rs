use crate::common::run_test;
use crate::common::runtime;
use alloy_primitives::{Address, U256, address, hex, keccak256};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::{
    constants::{
        BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, GAS_PRICE,
        MSG_SENDER_ADDRESS, MSG_VALUE,
    },
    wasm_runner::{CrossContractCallType, ExecutionData, RuntimeSandbox},
};
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    struct NestedStruct {
        uint32 a;
        address b;
        uint128 c;
    }

    enum TestEnum {
        One,
        Two,
        Three,
    }

    function emitTestEvent1(uint32 n) external;
    function emitTestEvent2(uint32 a, address b, uint128 c) external;
    function emitTestEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
    function emitTestEvent4(uint32 a, address b, uint128 c, uint8[] d, uint32 e, address f, uint128 g) external;
    function emitTestEvent5(uint32 a, address b, uint8[] c) external;
    function emitTestEvent6(uint32 a, address b, uint32 c, address d, uint128 e) external;
    function emitTestEvent7(uint32 a, uint8[] b, uint32 c, address d, uint128 e) external;
    function emitTestEvent8(uint64 a, string b) external;
    function emitTestEvent9(uint64 a, string b) external;
    function emitTestEvent10(uint32 a, address b, uint8[][] c) external;
    function emitTestEvent11(uint32 a, address b, uint32 c, uint16[] d, string e) external;
    function emitTestEvent12(uint64 a, string[] b) external;
    function emitTestEvent13(uint64 a, TestEnum[] b) external;
    function emitTestEvent14(uint32 a, address b, uint32 c, uint16[] d, TestEnum e) external;
    function emitTestEvent15(TestEnum a, address b, uint128 c) external;
    function emitTestAnonEvent1(uint32 n) external;
    function emitTestAnonEvent2(uint32 a, address b, uint128 c) external;
    function emitTestAnonEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
    function emitTestAnonEvent4(uint32 a, address b, uint128 c, uint8[] d, uint32 e, address f, uint128 g) external;
    function emitTestAnonEvent5(uint32 a, address b, uint8[] c) external;
    function emitTestAnonEvent6(uint32 a, address b, uint32 c, address d, uint128 e) external;
    function emitTestAnonEvent7(uint32 a, uint8[] b, uint32 c, address d, uint128 e) external;
    function emitTestAnonEvent8(uint64 a, string b) external;
    function emitTestAnonEvent9(uint64 a, string b) external;
    function emitTestAnonEvent10(uint32 a, address b, uint32[][] c) external;
    function emitTestAnonEvent11(uint32 a, address b, uint32 c, uint16[] d, string e) external;
    function emitTestAnonEvent12(uint64 a, string[] b) external;
    function emitTestAnonEvent13(uint64 a, TestEnum[] b) external;
    function emitTestAnonEvent14(uint32 a, address b, uint32 c, uint16[] d, TestEnum e) external;
    function emitTestAnonEvent15(TestEnum a, address b, uint128 c) external;
    function emitTestAnonymous(uint32 a, uint128 b, uint8[] c, uint32 d, address e, uint128 f) external;
);

#[rstest]
#[case(emitTestEvent1Call::new((42,)), 2, [
        keccak256(b"TestEvent1(uint32)").to_vec(),
        42.abi_encode().to_vec()
    ].concat())]
#[case(emitTestEvent2Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 4, [
        keccak256(b"TestEvent2(uint32,address,uint128)").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec()
    ].concat())]
#[case(emitTestEvent3Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5]
    )), 3,
    [
        keccak256(b"TestEvent3(uint32,address,uint128,uint8[])").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005").unwrap()
    ].concat())]
#[case(emitTestEvent4Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 3,
    [
        keccak256(b"TestEvent4(uint32,address,uint128,uint8[],(uint32,address,uint128))").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
      hex::decode("0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000cafe00000000000000000000000000000000735700000000000000000000000000000000ffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005").unwrap()
    ].concat())]
#[case(emitTestEvent5Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        vec![1, 2, 3, 4, 5],
    )), 4,
    [
        keccak256(b"TestEvent5(uint32,address,uint8[])").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x5917e5a395fb9b454434de59651d36822a9e29c5ec57474df3e67937b969460c").unwrap()
        //keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
    ].concat())]
#[case(emitTestEvent6Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        43,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 4,
    [
        keccak256(b"TestEvent6(uint32,address,(uint32,address,uint128))").to_vec(),
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
#[case(emitTestEvent7Call::new((
        42,
        vec![1, 2, 3, 4, 5],
        43,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 4,
    [
        keccak256(b"TestEvent7(uint32,uint8[],(uint32,address,uint128))").to_vec(),
        42.abi_encode().to_vec(),
        hex::decode("0x5917e5a395fb9b454434de59651d36822a9e29c5ec57474df3e67937b969460c").unwrap(),
        // keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
#[case(emitTestEvent8Call::new((
        42,
        "test string".into(),
    )), 2,
    [
        keccak256(b"TestEvent8(uint64,string)").to_vec(),
        42.abi_encode().to_vec(),
        "test string".abi_encode(),
    ].concat())]
#[case(emitTestEvent9Call::new((
        42,
        "test string".into(),
    )), 3,
    [
        keccak256(b"TestEvent9(uint64,string)").to_vec(),
        42.abi_encode().to_vec(),
        keccak256(b"test string").to_vec(),
    ].concat())]
#[case(emitTestEvent10Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        vec![vec![1, 2], vec![3, 4], vec![5, 6]],
    )), 4,
    [
        keccak256(b"TestEvent10(uint32,address,uint8[][])").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x67fd5a843da88fc165a797990d9a7825dcc0af1c9931a6aebababf15e4f2ac41").unwrap()
    ].concat())]
#[case(emitTestEvent11Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        43,
        vec![1, 2, 3, 4, 5],
        "test string".into(),
    )), 4,
    [
        keccak256(b"TestEvent11(uint32,address,(uint32,uint16[],string))").to_vec(),
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0xc37ba50506dec0245492dceb29917e1743c990c285df6a31baf211c204ad8c39").unwrap(),
    ].concat())]
#[case(emitTestEvent12Call::new((
        42,
        vec!["test string".into(), "hello world".into()],
    )), 3,
    [
        keccak256(b"TestEvent12(uint64,string[])").to_vec(),
        42.abi_encode().to_vec(),
        hex::decode("0x4262f685f28afa73e3ac58a6f7cbef13d4d78bc1b4a8ca117c3e4bccb5e6b47e").unwrap(),
    ].concat())]
#[case(emitTestEvent13Call::new((
        42,
        vec![TestEnum::One, TestEnum::Two, TestEnum::Three],
    )), 3,
    [
        keccak256(b"TestEvent13(uint64,uint8[])").to_vec(),
        42.abi_encode().to_vec(),
        hex::decode("0xe682b7c401097344fed1af3e3492f018caf2a2491b45159ba612453495164301").unwrap(),
    ].concat())]
#[case(emitTestEvent14Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        42,
        vec![1, 2, 3, 4, 5],
        TestEnum::Two,
    )), 4,
    [
        keccak256(b"TestEvent14(uint32,address,(uint32,uint16[],uint8))").to_vec(),
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0xb938c742591e76b1d9405e45bbaf979fb5fa6e2fdc73269e4c19be276687ccb3").unwrap(),
    ].concat())]
#[case(emitTestEvent15Call::new((
        TestEnum::Two,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
    )), 4,
    [
        keccak256(b"TestEvent15(uint8,address,uint128)").to_vec(),
        1.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec(),
    ].concat())]
#[case(emitTestAnonEvent1Call::new((42,)), 1, [42.abi_encode().to_vec()].concat())]
#[case(emitTestAnonEvent2Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 3, [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec()
    ].concat())]
#[case(emitTestAnonEvent3Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5]
    )), 2,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
       hex::decode("0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005").unwrap()
    ].concat())]
#[case(emitTestAnonEvent4Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 2,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000cafe00000000000000000000000000000000735700000000000000000000000000000000ffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005").unwrap()
    ].concat())]
#[case(emitTestAnonEvent5Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        vec![1, 2, 3, 4, 5],
    )), 3,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x5917e5a395fb9b454434de59651d36822a9e29c5ec57474df3e67937b969460c").unwrap(),
        // keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
    ].concat())]
#[case(emitTestAnonEvent6Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        43,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 3,
    [
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
#[case(emitTestAnonEvent7Call::new((
        42,
        vec![1, 2, 3, 4, 5],
        43,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 3,
    [
        42.abi_encode().to_vec(),
        //keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        hex::decode("0x5917e5a395fb9b454434de59651d36822a9e29c5ec57474df3e67937b969460c").unwrap(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
#[case(emitTestAnonEvent8Call::new((
        42,
        "test string".into(),
    )), 1,
    [
        42.abi_encode().to_vec(),
        "test string".abi_encode(),
    ].concat())]
#[case(emitTestAnonEvent9Call::new((
        42,
        "test string".into(),
    )), 2,
    [
        42.abi_encode().to_vec(),
        keccak256(b"test string").to_vec(),
    ].concat())]
#[case(emitTestAnonEvent10Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        vec![vec![1, 2], vec![3, 4], vec![5, 6]],
    )), 3,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0x67fd5a843da88fc165a797990d9a7825dcc0af1c9931a6aebababf15e4f2ac41").unwrap()
    ].concat())]
#[case(emitTestAnonEvent11Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        43,
        vec![1, 2, 3, 4, 5],
        "test string".into(),
    )), 3,
    [
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0xc37ba50506dec0245492dceb29917e1743c990c285df6a31baf211c204ad8c39").unwrap(),
    ].concat())]
#[case(emitTestAnonEvent12Call::new((
        42,
        vec!["test string".into(), "hello world".into()],
    )), 2,
    [
        42.abi_encode().to_vec(),
        hex::decode("0x4262f685f28afa73e3ac58a6f7cbef13d4d78bc1b4a8ca117c3e4bccb5e6b47e").unwrap(),
    ].concat())]
#[case(emitTestAnonEvent13Call::new((
        42,
        vec![TestEnum::One, TestEnum::Two, TestEnum::Three],
    )), 2,
    [
        42.abi_encode().to_vec(),
        hex::decode("0xe682b7c401097344fed1af3e3492f018caf2a2491b45159ba612453495164301").unwrap(),
    ].concat())]
#[case(emitTestAnonEvent14Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        42,
        vec![1, 2, 3, 4, 5],
        TestEnum::Two,
    )), 3,
    [
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        hex::decode("0xb938c742591e76b1d9405e45bbaf979fb5fa6e2fdc73269e4c19be276687ccb3").unwrap(),
    ].concat())]
#[case(emitTestAnonEvent15Call::new((
        TestEnum::Two,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
    )), 3,
    [
        1.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec(),
    ].concat())]
#[case(emitTestAnonymousCall::new((
        42,
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        43,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 4,
    [
        42.abi_encode().to_vec(),
        u128::MAX.abi_encode(),
        hex::decode("0x5917e5a395fb9b454434de59651d36822a9e29c5ec57474df3e67937b969460c").unwrap(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
fn test_emit_event<T: SolCall>(
    #[with("event", "tests/framework/move_sources/event.move")] runtime: RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_topic: u32,
    #[case] expected_data: Vec<u8>,
) {
    let (result, _) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(result, 0, "Function returned non-zero exit code: {result}");

    let (topic, data) = runtime.log_events.lock().unwrap().recv().unwrap();
    assert_eq!(expected_topic, topic);
    assert_eq!(expected_data, data.as_slice());
}
