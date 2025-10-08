//! This module contains the implementation for the native functions.
//!
//! Native functions in Move are functions directly implemented inside the Move VM. To emulate that
//! mechanism, we direcly implement them in WASM and limk them into the file.
mod dynamic_field;
mod event;
pub mod object;
mod tests;
mod transaction;
pub mod transfer;
mod types;

use walrus::{FunctionId, Module};

use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{
            SF_MODULE_NAME_DYNAMIC_FIELD, SF_MODULE_NAME_EVENT, SF_MODULE_NAME_OBJECT,
            SF_MODULE_NAME_TRANSFER, SF_MODULE_NAME_TX_CONTEXT, SF_MODULE_NAME_TYPES,
            STYLUS_FRAMEWORK_ADDRESS,
        },
    },
    hostio,
    runtime::RuntimeFunction,
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
    // This is for objects with UID as id.
    pub const NATIVE_DELETE_OBJECT: &str = "delete";
    // This is for objects with NamedId as id.
    pub const NATIVE_REMOVE_OBJECT: &str = "remove";
    pub const NATIVE_COMPUTE_NAMED_ID: &str = "compute_named_id";
    pub const NATIVE_AS_UID: &str = "as_uid";
    pub const NATIVE_AS_UID_MUT: &str = "as_uid_mut";

    // Dynamic fields
    #[cfg(debug_assertions)]
    pub const NATIVE_GET_LAST_MEMORY_POSITION: &str = "get_last_memory_position";
    const NATIVE_HASH_TYPE_AND_KEY: &str = "hash_type_and_key";
    const NATIVE_ADD_CHILD_OBJECT: &str = "add_child_object";
    const NATIVE_BORROW_CHILD_OBJECT: &str = "borrow_child_object";
    const NATIVE_BORROW_CHILD_OBJECT_MUT: &str = "borrow_child_object_mut";
    const NATIVE_REMOVE_CHILD_OBJECT: &str = "remove_child_object";
    const NATIVE_HAS_CHILD_OBJECT: &str = "has_child_object";

    // Host functions
    const HOST_BLOCK_NUMBER: &str = "block_number";
    const HOST_BLOCK_GAS_LIMIT: &str = "block_gas_limit";
    const HOST_BLOCK_TIMESTAMP: &str = "block_timestamp";
    const HOST_CHAIN_ID: &str = "chainid";

    /// Links the function into the module and returns its id. If the function is already present
    /// it just returns the id.
    ///
    /// This function is idempotent.
    pub fn get(
        name: &str,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_id: &ModuleId,
    ) -> FunctionId {
        let ModuleId {
            address,
            module_name,
        } = module_id;
        // Some functions are implemented by host functions directly. For those, we just import and
        // use them without wrapping them.
        if let Some(host_fn_name) = Self::host_fn_name(name) {
            if let Ok(function_id) = module.imports.get_func("vm_hooks", host_fn_name) {
                return function_id;
            } else {
                match (host_fn_name, *address, module_name.as_str()) {
                    (
                        Self::HOST_BLOCK_NUMBER,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_number(module);
                        return function_id;
                    }
                    (
                        Self::HOST_BLOCK_GAS_LIMIT,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_gas_limit(module);
                        return function_id;
                    }
                    (
                        Self::HOST_BLOCK_TIMESTAMP,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_timestamp(module);
                        return function_id;
                    }
                    (Self::HOST_CHAIN_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
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
            match (name, *address, module_name.as_str()) {
                (Self::NATIVE_SENDER, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_sender_fn(module, compilation_ctx)
                }
                (Self::NATIVE_MSG_VALUE, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_msg_value_fn(module, compilation_ctx)
                }
                (
                    Self::NATIVE_BLOCK_BASEFEE,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_TX_CONTEXT,
                ) => transaction::add_native_block_basefee_fn(module, compilation_ctx),
                (Self::NATIVE_GAS_PRICE, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_tx_gas_price_fn(module, compilation_ctx)
                }
                (Self::NATIVE_FRESH_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    object::add_native_fresh_id_fn(module, compilation_ctx)
                }
                (
                    Self::NATIVE_HAS_CHILD_OBJECT,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_DYNAMIC_FIELD,
                ) => dynamic_field::add_has_child_object_fn(module, compilation_ctx),
                #[cfg(debug_assertions)]
                (Self::NATIVE_GET_LAST_MEMORY_POSITION, _, _) => {
                    tests::add_get_last_memory_position_fn(module, compilation_ctx)
                }
                _ => panic!("native function {module_id}::{name} not supported yet"),
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
        module_id: &ModuleId,
        generics: &[IntermediateType],
    ) -> FunctionId {
        let ModuleId {
            address,
            module_name,
        } = module_id;

        match (name, *address, module_name.as_str()) {
            //
            // Transfer
            //
            (Self::NATIVE_SHARE_OBJECT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TRANSFER) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                transfer::add_share_object_fn(module, compilation_ctx, &generics[0])
            }
            (Self::NATIVE_TRANSFER_OBJECT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TRANSFER) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                transfer::add_transfer_object_fn(module, compilation_ctx, &generics[0])
            }
            (Self::NATIVE_FREEZE_OBJECT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TRANSFER) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                transfer::add_freeze_object_fn(module, compilation_ctx, &generics[0])
            }
            (Self::NATIVE_DELETE_OBJECT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TRANSFER)
            | (Self::NATIVE_REMOVE_OBJECT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TRANSFER) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::DeleteFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&generics[0]],
                )
            }
            //
            // Event
            //
            (Self::NATIVE_EMIT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_EVENT) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                event::add_emit_log_fn(module, compilation_ctx, &generics[0])
            }

            //
            // Types
            //
            (Self::NATIVE_IS_ONE_TIME_WITNESS, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TYPES) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                types::add_is_one_time_witness_fn(module, compilation_ctx, &generics[0])
            }

            //
            // Object
            //
            (Self::NATIVE_COMPUTE_NAMED_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                object::add_compute_named_id_fn(module, compilation_ctx, &generics[0])
            }
            (Self::NATIVE_AS_UID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT)
            | (Self::NATIVE_AS_UID_MUT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT) => {
                // Generics are not used in this function because it just converts &NamedId to &UID,
                // which, under the hood they have the same structure. Generic type is not used in
                // the function, just to detect that the function was called correctly
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                object::add_as_uid_fn(module, compilation_ctx)
            }

            //
            // Dynamic field
            //
            (
                Self::NATIVE_HASH_TYPE_AND_KEY,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                dynamic_field::add_hash_type_and_key_fn(module, compilation_ctx, &generics[0])
            }
            (
                Self::NATIVE_ADD_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                dynamic_field::add_child_object_fn(module, compilation_ctx, &generics[0])
            }
            (
                Self::NATIVE_BORROW_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
            )
            | (
                Self::NATIVE_BORROW_CHILD_OBJECT_MUT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                dynamic_field::add_borrow_object_fn(module, compilation_ctx, &generics[0])
            }
            (
                Self::NATIVE_REMOVE_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                dynamic_field::add_remove_child_object_fn(module, compilation_ctx, &generics[0])
            }

            // This native function is only available in debug mode to help with testing. It should
            // not be compiled in release mode.
            #[cfg(debug_assertions)]
            (Self::SAVE_IN_SLOT, _, _) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
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
            (Self::READ_SLOT, _, _) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);

                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::ReadAndDecodeFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&generics[0]],
                )
            }
            #[cfg(debug_assertions)]
            (Self::NATIVE_HASH_TYPE_AND_KEY, _, _) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id);
                dynamic_field::add_hash_type_and_key_fn(module, compilation_ctx, &generics[0])
            }

            _ => panic!("generic native function {module_id}::{name} not supported yet"),
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

    fn assert_generics_length(len: usize, expected: usize, name: &str, module_id: &ModuleId) {
        assert_eq!(
            expected, len,
            "there was an error linking {module_id}::{name} expected {expected} type parameter(s), found {len}"
        );
    }
}
