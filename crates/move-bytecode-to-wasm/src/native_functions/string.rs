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

    let (print_i32, _, print_m, _, _, _) = crate::declare_host_debug_functions!(module);

    builder.local_get(vec_ptr).i32_const(128).call(print_m);
    // Since it is a reference, we de-reference it once
    /*
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
    */

    builder.local_get(vec_ptr).i32_const(128).call(print_m);
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

    builder
        .loop_(ValType::I32, |loop_| {
            let loop_id = loop_.id();

            // If i >= length, we finished the loop and return 1 (valid UTF-8)
            loop_
                .local_get(i)
                .local_get(length)
                .binop(BinaryOp::I32GeU)
                .if_else(
                    ValType::I32,
                    |then_| {
                        then_.i32_const(1).return_();
                    },
                    |else_| {
                        // First we load the byte at position i to determine the character size
                        else_
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

                        else_.local_get(i).call(print_i32);
                        else_.local_get(length).call(print_i32);
                        else_.local_get(current_char).call(print_i32);

                        // This code is based off
                        // https://stackoverflow.com/questions/66715611/check-for-valid-utf-8-encoding-in-c#answer-66723102
                        // We are implementing:
                        // 1:  if (c <= 0x7F) continue;
                        // 2:  if (0xC080 == c) continue;   // Accept 0xC080 as representation for '\0'
                        // 3:  if (0xC280 <= c && c <= 0xDFBF) {
                        //         if ((c & 0xE0C0) == 0xC080) { continue; }
                        //         else { return 0; }
                        //     }
                        // 4:  if (0xEDA080 <= c && c <= 0xEDBFBF) return 0; // Reject UTF-16 surrogates
                        // 5:  if (0xE0A080 <= c && c <= 0xEFBFBF) {
                        //         if ((c & 0xF0C0C0) == 0xE08080) { continue; }
                        //         else { return 0; }
                        //      }
                        // 6:  if (0xF0908080 <= c && c <= 0xF48FBFBF) {
                        //         if ((c & 0xF8C0C0C0) == 0xF0808080) { continue; }
                        //         else { return 0; }
                        //      }
                        else_.i32_const(0x7F).binop(BinaryOp::I32LeU).if_else(
                            ValType::I32,
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
                                        ValType::I32,
                                        |then_| {
                                            // First we load the 2-byte character
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

                                            // 2:  if (0xC080 == c) continue;   // Accept 0xC080 as representation for '\0'
                                            then_
                                                .i32_const(0xC080)
                                                .local_get(current_char)
                                                .binop(BinaryOp::I32Eq)
                                                .if_else(
                                                    None,
                                                    |then_| {
                                                        // It's the representation for '\0', continue
                                                        then_
                                                            .local_get(i)
                                                            .i32_const(2)
                                                            .binop(BinaryOp::I32Add)
                                                            .local_set(i)
                                                            .br(loop_id);
                                                    },
                                                    |_| {},
                                                );

                                            // 3:  if (0xC280 <= c && c <= 0xDFBF) {
                                            //         if ((c & 0xE0C0) == 0xC080) { continue; }
                                            //         else { return 0; }
                                            //     }
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

                                            // 4:  if (0xEDA080 <= c && c <= 0xEDBFBF) return 0; // Reject UTF-16 surrogates
                                            else_
                                                .i32_const(0xEDA080)
                                                .local_get(current_char)
                                                .binop(BinaryOp::I32LeU)
                                                .local_get(current_char)
                                                .i32_const(0xEDBFBF)
                                                .binop(BinaryOp::I32LeU)
                                                .binop(BinaryOp::I32And)
                                                .if_else(
                                                    None,
                                                    // Invalid UTF-8 byte sequence (UTF-16 surrogate)
                                                    |then_| {
                                                        then_.i32_const(0).return_();
                                                    },
                                                    |_| {},
                                                );

                                            // 5:  if (0xE0A080 <= c && c <= 0xEFBFBF) {
                                            //       if ((c & 0xF0C0C0) == 0xE08080) { continue; }
                                            //       else { return 0; }
                                            //  }
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
                                                        // 6:  if (0xF0908080 <= c && c <= 0xF48FBFBF) {
                                                        //          if ((c & 0xF8C0C0C0) == 0xF0808080) { continue; }
                                                        //          else { return 0; }
                                                        //     }
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
                                                                        .i32_const(
                                                                            0xF8C0C0C0_u32 as i32,
                                                                        )
                                                                        .binop(BinaryOp::I32And)
                                                                        .i32_const(
                                                                            0xF0808080_u32 as i32,
                                                                        )
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
                                                                                else_
                                                                                    .i32_const(0)
                                                                                    .return_();
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
                    },
                );
        })
        .return_();

    function.finish(vec![vec_ptr], &mut module.funcs)
}
