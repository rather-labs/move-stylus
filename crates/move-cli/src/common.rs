// Copyright 2023-2024, Offchain Labs, Inc.
// Modified by Rather Labs, Inc. in 2026.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::PathBuf};

use alloy::signers::Signer;
use alloy::{network::EthereumWallet, primitives::FixedBytes, signers::local::PrivateKeySigner};
use anyhow::{Context, Result, anyhow, bail};
use clap::{ArgGroup, Args};

use crate::{constants::DEFAULT_ENDPOINT, deploy::util::text::decode0x};

#[derive(Args, Clone, Debug)]
pub struct CommonConfig {
    /// Arbitrum RPC endpoint.
    #[arg(short, long, default_value = DEFAULT_ENDPOINT)]
    pub endpoint: String,

    /// Whether to print debug info.
    #[arg(long)]
    pub verbose: bool,

    /// Optional max fee per gas in gwei units.
    #[arg(long)]
    pub max_fee_per_gas_gwei: Option<String>,
}

pub trait GasFeeConfig {
    fn get_max_fee_per_gas_wei(&self) -> Result<Option<u128>>;
    fn get_fee_str(&self) -> &Option<String>;
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
        bail!(
            "Overflow occurred in floating point multiplication of --max-fee-per-gas-gwei converting"
        );
    }

    if wei < 0.0 || wei >= u128::MAX as f64 {
        bail!("Result outside valid range for wei");
    }

    Ok(wei as u128)
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

#[derive(Clone, Debug, Args)]
#[clap(group(ArgGroup::new("key").required(true).args(&["private_key_path", "private_key"])))]
pub struct AuthOpts {
    /// File path to a text file containing a hex-encoded private key.
    #[arg(long)]
    pub private_key_path: Option<PathBuf>,
    /// Private key as a hex string. Warning: this exposes your key to shell history.
    #[arg(long)]
    pub private_key: Option<String>,
}

/// Loads a wallet for signing transactions.
impl AuthOpts {
    pub fn alloy_wallet(&self, chain_id: u64) -> Result<EthereumWallet> {
        if let Some(key) = &self.private_key {
            if key.is_empty() {
                return Err(anyhow!("empty private key"));
            }
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        if let Some(file) = &self.private_key_path {
            let key = fs::read_to_string(file).context("could not open private key file")?;
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        Err(anyhow!("no private key provided"))
    }
}
