use alloy::hex;
use alloy::primitives::address;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    primitives::Address, primitives::U256, providers::ProviderBuilder, sol,
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
        function create() public view;
        function read(address id) public view returns (uint64);
        function increment(address id) public view;
        function setValue(address id, uint64 value) public view;
        function deleteCounter(address id) public view;
        function slotReader(uint256 slot) public view returns (uint256);
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
    println!("Create Logs:");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }
    let slots = [
        U256::from_str("0x05d0ca05d46093310ab4a19866376b885c66147ea78c86c5ac6a70e8d0cfeb54")
            .unwrap(),
        U256::from_str("0x05d0ca05d46093310ab4a19866376b885c66147ea78c86c5ac6a70e8d0cfeb55")
            .unwrap(),
        U256::from_str("0x05d0ca05d46093310ab4a19866376b885c66147ea78c86c5ac6a70e8d0cfeb56")
            .unwrap(),
    ];
    for slot in slots {
        let res = example.slotReader(slot).call().await?;
        println!("slotReader = {}", res);
    }

    let id = address!("0x0000000000000000000000000000000000001234");
    // let res = example.read(U256::from(1234).to_le_bytes().into()).call().await?;
    let res = example.read(id).call().await?;
    println!("counter = {}", res);

    let pending_tx = example.increment(id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Increment Logs:");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }

    let res = example.read(id).call().await?;
    println!("counter = {}", res);

    for slot in slots {
        let res = example.slotReader(slot).call().await?;
        println!("slotReader = {}", res);
    }

    let pending_tx = example.setValue(id, 42).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    let res = example.read(id).call().await?;
    println!("counter = {}", res);

    let pending_tx = example.increment(id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    let res = example.read(id).call().await?;
    println!("counter = {}", res);

    let pending_tx = example.deleteCounter(id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Deleter Logs:");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }

    for slot in slots {
        let res = example.slotReader(slot).call().await?;
        println!("slotReader = {}", res);
    }

    Ok(())
}
