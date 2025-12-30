use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg},
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

    let swap_i32_fn = RuntimeFunction::SwapI32Bytes.get(module, None)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_compilation_context;
    use crate::test_tools::{
        build_module, get_linker_with_host_debug_functions, setup_wasmtime_module,
    };
    use rstest::rstest;
    use walrus::FunctionBuilder;

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
        func_body.i32_const(0);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer);
        let check_utf8_f =
            add_internal_check_utf8(&mut raw_module, &compilation_ctx, &module_id).unwrap();

        func_body.call(check_utf8_f);

        let function = function_builder.finish(vec![vec_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // We save length, capacity and the data
        let data = [
            &(string.len() as i32).to_le_bytes(),
            &(string.len() as i32).to_le_bytes(),
            string.as_slice(),
        ]
        .concat();
        let linker = get_linker_with_host_debug_functions();
        let (_, _, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let result: i32 = entrypoint.call(&mut store, 0).unwrap();

        assert_eq!(result != 0, expected);
    }
}
