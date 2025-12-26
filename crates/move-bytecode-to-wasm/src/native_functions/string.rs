use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg},
};

use crate::{
    CompilationContext, compilation_context::ModuleId,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::NativeFunction;

pub fn add_internal_check_utf8(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_INTERNAL_CHECK_UTF8,
            module_id,
        ))
        .func_body();

    // Arguments
    let vec_ptr = module.locals.add(ValType::I32);

    // Locals
    let length = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let current_char = module.locals.add(ValType::I32);

    // Since it is a reference, we de-reference it once
    builder
        .local_get(vec_ptr)
        .load(
            compilation_ctx.memory_id,
            walrus::ir::LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(vec_ptr);

    // Load vector's length
    builder
        .local_get(vec_ptr)
        .load(
            compilation_ctx.memory_id,
            walrus::ir::LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(length);

    // Skip the header and iterate over the bytes
    builder.skip_vec_header(vec_ptr);

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        // If i < length, continue
        loop_
            .local_get(i)
            .local_get(length)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_id);

        // First we load the byte at position i to determine the character size
        loop_
            .local_get(vec_ptr)
            .local_get(i)
            .binop(BinaryOp::I32Add)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtend,
                },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_tee(current_char);

        // Determine character byte size. This block of code is basiacally doing the following:
        //  if b0 < 0x80 { 1 }
        //  else if b0 & 0xE0 == 0xC0 { 2 }
        //  else if b0 & 0xF0 == 0xE0 { 3 }
        //  else if b0 & 0xF8 == 0xF0 { 4 }
        //  else { invalid }
        loop_.i32_const(0x7F).binop(BinaryOp::I32LeU).if_else(
            None,
            // Valid Ascii, continue the loop
            |then_| {
                then_
                    .local_get(i)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(i)
                    .br(loop_id);
            },
            // Multi-byte character
            |else_| {
                // (byte & 0xE0) == 0xC0) -> 2-byte character
                else_
                    .local_get(current_char)
                    .i32_const(0xE0)
                    .binop(BinaryOp::I32And)
                    .i32_const(0xC0)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then_| {
                            // if (0xC280 <= c && c <= 0xDFBF) return ((c & 0xE0C0) == 0xC080);
                            then_
                                .local_get(vec_ptr)
                                .local_get(i)
                                .binop(BinaryOp::I32Add)
                                .load(
                                    compilation_ctx.memory_id,
                                    LoadKind::I32_16 {
                                        kind: ExtendedLoad::ZeroExtend,
                                    },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                .local_tee(current_char);

                            then_
                                .i32_const(0xC280)
                                .local_get(current_char)
                                .binop(BinaryOp::I32LeU)
                                .local_get(current_char)
                                .i32_const(0xDFBF)
                                .binop(BinaryOp::I32LeU)
                                .binop(BinaryOp::I32And)
                                .if_else(
                                    None,
                                    |then_| {
                                        then_
                                            .local_get(current_char)
                                            .i32_const(0xE0C0)
                                            .binop(BinaryOp::I32And)
                                            .i32_const(0xC080)
                                            .binop(BinaryOp::I32Eq)
                                            .if_else(
                                                None,
                                                |then_| {
                                                    then_
                                                        .local_get(i)
                                                        .i32_const(2)
                                                        .binop(BinaryOp::I32Add)
                                                        .local_set(i)
                                                        .br(loop_id);
                                                },
                                                |else_| {
                                                    // Invalid UTF-8 byte sequence
                                                    else_.i32_const(0).return_();
                                                },
                                            );
                                    },
                                    |else_| {
                                        // Invalid UTF-8 byte sequence
                                        else_.i32_const(0).return_();
                                    },
                                );
                        },
                        |else_| {
                            // 3 bytes
                            // if (0xE0A080 <= c && c <= 0xEFBFBF) return ((c & 0xF0C0C0) == 0xE08080);
                            else_
                                .local_get(vec_ptr)
                                .local_get(i)
                                .binop(BinaryOp::I32Add)
                                .load(
                                    compilation_ctx.memory_id,
                                    LoadKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                .local_tee(current_char);

                            else_
                                .i32_const(0xE0A080)
                                .local_get(current_char)
                                .binop(BinaryOp::I32LeU)
                                .local_get(current_char)
                                .i32_const(0xEFBFBF)
                                .binop(BinaryOp::I32LeU)
                                .binop(BinaryOp::I32And)
                                .if_else(
                                    None,
                                    |then_| {
                                        then_
                                            .local_get(current_char)
                                            .i32_const(0xF0C0C0)
                                            .binop(BinaryOp::I32And)
                                            .i32_const(0xE08080)
                                            .binop(BinaryOp::I32Eq)
                                            .if_else(
                                                None,
                                                |then_| {
                                                    then_
                                                        .local_get(i)
                                                        .i32_const(3)
                                                        .binop(BinaryOp::I32Add)
                                                        .local_set(i)
                                                        .br(loop_id);
                                                },
                                                |else_| {
                                                    // Invalid UTF-8 byte sequence
                                                    else_.i32_const(0).return_();
                                                },
                                            );
                                    },
                                    |else_| {
                                        // 4 bytes
                                        // if (0xF0908080 <= c && c <= 0xF48FBFBF) return ((c & 0xF8C0C0C0) == 0xF0808080);
                                        else_
                                            .i32_const(0xF0908080_u32 as i32)
                                            .local_get(current_char)
                                            .binop(BinaryOp::I32LeU)
                                            .local_get(current_char)
                                            .i32_const(0xF48FBFBF_u32 as i32)
                                            .binop(BinaryOp::I32LeU)
                                            .binop(BinaryOp::I32And)
                                            .if_else(
                                                None,
                                                |then_| {
                                                    then_
                                                        .local_get(current_char)
                                                        .i32_const(0xF8C0C0C0_u32 as i32)
                                                        .binop(BinaryOp::I32And)
                                                        .i32_const(0xF0808080_u32 as i32)
                                                        .binop(BinaryOp::I32Eq)
                                                        .if_else(
                                                            None,
                                                            |then_| {
                                                                then_
                                                                    .local_get(i)
                                                                    .i32_const(4)
                                                                    .binop(BinaryOp::I32Add)
                                                                    .local_set(i)
                                                                    .br(loop_id);
                                                            },
                                                            |else_| {
                                                                // Invalid UTF-8 byte sequence
                                                                else_.i32_const(0).return_();
                                                            },
                                                        );
                                                },
                                                |else_| {
                                                    // Invalid UTF-8 byte sequence
                                                    else_.i32_const(0).return_();
                                                },
                                            );
                                    },
                                );
                        },
                    );
            },
        );
    });

    // Return 1 (valid UTF-8)
    builder.i32_const(1).return_();

    function.finish(vec![vec_ptr], &mut module.funcs)
}
