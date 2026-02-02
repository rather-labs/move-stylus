//! This module contains the implementation for the native functions.
//!
//! Native functions in Move are functions directly implemented inside the Move VM. To emulate that
//! mechanism, we direcly implement them in WASM and limk them into the file.
mod abi_error;
mod account;
mod contract_calls;
mod dynamic_field;
pub mod error;
mod event;
pub mod object;
mod peep;
mod string;
mod tests;
mod transaction;
pub mod transfer;
mod types;
mod unit_test;

use std::hash::Hasher;

use error::NativeFunctionError;
use move_symbol_pool::Symbol;
use walrus::{FunctionId, Module};

use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{
            SF_MODULE_NAME_ACCOUNT, SF_MODULE_NAME_DYNAMIC_FIELD, SF_MODULE_NAME_ERROR,
            SF_MODULE_NAME_EVENT, SF_MODULE_NAME_OBJECT, SF_MODULE_NAME_PEEP,
            SF_MODULE_NAME_SOL_TYPES, SF_MODULE_NAME_TRANSFER, SF_MODULE_NAME_TX_CONTEXT,
            SF_MODULE_NAME_TYPES, SF_MODULE_TEST_SCENARIO, STANDARD_LIB_ADDRESS,
            STDLIB_MODULE_NAME_STRING, STDLIB_MODULE_UNIT_TEST, STYLUS_FRAMEWORK_ADDRESS,
        },
    },
    data::RuntimeErrorData,
    hasher::get_hasher,
    hostio::{self, host_functions::HOST_MODULE_NAME, host_test_functions::TEST_HOST_MODULE_NAME},
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
    const NATIVE_DATA: &str = "native_data";

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

    // Error functions
    const NATIVE_REVERT: &str = "revert";

    // Object functions
    // This is for objects with UID as id.
    pub const NATIVE_DELETE_OBJECT: &str = "delete";
    // This is for objects with NamedId as id.
    pub const NATIVE_REMOVE_OBJECT: &str = "remove";
    pub const NATIVE_COMPUTE_NAMED_ID: &str = "compute_named_id";
    pub const NATIVE_AS_UID: &str = "as_uid";
    pub const NATIVE_AS_UID_MUT: &str = "as_uid_mut";
    // Peep function
    pub const NATIVE_PEEP: &str = "peep";

    // Dynamic fields
    #[cfg(debug_assertions)]
    pub const NATIVE_GET_LAST_MEMORY_POSITION: &str = "get_last_memory_position";
    const NATIVE_HASH_TYPE_AND_KEY: &str = "hash_type_and_key";
    const NATIVE_ADD_CHILD_OBJECT: &str = "add_child_object";
    const NATIVE_BORROW_CHILD_OBJECT: &str = "borrow_child_object";
    const NATIVE_BORROW_CHILD_OBJECT_MUT: &str = "borrow_child_object_mut";
    const NATIVE_REMOVE_CHILD_OBJECT: &str = "remove_child_object";
    const NATIVE_HAS_CHILD_OBJECT: &str = "has_child_object";

    // Account functions
    const NATIVE_ACCOUNT_CODE_SIZE: &str = "account_code_size";
    const NATIVE_ACCOUNT_BALANCE: &str = "account_balance";

    // Bytes functions
    const NATIVE_AS_VEC_BYTES_N: &str = "as_vec_bytes_n";

    // String
    const NATIVE_INTERNAL_CHECK_UTF8: &str = "internal_check_utf8";
    const NATIVE_INTERNAL_IS_CHAR_BOUNDARY: &str = "internal_is_char_boundary";
    const NATIVE_INTERNAL_INDEX_OF: &str = "internal_index_of";
    const NATIVE_INTERNAL_SUB_STRING: &str = "internal_sub_string";

    // Tests
    const NATIVE_POISON: &str = "poison";
    const NATIVE_NEW_TX_CONTEXT: &str = "new_tx_context";
    const NATIVE_DROP_STORAGE_OBJECT: &str = "drop_storage_object";
    const NATIVE_SET_SENDER_ADDRESS: &str = "set_sender_address";
    const NATIVE_SET_SIGNER_ADDRESS: &str = "set_signer_address";
    const NATIVE_SET_BLOCK_BASEFEE: &str = "set_block_basefee";
    const NATIVE_SET_GAS_PRICE: &str = "set_gas_price";
    const NATIVE_SET_BLOCK_NUMBER: &str = "set_block_number";
    const NATIVE_SET_GAS_LIMIT: &str = "set_gas_limit";
    const NATIVE_SET_BLOCK_TIMESTAMP: &str = "set_block_timestamp";
    const NATIVE_SET_CHAIN_ID: &str = "set_chain_id";

    // Host functions
    const HOST_BLOCK_NUMBER: &str = "block_number";
    const HOST_BLOCK_GAS_LIMIT: &str = "block_gas_limit";
    const HOST_BLOCK_TIMESTAMP: &str = "block_timestamp";
    const HOST_CHAIN_ID: &str = "chainid";

    // Host test functions
    const HOST_SET_SENDER_ADDRESS: &str = "set_sender_address";
    const HOST_SET_SIGNER_ADDRESS: &str = "set_signer_address";
    const HOST_SET_BLOCK_BASEFEE: &str = "set_block_basefee";
    const HOST_SET_GAS_PRICE: &str = "set_gas_price";
    const HOST_SET_BLOCK_NUMBER: &str = "set_block_number";
    const HOST_SET_GAS_LIMIT: &str = "set_gas_limit";
    const HOST_SET_BLOCK_TIMESTAMP: &str = "set_block_timestamp";
    const HOST_SET_CHAIN_ID: &str = "set_chain_id";

    /// Links the function into the module and returns its id. If the function is already present
    /// it just returns the id.
    ///
    /// This function is idempotent.
    pub fn get(
        name: &str,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_id: &ModuleId,
    ) -> Result<FunctionId, NativeFunctionError> {
        let ModuleId {
            address,
            module_name,
        } = module_id;
        let native_fn_name = Self::get_function_name(name, module_id);

        // Some functions are implemented by host functions directly. For those, we just import and
        // use them without wrapping them.
        if let Some(host_fn_name) = Self::host_fn_name(name) {
            let host_fn = if let Ok(function_id) =
                module.imports.get_func(HOST_MODULE_NAME, host_fn_name)
            {
                function_id
            } else {
                match (host_fn_name, *address, module_name.as_str()) {
                    (
                        Self::HOST_BLOCK_NUMBER,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_number(module);
                        function_id
                    }
                    (
                        Self::HOST_BLOCK_GAS_LIMIT,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_gas_limit(module);
                        function_id
                    }
                    (
                        Self::HOST_BLOCK_TIMESTAMP,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_NAME_TX_CONTEXT,
                    ) => {
                        let (function_id, _) = hostio::host_functions::block_timestamp(module);
                        function_id
                    }
                    (Self::HOST_CHAIN_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                        let (function_id, _) = hostio::host_functions::chain_id(module);
                        function_id
                    }
                    _ => {
                        return Err(NativeFunctionError::HostFunctionNotSupported(
                            host_fn_name.to_string(),
                        ));
                    }
                }
            };

            return Ok(host_fn);
        }

        if let Some(host_test_fn_name) = Self::host_test_fn_name(name) {
            let host_fn = if let Ok(function_id) = module
                .imports
                .get_func(TEST_HOST_MODULE_NAME, host_test_fn_name)
            {
                function_id
            } else {
                match (host_test_fn_name, *address, module_name.as_str()) {
                    (
                        Self::HOST_SET_SENDER_ADDRESS,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) =
                            hostio::host_test_functions::set_sender_address(module);

                        function_id
                    }
                    (
                        Self::HOST_SET_SIGNER_ADDRESS,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) =
                            hostio::host_test_functions::set_signer_address(module);
                        function_id
                    }
                    (
                        Self::HOST_SET_BLOCK_BASEFEE,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) =
                            hostio::host_test_functions::set_block_basefee(module);
                        function_id
                    }
                    (
                        Self::HOST_SET_GAS_PRICE,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) = hostio::host_test_functions::set_gas_price(module);
                        function_id
                    }
                    (
                        Self::HOST_SET_BLOCK_NUMBER,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) =
                            hostio::host_test_functions::set_block_number(module);
                        function_id
                    }

                    (
                        Self::HOST_SET_GAS_LIMIT,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) = hostio::host_test_functions::set_gas_limit(module);
                        function_id
                    }
                    (
                        Self::HOST_SET_BLOCK_TIMESTAMP,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) =
                            hostio::host_test_functions::set_block_timestamp(module);
                        function_id
                    }
                    (
                        Self::HOST_SET_CHAIN_ID,
                        STYLUS_FRAMEWORK_ADDRESS,
                        SF_MODULE_TEST_SCENARIO,
                    ) => {
                        let (function_id, _) = hostio::host_test_functions::set_chain_id(module);
                        function_id
                    }
                    _ => {
                        return Err(NativeFunctionError::HostFunctionNotSupported(
                            host_test_fn_name.to_string(),
                        ));
                    }
                }
            };

            return Ok(host_fn);
        }

        if let Some(function) = module.funcs.by_name(&native_fn_name) {
            Ok(function)
        } else {
            Ok(match (name, *address, module_name.as_str()) {
                (Self::NATIVE_POISON, STANDARD_LIB_ADDRESS, STDLIB_MODULE_UNIT_TEST) => {
                    unit_test::add_poison_fn(module, module_id)
                }
                (
                    Self::NATIVE_NEW_TX_CONTEXT,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_TEST_SCENARIO,
                ) => unit_test::add_new_tx_context_fn(module, module_id, compilation_ctx)?,

                (Self::NATIVE_SENDER, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_sender_fn(module, compilation_ctx, module_id)
                }

                (Self::NATIVE_MSG_VALUE, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_msg_value_fn(module, compilation_ctx, module_id)
                }
                (
                    Self::NATIVE_BLOCK_BASEFEE,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_TX_CONTEXT,
                ) => transaction::add_native_block_basefee_fn(module, compilation_ctx, module_id),
                (Self::NATIVE_GAS_PRICE, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_tx_gas_price_fn(module, compilation_ctx, module_id)
                }
                (Self::NATIVE_FRESH_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    object::add_native_fresh_id_fn(module, compilation_ctx, module_id)
                }
                (Self::NATIVE_DATA, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_TX_CONTEXT) => {
                    transaction::add_native_data_fn(module, compilation_ctx, module_id)?
                }
                (
                    Self::NATIVE_HAS_CHILD_OBJECT,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_DYNAMIC_FIELD,
                ) => dynamic_field::add_has_child_object_fn(module, compilation_ctx, module_id)?,
                #[cfg(debug_assertions)]
                (Self::NATIVE_GET_LAST_MEMORY_POSITION, _, _) => {
                    tests::add_get_last_memory_position_fn(module, compilation_ctx)
                }
                (
                    Self::NATIVE_ACCOUNT_CODE_SIZE,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_ACCOUNT,
                ) => account::add_native_account_code_size_fn(module, compilation_ctx, module_id),
                (
                    Self::NATIVE_ACCOUNT_BALANCE,
                    STYLUS_FRAMEWORK_ADDRESS,
                    SF_MODULE_NAME_ACCOUNT,
                ) => account::add_native_account_balance_fn(module, compilation_ctx, module_id)?,
                (
                    Self::NATIVE_INTERNAL_CHECK_UTF8,
                    STANDARD_LIB_ADDRESS,
                    STDLIB_MODULE_NAME_STRING,
                ) => string::add_internal_check_utf8(module, compilation_ctx, module_id)?,
                (
                    Self::NATIVE_INTERNAL_IS_CHAR_BOUNDARY,
                    STANDARD_LIB_ADDRESS,
                    STDLIB_MODULE_NAME_STRING,
                ) => string::add_internal_is_char_boundary(module, compilation_ctx, module_id)?,
                (
                    Self::NATIVE_INTERNAL_INDEX_OF,
                    STANDARD_LIB_ADDRESS,
                    STDLIB_MODULE_NAME_STRING,
                ) => string::add_internal_index_of(module, compilation_ctx, module_id)?,
                (
                    Self::NATIVE_INTERNAL_SUB_STRING,
                    STANDARD_LIB_ADDRESS,
                    STDLIB_MODULE_NAME_STRING,
                ) => string::add_internal_sub_string(module, compilation_ctx, module_id)?,
                _ => {
                    return Err(NativeFunctionError::NativeFunctionNotSupported(
                        *module_id,
                        name.to_string(),
                    ));
                }
            })
        }
    }

    /// Links a function marked as #[external_call] into themodule and returns its id. If the
    /// function is already present it just returns the id.
    ///
    /// This function is idempotent.
    pub fn get_external_call(
        name: &str,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        runtime_error_data: &mut RuntimeErrorData,
        module_id: &ModuleId,
        arguments_types: &[IntermediateType],
        named_ids: &[IntermediateType],
    ) -> Result<FunctionId, NativeFunctionError> {
        let native_fn_name = Self::get_function_name(name, module_id);

        if let Some(function) = module.funcs.by_name(&native_fn_name) {
            Ok(function)
        } else {
            let module_data = compilation_ctx.get_module_data_by_id(module_id)?;

            let function_information = module_data.functions.get_information_by_identifier(name)?;

            if let Some(special_attributes) = module_data
                .special_attributes
                .external_calls
                .get(&Symbol::from(name))
            {
                contract_calls::add_external_contract_call_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    module_id,
                    function_information,
                    &special_attributes.modifiers,
                    arguments_types,
                    named_ids,
                )
            } else {
                Err(NativeFunctionError::NotExternalCall(
                    *module_id,
                    name.to_owned(),
                ))
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
        runtime_error_data: Option<&mut RuntimeErrorData>,
        module_id: &ModuleId,
        generics: &[IntermediateType],
    ) -> Result<FunctionId, NativeFunctionError> {
        let ModuleId {
            address,
            module_name,
        } = module_id;

        let function_id = match (name, *address, module_name.as_str(), runtime_error_data) {
            //
            // Tests
            //
            (
                Self::NATIVE_DROP_STORAGE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_TEST_SCENARIO,
                _,
            ) => unit_test::add_drop_storage_object_fn(module, module_id),
            //
            // Transfer
            //
            (
                Self::NATIVE_SHARE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                transfer::add_share_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_TRANSFER_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                transfer::add_transfer_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_FREEZE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                transfer::add_freeze_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_DELETE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER,
                Some(runtime_error_data),
            )
            | (
                Self::NATIVE_REMOVE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::DeleteFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    Some(runtime_error_data),
                    &[&generics[0]],
                )?
            }
            //
            // Event
            //
            (
                Self::NATIVE_EMIT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_EVENT,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                event::add_emit_log_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }

            //
            // Error
            //
            (
                Self::NATIVE_REVERT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_ERROR,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                abi_error::add_revert_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }

            //
            // Types
            //
            (
                Self::NATIVE_IS_ONE_TIME_WITNESS,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TYPES,
                _,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                types::add_is_one_time_witness_fn(module, compilation_ctx, &generics[0], module_id)?
            }

            //
            // Object
            //
            (Self::NATIVE_COMPUTE_NAMED_ID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT, _) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                object::add_compute_named_id_fn(module, compilation_ctx, &generics[0], module_id)?
            }
            (Self::NATIVE_AS_UID, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT, _)
            | (Self::NATIVE_AS_UID_MUT, STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_OBJECT, _) => {
                // Generics are not used in this function because it just converts &NamedId to &UID,
                // which, under the hood they have the same structure. Generic type is not used in
                // the function, just to detect that the function was called correctly
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                object::add_as_uid_fn(module, compilation_ctx, module_id)
            }

            //
            // Dynamic field
            //
            (
                Self::NATIVE_HASH_TYPE_AND_KEY,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
                _,
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                dynamic_field::add_hash_type_and_key_fn(
                    module,
                    compilation_ctx,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_ADD_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                dynamic_field::add_child_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_BORROW_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
                Some(runtime_error_data),
            )
            | (
                Self::NATIVE_BORROW_CHILD_OBJECT_MUT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                dynamic_field::add_borrow_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_REMOVE_CHILD_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_DYNAMIC_FIELD,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                dynamic_field::add_remove_child_object_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }

            //
            // Bytes
            //
            (
                Self::NATIVE_AS_VEC_BYTES_N,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_SOL_TYPES,
                _,
            ) => RuntimeFunction::BytesToVec.get(module, Some(compilation_ctx), None)?,

            // This native function is only available in debug mode to help with testing. It should
            // not be compiled in release mode.
            #[cfg(debug_assertions)]
            (Self::SAVE_IN_SLOT, _, _, Some(runtime_error_data)) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                // In this case the native function implementation is the same as the runtime one.
                // So we reuse the runtime function.
                RuntimeFunction::EncodeAndSaveInStorage.get_generic(
                    module,
                    compilation_ctx,
                    Some(runtime_error_data),
                    &[&generics[0]],
                )?
            }
            // This native function is only available in debug mode to help with testing. It should
            // not be compiled in release mode.
            #[cfg(debug_assertions)]
            (Self::READ_SLOT, _, _, Some(runtime_error_data)) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;

                tests::add_read_slot_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }
            #[cfg(debug_assertions)]
            (Self::NATIVE_HASH_TYPE_AND_KEY, _, _, _) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                dynamic_field::add_hash_type_and_key_fn(
                    module,
                    compilation_ctx,
                    &generics[0],
                    module_id,
                )?
            }
            (
                Self::NATIVE_PEEP,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_PEEP,
                Some(runtime_error_data),
            ) => {
                Self::assert_generics_length(generics.len(), 1, name, module_id)?;
                peep::add_peep_fn(
                    module,
                    compilation_ctx,
                    runtime_error_data,
                    &generics[0],
                    module_id,
                )?
            }

            _ => {
                return Err(NativeFunctionError::GenericdNativeFunctionNotSupported(
                    *module_id,
                    name.to_owned(),
                ));
            }
        };

        Ok(function_id)
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

    /// Maps the native function name to the host function name.
    fn host_test_fn_name(name: &str) -> Option<&'static str> {
        match name {
            Self::NATIVE_SET_SENDER_ADDRESS => Some(Self::HOST_SET_SENDER_ADDRESS),
            Self::NATIVE_SET_SIGNER_ADDRESS => Some(Self::HOST_SET_SIGNER_ADDRESS),
            Self::NATIVE_SET_BLOCK_BASEFEE => Some(Self::HOST_SET_BLOCK_BASEFEE),
            Self::NATIVE_SET_GAS_PRICE => Some(Self::HOST_SET_GAS_PRICE),
            Self::NATIVE_SET_BLOCK_NUMBER => Some(Self::HOST_SET_BLOCK_NUMBER),
            Self::NATIVE_SET_GAS_LIMIT => Some(Self::HOST_SET_GAS_LIMIT),
            Self::NATIVE_SET_BLOCK_TIMESTAMP => Some(Self::HOST_SET_BLOCK_TIMESTAMP),
            Self::NATIVE_SET_CHAIN_ID => Some(Self::HOST_SET_CHAIN_ID),
            _ => None,
        }
    }

    fn assert_generics_length(
        len: usize,
        expected: usize,
        name: &str,
        module_id: &ModuleId,
    ) -> Result<(), NativeFunctionError> {
        if expected != len {
            return Err(NativeFunctionError::WrongNumberOfTypeParameters {
                module_id: *module_id,
                function_name: name.to_owned(),
                expected,
                found: len,
            });
        }

        Ok(())
    }

    pub fn get_function_name(name: &str, module_id: &ModuleId) -> String {
        format!("___{name}_{:x}", module_id.hash())
    }

    pub fn get_generic_function_name(
        name: &str,
        compilation_ctx: &CompilationContext,
        generics: &[&IntermediateType],
        module_id: &ModuleId,
    ) -> Result<String, NativeFunctionError> {
        if generics.is_empty() {
            return Err(NativeFunctionError::GetGenericFunctionNameNoGenerics(
                *module_id,
                name.to_owned(),
            ));
        }

        let mut hasher = get_hasher();
        for t in generics {
            t.process_hash(&mut hasher, compilation_ctx)?;
        }
        let hash = format!("{:x}", hasher.finish());

        Ok(format!("___{name}_{hash}_{:x}", module_id.hash()))
    }

    pub fn is_fallible(name: &str, module_id: &ModuleId) -> bool {
        let ModuleId {
            address,
            module_name,
        } = module_id;

        matches!(
            (name, *address, module_name.as_str()),
            (
                Self::NATIVE_PEEP,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_PEEP
            ) | (
                Self::NATIVE_SHARE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER
            ) | (
                Self::NATIVE_FREEZE_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER
            ) | (
                Self::NATIVE_TRANSFER_OBJECT,
                STYLUS_FRAMEWORK_ADDRESS,
                SF_MODULE_NAME_TRANSFER
            )
        )
    }
}
