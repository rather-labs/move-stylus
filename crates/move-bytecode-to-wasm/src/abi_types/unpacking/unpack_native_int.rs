use alloy_sol_types::{SolType, sol_data};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        boolean::IBool,
        simple_integers::{IU8, IU16, IU32, IU64},
    },
};

impl IBool {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size = sol_data::Bool::ENCODED_SIZE.expect("Bool should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );
    }
}

impl IU8 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );
    }
}

impl IU16 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Uint::<16>::ENCODED_SIZE.expect("U16 should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );
    }
}

impl IU32 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Uint::<32>::ENCODED_SIZE.expect("U32 should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );
    }
}

impl IU64 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Uint::<64>::ENCODED_SIZE.expect("U64 should have a fixed size");
        unpack_i64_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );
    }
}

pub fn unpack_i32_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    reader_pointer: LocalId,
    encoded_size: usize,
) {
    // Load the value
    block.local_get(reader_pointer);
    block.load(
        memory,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 28,
        },
    );
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);
    block.call(swap_i32_bytes_function);

    // increment reader pointer
    block.local_get(reader_pointer);
    block.i32_const(encoded_size as i32);
    block.binop(BinaryOp::I32Add);
    block.local_set(reader_pointer);
}

pub fn unpack_i64_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    reader_pointer: LocalId,
    encoded_size: usize,
) {
    // Load the value
    block.local_get(reader_pointer);
    block.load(
        memory,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 24,
        },
    );
    // Big-endian to Little-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None);
    block.call(swap_i64_bytes_function);

    // increment reader pointer
    block.local_get(reader_pointer);
    block.i32_const(encoded_size as i32);
    block.binop(BinaryOp::I32Add);
    block.local_set(reader_pointer);
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use alloy_sol_types::sol;
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::WasmResults;

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    use super::*;

    fn test_uint<T: WasmResults + PartialEq + Debug>(
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Args data should already be stored in memory
        int_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            args_pointer,
            &compilation_ctx,
        );

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
        test_uint(int_type.clone(), &data, 88, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IU16;

        let data = SolType::abi_encode_params(&(1616,));
        test_uint(int_type.clone(), &data, 1616, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u32() {
        let int_type = IntermediateType::IU32;
        type IntType = u32;
        type SolType = sol!((uint32,));

        let data = SolType::abi_encode_params(&(323232,));
        test_uint(int_type.clone(), &data, 323232, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u64() {
        let int_type = IntermediateType::IU64;
        type IntType = u64;
        type SolType = sol!((uint64,));

        let data = SolType::abi_encode_params(&(6464646464,));
        test_uint(int_type.clone(), &data, 6464646464i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), &data, IntType::MAX as i64, ValType::I64); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), &data, IntType::MIN as i64, ValType::I64); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i64,
            ValType::I64,
        ); // max -1 (avoid symmetry)
    }
}
