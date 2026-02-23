// Copyright 2023-2024, Offchain Labs, Inc.
// Modified by Rather Labs, Inc. in 2026.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

pub mod check;
pub mod project;
pub mod util;

use std::{path::PathBuf};

use crate::common::{AuthOpts, CommonConfig, GasFeeConfig};
use crate::constants::{ARB_WASM_ADDRESS};
use crate::deploy::{
    check::ContractCheck,
    util::{
        color::{Color, DebugColor},
    },
};
use alloy::{
    network::{TransactionBuilder},
    primitives::{
        Address, B256, U256,
        utils::{format_units, parse_ether},
    },
    providers::{Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    sol,
    sol_types::SolCall,
};
use clap::{Args};
use eyre::{Result, WrapErr, bail, eyre};

macro_rules! greyln {
    ($($msg:expr),*) => {{
        let msg = format!($($msg),*);
        println!("{}", $crate::deploy::util::color::Color::grey(&msg))
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
    pub(crate) common_cfg: CommonConfig,
    #[command(flatten)]
    pub data_fee: DataFeeOpts,
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    pub wasm_file: Option<PathBuf>,
    /// Where to deploy and activate the contract (defaults to a random address).
    #[arg(long)]
    pub contract_address: Option<Address>,
}

#[derive(Clone, Debug, Args)]
pub struct DataFeeOpts {
    /// Percent to bump the estimated activation data fee by.
    #[arg(long, default_value = "20")]
    pub data_fee_bump_percent: u64,
}

#[derive(Args, Clone, Debug)]
pub struct DeployConfig {
    #[command(flatten)]
    pub check_config: CheckConfig,
    /// Wallet source to use.
    #[command(flatten)]
    pub auth: AuthOpts,
    /// Only perform gas estimation.
    #[arg(long)]
    pub estimate_gas: bool,
    /// If specified, will not run the command in a reproducible docker container. Useful for local
    /// builds, but at the risk of not having a reproducible contract for verification purposes.
    #[arg(long)]
    pub no_verify: bool,
    /// Cargo stylus version when deploying reproducibly to downloads the corresponding cargo-stylus-base Docker image.
    /// If not set, uses the default version of the local cargo stylus binary.
    #[arg(long)]
    pub cargo_stylus_version: Option<String>,
    /// If set, do not activate the program after deploying it
    #[arg(long)]
    pub no_activate: bool,
    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    pub deployer_address: Address,
    /// The salt passed to the stylus deployer.
    #[arg(long, default_value_t = B256::ZERO)]
    pub deployer_salt: B256,
    /// The constructor arguments.
    #[arg(
        long,
        num_args(0..),
        value_name = "ARGS",
        allow_hyphen_values = true,
    )]
    pub constructor_args: Vec<String>,
    /// The amount of Ether sent to the contract through the constructor.
    #[arg(long, value_parser = parse_ether, default_value = "0")]
    pub constructor_value: U256,
    /// The constructor signature when using the --wasm-file flag.
    #[arg(long)]
    pub constructor_signature: Option<String>,
}
