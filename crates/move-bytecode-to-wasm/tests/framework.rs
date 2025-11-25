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
        struct NestedStruct {
            uint32 a;
            address b;
            uint128 c;
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
        function emitTestAnonEvent1(uint32 n) external;
        function emitTestAnonEvent2(uint32 a, address b, uint128 c) external;
        function emitTestAnonEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
        function emitTestAnonEvent4(uint32 a, address b, uint128 c, uint8[] d, uint32 e, address f, uint128 g) external;
        function emitTestAnonEvent5(uint32 a, address b, uint8[] c) external;
        function emitTestAnonEvent6(uint32 a, address b, uint32 c, address d, uint128 e) external;
        function emitTestAnonEvent7(uint32 a, uint8[] b, uint32 c, address d, uint128 e) external;
        function emitTestAnonEvent8(uint64 a, string b) external;
        function emitTestAnonEvent9(uint64 a, string b) external;
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
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec()
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
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec(),
        NestedStruct {
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
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
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
        keccak256("test string".abi_encode()).to_vec(),
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
        42,
        address!("0xcafe000000000000000000000000000000007357"),
        u128::MAX
    )), 2,
    [
        42.abi_encode().to_vec(),
        address!("0xcafe000000000000000000000000000000007357").abi_encode().to_vec(),
        u128::MAX.abi_encode().to_vec(),
        vec![1, 2, 3, 4, 5].abi_encode().to_vec(),
        NestedStruct {
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
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
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
        keccak256("test string".abi_encode()).to_vec(),
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
        keccak256(vec![1, 2, 3, 4, 5].abi_encode()).to_vec(),
        keccak256(NestedStruct {
            a: 43,
            b: address!("0xcafe000000000000000000000000000000007357"),
            c: u128::MAX,
        }.abi_encode()).to_vec()
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

mod cross_contract_calls {
    use alloy_primitives::{Address, U256, address};

    use crate::common::{
        runtime_sandbox::CrossContractCallType, translate_test_complete_package_with_framework,
    };

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "cross_contract_calls";
        const SOURCE_PATH: &str = "tests/framework";

        let mut translated_packages = translate_test_complete_package_with_framework(SOURCE_PATH);
        let translated_package = translated_packages.get_mut(MODULE_NAME).unwrap();
        RuntimeSandbox::new(translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        struct Baz {
            uint16 a;
            uint128 b;
        }

        struct Foo {
            address q;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Baz baz;
        }

        struct Bazz {
            uint16 a;
            uint256[] b;
        }

        struct Bar {
            address q;
            uint32[] r;
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bazz bazz;
            Baz baz;
        }

        function ccCallEmptyRes1(address contract_address) external returns (bool);
        function ccCallEmptyRes2(address contract_address, uint64 v) external returns (bool);
        function ccCallEmptyRes3(address contract_address, Foo v) external returns (bool);
        function ccCallEmptyRes4(address contract_address, Bar v) external returns (bool);
        function ccCallEmptyRes5(address contract_address, uint8[] v) external returns (bool);
        function ccCallEmptyRes1WithGas(address contract_address, uint64 gas) external returns (bool);
        function ccCallEmptyRes2WithGas(address contract_address, uint64 gas, uint64 v) external returns (bool);
        function ccCallEmptyRes3WithGas(address contract_address, uint64 gas, Foo v) external returns (bool);
        function ccCallEmptyRes1Payable(address contract_address, uint256 payable_value) external returns (bool);
        function ccCallEmptyRes2Payable(address contract_address, uint256 payable_value, uint64 v) external returns (bool);
        function ccCallEmptyRes3Payable(address contract_address, uint256 payable_value, Foo v) external returns (bool);
        function ccCallEmptyRes4Payable(address contract_address, uint256 payable_value, Bar v) external returns (bool);
        function ccCallEmptyRes5Payable(address contract_address, uint256 payable_value, uint8[] v) external returns (bool);
        function ccCallEmptyRes1PayableGas(address contract_address, uint256 payable_value, uint64 gas) external returns (bool);
        function ccCallEmptyRes2PayableGas(address contract_address, uint256 payable_value, uint64 gas, uint64 v) external returns (bool);
        function ccCallEmptyRes3PayableGas(address contract_address, uint256 payable_value, uint64 gas, Foo v) external returns (bool);
        function ccCallEmptyRes1Delegate(address contract_address) external returns (bool);
        function ccCallEmptyRes2Delegate(address contract_address, uint64 v) external returns (bool);
        function ccCallEmptyRes3Delegate(address contract_address, Foo v) external returns (bool);
        function ccCallEmptyRes4Delegate(address contract_address, Bar v) external returns (bool);
        function ccCallEmptyRes5Delegate(address contract_address, uint8[] v) external returns (bool);
        function ccCallEmptyRes1WithGasDelegate(address contract_address, uint64 gas) external returns (bool);
        function ccCallEmptyRes2WithGasDelegate(address contract_address, uint64 gas, uint64 v) external returns (bool);
        function ccCallEmptyRes3WithGasDelegate(address contract_address, uint64 gas, Foo v) external returns (bool);

        // The following functions are used to obtain their calldata and compare them
        function callEmptyRes1() external;
        function callEmptyRes2(uint64 v) external;
        function callEmptyRes3(Foo v) external;
        function callEmptyRes4(Bar v) external;
        function callEmptyRes5(uint8[] v) external;
        function callEmptyRes1Payable() external;
        function callEmptyRes2Payable(uint64 v) external;
        function callEmptyRes3Payable(Foo v) external;
        function callEmptyRes4Payable(Bar v) external;
        function callEmptyRes5Payable(uint8[] v) external;
    );

    const ADDRESS: alloy_primitives::Address =
        address!("0xbeefbeef00000000000000000000000000007357");

    fn get_foo() -> Foo {
        Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            baz: Baz { a: 42, b: 4242 },
        }
    }

    fn get_bar() -> Bar {
        Bar {
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bazz: Bazz {
                a: 42,
                b: vec![U256::MAX, U256::from(8), U256::from(7), U256::from(6)],
            },
            baz: Baz {
                a: 111,
                b: 1111111111,
            },
        }
    }

    #[rstest]
    #[case(
        ccCallEmptyRes1Call::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes2Call::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes3Call::new((ADDRESS, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes4Call::new((ADDRESS, get_bar())),
        callEmptyRes4Call::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes5Call::new((ADDRESS, vec![1,2,3,4,5])),
        callEmptyRes5Call::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes1WithGasCall::new((ADDRESS, 1)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        1,
    )]
    #[case(
        ccCallEmptyRes2WithGasCall::new((ADDRESS, 2, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        2,
    )]
    #[case(
        ccCallEmptyRes3WithGasCall::new((ADDRESS, 3, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
    )]
    #[case(
        ccCallEmptyRes1PayableCall::new((ADDRESS, U256::from(u16::MAX))),
        callEmptyRes1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes2PayableCall::new((ADDRESS, U256::from(u32::MAX), 42)),
        callEmptyRes2PayableCall::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes3PayableCall::new((ADDRESS, U256::from(u64::MAX), get_foo())),
        callEmptyRes3PayableCall::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes4PayableCall::new((ADDRESS, U256::from(u128::MAX), get_bar())),
        callEmptyRes4PayableCall::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes5PayableCall::new((ADDRESS, U256::MAX, vec![1,2,3,4,5])),
        callEmptyRes5PayableCall::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::MAX,
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes1PayableGasCall::new((ADDRESS, U256::from(u16::MAX), 1)),
        callEmptyRes1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        1,
    )]
    #[case(
        ccCallEmptyRes2PayableGasCall::new((ADDRESS, U256::from(u32::MAX), 2, 42)),
        callEmptyRes2PayableCall::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        2,
    )]
    #[case(
        ccCallEmptyRes3PayableGasCall::new((ADDRESS, U256::from(u64::MAX), 3, get_foo())),
        callEmptyRes3PayableCall::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        3,
    )]
    #[case(
        ccCallEmptyRes1DelegateCall::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes2DelegateCall::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes3DelegateCall::new((ADDRESS, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes4DelegateCall::new((ADDRESS, get_bar())),
        callEmptyRes4Call::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes5DelegateCall::new((ADDRESS, vec![1,2,3,4,5])),
        callEmptyRes5Call::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes1WithGasDelegateCall::new((ADDRESS, 1)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        1,
    )]
    #[case(
        ccCallEmptyRes2WithGasDelegateCall::new((ADDRESS, 2, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        2,
    )]
    #[case(
        ccCallEmptyRes3WithGasDelegateCall::new((ADDRESS, 3, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        3,
    )]
    // --
    #[case(
        ccCallEmptyRes1Call::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        false,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    #[case(
        ccCallEmptyRes2Call::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        false,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
    fn test_cross_contract_call_empty_calls<T: SolCall>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_cross_contract_calldata: Vec<u8>,
        #[case] success: bool,
        #[case] expected_call_type: CrossContractCallType,
        #[case] expected_payable_value: U256,
        #[case] expected_gas: u64,
    ) {
        runtime.set_cross_contract_call_success(success);

        let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
        assert_eq!(0, result);

        if success {
            assert_eq!(true.abi_encode(), return_data);

            let result = runtime.cross_contract_calls.lock().unwrap().recv().unwrap();
            assert_eq!(expected_call_type, result.call_type);
            assert_eq!(ADDRESS, Address::from(result.address));
            assert_eq!(expected_gas, result.gas);
            assert_eq!(expected_cross_contract_calldata, result.calldata);
            assert_eq!(expected_payable_value, result.value);
        } else {
            assert_eq!(false.abi_encode(), return_data);
        }
    }
}

mod cross_contract_calls_result {
    #![allow(clippy::too_many_arguments)]
    use alloy_primitives::{Address, U256, address};

    use crate::common::{
        runtime_sandbox::CrossContractCallType, translate_test_complete_package_with_framework,
    };

    use super::*;

    const GET_RESULT_ERROR_CODE: &str = "101";
    const DATA_ABORT_MESSAGE_PTR_OFFSET: usize = 256;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "cross_contract_calls_result";
        const SOURCE_PATH: &str = "tests/framework";

        let mut translated_packages = translate_test_complete_package_with_framework(SOURCE_PATH);
        let translated_package = translated_packages.get_mut(MODULE_NAME).unwrap();
        RuntimeSandbox::new(translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        struct Baz {
            uint16 a;
            uint128 b;
        }

        struct Foo {
            address q;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Baz baz;
        }

        struct Bazz {
            uint16 a;
            uint256[] b;
        }

        struct Bar {
            address q;
            uint32[] r;
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bazz bazz;
            Baz baz;
        }

        function ccCallViewRes1(address contract_address) external returns (uint64);
        function ccCallViewRes2(address contract_address) external returns (Foo);
        function ccCallViewRes3(address contract_address) external returns (Bar);
        function ccCallViewRes4(address contract_address) external returns (uint8[]);
        function ccCallPureRes1(address contract_address) external returns (uint64);
        function ccCallPureRes2(address contract_address) external returns (Foo);
        function ccCallPureRes3(address contract_address) external returns (Bar);
        function ccCallPureRes4(address contract_address) external returns (uint8[]);
        function ccCallViewPureRes1(address contract_address) external returns (uint64);
        function ccCallViewPureRes2(address contract_address) external returns (Foo);
        function ccCallViewPureRes3(address contract_address) external returns (Bar);
        function ccCallViewPureRes4(address contract_address) external returns (uint8[]);
        function ccCall1(address contract_address) external returns (uint64);
        function ccCall2(address contract_address) external returns (Foo);
        function ccCall3(address contract_address) external returns (Bar);
        function ccCall4(address contract_address) external returns (uint8[]);
        function ccCall1WithGas(address contract_address, uint64 gas) external returns (uint64);
        function ccCall2WithGas(address contract_address, uint64 gas) external returns (Foo);
        function ccCall3WithGas(address contract_address, uint64 gas) external returns (Bar);
        function ccCall4WithGas(address contract_address, uint64 gas) external returns (uint8[]);
        function ccCall1WithGasDelegate(address contract_address, uint64 gas) external returns (uint64);
        function ccCall2WithGasDelegate(address contract_address, uint64 gas) external returns (Foo);
        function ccCall3WithGasDelegate(address contract_address, uint64 gas) external returns (Bar);
        function ccCall4WithGasDelegate(address contract_address, uint64 gas) external returns (uint8[]);
        function ccCall1WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (uint64);
        function ccCall2WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (Foo);
        function ccCall3WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (Bar);
        function ccCall4WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (uint8[]);
        function ccCall1Payable(address contract_address, uint256 value) external returns (uint64);
        function ccCall2Payable(address contract_address, uint256 value) external returns (Foo);
        function ccCall3Payable(address contract_address, uint256 value) external returns (Bar);
        function ccCall4Payable(address contract_address, uint256 value) external returns (uint8[]);
        function ccCall1Delegate(address contract_address) external returns (uint64);
        function ccCall2Delegate(address contract_address) external returns (Foo);
        function ccCall3Delegate(address contract_address) external returns (Bar);
        function ccCall4Delegate(address contract_address) external returns (uint8[]);
        function ccCall1WithArgs(address contract_address, uint64) external returns (uint64);
        function ccCall2WithArgs(address contract_address, uint256 value, uint64, Foo) external returns (Foo);
        function ccCall3WithArgs(address contract_address, uint64 gas, uint64, Foo, Bar) external returns (Bar);
        function ccCall4WithArgs(address contract_address, uint64, Foo, Bar, uint8[]) external returns (uint8[]);

        // The following functions are used to obtain their calldata and compare them
        function callView1() external;
        function callView2() external;
        function callView3() external;
        function callView4() external;
        function callPure1() external;
        function callPure2() external;
        function callPure3() external;
        function callPure4() external;
        function callViewPure1() external;
        function callViewPure2() external;
        function callViewPure3() external;
        function callViewPure4() external;
        function call1() external;
        function call2() external;
        function call3() external;
        function call4() external;
        function call1Payable() external;
        function call2Payable() external;
        function call3Payable() external;
        function call4Payable() external;
        function call1WithArgs(uint64) external;
        function call2WithArgs(uint64,Foo) external;
        function call3WithArgs(uint64,Foo,Bar) external;
        function call4WithArgs(uint64,Foo,Bar,uint8[]) external;
    );

    const ADDRESS: alloy_primitives::Address =
        address!("0xbeefbeef00000000000000000000000000007357");

    fn get_foo() -> Foo {
        Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            baz: Baz { a: 42, b: 4242 },
        }
    }

    fn get_bar() -> Bar {
        Bar {
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bazz: Bazz {
                a: 42,
                b: vec![U256::MAX, U256::from(8), U256::from(7), U256::from(6)],
            },
            baz: Baz {
                a: 111,
                b: 1111111111,
            },
        }
    }

    #[rstest]
    #[case(
        ccCallViewRes1Call::new((ADDRESS,)),
        callView1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCallViewRes2Call::new((ADDRESS,)),
        callView2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCallViewRes3Call::new((ADDRESS,)),
        callView3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCallViewRes4Call::new((ADDRESS,)),
        callView4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCallPureRes1Call::new((ADDRESS,)),
        callPure1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCallPureRes2Call::new((ADDRESS,)),
        callPure2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCallPureRes3Call::new((ADDRESS,)),
        callPure3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCallPureRes4Call::new((ADDRESS,)),
        callPure4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCallViewPureRes1Call::new((ADDRESS,)),
        callViewPure1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCallViewPureRes2Call::new((ADDRESS,)),
        callViewPure2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCallViewPureRes3Call::new((ADDRESS,)),
        callViewPure3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCallViewPureRes4Call::new((ADDRESS,)),
        callViewPure4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1Call::new((ADDRESS,)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2Call::new((ADDRESS,)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3Call::new((ADDRESS,)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4Call::new((ADDRESS,)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1DelegateCall::new((ADDRESS,)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2DelegateCall::new((ADDRESS,)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3DelegateCall::new((ADDRESS,)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4DelegateCall::new((ADDRESS,)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1WithGasCall::new((ADDRESS, 1)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        1,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2WithGasCall::new((ADDRESS, 2)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        2,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3WithGasCall::new((ADDRESS, 3)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4WithGasCall::new((ADDRESS, 4)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1WithGasDelegateCall::new((ADDRESS, 1)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        1,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2WithGasDelegateCall::new((ADDRESS, 2)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        2,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3WithGasDelegateCall::new((ADDRESS, 3)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4WithGasDelegateCall::new((ADDRESS, 4)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1WithGasPayableCall::new((ADDRESS, 1, U256::from(u16::MAX))),
        call1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        1,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2WithGasPayableCall::new((ADDRESS, 2, U256::from(u32::MAX))),
        call2PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        2,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3WithGasPayableCall::new((ADDRESS, 3, U256::from(u64::MAX))),
        call3PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        3,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4WithGasPayableCall::new((ADDRESS, 4, U256::from(u128::MAX))),
        call4PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1PayableCall::new((ADDRESS, U256::from(u16::MAX))),
        call1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2PayableCall::new((ADDRESS, U256::from(u32::MAX))),
        call2PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3PayableCall::new((ADDRESS, U256::from(u64::MAX))),
        call3PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        u64::MAX,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4PayableCall::new((ADDRESS, U256::from(u128::MAX))),
        call4PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    #[case(
        ccCall1WithArgsCall::new((ADDRESS, 84)),
        call1WithArgsCall::new((84,)).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
    #[case(
        ccCall2WithArgsCall::new((ADDRESS, U256::from(u32::MAX), 84, get_foo())),
        call2WithArgsCall::new((84, get_foo())).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
        get_foo().abi_encode(),
    )]
    #[case(
        ccCall3WithArgsCall::new((ADDRESS, 3, 84, get_foo(), get_bar())),
        call3WithArgsCall::new((84, get_foo(), get_bar())).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
    #[case(
        ccCall4WithArgsCall::new((ADDRESS, 84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])),
        call4WithArgsCall::new((84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
    fn test_cross_contract_call_with_result_calls<T: SolCall>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_cross_contract_calldata: Vec<u8>,
        #[case] success: bool,
        #[case] expected_call_type: CrossContractCallType,
        #[case] expected_payable_value: U256,
        #[case] expected_gas: u64,
        #[case] expected_result: Vec<u8>,
    ) {
        runtime.set_cross_contract_call_success(success);
        runtime.set_cross_contract_return_data(expected_result.clone());

        let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
        assert_eq!(0, result);

        if success {
            assert_eq!(return_data, expected_result);

            let result = runtime.cross_contract_calls.lock().unwrap().recv().unwrap();
            assert_eq!(expected_call_type, result.call_type);
            assert_eq!(ADDRESS, Address::from(result.address));
            assert_eq!(expected_gas, result.gas);
            assert_eq!(expected_cross_contract_calldata, result.calldata);
            assert_eq!(expected_payable_value, result.value);
        } else {
            assert_eq!(false.abi_encode(), return_data);
        }
    }

    #[rstest]
    #[case(ccCallViewRes1Call::new((ADDRESS,)))]
    #[case(ccCallViewRes2Call::new((ADDRESS,)))]
    #[case(ccCallViewRes3Call::new((ADDRESS,)))]
    #[case(ccCallViewRes4Call::new((ADDRESS,)))]
    #[case(ccCallPureRes1Call::new((ADDRESS,)))]
    #[case(ccCallPureRes2Call::new((ADDRESS,)))]
    #[case(ccCallPureRes3Call::new((ADDRESS,)))]
    #[case(ccCallPureRes4Call::new((ADDRESS,)))]
    #[case(ccCallViewPureRes1Call::new((ADDRESS,)))]
    #[case(ccCallViewPureRes2Call::new((ADDRESS,)))]
    #[case(ccCallViewPureRes3Call::new((ADDRESS,)))]
    #[case(ccCallViewPureRes4Call::new((ADDRESS,)))]
    #[case(ccCall1Call::new((ADDRESS,)))]
    #[case(ccCall2Call::new((ADDRESS,)))]
    #[case(ccCall3Call::new((ADDRESS,)))]
    #[case(ccCall4Call::new((ADDRESS,)))]
    #[case(ccCall1DelegateCall::new((ADDRESS,)))]
    #[case(ccCall2DelegateCall::new((ADDRESS,)))]
    #[case(ccCall3DelegateCall::new((ADDRESS,)))]
    #[case(ccCall4DelegateCall::new((ADDRESS,)))]
    #[case(ccCall1WithGasCall::new((ADDRESS, 1)))]
    #[case(ccCall2WithGasCall::new((ADDRESS, 2)))]
    #[case(ccCall3WithGasCall::new((ADDRESS, 3)))]
    #[case(ccCall4WithGasCall::new((ADDRESS, 4)))]
    #[case(ccCall1WithGasDelegateCall::new((ADDRESS, 1)))]
    #[case(ccCall2WithGasDelegateCall::new((ADDRESS, 2)))]
    #[case(ccCall3WithGasDelegateCall::new((ADDRESS, 3)))]
    #[case(ccCall4WithGasDelegateCall::new((ADDRESS, 4)))]
    #[case(ccCall1WithGasPayableCall::new((ADDRESS, 1, U256::from(u16::MAX))))]
    #[case(ccCall2WithGasPayableCall::new((ADDRESS, 2, U256::from(u32::MAX))))]
    #[case(ccCall3WithGasPayableCall::new((ADDRESS, 3, U256::from(u64::MAX))))]
    #[case(ccCall4WithGasPayableCall::new((ADDRESS, 4, U256::from(u128::MAX))))]
    #[case(ccCall1PayableCall::new((ADDRESS, U256::from(u16::MAX))))]
    #[case(ccCall2PayableCall::new((ADDRESS, U256::from(u32::MAX))))]
    #[case(ccCall3PayableCall::new((ADDRESS, U256::from(u64::MAX))))]
    #[case(ccCall4PayableCall::new((ADDRESS, U256::from(u128::MAX))))]
    #[case(ccCall1WithArgsCall::new((ADDRESS, 84)))]
    #[case(ccCall2WithArgsCall::new((ADDRESS, U256::from(u32::MAX), 84, get_foo())))]
    #[case(ccCall3WithArgsCall::new((ADDRESS, 3, 84, get_foo(), get_bar())))]
    #[case(ccCall4WithArgsCall::new((ADDRESS, 84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])))]
    fn test_cross_contract_call_with_result_get_result_panic_if_fails<T: SolCall>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
    ) {
        use crate::common::runtime_sandbox::ExecutionData;

        runtime.set_cross_contract_call_success(false);
        let ExecutionData {
            result: _,
            return_data: _,
            instance,
            mut store,
        } = runtime
            .call_entrypoint_with_data(call_data.abi_encode())
            .unwrap();

        // Read where the encoded error is
        let error_ptr = RuntimeSandbox::read_memory_from(
            &instance,
            &mut store,
            DATA_ABORT_MESSAGE_PTR_OFFSET,
            4,
        )
        .unwrap();
        let error_ptr = u32::from_le_bytes(error_ptr.try_into().unwrap());

        // Read the actual message length from the ABI header (4 bytes big-endian at offset 68)
        let msg_len_bytes =
            RuntimeSandbox::read_memory_from(&instance, &mut store, error_ptr as usize + 68, 4)
                .unwrap();
        let msg_len = u32::from_be_bytes(msg_len_bytes.try_into().unwrap()) as usize;

        // Read the raw error message bytes (not ABI-encoded, just UTF-8 string bytes)
        let error_bytes = RuntimeSandbox::read_memory_from(
            &instance,
            &mut store,
            // raw error data (skip 4-byte length header + 68-byte ABI header)
            error_ptr as usize + 72,
            // Use the actual message length from the ABI header
            msg_len,
        )
        .unwrap();

        // Convert raw UTF-8 bytes to string
        let error = String::from_utf8(error_bytes).unwrap();

        assert_eq!(GET_RESULT_ERROR_CODE, error);
    }
}

mod error {
    use alloy_primitives::{U256, address, keccak256};

    use crate::common::translate_test_package_with_framework;

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "error";
        const SOURCE_PATH: &str = "tests/framework/error.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        struct SimpleError {
            string e;
        }

        struct CustomError {
            string error_message;
            uint64 error_code;
        }

        struct CustomError2 {
            bool a;
            uint8 b;
            uint16 c;
            uint32 d;
            uint64 e;
            uint128 f;
            uint256 g;
            address h;
        }

        struct CustomError3 {
            uint32[] a;
            uint128[] b;
            uint64[][] c;
        }

        struct CustomError4 {
            SimpleError a;
            CustomError b;
        }

        struct NestedStruct1 {
            string e;
        }
        struct NestedStruct2 {
            string a;
            uint64 b;
        }

        function revertStandardError(string s) external;
        function revertCustomError(string s, uint64 code) external;
        function revertCustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h) external;
        function revertCustomError3(uint32[] a, uint128[] b, uint64[][] c) external;
        function revertCustomError4(string a, string b, uint64 c) external;
    );

    #[rstest]
    #[case(
        revertStandardErrorCall::new((String::from("Not enough Ether provided."),)),
        [
            keccak256(b"SimpleError(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&("Not enough Ether provided.",)),
        ].concat()
    )]
    #[case(
        revertCustomErrorCall::new((
            String::from("Custom error message"),
            42,
        )),
        [
            keccak256(b"CustomError(string,uint64)")[..4].to_vec(),
            <sol!((string, uint64))>::abi_encode_params(&(
                "Custom error message",
                42,
            )),
        ].concat()
    )]
    #[case(
        revertCustomError2Call::new((true, 2u8, 3u16, 4u32, 5u64, 5u128, U256::from(5), address!("0xffffffffffffffffffffffffffffffffffffffff"))),
        [
            keccak256(b"CustomError2(bool,uint8,uint16,uint32,uint64,uint128,uint256,address)")[..4].to_vec(),
            <sol!((bool, uint8, uint16, uint32, uint64, uint128, uint256, address))>::abi_encode_params(&(true, 2u8, 3u16, 4u32, 5u64, 5u128, U256::from(5), address!("0xffffffffffffffffffffffffffffffffffffffff"))),
        ].concat()
    )]
    #[case(
        revertCustomError3Call::new((vec![1, 2, 3], vec![4, 5], vec![vec![6, 7, 8], vec![9, 10, 11]])),
        [
            keccak256(b"CustomError3(uint32[],uint128[],uint64[][])")[..4].to_vec(),
            <sol!((uint32[], uint128[], uint64[][]))>::abi_encode_params(&(vec![1, 2, 3], vec![4, 5], vec![vec![6, 7, 8], vec![9, 10, 11]])),
        ].concat()
    )]
    #[case(
        revertCustomError4Call::new((
            String::from("Custom error message"),
            String::from("Custom error message 2"),
            42,
        )),
        [
            keccak256(b"CustomError4((string),(string,uint64))")[..4].to_vec(),
            {
                let params = (
                    NestedStruct1 { e: String::from("Custom error message") },
                    NestedStruct2 { a: String::from("Custom error message 2"), b: 42 },
                );
                <sol!((NestedStruct1, NestedStruct2)) as alloy_sol_types::SolValue>::abi_encode_params(&params)
            },
        ].concat()
    )]
    fn test_revert<T: SolCall>(
        runtime: RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_data: Vec<u8>,
    ) {
        let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
        assert_eq!(1, result);

        assert_eq!(return_data, expected_data);
    }
}
