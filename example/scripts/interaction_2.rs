//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use alloy::hex;
use alloy::primitives::keccak256;
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    primitives::{Address, address},
    providers::ProviderBuilder,
    sol,
    sol_types::SolValue,
    transports::http::reqwest::Url,
};
use dotenv::dotenv;
use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;

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
        struct TestEvent3 {
            TestEvent1 a;
            TestEvent2 b;
        }

        #[derive(Debug, PartialEq)]
        struct TestGenericEvent1 {
            uint32 o;
            bool p;
            TestEvent1 q;
        }

        #[derive(Debug, PartialEq)]
        struct TestGenericEvent2 {
            uint32 o;
            bool p;
            TestEvent1 q;
            uint32[] r;
            TestGenericEvent1 s;
        }

        #[derive(Debug, PartialEq)]
        struct Stack {
            uint32[] pos0;
        }

        function emitTestEvent1(uint32 n) public view;
        function emitTestEvent2(uint32 a, uint8[] b, uint128 c) public view;
        function emitTestEvent3(TestEvent1 a, TestEvent2 b) public view;
        function emitTestEventGeneric1(uint32 o, bool p, TestEvent1 q) public view;
        function emitTestEventGeneric2(uint32 o, bool p, TestEvent1 q, uint32[] r) public view;
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
    println!("Storage value at init_key: {:?}", init_value_le);

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
    println!("Counter value: {:?}", storage_value_le);

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("getUniqueId - Emitted UID: 0x{}", hex::encode(raw));
    }
    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {:?}", storage_value_le);

    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("getUniqueId - Emitted UID: 0x{}", hex::encode(raw));
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {:?}", storage_value_le);

    let ret = example.getFreshObjectAddress().call().await?;
    println!("fresh new id {ret:?}");

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("Counter value: {:?}", storage_value_le);

    // Events
    // Emit test event 1
    let pending_tx = example.emitTestEvent1(43).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent1 { n: 43 };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent1 as SolValue>::abi_decode(&data)?;
        assert_eq!(event, decoded_event);
        println!("Decoded event data = {:?}", decoded_event);
    }

    // Emit test event 2
    let pending_tx = example
        .emitTestEvent2(43, vec![1, 2, 3], 1234)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent2 {
        a: 43,
        b: vec![1, 2, 3],
        c: 1234,
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent2 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    // Emit test event 3
    let pending_tx = example
        .emitTestEvent3(
            Example::TestEvent1 { n: 43 },
            Example::TestEvent2 {
                a: 43,
                b: vec![1, 2, 3],
                c: 1234,
            },
        )
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent3 {
        a: Example::TestEvent1 { n: 43 },
        b: Example::TestEvent2 {
            a: 43,
            b: vec![1, 2, 3],
            c: 1234,
        },
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent3 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    // Emit test event with generics 1
    let pending_tx = example
        .emitTestEventGeneric1(43, true, Example::TestEvent1 { n: 43 })
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestGenericEvent1 {
        o: 43,
        p: true,
        q: Example::TestEvent1 { n: 43 },
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestGenericEvent1 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    // Emit test event with generics 2
    let pending_tx = example
        .emitTestEventGeneric2(43, true, Example::TestEvent1 { n: 43 }, vec![1, 2, 3])
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestGenericEvent2 {
        o: 43,
        p: true,
        q: Example::TestEvent1 { n: 43 },
        r: vec![1, 2, 3],
        s: Example::TestGenericEvent1 {
            o: 43,
            p: true,
            q: Example::TestEvent1 { n: 43 },
        },
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestGenericEvent2 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:#?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    let s = example.testStack1().call().await?;
    println!("testStack1\nelements: {:?} len: {}", s._0.pos0, s._1);

    let s = example.testStack2().call().await?;
    println!("testStack2\nelements: {:?} len: {}", s._0.pos0, s._1);

    let s = example.testStack3().call().await?;
    println!("testStack3\nelements: {:?} len: {}", s._0.pos0, s._1);

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
