use alloy::primitives::{Address, U256, address, keccak256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    providers::ProviderBuilder,
    sol,
    sol_types::{SolEvent, SolValue},
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

    // Helper to get last 8 hex chars of an address (0x + last 4 bytes)
    let short_addr = |addr: &Address| -> String {
        let s = format!("{addr}");
        format!("0x..{}", &s[s.len() - 8..])
    };

    let sender_short = short_addr(&sender);

    // ==================== Generic Functions ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Generic Functions            ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.echoWithGenericFunctionU16(42).call().await?;
    assert_eq!(res, 42u16);
    println!("  ✓ echoWithGenericFunctionU16(42) = {res}");

    let res = example
        .echoWithGenericFunctionVec32(vec![1, 2, 3])
        .call()
        .await?;
    assert_eq!(res, vec![1, 2, 3]);
    println!("  ✓ echoWithGenericFunctionVec32([1,2,3]) = {res:?}");

    let res = example
        .echoWithGenericFunctionU16Vec32(42, vec![4, 5, 6])
        .call()
        .await?;
    assert_eq!(res._0, 42u16);
    assert_eq!(res._1, vec![4, 5, 6]);
    println!(
        "  ✓ echoWithGenericFunctionU16Vec32(42, [4,5,6]) = ({}, {:?})",
        res._0, res._1
    );

    let test_addr = address!("0x1234567890abcdef1234567890abcdef12345678");
    let res = example
        .echoWithGenericFunctionAddressVec128(test_addr, vec![7, 8, 9])
        .call()
        .await?;
    assert_eq!(res._0, test_addr);
    assert_eq!(res._1, vec![7, 8, 9]);
    println!(
        "  ✓ echoWithGenericFunctionAddressVec128({}, [7,8,9]) = ({}, {:?})",
        short_addr(&test_addr),
        short_addr(&res._0),
        res._1
    );

    // ==================== Storage & Constructor ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║       Storage & Constructor          ║");
    println!("╚══════════════════════════════════════╝");

    let init_key = U256::from_be_bytes(keccak256(b"init_key").into());
    let init_value_le = storage_value_to_le(&provider, address, init_key).await?;
    println!("  init_key storage value: {init_value_le}");

    let counter_key = U256::from_be_bytes(keccak256(b"global_counter_key").into());
    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("  global_counter_key: {storage_value_le}");

    // ==================== Unique IDs ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║            Unique IDs                ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Calling getUniqueIds() (generates 3 UIDs)...");
    let pending_tx = example.getUniqueIds().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("    ✓ Emitted UID: 0x{}", decoded_uid.data.uid);
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("  Counter value: {storage_value_le}");

    println!("\n  Calling getUniqueId() (generates 1 UID)...");
    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("    ✓ Emitted UID: 0x{}", decoded_uid.data.uid);
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("  Counter value: {storage_value_le}");

    println!("\n  Calling getUniqueId() again...");
    let pending_tx = example.getUniqueId().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let log: alloy::primitives::Log = log.clone().into();
        let decoded_uid = Example::NewUID::decode_log(&log).unwrap();
        println!("    ✓ Emitted UID: 0x{}", decoded_uid.data.uid);
    }

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("  Counter value: {storage_value_le}");

    println!("\n  Calling getFreshObjectAddress()...");
    let res = example.getFreshObjectAddress().call().await?;
    println!("    ✓ Fresh address: {}", short_addr(&res));

    let storage_value_le = storage_value_to_le(&provider, address, counter_key).await?;
    println!("  Counter value: {storage_value_le}");

    // ==================== Event Emission ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║          Event Emission              ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  TestEvent1 (indexed uint32):");
    let pending_tx = example.emitTestEvent1(43).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    let expected = Example::TestEvent1 { n: 43 };
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded = Example::TestEvent1::decode_log(&primitive_log)?;
        assert_eq!(expected, decoded.data);
    }
    println!("    ✓ emitTestEvent1(43) decoded correctly");

    println!("\n  TestEvent2 (indexed uint32, indexed uint8[], uint128):");
    let pending_tx = example
        .emitTestEvent2(42, vec![43u8, 44, 45], 46)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let expected = Example::TestEvent2 {
        a: 42,
        b: keccak256(&vec![43, 44, 45].abi_encode()[64..]),
        c: 46,
    };

    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded = Example::TestEvent2::decode_log(&primitive_log)?;
        assert_eq!(expected, decoded.data);
    }
    println!("    ✓ emitTestEvent2(42, [43,44,45], 46) decoded correctly");

    println!("\n  TestEvent3 (indexed struct, struct):");
    let pending_tx = example
        .emitTestEvent3(42, 43, vec![44, 45, 46], 47)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let expected = Example::TestEvent3 {
        a: keccak256(Example::NestedStruct1 { n: 42 }.abi_encode()),
        b: Example::NestedStruct2 {
            a: 43,
            b: vec![44, 45, 46],
            c: 47,
        },
    };
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded = Example::TestEvent3::decode_log(&primitive_log)?;
        assert_eq!(expected, decoded.data);
    }
    println!("    ✓ emitTestEvent3(42, 43, [44,45,46], 47) decoded correctly");

    println!("\n  TestEvent4 (indexed uint32, uint16[], uint8[], uint32[]):");
    let pending_tx = example
        .emitTestEvent4(42, vec![43, 44, 45], vec![46, 47, 48], vec![49, 50, 51])
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let expected = Example::TestEvent4 {
        a: 42,
        b: vec![43, 44, 45],
        c: vec![46, 47, 48],
        d: vec![49, 50, 51],
    };
    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded = Example::TestEvent4::decode_log(&primitive_log)?;
        assert_eq!(expected, decoded.data);
    }
    println!("    ✓ emitTestEvent4(42, [43,44,45], [46,47,48], [49,50,51]) decoded correctly");

    // ==================== Stack Operations ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Stack Operations             ║");
    println!("╚══════════════════════════════════════╝");

    let s = example.testStack1().call().await?;
    println!("  ✓ testStack1(): elements={:?}, len={}", s._0.pos0, s._1);

    let s = example.testStack2().call().await?;
    println!("  ✓ testStack2(): elements={:?}, len={}", s._0.pos0, s._1);

    let s = example.testStack3().call().await?;
    println!("  ✓ testStack3(): elements={:?}, len={}", s._0.pos0, s._1);

    // ==================== Receive & Fallback ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        Receive & Fallback            ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Plain ETH transfer (triggers receive):");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .value(U256::from(1_000_000_000_000_000_000u128));
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;

    for log in receipt.logs() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let expected = Example::ReceiveEvent {
            sender,
            data_length: 0,
            data: vec![],
        };
        assert_eq!(decoded.data, expected);
    }
    println!("    ✓ Sent 1 ETH, ReceiveEvent decoded (sender={sender_short}, data_length=0)");

    println!("\n  ETH with calldata [43,44,45] (triggers fallback):");
    let calldata = vec![43u8, 44, 45].abi_encode();
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
        let decoded = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let expected = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded.data, expected);
    }
    println!("    ✓ Sent 1 ETH + calldata, ReceiveEvent decoded (data_length={calldata_len})");

    println!("\n  ETH with string calldata (triggers fallback):");
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
        let decoded = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let expected = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded.data, expected);
    }
    println!(
        "    ✓ Sent 1 ETH + \"hola como estas\", ReceiveEvent decoded (data_length={calldata_len})"
    );

    println!("\n  ETH with U256::MAX calldata (triggers fallback):");
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
        let decoded = Example::ReceiveEvent::decode_log(&primitive_log)?;
        let expected = Example::ReceiveEvent {
            sender,
            data_length: calldata_len,
            data: calldata.clone(),
        };
        assert_eq!(decoded.data, expected);
    }
    println!("    ✓ Sent 1 ETH + U256::MAX, ReceiveEvent decoded (data_length={calldata_len})");

    // ==================== Done ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        ✓ All tests passed!           ║");
    println!("╚══════════════════════════════════════╝\n");

    Ok(())
}

/// Converts a storage value from big-endian (as read from storage) to little-endian (as stored)
async fn storage_value_to_le<T: Provider>(
    provider: &T,
    address: Address,
    key: U256,
) -> eyre::Result<U256> {
    let value = provider.get_storage_at(address, key).await?;
    Ok(U256::from_le_bytes(value.to_be_bytes::<32>()))
}
