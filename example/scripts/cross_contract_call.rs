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
        function balanceOfErc20(address contract, address token_owner) public view returns (uint256);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;

    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address_erc20 = std::env::var("CONTRACT_ADDRESS_ERC20")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC20"))?;
let contract_address_erc20 = Address::from_str(&contract_address_erc20)?;

    let contract_address = std::env::var("CONTRACT_ADDRESS_CROSS_CALL")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_CROSS_CALL"))?;

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

    let res = example.balanceOfErc20(contract_address_erc20, sender).call().await?;
    println!("Balance of {sender} = {res}");

    let res = example.balanceOfErc20(contract_address_erc20, sender).send().await?;
    println!("Balance of {sender} = {res:?}");


    Ok(())
}
