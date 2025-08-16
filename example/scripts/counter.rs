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
        function create(bool share) public view;
        function read(address id) public view returns (uint64);
        function readOwner(address id) public view returns (address);
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

    let share_flag = false;
    let pending_tx = example.create(share_flag).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Create Logs:");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }
    let id = address!("0x0000000000000000000000000000000000001234");

    let pending_tx = example.readOwner(id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Read Owner Logs:");

    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("  - 0x{}", hex::encode(raw));
    }

    let res = example.readOwner(id).call().await?;
    println!("owner = {}", res);

    let slots = [
        U256::from_str("0x221689f749568568ffabb655ec216a45ca02fc4d4a4e7184a9569d4cd6113749")
            .unwrap(),
        U256::from_str("0x221689f749568568ffabb655ec216a45ca02fc4d4a4e7184a9569d4cd611374a")
            .unwrap(),
        U256::from_str("0x221689f749568568ffabb655ec216a45ca02fc4d4a4e7184a9569d4cd611374b")
            .unwrap(),
    ];
    for slot in slots {
        let res = example.slotReader(slot).call().await?;
        println!("slotReader = {}", res);
    }

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
