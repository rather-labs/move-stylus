//! This module is in charge of injecting the datatypes that can only be created or are
//! automatically injected by the VM, such as the primitive type Signer or the TxContext struct
//! from the stylus framework.

use std::sync::LazyLock;

use walrus::{FunctionId, InstrSeqBuilder, Module, ValType, ir::BinaryOp};

use crate::{
    compilation_context::{ModuleId, module_data::Address},
    translation::intermediate_types::signer::ISigner,
};

pub fn inject_signer(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    allocator_func: FunctionId,
    tx_origin_function: FunctionId,
    #[cfg(debug_assertions)] emit_log_function: FunctionId,
) {
    let signer_pointer = module.locals.add(ValType::I32);
    block.i32_const(ISigner::HEAP_SIZE);
    block.call(allocator_func);
    block.local_tee(signer_pointer);
    // We add 12 to the pointer returned by the allocator because stylus writes 20
    // bytes, and those bytes need to be at the end.
    block.i32_const(12);
    block.binop(BinaryOp::I32Add);
    block.call(tx_origin_function);
    block.local_get(signer_pointer);

    // If we are building in debug mode, we call `emit_log` to log the signer's
    // address. This is useful for debugging in this stage.
    // TODO: Remove this when is no longer necessary
    #[cfg(debug_assertions)]
    {
        block.local_get(signer_pointer);
        block.i32_const(ISigner::HEAP_SIZE);
        block.i32_const(0);
        block.call(emit_log_function);
    }
}

pub struct TxContext;

static TX_CONTEXT_MODULE: LazyLock<ModuleId> = LazyLock::new(|| ModuleId {
    address: Address::from([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
    ]),
    module_name: "tx_context".to_string(),
});

impl TxContext {
    const TX_CONTEXT_IDENTIFIER: &str = "TxContext";

    /// The only valid TxContext is the one defined in the stylus framework. Any other struct named
    /// TxContext from any other module must be reported as invalid.
    pub fn struct_is_tx_context(module_id: &ModuleId, identifier: &str) -> bool {
        if identifier == Self::TX_CONTEXT_IDENTIFIER {
            if *module_id != *TX_CONTEXT_MODULE {
                panic!(
                    "Using invalid TxContext struct from module {module_id}. The only valid TxContext object is from the module stylus::{}",
                    *TX_CONTEXT_MODULE
                );
            }
            return true;
        }
        false
    }

    /// TxContext is an empty struct. We just reserve 4 bytes and return a pointer to that.
    pub fn inject_tx_context(block: &mut InstrSeqBuilder, allocator_func: FunctionId) {
        block.i32_const(4).call(allocator_func);
    }
}
