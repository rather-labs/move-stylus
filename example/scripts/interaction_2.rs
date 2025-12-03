//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use std::str::FromStr;
use std::sync::Arc;

use dotenv::dotenv;
use eyre::eyre;

use alloy::{
    primitives::{Address, U256, address, keccak256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::{SolEvent, SolValue},
    transports::http::reqwest::Url,
};

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Example {
        #[derive(Debug)]
        struct ID {
           bytes32 bytes;
        }

        #[derive(Debug)]
        struct UID {
           ID id;
        }

        #[derive(Debug)]
        event NewUID(address indexed uid);

        #[derive(Debug, PartialEq)]
        event TestEvent1 (
            uint32 indexed n,
        );

        #[derive(Debug, PartialEq)]
        event TestEvent2 (
            uint32 indexed a,
            uint8[] indexed b,
            uint128 c,
        );

        #[derive(Debug, PartialEq)]
        struct NestedStruct1 {
            uint32 n;
        }

        #[derive(Debug, PartialEq)]
        struct NestedStruct2 {
            uint32 a;
            uint8[] b;
            uint128 c;
        }

        #[derive(Debug, PartialEq)]
        event TestEvent3 (
            NestedStruct1 indexed a,
            NestedStruct2 b,
        );

        #[derive(Debug, PartialEq)]
        event TestEvent4 (
            uint32 indexed a,
            uint16[] b,
            uint8[] c,
            uint32[] d,
        );

        #[derive(Debug, PartialEq)]
        struct Stack {
            uint32[] pos0;
        }

        #[derive(Debug, PartialEq)]
        event ReceiveEvent (
            address indexed sender,
            uint32 data_length,
            uint8[] data,
        );

        function emitTestEvent1(uint32 n) public view;
        function emitTestEvent2(uint32 a, uint8[] b, uint128 c) public view;
        function emitTestEvent3(uint32 n, uint32 a, uint8[] b, uint128 c) public view;
        function emitTestEvent4(uint32 a, uint16[] b, uint8[] c, uint32[] d) public view;
        function echoWithGenericFunctionU16(uint16 x) external view returns (uint16);
        function echoWithGenericFunctionVec32(uint32[] x) external view returns (uint32[]);
        function echoWithGenericFunctionU16Vec32(uint16 x, uint32[] y) external view returns (uint16, uint32[]);
        function echoWithGenericFunctionAddressVec128(address x, uint128[] y) external view returns (address, uint128[]);
        function getUniqueIds() external view returns (UID, UID, UID);
        function getUniqueId() external view returns (UID);
        function getFreshObjectAddress() external view returns (address);
        function testStack1() external view returns (Stack, uint64);
        function testStack2() external view returns (Stack, uint64);
        function testStack3() external view returns (Stack, uint64);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;

    let contract_address = std::env::var("CONTRACT_ADDRESS_2")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_2"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;
    let sender = signer.address();

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    let ret = example.echoWithGenericFunctionU16(42).call().await?;
    println!("echoWithGenericFunctionU16 {ret}");

    let ret = example
        .echoWithGenericFunctionVec32(vec![1, 2, 3])
        .call()
        .await?;
    println!("echoWithGenericFunctionVec32 {ret:?}");

    let ret = example
        .echoWithGenericFunctionU16Vec32(42, vec![4, 5, 6])
        .call()
        .await?;
    println!("echoWithGenericFunctionU16Vec32 ({}, {:?})", ret._0, ret._1);

    let ret = example
        .echoWithGenericFunctionAddressVec128(
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            vec![7, 8, 9],
        )
        .call()
        .await?;
    println!(
        "echoWithGenericFunctionAddressVec256 ({}, {:?})",
        ret._0, ret._1
    );

    // If the constructor is called, the storage value at init_key is should be different from 0
    let init_key = alloy::primitives::U256::from_be_bytes(keccak256(b"init_key").into());
    let init_value_le = storage_value_to_le(&provider, address, init_key).await?;
    println!("Storage value at init_key: {init_value_le:?}");

    // Storage key for the counter
    let counter_key =
        alloy::primitives::U256::from_be_bytes(keccak256(b"global_counter_key").into());

    let pending_tx = example.getUniqueIds().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("getUniqueIds - Emitted UID: 0x{}", decoded_uid.data.uid);
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("getUniqueIds - Emitted UID: 0x{}", decoded_uid.data.uid);
    }
    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("getUniqueIds - Emitted UID: 0x{}", decoded_uid.data.uid);
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let ret = example.getFreshObjectAddress().call().await?;
    println!("fresh new id {ret:?}");

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    // Events
    // Emit test event 1
    println!("Emitting test event 1");
    let pending_tx = example.emitTestEvent1(43).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent1 { n: 43 };

    // Decode event 1 from logs
    let logs = receipt.logs();
    for log in logs {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::TestEvent1::decode_log(&primitive_log)?;
        assert_eq!(event, decoded_event.data);
    }

    // Emit test event 2
    println!("Emitting test event 2");
    let pending_tx = example
        .emitTestEvent2(42, vec![43, 44, 45], 46)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent2 {
        a: 42,
        b: keccak256(&vec![43, 44, 45].abi_encode()[64..]),
        c: 46,
    };

    // Decode event 2 from logs
    let logs = receipt.logs();
    for log in logs {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::TestEvent2::decode_log(&primitive_log)?;
        assert_eq!(event, decoded_event.data);
    }

    // Emit test event 3
    println!("Emitting test event 3");
    let pending_tx = example
        .emitTestEvent3(42, 43, vec![44, 45, 46], 47)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent3 {
        a: keccak256(Example::NestedStruct1 { n: 42 }.abi_encode()),
        b: Example::NestedStruct2 {
            a: 43,
            b: vec![44, 45, 46],
            c: 47,
        },
    };
    // Decode event 3 from logs
    let logs = receipt.logs();
    for log in logs {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::TestEvent3::decode_log(&primitive_log)?;
        assert_eq!(event, decoded_event.data);
    }

    // Emit test event 4
    println!("Emitting test event 4");
    let pending_tx = example
        .emitTestEvent4(42, vec![43, 44, 45], vec![46, 47, 48], vec![49, 50, 51])
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent4 {
        a: 42,
        b: vec![43, 44, 45],
        c: vec![46, 47, 48],
        d: vec![49, 50, 51],
    };
    let logs = receipt.logs();
    for log in logs {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::TestEvent4::decode_log(&primitive_log)?;
        assert_eq!(event, decoded_event.data);
    }

    let s = example.testStack1().call().await?;
    println!("testStack1\nelements: {:?} len: {}", s._0.pos0, s._1);

    let s = example.testStack2().call().await?;
    println!("testStack2\nelements: {:?} len: {}", s._0.pos0, s._1);

    let s = example.testStack3().call().await?;
    println!("testStack3\nelements: {:?} len: {}", s._0.pos0, s._1);

    println!("\nSending plain ETH transfer to the contract (empty calldata)");
    println!("This should trigger the receive() function if it exists");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128));
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Successfully sent 1 ETH to the contract (plain transfer)");

    // Decode ReceiveEvent from the receipt logs
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let event = Example::ReceiveEvent {
            sender,
            data_length: 0,
            data: vec![],
        };
        assert_eq!(decoded_event.data, event);
    }

    println!("\nSending ETH with calldata");
    println!("This should trigger the fallback function");
    let calldata = vec![43, 44, 45].abi_encode();
    let calldata_len = calldata.len() as u32;
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128))
        .input(calldata.clone().into());
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let event = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded_event.data, event);
    }

    println!("\nSending ETH with a string as calldata");
    let calldata = String::from("hola como estas").abi_encode();
    let calldata_len = calldata.len() as u32;
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128))
        .input(calldata.clone().into());

    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let event = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded_event.data, event);
    }

    println!("\nSending ETH with a u256 as calldata");
    let calldata = U256::MAX.abi_encode();
    let calldata_len = calldata.len() as u32;
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128))
        .input(calldata.clone().into());

    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let event = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded_event.data, event);
    }

    Ok(())
}

/// Converts a storage value from big-endian (as read from storage) to little-endian (as stored)
async fn storage_value_to_le<T: Provider>(
    provider: &T,
    address: Address,
    key: alloy::primitives::U256,
) -> eyre::Result<alloy::primitives::U256> {
    let value = provider.get_storage_at(address, key).await?;
    Ok(alloy::primitives::U256::from_le_bytes(
        value.to_be_bytes::<32>(),
    ))
}
