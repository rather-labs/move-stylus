#![allow(dead_code)]
//! Contants for the sandbox

use alloy_primitives::U256;

pub const SIGNER_ADDRESS: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7];
pub const CONTRACT_ADDRESS: &str = "0xcafe000000000000000000000000000000007357";

pub const MSG_SENDER_ADDRESS: [u8; 20] =
    [7, 3, 5, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7];

pub const MSG_VALUE: U256 = U256::MAX;

pub const BLOCK_BASEFEE: U256 = U256::from_le_bytes([
    1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

pub const BLOCK_NUMBER: u64 = 3141592;
pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;
