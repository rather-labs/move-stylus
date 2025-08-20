use alloy::hex;
use alloy::primitives::FixedBytes;
use alloy::providers::Provider;
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
        function walkTheDog(bytes32 capability) public view;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let priv_key_2 = std::env::var("PRIV_KEY_2").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_DOG_WALKER")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_DOG_WALKER"))?;

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

    println!("Creating a new capability and capturing its id");
    let tx = example.create().into_transaction_request().from(sender);
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    let capability_id = receipt.logs()[0].data().data.0.clone();
    let capability_id = FixedBytes::<32>::new(<[u8; 32]>::try_from(capability_id.to_vec()).unwrap());
    println!("Captured capability {:?}", capability_id);
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("constructor 0x{}", hex::encode(&raw));
    }

    println!("\nWalking the dog with owner {sender}");
    let pending_tx = example.walkTheDog(capability_id).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("walking the dog logs: 0x{}", hex::encode(raw));
    }

    let signer = PrivateKeySigner::from_str(&priv_key_2)?;
    let sender = signer.address();
    println!("\nWalking the dog with another user {sender} (should fail)");

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );

    let example_2 = Example::new(address, provider.clone());

    let pending_tx = example_2.walkTheDog(capability_id).send().await;
    println!("Tx failed?: {:?}", pending_tx.is_err());

    Ok(())
}
