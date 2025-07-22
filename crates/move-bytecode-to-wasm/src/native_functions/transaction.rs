//! This module contains all the functions retaled to transaction information.

use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

use crate::{
    CompilationContext, hostio::host_functions::msg_sender,
    translation::intermediate_types::address::IAddress,
};

use super::NativeFunction;

pub fn add_native_sender_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let (msg_sender_function_id, _) = msg_sender(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let address_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::NATIVE_SENDER.to_owned())
        .func_body();

    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_tee(address_ptr)
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .call(msg_sender_function_id)
        .local_get(address_ptr);

    function.finish(vec![], &mut module.funcs)
}
