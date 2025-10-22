use alloy::primitives::{U256, address};
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
        function totalSupply(address contract) public view returns (uint256);
        function transferFromErc20(
            address contract,
            address sender,
            address recipient,
            uint256 amount
        ) external returns (bool);
    }
);

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Erc20{
        function constructor() public view;
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
    let contract_address_erc20 = std::env::var("CONTRACT_ADDRESS_ERC20")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC20"))?;

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

    let contract_address_erc20 = Address::from_str(&contract_address_erc20)?;
    let erc20 = Erc20::new(contract_address_erc20, provider.clone());

    let address_2 = address!("0xcafecafecafecafecafecafecafecafecafecafe");

    println!("====================");
    println!("Querying balances of ERC20 from other contract..");
    println!("====================");
    let res = example.totalSupply(contract_address_erc20).call().await?;
    println!("Total Supply = {res}");

    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    println!("Balance of {sender} = {res:?}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    println!("Balance of {address_2} = {res:?}");

    println!("====================");
    println!("Allow this contract ({address}) to spend 1000 TST from {sender}");
    println!("====================\n");
    let res = erc20.allowance(sender, address).call().await?;
    println!("  Current allowance = {}", res);

    println!();

    println!("Executing allow...");
    let pending_tx = erc20.approve(address, U256::from(1000)).send().await?;
    let receipt = pending_tx.get_receipt().await;
    println!("Approval succeded {}", receipt.is_ok());

    let res = erc20.allowance(sender, address).call().await?;
    println!("  Current allowance = {}", res);

    println!("====================");
    println!("Using transferFrom function from this contract ({address})");
    println!("====================\n");

    println!("Balances and allowance BEFORE corss call to transferFrom:");
    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    println!("\tBalance of {sender} = {res:?}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    println!("\tBalance of {address_2} = {res:?}");

    let res = erc20.allowance(sender, address).call().await?;
    println!("\tCurrent allowance = {}", res);

    let pending_tx = example
        .transferFromErc20(contract_address_erc20, sender, address_2, U256::from(1000))
        .send()
        .await?;
    let _receipt = pending_tx.get_receipt().await?;
    println!("Transfer From succeded");

    println!("Balances and allowance AFTER corss call to transferFrom:");
    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    println!("\tBalance of {sender} = {res:?}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    println!("\tBalance of {address_2} = {res:?}");

    let res = erc20.allowance(sender, address).call().await?;
    println!("\tCurrent allowance = {}", res);

    Ok(())
}
