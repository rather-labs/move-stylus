use alloy_sol_types::SolValue;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::runtime_sandbox::RuntimeSandbox;
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

mod tx_context {
    use alloy_primitives::{Address, hex};

    use crate::common::{
        runtime_sandbox::constants::{
            BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, GAS_PRICE,
            MSG_SENDER_ADDRESS, MSG_VALUE,
        },
        translate_test_package_with_framework,
    };

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "tx_context";
        const SOURCE_PATH: &str = "tests/framework/tx_context.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getSender() external returns (address);
        function getMsgValue() external returns (uint256);
        function getBlockNumber() external returns (uint64);
        function getBlockBasefee() external returns (uint256);
        function getBlockGasLimit() external returns (uint64);
        function getBlockTimestamp() external returns (uint64);
        function getGasPrice() external returns (uint256);
        function getFreshObjectAddress() external returns (address, address, address);
    );

    #[rstest]
    #[case(getSenderCall::new(()), (Address::new(MSG_SENDER_ADDRESS),))]
    #[case(getMsgValueCall::new(()), (MSG_VALUE,))]
    #[case(getBlockNumberCall::new(()), (BLOCK_NUMBER,))]
    #[case(getBlockBasefeeCall::new(()), (BLOCK_BASEFEE,))]
    #[case(getBlockGasLimitCall::new(()), (BLOCK_GAS_LIMIT,))]
    #[case(getBlockTimestampCall::new(()), (BLOCK_TIMESTAMP,))]
    #[case(getGasPriceCall::new(()), (GAS_PRICE,))]
    fn test_tx_context<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }

    #[rstest]
    #[case(
        getFreshObjectAddressCall::new(()),
        (
            hex::decode("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap(),
            hex::decode("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap(),
            hex::decode("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255")
                .map(|h| <[u8; 32]>::try_from(h).unwrap())
                .unwrap()
        )
    )]
    fn test_tx_fresh_id<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: ([u8; 32], [u8; 32], [u8; 32]),
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }
}

mod event {
    use alloy_primitives::{address, keccak256};

    use crate::common::translate_test_package_with_framework;

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "event";
        const SOURCE_PATH: &str = "tests/framework/event.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        struct TestEvent2 {
            uint32 a;
            address b;
            uint128 c;
        }


        function emitTestEvent1(uint32 n) external;
        function emitTestEvent2(uint32 a, address b, uint128 c) external;
        function emitTestEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
        function emitTestEvent4(uint32 a, address b, uint128 c, uint8[] d, TestEvent2 e) external;
        function emitTestEvent5(uint32 a, address b, uint8[] c) external;
        function emitTestEvent6(uint32 a, address b, TestEvent2 c) external;
        function emitTestEvent7(uint32 a, uint8[] b, TestEvent2 c) external;
        function emitTestAnonEvent1(uint32 n) external;
        function emitTestAnonEvent2(uint32 a, address b, uint128 c) external;
        function emitTestAnonEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
        function emitTestAnonEvent4(uint32 a, address b, uint128 c, uint8[] d, TestEvent2 e) external;
        function emitTestAnonEvent5(uint32 a, address b, uint8[] c) external;
        function emitTestAnonEvent6(uint32 a, address b, TestEvent2 c) external;
        function emitTestAnonEvent7(uint32 a, uint8[] b, TestEvent2 c) external;
        function emitTestAnonymous(uint32 a, uint128 b, uint8[] c, TestEvent2 d) external;
        function emitTestAnonymous2(
            uint32 a,
            uint128 b,
            uint8[] c,
            TestEvent2 d,
            uint32 e,
            address f,
            uint128 g,
            uint8[] h,
            TestEvent2 i,
        ) external;
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
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec()
    ].concat())]
    #[case(emitTestEvent4Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 42,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 3,
    [
        keccak256(b"TestEvent4(uint32,address,uint128,uint8[],(uint32,address,uint128))").to_vec(),
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec(),
        TestEvent2 {
            a: 42,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode().to_vec()
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
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
    ].concat())]
    #[case(emitTestEvent6Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 4,
    [
        keccak256(b"TestEvent6(uint32,address,(uint32,address,uint128))").to_vec(),
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
    #[case(emitTestEvent7Call::new((
        42,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 4,
    [
        keccak256(b"TestEvent7(uint32,uint8[],(uint32,address,uint128))").to_vec(),
        42.abi_encode().to_vec(),
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
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
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec()
    ].concat())]
    #[case(emitTestAnonEvent4Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 42,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 2,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec(),
        TestEvent2 {
            a: 42,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode().to_vec()
    ].concat())]
    #[case(emitTestAnonEvent5Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        vec![1, 2, 3, 4, 5],
    )), 3,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
    ].concat())]
    #[case(emitTestAnonEvent6Call::new((
        41,
        address!("0xcafe000000000000000000000000000000007357"),
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 3,
    [
        41.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
    #[case(emitTestAnonEvent7Call::new((
        42,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 3,
    [
        42.abi_encode().to_vec(),
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
    #[case(emitTestAnonymousCall::new((
        42,
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 4,
    [
        42.abi_encode().to_vec(),
        u128::MAX.abi_encode(),
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
    ].concat())]
    #[case(emitTestAnonymous2Call::new((
        42,
        u128::MAX,
        vec![1, 2, 3, 4, 5],
        TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        },
        84,
        address!("0xcafecafecafe0000000000000000000073577357"),
        u64::MAX as u128,
        vec![9, 8, 7, 6, 5],
        TestEvent2 {
            a: 85,
            b: address!("0xbeefbeef00000000000000000000000000007357"),
            c: u128::MAX,
        }
    )), 4,
    [
        42.abi_encode().to_vec(),
        u128::MAX.abi_encode(),
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(TestEvent2 {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec(),
        84.abi_encode().to_vec(),
        address!("0xcafecafecafe0000000000000000000073577357").abi_encode().to_vec(),
        (u64::MAX as u128).abi_encode().to_vec(),
        vec![9, 8, 7, 6, 5].abi_encode().to_vec(),
        TestEvent2 {
            a: 85,
            b: address!("0xbeefbeef00000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode().to_vec()
    ].concat())]
    fn test_emit_event<T: SolCall>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_topic: u32,
        #[case] expected_data: Vec<u8>,
    ) {
        let (result, _) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
        assert_eq!(result, 0, "Function returned non-zero exit code: {result}");

        let (topic, data) = runtime.log_events.lock().unwrap().recv().unwrap();
        println!("Topic {topic}");
        println!("Data {data:?}");
        assert_eq!(expected_topic, topic);
        assert_eq!(expected_data, data.as_slice());
    }
}
