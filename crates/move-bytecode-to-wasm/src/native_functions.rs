//! This module contains the implementation for the native functions.
//!
//! Native functions in Move are functions directly implemented inside the Move VM. To emulate that
//! mechanism, we direcly implement them in WASM and limk them into the file.
mod object;
mod storage;
mod transaction;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    panic::panic_any,
};

use walrus::{FunctionId, Module};

use crate::{
    CompilationContext, hostio,
    translation::intermediate_types::{IType, IntermediateType},
};

pub struct NativeFunction;

impl NativeFunction {
    const NATIVE_SENDER: &str = "native_sender";
    const NATIVE_MSG_VALUE: &str = "native_msg_value";
    const NATIVE_BLOCK_NUMBER: &str = "native_block_number";
    const NATIVE_BLOCK_BASEFEE: &str = "native_block_basefee";
    const NATIVE_BLOCK_GAS_LIMIT: &str = "native_block_gas_limit";
    const NATIVE_BLOCK_TIMESTAMP: &str = "native_block_timestamp";
    const NATIVE_CHAIN_ID: &str = "native_chain_id";
    const NATIVE_GAS_PRICE: &str = "native_gas_price";
    const NATIVE_FRESH_ID: &str = "fresh_id";
    const NATIVE_STORAGE_SAVE: &str = "save_in_slot";

    const HOST_BLOCK_NUMBER: &str = "block_number";
    const HOST_BLOCK_GAS_LIMIT: &str = "block_gas_limit";
    const HOST_BLOCK_TIMESTAMP: &str = "block_timestamp";
    const HOST_CHAIN_ID: &str = "chainid";

    /// Links the function into the module and returns its id. If the function is already present
    /// it just returns the id.
    ///
    /// This funciton is idempotent.
    pub fn get(name: &str, module: &mut Module, compilaton_ctx: &CompilationContext) -> FunctionId {
        // Some functions are implemented by host functions directly. For those, we just import and
        // use them without wrapping them.
        if let Some(host_fn_name) = Self::host_fn_name(name) {
            if let Ok(function_id) = module.imports.get_func("vm_hooks", host_fn_name) {
                return function_id;
            } else {
                match host_fn_name {
                    Self::HOST_BLOCK_NUMBER => {
                        let (function_id, _) = hostio::host_functions::block_number(module);
                        return function_id;
                    }
                    Self::HOST_BLOCK_GAS_LIMIT => {
                        let (function_id, _) = hostio::host_functions::block_gas_limit(module);
                        return function_id;
                    }
                    Self::HOST_BLOCK_TIMESTAMP => {
                        let (function_id, _) = hostio::host_functions::block_timestamp(module);
                        return function_id;
                    }
                    Self::HOST_CHAIN_ID => {
                        let (function_id, _) = hostio::host_functions::chain_id(module);
                        return function_id;
                    }
                    _ => {
                        panic!("host function {host_fn_name} not supported yet");
                    }
                }
            }
        }

        if let Some(function) = module.funcs.by_name(name) {
            function
        } else {
            match name {
                Self::NATIVE_SENDER => transaction::add_native_sender_fn(module, compilaton_ctx),
                Self::NATIVE_MSG_VALUE => {
                    transaction::add_native_msg_value_fn(module, compilaton_ctx)
                }
                Self::NATIVE_BLOCK_BASEFEE => {
                    transaction::add_native_block_basefee_fn(module, compilaton_ctx)
                }
                Self::NATIVE_GAS_PRICE => {
                    transaction::add_native_tx_gas_price_fn(module, compilaton_ctx)
                }
                Self::NATIVE_FRESH_ID => object::add_native_fresh_id_fn(module, compilaton_ctx),

                _ => panic!("native function {name} not supported yet"),
            }
        }
    }

    pub fn get_generic(
        name: &str,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        generics: &[IntermediateType],
    ) -> FunctionId {
        // Thid hash will uniquely identify this native fn
        let mut hasher = DefaultHasher::new();
        let function_hash = generics.iter().for_each(|t| t.hash(&mut hasher));
        let function_name = format!("{name}_{:x}", hasher.finish());

        if let Some(function) = module.funcs.by_name(&function_name) {
            function
        } else {
            match name {
                Self::NATIVE_STORAGE_SAVE => {
                    assert_eq!(
                        1,
                        generics.len(),
                        "there was an error linking {function_name} expected 1 type parameter, found {}",
                        generics.len(),
                    );

                    let struct_ = match generics.get(0) {
                        Some(IntermediateType::IStruct { module_id, index }) => compilation_ctx
                            .get_user_data_type_by_index(module_id, *index)
                            .unwrap(),
                        Some(_) => todo!(),
                        None => todo!(),
                    };
                    storage::add_storage_save_fn(function_name, module, compilation_ctx, struct_)
                }
                _ => panic!("generic native function {name} not supported yet"),
            }
        }
    }

    /// Maps the native function name to the host function name.
    fn host_fn_name(name: &str) -> Option<&'static str> {
        match name {
            Self::NATIVE_BLOCK_NUMBER => Some(Self::HOST_BLOCK_NUMBER),
            Self::NATIVE_BLOCK_GAS_LIMIT => Some(Self::HOST_BLOCK_GAS_LIMIT),
            Self::NATIVE_BLOCK_TIMESTAMP => Some(Self::HOST_BLOCK_TIMESTAMP),
            Self::NATIVE_CHAIN_ID => Some(Self::HOST_CHAIN_ID),
            _ => None,
        }
    }
}
