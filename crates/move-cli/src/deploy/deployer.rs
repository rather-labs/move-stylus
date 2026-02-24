// Copyright 2023-2024, Offchain Labs, Inc.
// Modified by Rather Labs, Inc. in 2026.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

use crate::deploy::{
    DeployConfig, calculate_fee_per_gas,
    util::color::{Color, DebugColor},
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256, address},
    providers::Provider,
    rpc::types::{TransactionReceipt, TransactionRequest},
    sol,
    sol_types::SolEvent,
};
use eyre::{Context, Result, bail, eyre};

pub const STYLUS_DEPLOYER_ADDRESS: Address = address!("cEcba2F1DC234f70Dd89F2041029807F8D03A990");

sol! {
    #[sol(rpc)]
    interface StylusDeployer {
        event ContractDeployed(address deployedContract);

        function deploy(
            bytes calldata bytecode,
            bytes calldata initData,
            uint256 initValue,
            bytes32 salt
        ) public payable returns (address);
    }

    function stylus_constructor();
}

pub struct DeployerArgs {
    /// Factory address
    address: Address,
    /// Value to be sent in the tx
    tx_value: U256,
    /// Calldata to be sent in the tx
    tx_calldata: Vec<u8>,
}

/// Deploys, activates, and initializes the contract using the Stylus deployer.
pub async fn deploy(
    cfg: &DeployConfig,
    deployer: DeployerArgs,
    sender: Address,
    provider: &impl Provider,
) -> Result<()> {
    if cfg.check_config.common_cfg.verbose {
        greyln!(
            "deploying contract using deployer at address: {}",
            deployer.address.debug_lavender()
        );
    }
    let tx = TransactionRequest::default()
        .with_to(deployer.address)
        .with_from(sender)
        .with_value(deployer.tx_value)
        .with_input(deployer.tx_calldata);

    let gas = provider
        .estimate_gas(tx.clone())
        .await
        .wrap_err("deployment failed during gas estimation")?;

    let gas_price = provider.get_gas_price().await?;

    if cfg.check_config.common_cfg.verbose || cfg.estimate_gas {
        super::print_gas_estimate("deployer deploy, activate, and init", gas, gas_price).await?;
    }
    if cfg.estimate_gas {
        return Ok(());
    }

    let fee_per_gas = calculate_fee_per_gas(&cfg.check_config.common_cfg, gas_price)?;

    let receipt = super::run_tx(
        "deploy_activate_init",
        tx,
        Some(gas),
        fee_per_gas,
        provider,
        cfg.check_config.common_cfg.verbose,
    )
    .await?;
    let contract = get_address_from_receipt(&receipt)?;
    let address = contract.debug_lavender();

    if cfg.check_config.common_cfg.verbose {
        let gas = super::format_gas(receipt.gas_used);
        greyln!(
            "deployed code at address: {address} {} {gas}",
            "with".grey()
        );
    } else {
        greyln!("deployed code at address: {address}");
    }
    let tx_hash = receipt.transaction_hash.debug_lavender();
    greyln!("deployment tx hash: {tx_hash}");
    Ok(())
}

/// Gets the Stylus-contract address that was deployed using the deployer.
fn get_address_from_receipt(receipt: &TransactionReceipt) -> Result<Address> {
    let receipt = receipt.clone().into_inner();
    for log in receipt.logs().iter() {
        if let Some(topic) = log.topics().first() {
            if topic.0 == StylusDeployer::ContractDeployed::SIGNATURE_HASH {
                if log.data().data.len() != 32 {
                    bail!("address missing from ContractDeployed log");
                }
                return Ok(Address::from_slice(&log.data().data[12..32]));
            }
        }
    }
    Err(eyre!("contract address not found in receipt"))
}
