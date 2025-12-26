use move_compiler::linters::loop_without_exit;
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg},
};

use crate::{CompilationContext, wasm_builder_extensions::WasmBuilderExtension};

use super::{RuntimeFunction, error::RuntimeFunctionError};

pub fn internal_check_utf8(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::LocateStorageData.name().to_owned())
        .func_body();

    // Arguments
    let vec_ptr = module.locals.add(ValType::I32);

    // Locals
    let length = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let char_byte_size = module.locals.add(ValType::I32);
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
            .local_tee(char_byte_size);

        // Determine character byte size. This block of code is basiacally doing the following:
        //  if b0 < 0x80 { 1 }
        //  else if b0 & 0xE0 == 0xC0 { 2 }
        //  else if b0 & 0xF0 == 0xE0 { 3 }
        //  else if b0 & 0xF8 == 0xF0 { 4 }
        //  else { invalid }
        loop_.i32_const(0x80).binop(BinaryOp::I32LtU).if_else(
            None,
            // 1-byte character (byte < 0x80)
            |then_| {
                then_.i32_const(1).local_set(char_byte_size);
            },
            // Multi-byte character
            |else_| {
                // (byte & 0xE0) == 0xC0) -> 2-byte character
                else_
                    .local_get(char_byte_size)
                    .i32_const(0xE0)
                    .binop(BinaryOp::I32And)
                    .i32_const(0xC0)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then_| {
                            then_.i32_const(2).local_set(char_byte_size);
                        },
                        |else_| {
                            // (byte & 0xF0) == 0xE0) -> 3-byte character
                            else_
                                .local_get(char_byte_size)
                                .i32_const(0xF0)
                                .binop(BinaryOp::I32And)
                                .i32_const(0xE0)
                                .binop(BinaryOp::I32Eq)
                                .if_else(
                                    None,
                                    |then_| {
                                        then_.i32_const(3).local_set(char_byte_size);
                                    },
                                    |else_| {
                                        else_
                                            .local_get(char_byte_size)
                                            .i32_const(0xF8)
                                            .binop(BinaryOp::I32And)
                                            .i32_const(0xF0)
                                            .binop(BinaryOp::I32Eq)
                                            .if_else(
                                                None,
                                                |then_| {
                                                    then_.i32_const(8).local_set(char_byte_size);
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

    Ok(function.finish(vec![vec_ptr], &mut module.funcs))
}
