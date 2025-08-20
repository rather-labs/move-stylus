use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

use crate::{
    CompilationContext, hostio::host_functions::emit_log,
    translation::intermediate_types::structs::IStruct,
};

use super::NativeFunction;

pub fn add_emit_log_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let (emit_log_fn, _) = emit_log(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function
        .name(NativeFunction::NATIVE_EMIT.to_owned())
        .func_body();

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);

    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(struct_.solidity_abi_encode_size(compilation_ctx) as i32)
        .call(compilation_ctx.allocator)
        .local_tee(writer_pointer)
        .local_tee(calldata_reference_pointer)
        .local_set(packed_data_begin);

    // ABI pack the struct before emitting the event
    struct_.add_pack_instructions(
        &mut builder,
        module,
        struct_ptr,
        writer_pointer,
        calldata_reference_pointer,
        compilation_ctx,
        None,
    );

    // Emit the event with the ABI packed struct

    // Beginning of the packed data
    builder.local_get(packed_data_begin);

    // Call the allocator function to store in stack the end of the calldata
    // builder.i32_const(0).call(compilation_ctx.allocator);

    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(packed_data_begin)
        .binop(BinaryOp::I32Sub);

    // Log 0
    builder.i32_const(0).call(emit_log_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}
