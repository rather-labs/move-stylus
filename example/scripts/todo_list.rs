use alloy::hex;
use alloy::primitives::{FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::{primitives::Address, providers::ProviderBuilder, sol, transports::http::reqwest::Url};
use dotenv::dotenv;
use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]

    contract Example {
        
        #[derive(Debug)]
        struct String {
            uint8[] bytes;
        }

        function new() public view;
        function add(bytes32 id, String item) public;
        function remove(bytes32 id, uint64 index) public view returns (String);
        function delete(bytes32 id) public view;
        function length(bytes32 id) public view returns (uint64);
        function read(bytes32 id) public view returns (String[]);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_TODO_LIST")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_TODO_LIST"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    let pending_tx = example.new_call().send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Creating a new todo list and capturing its id");
    let list_id = receipt.logs()[0].data().data.0.clone();
    let list_id = FixedBytes::<32>::new(<[u8; 32]>::try_from(list_id.to_vec()).unwrap());
    println!("Captured todo list id {:?}", list_id);
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }

    let items = example.read(list_id).call().await?;
    println!("Todo list");
    for item in items {
        println!(" - {}", String::from_utf8(item.bytes).unwrap());
    }
    let len = example.length(list_id).call().await?;
    assert_eq!(len, 0);

    let pending_tx = example.add(list_id, Example::String { bytes: "Buy groceries".as_bytes().to_vec() }).send().await?;
    let _ = pending_tx.get_receipt().await?;
    println!("Added item to todo list");

    let items = example.read(list_id).call().await?;
    println!("Todo list");
    for item in items {
        println!(" - {}", String::from_utf8(item.bytes).unwrap());
    }

    let len = example.length(list_id).call().await?;
    assert_eq!(len, 1);

    let pending_tx = example.add(list_id, Example::String { bytes: "Go to the gym".as_bytes().to_vec() }).send().await?;
    let _ = pending_tx.get_receipt().await?;

    println!("Added item to todo list");

    let items = example.read(list_id).call().await?;
    println!("Todo list");
    for item in items {
        println!(" - {}", String::from_utf8(item.bytes).unwrap());
    }

    let len = example.length(list_id).call().await?;
    assert_eq!(len, 2);

    let pending_tx = example.remove(list_id, 1).send().await?;
    let _ = pending_tx.get_receipt().await?;
    println!("Removed item from todo list");

    let items = example.read(list_id).call().await?;
    println!("Todo list");
    for item in items {
        println!(" - {}", String::from_utf8(item.bytes).unwrap());
    }

    let len = example.length(list_id).call().await?;
    assert_eq!(len, 1);

    let pending_tx = example.delete(list_id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Deleted todo list");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("delete tx 0x{}", hex::encode(&raw));
    }

    Ok(())
}
