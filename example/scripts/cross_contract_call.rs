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

        function balanceOfErc20(address erc20_address, address balance_address) external returns (uint256);
        function totalSupply(address erc20_address) external returns (uint256);
        function transferFromErc20(
            address erc20_address,
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
        function allowance(address owner, address spender) external returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external returns (uint256);
        function burn(address from, uint256 amount) external;
        function decimals() external returns (uint8);
        function mint(address to, uint256 amount) external;
        function name() external returns (string);
        function symbol() external returns (string);
        function totalSupply() external returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
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

    println!("==============================================================================");
    println!(" Querying ERC20 balances via cross-contract call");
    println!("==============================================================================");
    let res = example.totalSupply(contract_address_erc20).call().await?;
    println!("Total Supply: {res}");

    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    let sender_balance_initial = res;
    println!("Balance of {sender}: {sender_balance_initial}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    let address_2_balance_initial = res;
    println!("Balance of {address_2}: {address_2_balance_initial}");

    println!("\n==============================================================================");
    println!(" Approving cross-contract call: allow contract to spend 1000 tokens");
    println!("==============================================================================");
    let res = erc20.allowance(sender, address).call().await?;
    let allowance_before = res;
    println!("Current allowance: {allowance_before}");

    println!("\nExecuting approve transaction...");
    let pending_tx = erc20.approve(address, U256::from(1000)).send().await?;
    let _receipt = pending_tx.get_receipt().await?;
    println!("✓ Approval succeeded");

    let res = erc20.allowance(sender, address).call().await?;
    let allowance_after = res;
    println!("Current allowance: {allowance_after}");
    assert_eq!(
        allowance_after,
        U256::from(1000),
        "Allowance should be set to 1000"
    );

    println!("\n==============================================================================");
    println!(" Executing cross-contract transferFrom");
    println!("==============================================================================");

    println!("\nBalances and allowance BEFORE cross-contract call to transferFrom:");
    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    let sender_balance_before = res;
    println!("  Balance of {sender}: {sender_balance_before}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    let address_2_balance_before = res;
    println!("  Balance of {address_2}: {address_2_balance_before}");

    let res = erc20.allowance(sender, address).call().await?;
    let allowance_before_transfer = res;
    println!("  Current allowance: {allowance_before_transfer}");

    println!("\nExecuting transferFrom transaction (1000 tokens)...");
    let transfer_amount = U256::from(1000);
    let pending_tx = example
        .transferFromErc20(contract_address_erc20, sender, address_2, transfer_amount)
        .send()
        .await?;
    let _receipt = pending_tx.get_receipt().await?;
    println!("✓ TransferFrom succeeded");

    println!("\nBalances and allowance AFTER cross-contract call to transferFrom:");
    let res = example
        .balanceOfErc20(contract_address_erc20, sender)
        .call()
        .await?;
    let sender_balance_after = res;
    println!("  Balance of {sender}: {sender_balance_after}");

    let res = example
        .balanceOfErc20(contract_address_erc20, address_2)
        .call()
        .await?;
    let address_2_balance_after = res;
    println!("  Balance of {address_2}: {address_2_balance_after}");

    let res = erc20.allowance(sender, address).call().await?;
    let allowance_after_transfer = res;
    println!("  Current allowance: {allowance_after_transfer}");

    // Verify the transfer worked correctly
    assert_eq!(
        sender_balance_after,
        sender_balance_before - transfer_amount,
        "Sender balance should decrease by transfer amount"
    );
    assert_eq!(
        address_2_balance_after,
        address_2_balance_before + transfer_amount,
        "Recipient balance should increase by transfer amount"
    );
    assert_eq!(
        allowance_after_transfer,
        allowance_before_transfer - transfer_amount,
        "Allowance should decrease by transfer amount"
    );
    println!("\n✓ Cross-contract transfer verified: balances and allowance updated correctly");

    Ok(())
}
