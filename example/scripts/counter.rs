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
        function create() public view;
        function read(bytes32 id) public view returns (uint64);
        function increment(bytes32 id) public view;
        function setValue(bytes32 id, uint64 value) public view;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_COUNTER")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_COUNTER"))?;

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

    let pending_tx = example.create().send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Creating a new counter and capturing its id");
    let counter_id = receipt.logs()[0].data().data.0.clone();
    let counter_id = FixedBytes::<32>::new(<[u8; 32]>::try_from(counter_id.to_vec()).unwrap());
    println!("Captured counter_id {:?}", counter_id);
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }

    println!("\nReading value before increment");
    let res = example.read(counter_id).call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");
    let pending_tx = example.increment(counter_id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read(counter_id).call().await?;
    println!("counter = {}", res);

    println!("\nSetting counter to number 42");
    let pending_tx = example.setValue(counter_id, 42).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading counter after set");
    let res = example.read(counter_id).call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");
    let pending_tx = example.increment(counter_id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read(counter_id).call().await?;
    println!("counter = {}", res);

    // Add a new sender and try to set the value
    let priv_key_2 =
        std::env::var("PRIV_KEY_2").map_err(|_| eyre!("No {} env var set", "PRIV_KEY_2"))?;
    let signer_2 = PrivateKeySigner::from_str(&priv_key_2)?;
    let sender_2 = signer_2.address();

    let provider_2 = Arc::new(
        ProviderBuilder::new()
            .wallet(signer_2)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let example_2 = Example::new(address, provider_2.clone());

    println!("\nFunding {sender_2} with some ETH to pay for the gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(5_000_000_000_000_000_000u128)); // 5 eth in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;

    println!("\nSending set value to 100 tx with the account that is not the owner");
    let pending_tx = example_2.setValue(counter_id, 100).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("set value logs 0: 0x{}", hex::encode(raw));
    }

    // Value did not change as the sender is not the owner
    println!("\nReading value after set value");
    let res = example_2.read(counter_id).call().await?;
    println!("counter = {}", res);

    Ok(())
}
