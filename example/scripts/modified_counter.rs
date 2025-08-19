use alloy::hex;
use alloy::primitives::FixedBytes;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    sol,
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
        function create(bool share) public view;
        function read(bytes32 id) public view returns (uint64);
        function increment(bytes32 id) public view;
        function setValue(bytes32 id, uint64 value) public view;
        function deleteCounter(bytes32 id) public view;
        function freezeCounter(bytes32 id) public view;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_MODIFIED_COUNTER")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_MODIFIED_COUNTER"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    let share_flag = true;
    let pending_tx = example.create(share_flag).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Create Logs:");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }

    // Read storage at the slot where the counter is stored
    let storage_slot = receipt.logs()[1].data().data.0.clone();
    let storage_slot = FixedBytes::<32>::new(<[u8; 32]>::try_from(storage_slot.to_vec()).unwrap());
    let storage = provider
        .get_storage_at(address, storage_slot.into())
        .await?;
    println!("Storage slot: {:?}", storage_slot);
    println!("Storage after transfer: {:?}", storage);

    let counter_id = receipt.logs()[0].data().data.0.clone();
    let counter_id = FixedBytes::<32>::new(<[u8; 32]>::try_from(counter_id.to_vec()).unwrap());
    println!("Captured counter_id {:?}", counter_id);

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

    println!("\nSending freeze tx");
    let pending_tx = example.freezeCounter(counter_id).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    let frozen_storage_slot = receipt.logs()[3].data().data.0.clone();
    let frozen_storage_slot =
        FixedBytes::<32>::new(<[u8; 32]>::try_from(frozen_storage_slot.to_vec()).unwrap());
    let frozen_storage = provider
        .get_storage_at(address, frozen_storage_slot.into())
        .await?;
    println!("Frozen storage slot: {:?}", frozen_storage_slot);
    println!("Frozen storage after freeze: {:?}", frozen_storage);

    let storage = provider
        .get_storage_at(address, storage_slot.into())
        .await?;
    println!("storage slot: {:?}", storage_slot);
    println!("storage after freeze: {:?}", storage);

    Ok(())
}
