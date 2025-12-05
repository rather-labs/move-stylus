#![allow(dead_code)]
//! Contants for the sandbox

use alloy_primitives::U256;

pub const SIGNER_ADDRESS: [u8; 20] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xbe, 0xef,
];
pub const CONTRACT_ADDRESS: &str = "0xcafe000000000000000000000000000000007357";

pub const MSG_SENDER_ADDRESS: [u8; 20] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xca, 0xfe,
];

pub const MSG_VALUE: U256 = U256::MAX;

pub const BLOCK_BASEFEE: U256 = U256::from_le_bytes([
    0x12, 0x34, 0x56, 0x78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
]);

pub const GAS_PRICE: U256 = U256::from_le_bytes([
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
]);

pub const BLOCK_NUMBER: u64 = 3141592;
pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;
pub const BLOCK_TIMESTAMP: u64 = 1438338373;
pub const CHAIN_ID: u64 = 42331;
