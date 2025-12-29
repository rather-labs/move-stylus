use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

pub fn unpack_u32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::UnpackU32.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size = module.locals.add(ValType::I32);

    // Load the value
    builder
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        )
        .call(swap_i32_bytes_function);

    // Set the global reader pointer to reader pointer + encoded size
    builder
        .local_get(reader_pointer)
        .local_get(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    Ok(function.finish(vec![reader_pointer, encoded_size], &mut module.funcs))
}

pub fn unpack_u64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    let mut builder = function
        .name(RuntimeFunction::UnpackU64.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<64>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)?;

    // Load the value
    builder
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 24,
            },
        )
        .call(swap_i64_bytes_function);

    // Increment reader pointer
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size as i32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::WasmResults;

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    /// Test helper for unpacking simple integer types that fit in WASM value types
    fn unpack_uint<T: WasmResults + PartialEq + std::fmt::Debug>(
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                args_pointer,
                args_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module::<_, T>(&mut raw_module, data.to_vec(), "test_function", None);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_unpack_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IU8;

        let data = SolType::abi_encode_params(&(88,));
        unpack_uint(int_type.clone(), &data, 88, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IU16;

        let data = SolType::abi_encode_params(&(1616,));
        unpack_uint(int_type.clone(), &data, 1616, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u32() {
        type IntType = u32;
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IU32;

        let data = SolType::abi_encode_params(&(323232,));
        unpack_uint(int_type.clone(), &data, 323232, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u64() {
        type IntType = u64;
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IU64;

        let data = SolType::abi_encode_params(&(6464646464,));
        unpack_uint(int_type.clone(), &data, 6464646464i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i64,
            ValType::I64,
        );
    }
}
