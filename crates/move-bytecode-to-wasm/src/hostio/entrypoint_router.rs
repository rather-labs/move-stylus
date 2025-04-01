use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{abi_types::function_encoding::AbiFunctionSelector, memory::get_allocator_function_id};

use super::host_functions;

/// Builds an entrypoint router for the list of functions provided
/// and adds it to the module exporting it as `user_entrypoint`
/// TODO: This should route to the actual functions
pub fn build_entrypoint_router(
    module: &mut Module,
    functions: &[(FunctionId, AbiFunctionSelector)],
) {
    let (allocator_func, memory_id) = get_allocator_function_id();

    let (read_args_function, _) = host_functions::read_args(module);

    let args_len = module.locals.add(ValType::I32);
    let selector = module.locals.add(ValType::I32);
    let args_pointer = module.locals.add(ValType::I32);

    let mut router = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let mut router_builder = router.func_body();

    // TODO: handle case where no args data, now we just panic
    router_builder.block(None, |block| {
        let block_id = block.id();

        // If args len is < 4 there is no selector
        block.local_get(args_len);
        block.i32_const(4);
        block.binop(BinaryOp::I32GeS);
        block.br_if(block_id);
        block.unreachable();
    });

    // // Load function args to memory
    router_builder.local_get(args_len);
    router_builder.call(allocator_func);
    router_builder.local_tee(args_pointer);
    router_builder.call(read_args_function);

    // Load selector from first 4 bytes of args
    router_builder.local_get(args_pointer);
    router_builder.load(
        memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );
    router_builder.local_set(selector);

    // TODO: build actual router
    router_builder.i32_const(0);

    let router = router.finish(vec![args_len], &mut module.funcs);
    add_entrypoint(module, router);
}

/// Add an entrypoint to the module with the interface defined by Stylus
pub fn add_entrypoint(module: &mut Module, func: FunctionId) {
    module.exports.add("user_entrypoint", func);
}
