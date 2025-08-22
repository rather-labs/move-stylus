use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

use crate::{
    CompilationContext, get_generic_function_name, hostio::host_functions::emit_log,
    translation::intermediate_types::IntermediateType,
};

use super::NativeFunction;

pub fn add_emit_log_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_EMIT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let (emit_log_fn, _) = emit_log(module);

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);

    let size = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
        32
    } else {
        struct_.solidity_abi_encode_size(compilation_ctx) as i32
    };
    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(size)
        .call(compilation_ctx.allocator)
        .local_tee(writer_pointer)
        .local_tee(calldata_reference_pointer)
        .local_set(packed_data_begin);

    // ABI pack the struct before emitting the event
    if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
        struct_.add_pack_instructions(
            &mut builder,
            module,
            struct_ptr,
            writer_pointer,
            calldata_reference_pointer,
            compilation_ctx,
            Some(calldata_reference_pointer),
        );
    } else {
        struct_.add_pack_instructions(
            &mut builder,
            module,
            struct_ptr,
            writer_pointer,
            calldata_reference_pointer,
            compilation_ctx,
            None,
        );
    }

    // Emit the event with the ABI packed struct

    // Beginning of the packed data
    builder.local_get(packed_data_begin);

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
