// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, UnaryOp},
};

use crate::{
    CompilationContext, compilation_context::ModuleId, runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::{NativeFunction, error::NativeFunctionError};

pub fn add_internal_check_utf8(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
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

    let swap_i32_fn = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;

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
    builder.skip_vec_header(vec_ptr).local_set(vec_ptr);

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

                        // This code is based off
                        // https://stackoverflow.com/questions/66715611/check-for-valid-utf-8-encoding-in-c#answer-66723102
                        // We are implementing:
                        // 1:  if (c <= 0x7F) continue;
                        // 2:  if (0xC080 == c) continue;   // Accept 0xC080 as representation for '\0'
                        // 3:  if (0xC280 <= c && c <= 0xDFBF) {
                        //         if ((c & 0xE0C0) == 0xC080) { continue; }
                        //     }
                        // 4:  if (0xEDA080 <= c && c <= 0xEDBFBF) return 0; // Reject UTF-16 surrogates
                        // 5:  if (0xE0A080 <= c && c <= 0xEFBFBF) {
                        //         if ((c & 0xF0C0C0) == 0xE08080) { continue; }
                        //      }
                        // 6:  if (0xF0908080 <= c && c <= 0xF48FBFBF) {
                        //         if ((c & 0xF8C0C0C0) == 0xF0808080) { continue; }
                        //      }

                        // =============================
                        //  1 byte character (ASCII)
                        // =============================
                        // 1:  if (c <= 0x7F) continue (ascii);
                        else_
                            .local_get(current_char)
                            .i32_const(0x7F)
                            .binop(BinaryOp::I32LeU)
                            .if_else(
                                None,
                                |then_| {
                                    then_
                                        .local_get(i)
                                        .i32_const(1)
                                        .binop(BinaryOp::I32Add)
                                        .local_set(i)
                                        .br(loop_id);
                                },
                                |_| {},
                            );

                        // ===================
                        //  2 bytes character
                        // ===================

                        // First we load the 2-byte character
                        else_
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
                            .call(swap_i32_fn)
                            .i32_const(16)
                            .binop(BinaryOp::I32ShrU)
                            .local_set(current_char);

                        // 2:  if (0xC080 == c) continue;   (Accept 0xC080 as representation for '\0')
                        else_
                            .i32_const(0xC080)
                            .local_get(current_char)
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
                                |_| {},
                            );

                        // 3:  if (0xC280 <= c && c <= 0xDFBF) && ((c & 0xE0C0) == 0xC080) {
                        //         i +=2;
                        //         continue;
                        //      }
                        else_
                            // (0xC280 <= c && c <= 0xDFBF)
                            .i32_const(0xC280)
                            .local_get(current_char)
                            .binop(BinaryOp::I32LeU)
                            .local_get(current_char)
                            .i32_const(0xDFBF)
                            .binop(BinaryOp::I32LeU)
                            .binop(BinaryOp::I32And)
                            // ((c & 0xE0C0) == 0xC080)
                            .local_get(current_char)
                            .i32_const(0xE0C0)
                            .binop(BinaryOp::I32And)
                            .i32_const(0xC080)
                            .binop(BinaryOp::I32Eq)
                            .binop(BinaryOp::I32And)
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
                                |_| {},
                            );

                        // ===================
                        //  3 bytes character
                        // ===================
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
                            .call(swap_i32_fn)
                            .i32_const(8)
                            .binop(BinaryOp::I32ShrU)
                            .local_set(current_char);

                        // 4:  if (0xEDA080 <= c && c <= 0xEDBFBF) return 0; (Reject UTF-16
                        //     surrogates)
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

                        // 5:  if (0xE0A080 <= c && c <= 0xEFBFBF) && ((c & 0xF0C0C0) == 0xE08080) {
                        //       i+=3;
                        //       continue;
                        //  }
                        else_
                            // (0xE0A080 <= c && c <= 0xEFBFBF)
                            .i32_const(0xE0A080)
                            .local_get(current_char)
                            .binop(BinaryOp::I32LeU)
                            .local_get(current_char)
                            .i32_const(0xEFBFBF)
                            .binop(BinaryOp::I32LeU)
                            .binop(BinaryOp::I32And)
                            // ((c & 0xF0C0C0) == 0xE08080)
                            .local_get(current_char)
                            .i32_const(0xF0C0C0)
                            .binop(BinaryOp::I32And)
                            .i32_const(0xE08080)
                            .binop(BinaryOp::I32Eq)
                            .binop(BinaryOp::I32And)
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
                                |_| {},
                            );

                        // ===================
                        //  4 bytes character
                        // ===================

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
                            .call(swap_i32_fn)
                            .local_set(current_char);

                        // 6:  if (0xF0908080 <= c && c <= 0xF48FBFBF) && ((c & 0xF8C0C0C0) == 0xF0808080) {
                        //        i+=4;
                        //        continue;
                        //     }
                        else_
                            // (0xF0908080 <= c && c <= 0xF48FBFBF)
                            .i32_const(0xF0908080_u32 as i32)
                            .local_get(current_char)
                            .binop(BinaryOp::I32LeU)
                            .local_get(current_char)
                            .i32_const(0xF48FBFBF_u32 as i32)
                            .binop(BinaryOp::I32LeU)
                            .binop(BinaryOp::I32And)
                            // ((c & 0xF8C0C0C0) == 0xF0808080)
                            .local_get(current_char)
                            .i32_const(0xF8C0C0C0_u32 as i32)
                            .binop(BinaryOp::I32And)
                            .i32_const(0xF0808080_u32 as i32)
                            .binop(BinaryOp::I32Eq)
                            .binop(BinaryOp::I32And)
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
                                |_| {},
                            );

                        else_.i32_const(0).return_();
                    },
                );
        })
        .return_();

    Ok(function.finish(vec![vec_ptr], &mut module.funcs))
}

/// Implementation of `native fun internal_is_char_boundary(v: &vector<u8>, index: u64): bool;`
pub fn add_internal_is_char_boundary(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    // Function declaration: (vec_ptr: i32, char_index: i64) -> i32 (bool)
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64],
        &[ValType::I32],
    );
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_INTERNAL_IS_CHAR_BOUNDARY,
            module_id,
        ))
        .func_body();

    // Arguments
    let vec_ptr = module.locals.add(ValType::I32);
    let char_index = module.locals.add(ValType::I64);

    // Locals
    let length = module.locals.add(ValType::I32);
    let is_boundary = module.locals.add(ValType::I32);
    let char_index_i32 = module.locals.add(ValType::I32);

    // Convert char_index to i32 (maybe we need to use downcast runtime fn here?)
    builder
        .local_get(char_index)
        .unop(UnaryOp::I32WrapI64)
        .local_set(char_index_i32);

    // Initialize is_boundary to 0
    builder.i32_const(0).local_set(is_boundary);

    // Load vector's length (stored as i32 at the header)
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

    // Check if character is boundary
    builder.local_get(char_index).unop(UnaryOp::I64Eqz).if_else(
        None,
        |then| {
            // First character is always a boundary
            then.i32_const(1).local_set(is_boundary);
        },
        |else_| {
            // Check if index >= length
            // Convert char_index to i32 for comparison with length
            else_
                .local_get(char_index_i32)
                .local_get(length)
                .binop(BinaryOp::I32GeU)
                .if_else(
                    None,
                    |then| {
                        // index == length ? true : false
                        then.local_get(char_index_i32)
                            .local_get(length)
                            .binop(BinaryOp::I32Eq)
                            .local_set(is_boundary);
                    },
                    |else_| {
                        // General Case: index is within [1, length-1]
                        // Get the byte at the index
                        else_
                            .vec_elem_ptr(vec_ptr, char_index_i32, 1)
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32_8 {
                                    kind: ExtendedLoad::SignExtend,
                                },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            // (self as i8) >= -64
                            .i32_const(-0x40) // -64
                            .binop(BinaryOp::I32GeS)
                            .local_set(is_boundary);
                    },
                );
        },
    );

    builder.local_get(is_boundary).return_();

    Ok(function.finish(vec![vec_ptr, char_index], &mut module.funcs))
}

/// Implementation of `native fun internal_index_of(v: &vector<u8>, r: &vector<u8>): u64;`
pub fn add_internal_index_of(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I64],
    );
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_INTERNAL_INDEX_OF,
            module_id,
        ))
        .func_body();

    // Arguments
    let vec_ptr = module.locals.add(ValType::I32);
    let pattern_ptr = module.locals.add(ValType::I32);

    // Locals
    let vec_len = module.locals.add(ValType::I32);
    let pattern_len = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let j = module.locals.add(ValType::I32);
    let search_limit = module.locals.add(ValType::I32);

    // 1. Load lengths
    builder
        .local_get(vec_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(vec_len);

    builder
        .local_get(pattern_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(pattern_len);

    // 2. Base cases
    // If pattern is empty, return 0
    builder
        .local_get(pattern_len)
        .unop(UnaryOp::I32Eqz)
        .if_else(
            None,
            |then| {
                then.i64_const(0).return_();
            },
            |_| {},
        );

    // If pattern > source, return source length
    builder
        .local_get(pattern_len)
        .local_get(vec_len)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                then.local_get(vec_len)
                    .unop(UnaryOp::I64ExtendUI32)
                    .return_();
            },
            |_| {},
        );

    // 3. Pre-calculate search_limit: vec_len - pattern_len
    // We know vec_len >= pattern_len here because of the check above.
    builder
        .local_get(vec_len)
        .local_get(pattern_len)
        .binop(BinaryOp::I32Sub)
        .local_set(search_limit);

    // 4. Outer Loop
    builder.i32_const(0).local_set(i);
    builder.loop_(None, |outer_loop| {
        let outer_id = outer_loop.id();

        // Inner Loop
        outer_loop.i32_const(0).local_set(j);
        outer_loop.loop_(None, |inner_loop| {
            let inner_id = inner_loop.id();

            // Load byte vec[i + j] and pattern[j]
            inner_loop
                .vec_elem_ptr(vec_ptr, i, 1)
                .local_get(j)
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
                .vec_elem_ptr(pattern_ptr, j, 1)
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
                .binop(BinaryOp::I32Ne); // If mismatch

            inner_loop.if_else(
                None,
                |then| {
                    // Mismatch: increment i
                    then.local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(i);
                },
                |else_| {
                    // Match: increment j
                    else_
                        .local_get(j)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(j);
                    // If j == pattern_len, match found
                    else_
                        .local_get(j)
                        .local_get(pattern_len)
                        .binop(BinaryOp::I32Eq)
                        .if_else(
                            None,
                            |found| {
                                found.local_get(i).unop(UnaryOp::I64ExtendUI32).return_();
                            },
                            |not_yet| {
                                not_yet.br(inner_id);
                            },
                        );
                },
            );
        });

        // Loop Condition: if i <= search_limit, continue
        outer_loop
            .local_get(i)
            .local_get(search_limit)
            .binop(BinaryOp::I32LeU)
            .br_if(outer_id);
    });

    // 5. Final return if no match found
    builder
        .local_get(vec_len)
        .unop(UnaryOp::I64ExtendUI32)
        .return_();

    Ok(function.finish(vec![vec_ptr, pattern_ptr], &mut module.funcs))
}

/// Implementation of `native fun internal_sub_string(v: &vector<u8>, i: u64, j: u64): vector<u8>;`
pub fn add_internal_sub_string(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    // Signature: (vec_ptr: i32, start_index: i64, end_index: i64) -> i32 (new_vec_ptr)
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64, ValType::I64],
        &[ValType::I32],
    );
    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_INTERNAL_SUB_STRING,
            module_id,
        ))
        .func_body();

    // Arguments
    let vec_ptr = module.locals.add(ValType::I32);
    let start_idx = module.locals.add(ValType::I64);
    let end_idx = module.locals.add(ValType::I64);

    // Locals
    let sub_vec_ptr = module.locals.add(ValType::I32);
    let start_idx_i32 = module.locals.add(ValType::I32);
    let end_idx_i32 = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);
    let sub_len = module.locals.add(ValType::I32);

    // Load vec length
    builder
        .local_get(vec_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Cast indices to i32
    builder
        .local_get(start_idx)
        .unop(UnaryOp::I32WrapI64)
        .local_set(start_idx_i32);

    builder
        .local_get(end_idx)
        .unop(UnaryOp::I32WrapI64)
        .local_set(end_idx_i32);

    // Index Validation: check start < end < len
    builder
        .local_get(start_idx_i32)
        .local_get(end_idx_i32)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                // start > end
                then.unreachable();
            },
            |else_| {
                else_
                    .local_get(end_idx_i32)
                    .local_get(len)
                    .binop(BinaryOp::I32GtU)
                    .if_else(
                        None,
                        |then| {
                            // end > len
                            then.unreachable();
                        },
                        |_| {},
                    );
            },
        );

    // sub_len = end - start
    builder
        .local_get(end_idx_i32)
        .local_get(start_idx_i32)
        .binop(BinaryOp::I32Sub)
        .local_set(sub_len);

    // Allocate Memory for the substring vector
    let allocate_vector_with_header_fn =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;
    builder
        .local_get(sub_len)
        .local_get(sub_len)
        .i32_const(1)
        .call(allocate_vector_with_header_fn)
        .local_set(sub_vec_ptr);

    // Copy data into the substring vector
    builder
        .skip_vec_header(sub_vec_ptr)
        .vec_elem_ptr(vec_ptr, start_idx_i32, 1)
        .local_get(sub_len)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Return the pointer to the new vector
    builder.local_get(sub_vec_ptr).return_();

    Ok(function.finish(vec![vec_ptr, start_idx, end_idx], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_compilation_context;
    use crate::test_tools::INITIAL_MEMORY_OFFSET;
    use crate::test_tools::{
        build_module, get_linker_with_host_debug_functions, setup_wasmtime_module,
    };
    use rstest::rstest;
    use walrus::FunctionBuilder;

    /// Helper function to create vector data in memory format: [len, capacity, data...]
    /// This is the standard format for vectors in WASM memory:
    /// - 4 bytes: length (i32, little-endian)
    /// - 4 bytes: capacity (i32, little-endian)
    /// - N bytes: actual data
    fn create_vector_data(data: &[u8]) -> Vec<u8> {
        let len = data.len() as i32;
        let mut result = Vec::new();
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(data);
        result
    }

    #[rstest]
    #[case::ascii(b"hello world".to_vec(), true)]
    #[case::ascii(b"RustLang and MoveLang".to_vec(), true)]
    #[case::ascii(b"the brown fox jumps over the lazy dog 0123456789".to_vec(), true)]
    #[case::ascii(b"this is a valif UTF-8 in ASCII".to_vec(), true)]
    #[case::byte_length_2("Привет мир".as_bytes().to_vec(), true)]
    #[case::byte_length_2("ñáéíóú".as_bytes().to_vec(), true)]
    #[case::byte_length_2("çãõé".as_bytes().to_vec(), true)]
    #[case::byte_length_2("àèìòù".as_bytes().to_vec(), true)]
    #[case::byte_length_2("ÄÖÜß".as_bytes().to_vec(), true)]
    #[case::byte_length_3("こんにちは 世界".as_bytes().to_vec(), true)]
    #[case::byte_length_3("ありがとう".as_bytes().to_vec(), true)]
    #[case::byte_length_3("漢字テスト".as_bytes().to_vec(), true)]
    #[case::byte_length_3("中国语言".as_bytes().to_vec(), true)]
    #[case::byte_length_3("北京上海广州".as_bytes().to_vec(), true)]
    #[case::byte_length_4("😀😁😂🤣😃".as_bytes().to_vec(), true)]
    #[case::byte_length_4("🐉🐍🐢🦄🐬".as_bytes().to_vec(), true)]
    #[case::byte_length_4("🚀🚗🚲🚂✈️".as_bytes().to_vec(), true)]
    #[case::byte_length_4("🦊🐼🐧🐙🐠".as_bytes().to_vec(), true)]
    #[case::byte_length_4("📱💻🖥️⌚️🖨️".as_bytes().to_vec(), true)]
    #[case::length_mixture("Hello, 世界! 👋".as_bytes().to_vec(), true)]
    #[case::length_mixture("Hello ñá Café € ☃ 😀😁".as_bytes().to_vec(), true)]
    #[case::length_mixture("Rust Über façade ♥ ♪ 🚀 🐉".as_bytes().to_vec(), true)]
    #[case::length_mixture("Cache mañana piñata ∑ ∆ √ 😂 🤣".as_bytes().to_vec(), true)]
    #[case::length_mixture("Valid jalapeño naïve ☺ ☼ ← ↑ 😀 😃".as_bytes().to_vec(), true)]
    #[case::truncated_2_bytes(b"\xC3".to_vec(), false)]
    #[case::truncated_3_bytes(b"\xE2\x82".to_vec(), false)]
    #[case::truncated_4_bytes(b"\xF0\x90\x8D".to_vec(), false)]
    #[case::bad_continuation(b"\xE2\x28\xA1".to_vec(), false)]
    #[case::bad_continuation_2(b"\xC2\x41".to_vec(), false)]
    #[case::utf_16_surrogate(b"\xED\xA0\x80".to_vec(), false)]
    #[case::utf_16_surrogate_2(b"\xED\xBF\xBF".to_vec(), false)]
    #[case::out_of_range(b"\xF4\x90\x80\x80".to_vec(), false)]
    fn test_utf8_strings(#[case] string: Vec<u8>, #[case] expected: bool) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer) =
            build_module(None);
        let module_id = ModuleId::default();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let vec_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Pointer for the allocated string
        func_body.i32_const(INITIAL_MEMORY_OFFSET);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer);
        let check_utf8_f =
            add_internal_check_utf8(&mut raw_module, &compilation_ctx, &module_id).unwrap();

        func_body.call(check_utf8_f);

        let function = function_builder.finish(vec![vec_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // Prepare vector data: [len, capacity, data...]
        let vector_data = create_vector_data(&string);
        let linker = get_linker_with_host_debug_functions();
        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vector_data, "test_function", Some(linker));

        let result: i32 = entrypoint.call(&mut store, 0).unwrap();

        assert_eq!(
            result != 0,
            expected,
            "UTF-8 validation failed for string: {string:?}"
        );
    }

    #[rstest]
    // --- Basic ASCII ---
    #[case::ascii_start(b"abc".to_vec(), 0, true)]
    #[case::ascii_middle(b"abc".to_vec(), 1, true)]
    #[case::ascii_end(b"abc".to_vec(), 3, true)]
    #[case::ascii_out_of_bounds(b"abc".to_vec(), 4, false)]
    // --- 2-Byte Character (ñ = \xC3\xB1) ---
    #[case::utf8_2_start("ñ".as_bytes().to_vec(), 0, true)]
    #[case::utf8_2_cont("ñ".as_bytes().to_vec(), 1, false)]
    #[case::utf8_2_end("ñ".as_bytes().to_vec(), 2, true)]
    // --- 3-Byte Character (界 = \xE7\x95\x8C) ---
    #[case::utf8_3_start("界".as_bytes().to_vec(), 0, true)]
    #[case::utf8_3_cont_1("界".as_bytes().to_vec(), 1, false)]
    #[case::utf8_3_cont_2("界".as_bytes().to_vec(), 2, false)]
    #[case::utf8_3_end("界".as_bytes().to_vec(), 3, true)]
    // --- 4-Byte Character (😀 = \xF0\x9F\x98\x80) ---
    #[case::utf8_4_start("😀".as_bytes().to_vec(), 0, true)]
    #[case::utf8_4_cont_1("😀".as_bytes().to_vec(), 1, false)]
    #[case::utf8_4_cont_2("😀".as_bytes().to_vec(), 2, false)]
    #[case::utf8_4_cont_3("😀".as_bytes().to_vec(), 3, false)]
    #[case::utf8_4_end("😀".as_bytes().to_vec(), 4, true)]
    // --- Mixture ---
    #[case::mixture_boundary("Añ😀".as_bytes().to_vec(), 1, true)] // Start of 'ñ'
    #[case::mixture_not_boundary("Añ😀".as_bytes().to_vec(), 4, false)] // Middle of '😀'
    // --- Empty String ---
    #[case::empty_string(b"".to_vec(), 0, true)]
    fn test_is_char_boundary(#[case] string: Vec<u8>, #[case] index: i64, #[case] expected: bool) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer) =
            build_module(None);
        let module_id = ModuleId::default();

        // Input signature matches our builder: (i32, i64) -> i32
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I64],
            &[ValType::I32],
        );

        let vec_ptr = raw_module.locals.add(ValType::I32);
        let char_index = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        // The data is placed at INITIAL_MEMORY_OFFSET
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_get(char_index); // The index to check

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer);

        let is_char_boundary_f =
            add_internal_is_char_boundary(&mut raw_module, &compilation_ctx, &module_id).unwrap();

        func_body.call(is_char_boundary_f);

        let function = function_builder.finish(vec![vec_ptr, char_index], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // Prepare vector data: [len, capacity, data...]
        let vector_data = create_vector_data(&string);
        let linker = get_linker_with_host_debug_functions();
        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vector_data, "test_function", Some(linker));

        // Call with index as the second argument (the i64)
        // Note: index 0 in .call() is INITIAL_MEMORY_OFFSET, index 1 is the actual 'index'
        let result: i32 = entrypoint.call(&mut store, (0, index)).unwrap();

        assert_eq!(
            result != 0,
            expected,
            "Character boundary check failed for string {string:?} at index {index}"
        );
    }

    #[rstest]
    // --- Basic Matches ---
    #[case::simple_start(b"hello".to_vec(), b"he".to_vec(), 0)]
    #[case::simple_middle(b"hello".to_vec(), b"el".to_vec(), 1)]
    #[case::simple_end(b"hello".to_vec(), b"lo".to_vec(), 3)]
    #[case::full_match(b"hello".to_vec(), b"hello".to_vec(), 0)]
    // --- Non-Matches (Should return source length) ---
    #[case::not_found(b"hello".to_vec(), b"world".to_vec(), 5)]
    #[case::partial_prefix(b"hello".to_vec(), b"hellos".to_vec(), 5)] // Pattern longer than source
    #[case::case_sensitive(b"Hello".to_vec(), b"h".to_vec(), 5)]
    // --- Empty Cases ---
    #[case::empty_pattern(b"hello".to_vec(), b"".to_vec(), 0)]
    #[case::empty_source(b"".to_vec(), b"a".to_vec(), 0)]
    #[case::both_empty(b"".to_vec(), b"".to_vec(), 0)]
    // --- Multi-byte UTF-8 ---
    #[case::utf8_start("🦀rust".as_bytes().to_vec(), "🦀".as_bytes().to_vec(), 0)]
    #[case::utf8_middle("hello 界 world".as_bytes().to_vec(), "界".as_bytes().to_vec(), 6)]
    #[case::utf8_emoji("123😀456".as_bytes().to_vec(), "😀".as_bytes().to_vec(), 3)]
    // --- Overlapping / Repeating ---
    #[case::repeat_pattern(b"aaaaa".to_vec(), b"aa".to_vec(), 0)] // Finds first occurrence
    #[case::late_match(b"mississippi".to_vec(), b"ppi".to_vec(), 8)]
    fn test_index_of(#[case] source: Vec<u8>, #[case] pattern: Vec<u8>, #[case] expected: u64) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer) =
            build_module(None);
        let module_id = ModuleId::default();

        // Signature: (source_ptr: i32, pattern_ptr: i32) -> i64
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I64],
        );

        let src_ptr_local = raw_module.locals.add(ValType::I32);
        let pat_ptr_local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Calculate memory layout: place source and pattern vectors sequentially
        // Source vector size: 8 (header) + source.len()
        let source_vector_size = 8 + source.len();
        // Pattern vector starts after source vector with some padding for alignment
        let pattern_addr = INITIAL_MEMORY_OFFSET + source_vector_size as i32;

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.i32_const(pattern_addr);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer);

        let index_of_f =
            add_internal_index_of(&mut raw_module, &compilation_ctx, &module_id).unwrap();

        func_body.call(index_of_f);

        let function =
            function_builder.finish(vec![src_ptr_local, pat_ptr_local], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // Prepare data block: [Source Vector] ... [Pattern Vector]
        let mut memory_data = create_vector_data(&source);

        // Pad to reach pattern_addr
        let padding_needed = (pattern_addr - INITIAL_MEMORY_OFFSET) as usize - memory_data.len();
        memory_data.extend(vec![0u8; padding_needed]);

        // Append pattern vector
        memory_data.extend_from_slice(&create_vector_data(&pattern));

        let linker = get_linker_with_host_debug_functions();
        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, memory_data, "test_function", Some(linker));

        let result: i64 = entrypoint.call(&mut store, (0, 0)).unwrap();

        assert_eq!(
            result as u64, expected,
            "Index search failed: pattern {pattern:?} in source {source:?}"
        );
    }

    #[rstest]
    // --- Basic Slices ---
    #[case::middle(b"hello world".to_vec(), 0, 5, b"hello".to_vec())]
    #[case::start(b"rustlang".to_vec(), 0, 4, b"rust".to_vec())]
    #[case::end(b"move_lang".to_vec(), 5, 9, b"lang".to_vec())]
    // --- Multi-byte UTF-8 ---
    #[case::utf8_emoji("👋 hello".as_bytes().to_vec(), 0, 4, "👋".as_bytes().to_vec())]
    #[case::utf8_mixed("Café".as_bytes().to_vec(), 0, 3, "Caf".as_bytes().to_vec())]
    #[case::utf8_accent("Café".as_bytes().to_vec(), 3, 5, "é".as_bytes().to_vec())]
    // --- Edge Cases ---
    #[case::empty_slice(b"anything".to_vec(), 3, 3, b"".to_vec())]
    #[case::full_copy(b"copy_me".to_vec(), 0, 7, b"copy_me".to_vec())]
    fn test_sub_string(
        #[case] source: Vec<u8>,
        #[case] start: i64,
        #[case] end: i64,
        #[case] expected: Vec<u8>,
    ) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer) =
            build_module(None);
        let module_id = ModuleId::default();

        // Signature: (src_ptr: i32, start: i64, end: i64) -> i32
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I64, ValType::I64],
            &[ValType::I32],
        );

        let src_ptr_local = raw_module.locals.add(ValType::I32);
        let start_local = raw_module.locals.add(ValType::I64);
        let end_local = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        // Prepare call to sub_string - use the src_ptr parameter
        func_body.local_get(src_ptr_local);
        func_body.local_get(start_local);
        func_body.local_get(end_local);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer);

        let sub_string_f =
            add_internal_sub_string(&mut raw_module, &compilation_ctx, &module_id).unwrap();

        func_body.call(sub_string_f);

        let function = function_builder.finish(
            vec![src_ptr_local, start_local, end_local],
            &mut raw_module.funcs,
        );
        raw_module.exports.add("test_function", function);

        // Prepare source vector data: [len, capacity, data...]
        let source_data = create_vector_data(&source);

        let linker = get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            source_data.clone(),
            "test_function",
            Some(linker),
        );

        // Update the allocator's global pointer to point after the source data
        // so that allocations don't overwrite the source vector
        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();
        // Set it to point after the source data
        global_next_free_memory_pointer
            .set(
                &mut store,
                wasmtime::Val::I32(INITIAL_MEMORY_OFFSET + source_data.len() as i32),
            )
            .unwrap();

        // 1. Execute the function to get the pointer to the NEW vector
        // Pass INITIAL_MEMORY_OFFSET as the source pointer since that's where the data is placed
        let new_vec_ptr: i32 = entrypoint
            .call(&mut store, (INITIAL_MEMORY_OFFSET, start, end))
            .unwrap();

        // 2. Access WASM memory to verify the content of the new vector
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let data = memory.data(&store);

        // Read header from the returned pointer
        let actual_len = i32::from_le_bytes(
            data[new_vec_ptr as usize..new_vec_ptr as usize + 4]
                .try_into()
                .unwrap(),
        );

        // Read the actual bytes (skipping 8-byte header)
        let start_byte = (new_vec_ptr + 8) as usize;
        let end_byte = start_byte + actual_len as usize;
        let actual_bytes = &data[start_byte..end_byte];

        // Assertions
        assert_eq!(
            actual_len as usize,
            expected.len(),
            "Substring vector length mismatch"
        );
        assert_eq!(
            actual_bytes, expected,
            "Substring vector content mismatch for slice {start}..{end}",
        );
    }
}
