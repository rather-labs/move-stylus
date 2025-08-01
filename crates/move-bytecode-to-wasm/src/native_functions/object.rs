use super::NativeFunction;
use crate::{
    CompilationContext,
    hostio::host_functions::{block_number, block_timestamp, native_keccak256},
    translation::intermediate_types::address::IAddress,
};
use walrus::{
    ConstExpr, FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

pub fn add_native_fresh_id_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let (native_keccak, _) = native_keccak256(module);
    let (block_number, _) = block_number(module);
    let (block_timestamp, _) = block_timestamp(module);

    // NOTE: This global variable is temporal. The counter will be stored in the contract's store.
    let global_counter = module.globals.add_local(
        ValType::I32,
        true,
        false,
        ConstExpr::Value(walrus::ir::Value::I32(0)),
    );

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let id_ptr = module.locals.add(ValType::I32);
    let data_to_hash_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::NATIVE_FRESH_ID.to_owned())
        .func_body();

    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(id_ptr);

    // Use block timestamp + block number + global counter to generate a unique ID
    builder
        .i32_const(16)
        .call(compilation_ctx.allocator)
        .local_tee(data_to_hash_ptr);

    builder
        .call(block_timestamp)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_get(data_to_hash_ptr)
        .call(block_number)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 8,
            },
        );

    // TODO: replace with counter from contract
    builder
        .global_get(global_counter)
        .i32_const(1)
        .binop(BinaryOp::I32Add)
        .global_set(global_counter);

    builder
        .local_get(data_to_hash_ptr)
        .global_get(global_counter)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 16,
            },
        );

    builder
        .local_get(data_to_hash_ptr)
        .i32_const(20)
        .local_get(id_ptr)
        .call(native_keccak);

    // Return the ID ptr
    builder.local_get(id_ptr);

    function.finish(vec![], &mut module.funcs)
}
