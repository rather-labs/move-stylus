use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{MemArg, StoreKind},
};

use crate::runtime::RuntimeFunction;

pub fn pack_i32_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    local: LocalId,
    writer_pointer: LocalId,
) {
    block.local_get(writer_pointer);

    // Load the local value to the stack
    block.local_get(local);

    // Little-endian to Big-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);
    block.call(swap_i32_bytes_function);

    block.store(
        memory,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 28,
        },
    );
}

pub fn pack_i64_type_instructions(
    block: &mut InstrSeqBuilder,
    module: &mut Module,
    memory: MemoryId,
    local: LocalId,
    writer_pointer: LocalId,
) {
    block.local_get(writer_pointer);

    // Load the local value to the stack
    block.local_get(local);

    // Little-endian to Big-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None);
    block.call(swap_i64_bytes_function);

    block.store(
        memory,
        StoreKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // Abi is left-padded to 32 bytes
            offset: 24,
        },
    );
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    enum Int {
        U32(u32),
        U64(u64),
    }

    fn test_uint(int_type: impl Packable, literal: Int, expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, alloc_function);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let local = match literal {
            Int::U32(literal) => {
                func_body.i32_const(literal as i32);
                raw_module.locals.add(ValType::I32)
            }
            Int::U64(literal) => {
                func_body.i64_const(literal as i64);
                raw_module.locals.add(ValType::I64)
            }
        };
        func_body.local_set(local);

        let writer_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(int_type.encoded_size(&compilation_ctx) as i32);
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        // Args data should already be stored in memory
        int_type.add_pack_instructions(
            &mut func_body,
            &mut raw_module,
            local,
            writer_pointer,
            writer_pointer, // unused for this type
            &compilation_ctx,
        );

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        // the return is the pointer to the packed value
        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IU8;

        let expected_result = SolType::abi_encode_params(&(88,));
        test_uint(int_type.clone(), Int::U32(88), &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            Int::U32(IntType::MAX as u32),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            Int::U32(IntType::MIN as u32),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            Int::U32((IntType::MAX - 1) as u32),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IU16;

        let expected_result = SolType::abi_encode_params(&(1616,));
        test_uint(int_type.clone(), Int::U32(1616), &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            Int::U32(IntType::MAX as u32),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            Int::U32(IntType::MIN as u32),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            Int::U32((IntType::MAX - 1) as u32),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u32() {
        type IntType = u32;
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IU32;

        let expected_result = SolType::abi_encode_params(&(323232,));
        test_uint(int_type.clone(), Int::U32(323232), &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), Int::U32(IntType::MAX), &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), Int::U32(IntType::MIN), &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            Int::U32(IntType::MAX - 1),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u64() {
        type IntType = u64;
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IU64;

        let expected_result = SolType::abi_encode_params(&(6464646464,));
        test_uint(int_type.clone(), Int::U64(6464646464), &expected_result);

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(int_type.clone(), Int::U64(IntType::MAX), &expected_result); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(int_type.clone(), Int::U64(IntType::MIN), &expected_result); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            Int::U64(IntType::MAX - 1),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }
}
