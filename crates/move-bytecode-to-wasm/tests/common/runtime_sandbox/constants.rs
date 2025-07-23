#![allow(dead_code)]
//! Contants for the sandbox

use alloy_primitives::U256;

pub const SIGNER_ADDRESS: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7];
pub const CONTRACT_ADDRESS: &str = "0xcafe000000000000000000000000000000007357";

pub const MSG_SENDER_ADDRESS: [u8; 20] =
    [7, 3, 5, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7];

pub const MSG_VALUE: [u8; 32] = U256::MAX.to_le_bytes();
