//! This module hold functtions used only in tests and debug builds.
#![cfg(debug_assertions)]

use super::NativeFunction;
use crate::{
    CompilationContext,
    data::DATA_SLOT_DATA_PTR_OFFSET,
    get_generic_function_name,
    hostio::host_functions::{
        block_number, block_timestamp, emit_log, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    runtime::RuntimeFunction,
    storage::encoding::field_size,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        heap_integers::{IU128, IU256},
        structs::IStruct,
    },
    utils::keccak_string_to_memory,
    vm_handled_types::{VmHandledType, named_id::NamedId, uid::Uid},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn add_get_last_memory_position_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    if let Some(function) = module
        .funcs
        .by_name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION)
    {
        return function;
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let mut builder = function
        .name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION.to_owned())
        .func_body();

    // Call allocator with size 0 to get the current memory position
    builder.i32_const(0).call(compilation_ctx.allocator);

    function.finish(vec![], &mut module.funcs)
}
