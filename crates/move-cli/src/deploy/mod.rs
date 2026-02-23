// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

pub mod check;
pub mod constants;
pub mod project;
pub mod util;
pub mod export_abi;

use std::{fs, path::PathBuf};

use crate::deploy::{
    check::ContractCheck,
    constants::{ARB_WASM_ADDRESS, DEFAULT_ENDPOINT},
    util::{color::{Color, DebugColor}, text::decode0x},
};
use alloy::{
    json_abi::Constructor, network::{EthereumWallet, TransactionBuilder}, primitives::{Address, B256, FixedBytes, U256, utils::{format_units, parse_ether}}, providers::{Provider, ProviderBuilder}, rpc::types::{TransactionReceipt, TransactionRequest}, signers::{Signer, local::{LocalSigner, PrivateKeySigner}}, sol, sol_types::SolCall
};
use clap::{ArgGroup, Args};
use eyre::{Result, WrapErr, bail, eyre};

macro_rules! greyln {
    ($($msg:expr),*) => {{
        let msg = format!($($msg),*);
        println!("{}", msg.grey())
    }};
}

macro_rules! mintln {
    ($($msg:expr),*) => {{
        let msg = format!($($msg),*);
        println!("{}", msg.mint())
    }};
}

macro_rules! egreyln {
    ($($msg:expr),*) => {{
        let msg = format!($($msg),*);
        eprintln!("{}", msg.grey())
    }};
}

pub(crate) use {egreyln, greyln};

pub mod deployer;

pub use deployer::STYLUS_DEPLOYER_ADDRESS;

sol! {
    #[sol(rpc)]
    interface ArbWasm {
        function activateProgram(address program)
            external
            payable
            returns (uint16 version, uint256 dataFee);
    }
}

/// Deploys a stylus contract, activating if needed.
pub async fn deploy(cfg: DeployConfig) -> Result<()> {
    let contract = check::check(&cfg.check_config)
        .await
        .expect("cargo stylus check failed");
    let verbose = cfg.check_config.common_cfg.verbose;
    let use_wasm_file = cfg.check_config.wasm_file.is_some();

    let constructor = if use_wasm_file {
        if let Some(signature) = &cfg.constructor_signature {
            Some(Constructor::parse(signature)?)
        } else {
            None
        }
    } else {
        if cfg.constructor_signature.is_some() {
            bail!("cannot set constructor signature without --wasm-file");
        }
        export_abi::get_constructor_signature()?
    };

    let deployer_args = match constructor {
        Some(constructor) => {
            let args = deployer::parse_constructor_args(&cfg, &constructor, &contract).await?;
            Some(args)
        }
        None => None,
    };

    // Check constructor flags for contracts without constructor
    if deployer_args.is_none() && !cfg.constructor_args.is_empty() {
        bail!("constructor arguments set but constructor was not found");
    }

    let provider = ProviderBuilder::new()
        .connect(&cfg.check_config.common_cfg.endpoint)
        .await?;
    let chain_id = provider.get_chain_id().await?;
    let wallet = cfg.auth.alloy_wallet(chain_id)?;
    let from_address = wallet.default_signer().address();
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&cfg.check_config.common_cfg.endpoint)
        .await?;

    if verbose {
        greyln!("sender address: {}", from_address.debug_lavender());
    }

    let data_fee = contract.suggest_fee() + cfg.constructor_value;

    if let ContractCheck::Ready { .. } = &contract {
        // check balance early
        let balance = provider
            .get_balance(from_address)
            .await
            .expect("failed to get balance");

        if balance < data_fee && !cfg.estimate_gas {
            bail!(
                "not enough funds in account {} to pay for data fee\n\
                 balance {} < {}\n\
                 please see the Quickstart guide for funding new accounts:\n{}",
                from_address.red(),
                balance.red(),
                format!("{data_fee} wei").red(),
                "https://docs.arbitrum.io/stylus/stylus-quickstart".yellow(),
            );
        }
    }

    if let Some(deployer_args) = deployer_args {
        return deployer::deploy(&cfg, deployer_args, from_address, &provider).await;
    }

    let contract_addr = cfg
        .deploy_contract(contract.code(), from_address, &provider)
        .await?;

    if cfg.estimate_gas {
        return Ok(());
    }

    match contract {
        ContractCheck::Ready { .. } => {
            if cfg.no_activate {
                mintln!(
                    r#"NOTE: You must activate the stylus contract before calling it. To do so, we recommend running:
cargo stylus activate --address {}"#,
                    hex::encode(contract_addr)
                );
            } else {
                cfg.activate(from_address, contract_addr, data_fee, &provider)
                    .await?
            }
        }
        ContractCheck::Active { .. } => greyln!("wasm already activated!"),
    }
    print_cache_notice(contract_addr);
    Ok(())
}

impl DeployConfig {
    async fn deploy_contract(
        &self,
        code: &[u8],
        sender: Address,
        provider: &impl Provider,
    ) -> Result<Address> {
        let init_code = contract_deployment_calldata(code);

        let tx = TransactionRequest::default()
            .with_from(sender)
            .with_deploy_code(init_code);

        let verbose = self.check_config.common_cfg.verbose;
        let gas = provider.estimate_gas(tx.clone()).await?;

        let gas_price = provider.get_gas_price().await?;

        if self.check_config.common_cfg.verbose || self.estimate_gas {
            print_gas_estimate("deployment", gas, gas_price).await?;
        }
        if self.estimate_gas {
            let nonce = provider.get_transaction_count(sender).await?;
            return Ok(sender.create(nonce));
        }

        let fee_per_gas = calculate_fee_per_gas(&self.check_config.common_cfg, gas_price)?;

        let receipt = run_tx(
            "deploy",
            tx,
            Some(gas),
            fee_per_gas,
            provider,
            self.check_config.common_cfg.verbose,
        )
        .await?;
        let contract = receipt.contract_address.ok_or(eyre!("missing address"))?;
        let address = contract.debug_lavender();

        if verbose {
            let gas = format_gas(receipt.gas_used);
            greyln!(
                "deployed code at address: {address} {} {gas}",
                "with".grey()
            );
        } else {
            greyln!("deployed code at address: {address}");
        }
        let tx_hash = receipt.transaction_hash.debug_lavender();
        greyln!("deployment tx hash: {tx_hash}");
        Ok(contract)
    }

    async fn activate(
        &self,
        sender: Address,
        contract_addr: Address,
        data_fee: U256,
        client: &impl Provider,
    ) -> Result<()> {
        let verbose = self.check_config.common_cfg.verbose;

        let data = ArbWasm::activateProgramCall {
            program: contract_addr,
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_from(sender)
            .with_to(ARB_WASM_ADDRESS)
            .with_value(data_fee)
            .with_input(data);

        let gas = client
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| eyre!("did not estimate correctly: {e}"))?;

        let gas_price = client.get_gas_price().await?;

        if self.check_config.common_cfg.verbose || self.estimate_gas {
            greyln!("activation gas estimate: {}", format_gas(gas));
        }

        let fee_per_gas = calculate_fee_per_gas(&self.check_config.common_cfg, gas_price)?;

        let receipt = run_tx(
            "activate",
            tx,
            Some(gas),
            fee_per_gas,
            client,
            self.check_config.common_cfg.verbose,
        )
        .await?;

        if verbose {
            let gas = format_gas(receipt.gas_used);
            greyln!("activated with {gas}");
        }
        greyln!(
            "contract activated and ready onchain with tx hash: {}",
            receipt.transaction_hash.debug_lavender()
        );
        Ok(())
    }
}

pub async fn print_gas_estimate(name: &str, gas: u64, gas_price: u128) -> Result<()> {
    greyln!("estimates");
    greyln!("{} tx gas: {}", name, gas.debug_lavender());
    greyln!(
        "gas price: {} gwei",
        format_units(gas_price, "gwei")?.debug_lavender()
    );
    let total_cost = gas_price.checked_mul(gas.into()).unwrap_or_default();
    let eth_estimate = format_units(total_cost, "ether")?;
    greyln!(
        "{} tx total cost: {} ETH",
        name,
        eth_estimate.debug_lavender()
    );
    Ok(())
}

pub fn print_cache_notice(contract_addr: Address) {
    let contract_addr = hex::encode(contract_addr);
    println!();
    mintln!(
        r#"NOTE: We recommend running cargo stylus cache bid {contract_addr} 0 to cache your activated contract in ArbOS.
Cached contracts benefit from cheaper calls. To read more about the Stylus contract cache, see
https://docs.arbitrum.io/stylus/how-tos/caching-contracts"#
    );
}

pub async fn run_tx(
    name: &str,
    tx: TransactionRequest,
    gas: Option<u64>,
    max_fee_per_gas_wei: u128,
    provider: &impl Provider,
    verbose: bool,
) -> Result<TransactionReceipt> {
    let mut tx = tx;
    if let Some(gas) = gas {
        tx.gas = Some(gas);
    }

    tx.max_fee_per_gas = Some(max_fee_per_gas_wei);
    tx.max_priority_fee_per_gas = Some(0);

    let tx = provider.send_transaction(tx).await?;
    let tx_hash = *tx.tx_hash();
    if verbose {
        greyln!("sent {name} tx: {}", tx_hash.debug_lavender());
    }
    let receipt = tx.get_receipt().await.wrap_err("tx failed to complete")?;
    if !receipt.status() {
        bail!("{name} tx reverted {}", tx_hash.debug_red());
    }
    Ok(receipt)
}

/// Prepares an EVM bytecode prelude for contract creation.
pub fn contract_deployment_calldata(code: &[u8]) -> Vec<u8> {
    let code_len: [u8; 32] = U256::from(code.len()).to_be_bytes();
    let mut deploy: Vec<u8> = vec![];
    deploy.push(0x7f); // PUSH32
    deploy.extend(code_len);
    deploy.push(0x80); // DUP1
    deploy.push(0x60); // PUSH1
    deploy.push(42 + 1); // prelude + version
    deploy.push(0x60); // PUSH1
    deploy.push(0x00);
    deploy.push(0x39); // CODECOPY
    deploy.push(0x60); // PUSH1
    deploy.push(0x00);
    deploy.push(0xf3); // RETURN
    deploy.push(0x00); // version
    deploy.extend(code);
    deploy
}

pub fn extract_contract_evm_deployment_prelude(calldata: &[u8]) -> Vec<u8> {
    // The length of the prelude, version part is 42 + 1 as per the code
    let metadata_length = 42 + 1;
    // Extract and return the metadata part
    calldata[0..metadata_length].to_vec()
}

pub fn extract_compressed_wasm(calldata: &[u8]) -> Vec<u8> {
    // The length of the prelude, version part is 42 + 1 as per the code
    let metadata_length = 42 + 1;
    // Extract and return the metadata part
    calldata[metadata_length..].to_vec()
}

pub fn format_gas(gas: u64) -> String {
    let text = format!("{gas} gas");
    if gas <= 3_000_000 {
        text.mint()
    } else if gas <= 7_000_000 {
        text.yellow()
    } else {
        text.pink()
    }
}

pub fn calculate_fee_per_gas<T: GasFeeConfig>(config: &T, gas_price: u128) -> Result<u128> {
    let fee_per_gas = match config.get_max_fee_per_gas_wei()? {
        Some(wei) => wei,
        None => gas_price,
    };
    Ok(fee_per_gas)
}

#[derive(Args, Clone, Debug)]
pub struct CheckConfig {
    #[command(flatten)]
    common_cfg: CommonConfig,
    #[command(flatten)]
    data_fee: DataFeeOpts,
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    wasm_file: Option<PathBuf>,
    /// Where to deploy and activate the contract (defaults to a random address).
    #[arg(long)]
    contract_address: Option<Address>,
}

#[derive(Clone, Debug, Args)]
struct DataFeeOpts {
    /// Percent to bump the estimated activation data fee by.
    #[arg(long, default_value = "20")]
    data_fee_bump_percent: u64,
}
#[derive(Args, Clone, Debug)]
struct DeployConfig {
    #[command(flatten)]
    check_config: CheckConfig,
    /// Wallet source to use.
    #[command(flatten)]
    auth: AuthOpts,
    /// Only perform gas estimation.
    #[arg(long)]
    estimate_gas: bool,
    /// If specified, will not run the command in a reproducible docker container. Useful for local
    /// builds, but at the risk of not having a reproducible contract for verification purposes.
    #[arg(long)]
    no_verify: bool,
    /// Cargo stylus version when deploying reproducibly to downloads the corresponding cargo-stylus-base Docker image.
    /// If not set, uses the default version of the local cargo stylus binary.
    #[arg(long)]
    cargo_stylus_version: Option<String>,
    /// If set, do not activate the program after deploying it
    #[arg(long)]
    no_activate: bool,
    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    deployer_address: Address,
    /// The salt passed to the stylus deployer.
    #[arg(long, default_value_t = B256::ZERO)]
    deployer_salt: B256,
    /// The constructor arguments.
    #[arg(
        long,
        num_args(0..),
        value_name = "ARGS",
        allow_hyphen_values = true,
    )]
    constructor_args: Vec<String>,
    /// The amount of Ether sent to the contract through the constructor.
    #[arg(long, value_parser = parse_ether, default_value = "0")]
    constructor_value: U256,
    /// The constructor signature when using the --wasm-file flag.
    #[arg(long)]
    constructor_signature: Option<String>,
}

pub trait GasFeeConfig {
    fn get_max_fee_per_gas_wei(&self) -> Result<Option<u128>>;
    fn get_fee_str(&self) -> &Option<String>;
}

#[derive(Args, Clone, Debug)]
struct CommonConfig {
    /// Arbitrum RPC endpoint.
    #[arg(short, long, default_value = DEFAULT_ENDPOINT)]
    endpoint: String,
    /// Whether to print debug info.
    #[arg(long)]
    verbose: bool,
    /// The path to source files to include in the project hash, which
    /// is included in the contract deployment init code transaction
    /// to be used for verification of deployment integrity.
    /// If not provided, all .rs files and Cargo.toml and Cargo.lock files
    /// in project's directory tree are included.
    #[arg(long)]
    source_files_for_project_hash: Vec<String>,
    #[arg(long)]
    /// Optional max fee per gas in gwei units.
    max_fee_per_gas_gwei: Option<String>,
    /// Specifies the features to use when building the Stylus binary.
    #[arg(long)]
    features: Option<String>,
}

impl GasFeeConfig for CommonConfig {
    fn get_fee_str(&self) -> &Option<String> {
        &self.max_fee_per_gas_gwei
    }

    fn get_max_fee_per_gas_wei(&self) -> Result<Option<u128>> {
        match self.get_fee_str() {
            Some(fee_str) => Ok(Some(convert_gwei_to_wei(fee_str)?)),
            None => Ok(None),
        }
    }
}

fn convert_gwei_to_wei(fee_str: &str) -> Result<u128> {
    let gwei = match fee_str.parse::<f64>() {
        Ok(fee) if fee >= 0.0 => fee,
        Ok(_) => bail!("Max fee per gas must be non-negative"),
        Err(_) => bail!("Invalid max fee per gas value: {}", fee_str),
    };

    if !gwei.is_finite() {
        bail!("Invalid gwei value: must be finite");
    }

    let wei = gwei * 1e9;
    if !wei.is_finite() {
        bail!("Overflow occurred in floating point multiplication of --max-fee-per-gas-gwei converting");
    }

    if wei < 0.0 || wei >= u128::MAX as f64 {
        bail!("Result outside valid range for wei");
    }

    Ok(wei as u128)
}

#[derive(Clone, Debug, Args)]
#[clap(group(ArgGroup::new("key").required(true).args(&["private_key_path", "private_key", "keystore_path"])))]
struct AuthOpts {
    /// File path to a text file containing a hex-encoded private key.
    #[arg(long)]
    private_key_path: Option<PathBuf>,
    /// Private key as a hex string. Warning: this exposes your key to shell history.
    #[arg(long)]
    private_key: Option<String>,
    /// Path to an Ethereum wallet keystore file (e.g. clef).
    #[arg(long)]
    keystore_path: Option<String>,
    /// Keystore password file.
    #[arg(long)]
    keystore_password_path: Option<PathBuf>,
}

/// Loads a wallet for signing transactions.
impl AuthOpts {
    pub fn alloy_wallet(&self, chain_id: u64) -> Result<EthereumWallet> {
        if let Some(key) = &self.private_key {
            if key.is_empty() {
                return Err(eyre!("empty private key"));
            }
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        if let Some(file) = &self.private_key_path {
            let key = fs::read_to_string(file).wrap_err("could not open private key file")?;
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        let keystore = self.keystore_path.as_ref().ok_or(eyre!("no keystore"))?;
        let password = self
            .keystore_password_path
            .as_ref()
            .map(fs::read_to_string)
            .unwrap_or(Ok("".into()))?;

        let signer =
            LocalSigner::decrypt_keystore(keystore, password)?.with_chain_id(Some(chain_id));
        Ok(EthereumWallet::new(signer))
    }
}
