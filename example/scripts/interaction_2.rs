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
    hex,
    primitives::{Address, U256, address, keccak256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolValue,
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

        #[derive(Debug, PartialEq)]
        struct TestEvent1 {
            uint32 n;
        }

        #[derive(Debug, PartialEq)]
        struct TestEvent2 {
            uint32 a;
            uint8[] b;
            uint128 c;
        }

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
        struct TestEvent3 {
            NestedStruct1 a;
            NestedStruct2 b;
        }

        #[derive(Debug, PartialEq)]
        struct Stack {
            uint32[] pos0;
        }

        #[derive(Debug, PartialEq)]
        struct ReceiveEvent {
            address sender;
        }

        function emitTestEvent1(uint32 n) public view;
        function emitTestEvent2(uint32 a, uint8[] b, uint128 c) public view;
        function emitTestEvent3(uint32 n, uint32 a, uint8[] b, uint128 c) public view;
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
        function fallback(address a, uint32 b) external payable;
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
        let raw = log.data().data.0.clone();
        println!("getUniqueIds - Emitted UID: 0x{}", hex::encode(raw));
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("getUniqueId - Emitted UID: 0x{}", hex::encode(raw));
    }
    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("getUniqueId - Emitted UID: 0x{}", hex::encode(raw));
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    let ret = example.getFreshObjectAddress().call().await?;
    println!("fresh new id {ret:?}");

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {storage_value_le:?}");

    // Events
    // Emit test event 1
    let pending_tx = example.emitTestEvent1(43).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent1 { n: 43 };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let topics = log.topics();
        let decoded_event = Example::TestEvent1::abi_decode(topics[1].as_slice())?;
        assert_eq!(event, decoded_event);
    }

    // Emit test event 2
    let pending_tx = example
        .emitTestEvent2(42, vec![43, 44, 45], 46)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent2 {
        a: 42,
        b: vec![43, 44, 45],
        c: 46,
    };

    // Decode the event data
    // TestEvent2 has indexes = 2, so fields 'a' and 'b' are indexed
    // topics[0] = event signature, topics[1] = indexed 'a' (u32), topics[2] = indexed 'b' (vector<u8> hash)
    // data field contains only 'c' (u128) which is not indexed
    let logs = receipt.logs();
    for log in logs {
        let topics = log.topics();
        let data = log.data().data.0.clone();

        // Decode indexed 'a' (u32) from topics[1] using SolValue
        let a = u32::abi_decode(topics[1].as_slice())?;

        // let b = event.b.abi_encode();
        // let b_hash = keccak256(b);
        // assert_eq!(topics[2].as_slice(), b_hash.as_slice(), "Hash of 'b' vector doesn't match");

        // Decode non-indexed 'c' (u128) from data using SolValue
        let c = u128::abi_decode(&data)?;

        assert_eq!(event.a, a);
        assert_eq!(event.c, c);
    }

    // Emit test event 3
    let pending_tx = example
        .emitTestEvent3(43, 43, vec![1, 2, 3], 1234_u128)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent3 {
        a: Example::NestedStruct1 { n: 43 },
        b: Example::NestedStruct2 {
            a: 43,
            b: vec![1, 2, 3],
            c: 1234_u128,
        },
    };
    // Decode the event data
    // TestEvent3 has indexes = 2, so fields 'a' and 'b' (both structs) are indexed
    // topics[0] = event signature, topics[1] = indexed 'a' (NestedStruct1 hash), topics[2] = indexed 'b' (NestedStruct2 hash)
    // data field is empty (both structs are indexed/hashed)
    let logs = receipt.logs();
    for log in logs {
        let _topics = log.topics();

        // // Verify the hash of 'a' (NestedStruct1) in topics[1]
        // let a_encoded = event.a.abi_encode();
        // let a_hash = keccak256(a_encoded);
        // assert_eq!(topics[1].as_slice(), a_hash.as_slice(), "Hash of 'a' struct doesn't match");

        // // Verify the hash of 'b' (NestedStruct2) in topics[2]
        // let b_encoded = event.b.abi_encode();
        // let b_hash = keccak256(b_encoded);
        // assert_eq!(topics[2].as_slice(), b_hash.as_slice(), "Hash of 'b' struct doesn't match");

        println!(
            "Decoded event: a={:?} (hash verified), b={:?} (hash verified)",
            event.a, event.b
        );
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
        let topics = log.topics();
        let decoded_event = Example::ReceiveEvent::abi_decode(topics[1].as_slice())?;
        let event = Example::ReceiveEvent { sender };
        assert_eq!(decoded_event, event);
    }

    println!("\nCalling fallback with b < 42 by sending ETH with calldata");
    println!("This should emit ReceiveEvent with sender = parameter 'a'");
    let other = address!("0x0000000000000000000000000000000000000001");
    // Encode fallback arguments (a: address, b: u32) as ABI parameters, without selector.
    // Any non-empty calldata that doesn't match a function selector will route to fallback;
    // here we also make the bytes ABI-compatible with the Move fallback's parameters.
    let calldata_lt = (other, 41_u32).abi_encode();
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128))
        .input(calldata_lt.into());
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let topics = log.topics();
        // ReceiveEvent has indexes = 1, so topics[1] holds the indexed sender address
        let decoded_event = Example::ReceiveEvent::abi_decode(topics[1].as_slice())?;
        let event = Example::ReceiveEvent { sender: other };
        assert_eq!(decoded_event, event);
        println!(
            "fallback(b < 42) emitted ReceiveEvent with sender: {:?}",
            decoded_event.sender
        );
    }

    println!("\nCalling fallback with b >= 42 by sending ETH with calldata");
    println!("This should emit ReceiveEvent with sender = ctx.sender()");
    let calldata_ge = (other, 43_u32).abi_encode();
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128))
        .input(calldata_ge.into());
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let topics = log.topics();
        let decoded_event = Example::ReceiveEvent::abi_decode(topics[1].as_slice())?;
        let event = Example::ReceiveEvent { sender };
        assert_eq!(decoded_event, event);
        println!(
            "fallback(b >= 42) emitted ReceiveEvent with sender: {:?}",
            decoded_event.sender
        );
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
