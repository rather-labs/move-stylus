use alloy::hex;
use alloy::primitives::FixedBytes;
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
        function constructor() public view;
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
    let contract_address = std::env::var("CONTRACT_ADDRESS_COUNTER_WITH_INIT")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_COUNTER_WITH_INIT"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    // Call the constructor
    // The idea is that the constructor will be called upon deployment of the contract
    let pending_tx = example.constructor().send().await?;
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
    let _ = pending_tx.get_receipt().await?;

    println!("\nReading value after increment");
    let res = example.read(counter_id).call().await?;
    println!("counter = {}", res);

    // Call it a second time to make sure the constructor is not called again
    let pending_tx = example.constructor().send().await?;
    let receipt = pending_tx.get_receipt().await?;

    // Check no log is emitted, meaning the constructor logic is not executed again
    assert_eq!(receipt.logs().len(), 0);
    // Read again and check the value has not changed
    let res = example.read(counter_id).call().await?;
    assert_eq!(res, 26);

    println!("\nSending increment tx");
    let pending_tx = example.increment(counter_id).send().await?;
    let _ = pending_tx.get_receipt().await?;

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

    Ok(())
}
