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
    use alloy_primitives::address;

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

        struct TestEvent1 {
            uint32 n;
        }

        struct TestEvent2 {
            uint32 a;
            address b;
            uint128 c;
        }

        struct TestEvent3 {
            uint32 a;
            address b;
            uint128 c;
            uint8[] d;
        }

        struct TestEvent4 {
            uint32 a;
            address b;
            uint128 c;
            uint8[] d;
            TestEvent2 e;
        }

        struct GenericEvent1 {
            uint32[] n;
            bool o;
            address p;
            uint128 q;
        }

        struct GenericEvent2 {
            uint64 n;
            bool o;
            TestEvent1 p;
            TestEvent2 q;
        }

        function emitTestEvent1(uint32 n) external;
        function emitTestEvent2(uint32 a, address b, uint128 c) external;
        function emitTestEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
        function emitTestEvent4(uint32 a, address b, uint128 c, uint8[] d, TestEvent2 e) external;
        function emitGenericEvent1(uint32[] n, bool o, address p, uint128 q) external;
        function emitGenericEvent2(uint64 n, bool o, TestEvent1 p, TestEvent2 q) external;
    );

    #[rstest]
    #[case(emitTestEvent1Call::new((42,)), TestEvent1 { n: 42 })]
    #[case(emitTestEvent2Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), TestEvent2 {
        a: 42,
        b: address!("0xcafe000000000000000000000000000000007357"),
        c: u128::MAX,
    })]
    #[case(emitTestEvent3Call::new((
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX,
        vec![1, 2, 3, 4, 5]
    )), TestEvent3 {
        a: 42,
        b: address!("0xcafe000000000000000000000000000000007357"),
        c: u128::MAX,
        d: vec![1, 2, 3, 4, 5],
    })]
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
    )), TestEvent4 {
        a: 42,
        b: address!("0xcafe000000000000000000000000000000007357"),
        c: u128::MAX,
        d: vec![1, 2, 3, 4, 5],
        e: TestEvent2 {
            a: 42,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }
    })]
    #[case(emitGenericEvent1Call::new((
        vec![1, 2, 3, 4, 5], false, address!("0xcafe000000000000000000000000000000007357"), u128::MAX
    )), GenericEvent1 {
        n: vec![1, 2, 3, 4, 5],
        o: false,
        p: address!("0xcafe000000000000000000000000000000007357"),
        q: u128::MAX
    })]
    #[case(emitGenericEvent2Call::new((
        u64::MAX, true, TestEvent1 { n: 42 }, TestEvent2 { a: 42, b: address!("0xcafe000000000000000000000000000000007357"), c: u128::MAX }
    )), GenericEvent2 {
        n: u64::MAX,
        o: true,
        p: TestEvent1 { n: 42 },
        q: TestEvent2 { a: 42, b: address!("0xcafe000000000000000000000000000000007357"), c: u128::MAX }
    })]
    fn test_emit_event<T: SolCall, V: SolValue>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        let (result, _) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
        assert_eq!(result, 0, "Function returned non-zero exit code: {result}");

        let event = runtime.log_events.lock().unwrap().recv().unwrap();
        assert_eq!(event, expected_result.abi_encode());
    }
}
