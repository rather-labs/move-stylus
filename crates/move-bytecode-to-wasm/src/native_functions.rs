//! This module contains the implementation for the native functions.
//!
//! Native functions in Move are functions directly implemented inside the Move VM. To emulate that
//! mechanism, we direcly implement them in WASM and limk them into the file.
mod event;
mod object;
mod transaction;
mod transfer;
mod types;

use walrus::{FunctionId, Module};

use crate::{
    CompilationContext, hostio, runtime::RuntimeFunction,
    translation::intermediate_types::IntermediateType,
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

    // Transfer functions
    pub const NATIVE_TRANSFER_OBJECT: &str = "transfer";
    pub const NATIVE_SHARE_OBJECT: &str = "share_object";
    pub const NATIVE_FREEZE_OBJECT: &str = "freeze_object";

    // Types functions
    pub const NATIVE_IS_ONE_TIME_WITNESS: &str = "is_one_time_witness";

    // Storage
    #[cfg(debug_assertions)]
    pub const SAVE_IN_SLOT: &str = "save_in_slot";
    #[cfg(debug_assertions)]
    pub const READ_SLOT: &str = "read_slot";

    // Event functions
    const NATIVE_EMIT: &str = "emit";

    // Object functions
    pub const NATIVE_DELETE_OBJECT: &str = "delete";

    // Host functions
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

    /// Links the function into the module and returns its id. The function generated depends on
    /// the types passed in the `generics` parameter.
    ///
    /// The idempotency of this function depends on the generator functions. This is designed this
    /// way to avoid errors when calculating the function name based on the types.
    pub fn get_generic(
        name: &str,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        generics: &[IntermediateType],
    ) -> FunctionId {
        match name {
            Self::NATIVE_SHARE_OBJECT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                transfer::add_share_object_fn(module, compilation_ctx, &generics[0])
            }
            Self::NATIVE_TRANSFER_OBJECT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                transfer::add_transfer_object_fn(module, compilation_ctx, &generics[0])
            }
            Self::NATIVE_FREEZE_OBJECT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                transfer::add_freeze_object_fn(module, compilation_ctx, &generics[0])
            }
            Self::NATIVE_DELETE_OBJECT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::DeleteFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&generics[0]],
                )
            }
            Self::NATIVE_EMIT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                event::add_emit_log_fn(module, compilation_ctx, &generics[0])
            }
            // This native function is only available in debug mode to help with testing. It should
            // not be compiled in release mode.
            #[cfg(debug_assertions)]
            Self::SAVE_IN_SLOT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::EncodeAndSaveInStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&generics[0]],
                )
            }
            // This native function is only available in debug mode to help with testing. It should
            // not be compiled in release mode.
            #[cfg(debug_assertions)]
            Self::READ_SLOT => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::DecodeAndReadFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&generics[0]],
                )
            }

            Self::NATIVE_IS_ONE_TIME_WITNESS => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {name} expected 1 type parameter, found {}",
                    generics.len(),
                );

                types::add_is_one_time_witness_fn(module, compilation_ctx, &generics[0])
            }
            _ => panic!("generic native function {name} not supported yet"),
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
