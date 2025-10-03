use alloy::{hex, primitives::address};
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
        function mint(address to, uint256 amount) external view;
        function burn(address from, uint256 amount) external view;
        function balanceOf(address account) public view returns (uint256);
        function totalSupply() external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_ERC20")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC20"))?;

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


    let address_1 = address!("0xcafecafecafecafecafecafecafecafecafecafe");

    println!("====================");
    println!("Creating a new erc20");
    println!("====================");
    let pending_tx = example.create().send().await?;
    let _receipt = pending_tx.get_receipt().await?;
    /*
    let counter_id = receipt.logs()[0].data().data.0.clone();
    let counter_id = FixedBytes::<32>::new(<[u8; 32]>::try_from(counter_id.to_vec()).unwrap());
    println!("Captured counter_id {:?}", counter_id);
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }
    */

    println!("created");


    println!("\n====================");
    println!("Contract Info");
    println!("====================");

    let res = example.totalSupply().call().await?;
    println!("Total Supply = {}", res);

    let res = example.decimals().call().await?;
    println!("decimals = {}", res);

    let res = example.name().call().await?;
    println!("name = {}", res);

    let res = example.symbol().call().await?;
    println!("symbol = {}", res);

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of target address = {}", res);

    println!("\n====================");
    println!("Mint");
    println!("====================");

    println!("Minting 555555 coins to target address");

    let pending_tx = example.mint(sender, U256::from(555555)).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Mint events");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }
    let res = example.totalSupply().call().await?;
    println!("Total Supply after mint = {}", res);

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of target address = {}", res);

    println!("\n====================");
    println!("Transfer");
    println!("====================");

    println!("Transfering 1000 TST to {address_1}");

    let res = example.balanceOf(sender).call().await?;
    println!("  Balance of origin address {sender} before transaction = {}", res);
    let res = example.balanceOf(address_1).call().await?;
    println!("  Balance of target address {address_1} before transaction = {}", res);

    let pending_tx = example.transfer(address_1, U256::from(1000)).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }


    let res = example.balanceOf(sender).call().await?;
    println!("  Balance of origin address {sender} after transaction = {}", res);
    let res = example.balanceOf(address_1).call().await?;
    println!("  Balance of target address {address_1} after transaction = {}", res);


    println!("\n====================");
    println!("Burn");
    println!("====================");

    println!("Burning 11111 coins to from {sender}");

    let pending_tx = example.burn(sender, U256::from(11111)).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Burn events");
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }
    let res = example.totalSupply().call().await?;
    println!("Total Supply after burn= {}", res);

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of target address = {}", res);

    Ok(())
}
