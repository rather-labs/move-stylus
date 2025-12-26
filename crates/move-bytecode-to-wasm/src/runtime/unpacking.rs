pub(crate) mod bytes;
pub(crate) mod enums;
pub(crate) mod heap_uint;
pub(crate) mod reference;
pub(crate) mod string;
pub(crate) mod structs;
pub(crate) mod uint;
pub(crate) mod vector;

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use std::rc::Rc;
    use walrus::{ConstExpr, FunctionBuilder, ValType, ir::Value};
    use wasmtime::WasmResults;

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Test helper for unpacking simple integer types that fit in WASM value types
    fn unpack_uint<T: WasmResults + PartialEq + std::fmt::Debug>(
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
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

    /// Test helper for unpacking heap-allocated types (u128, u256, address)
    fn unpack_heap_uint(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                args_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);

        let global_next_free_memory_pointer = global_next_free_memory_pointer
            .get(&mut store)
            .i32()
            .unwrap();
        assert_eq!(
            global_next_free_memory_pointer,
            (expected_result_bytes.len() + data.len()) as i32
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    /// Test helper for unpacking vector types
    fn unpack_vec(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        int_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    /// Test helper for unpacking reference types
    fn unpack_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_memory_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_memory_bytes,
            "Heap memory at returned pointer does not match expected content"
        );
    }

    // ============================================================================
    // Simple Integer Types (u8, u16, u32, u64)
    // ============================================================================

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

    // ============================================================================
    // Heap-Allocated Types (u128, u256, address)
    // ============================================================================

    #[test]
    fn test_unpack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let data = SolType::abi_encode_params(&(88,));
        unpack_heap_uint(&data, int_type.clone(), &88u128.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_heap_uint(&data, int_type.clone(), &(IntType::MAX - 1).to_le_bytes());
    }

    #[test]
    fn test_unpack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let data = SolType::abi_encode_params(&(U256::from(88),));
        unpack_heap_uint(&data, int_type.clone(), &U256::from(88).to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        unpack_heap_uint(
            &data,
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
        );
    }

    #[test]
    fn test_unpack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let data = SolType::abi_encode_params(&(Address::ZERO,));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        unpack_heap_uint(&data, int_type.clone(), &data);
    }

    // ============================================================================
    // Vector Types - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u8_empty() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params::<(Vec<u8>,)>(&(vec![],));
        let expected_result_bytes =
            [0u32.to_le_bytes().as_slice(), 0u32.to_le_bytes().as_slice()].concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u8() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(vec![1, 2],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Vector Types - Heap-Allocated Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u256() {
        type SolType = sol!((uint256[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IU256));

        let data =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            U256::from(1).to_le_bytes::<32>().as_slice(),
            U256::from(2).to_le_bytes::<32>().as_slice(),
            U256::from(3).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_address() {
        type SolType = sol!((address[],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IAddress));

        let data = SolType::abi_encode_params(&(vec![
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
        ],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Nested Vector Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU32,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU128,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),                        // len
            2u32.to_le_bytes().as_slice(),                        // capacity
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),  // first element pointer
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),  // second element pointer
            3u32.to_le_bytes().as_slice(),                        // first element length
            3u32.to_le_bytes().as_slice(),                        // first element capacity
            ((data.len() + 36) as u32).to_le_bytes().as_slice(), // first element - first value pointer
            ((data.len() + 52) as u32).to_le_bytes().as_slice(), // first element - second value pointer
            ((data.len() + 68) as u32).to_le_bytes().as_slice(), // first element - third value pointer
            1u128.to_le_bytes().as_slice(),                      // first element - first value
            2u128.to_le_bytes().as_slice(),                      // first element - second value
            3u128.to_le_bytes().as_slice(),                      // first element - third value
            3u32.to_le_bytes().as_slice(),                       // second element length
            3u32.to_le_bytes().as_slice(),                       // second element capacity
            ((data.len() + 104) as u32).to_le_bytes().as_slice(), // second element - first value pointer
            ((data.len() + 120) as u32).to_le_bytes().as_slice(), // second element - second value pointer
            ((data.len() + 136) as u32).to_le_bytes().as_slice(), // second element - third value pointer
            4u128.to_le_bytes().as_slice(),                       // second element - first value
            5u128.to_le_bytes().as_slice(),                       // second element - second value
            6u128.to_le_bytes().as_slice(),                       // second element - third value
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Reference Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u8() {
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88u8,));
        let expected = 88u8.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u16() {
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(88u16,));
        let expected = 88u16.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u32() {
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        unpack_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        unpack_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u128() {
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123u128,));
        let expected = 123u128.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u256() {
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Rc::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let expected = value.to_le_bytes::<32>().to_vec();

        let data = SolType::abi_encode_params(&(value,));
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Rc::new(IntermediateType::IAddress));

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_ref(&data, ref_type.clone(), &data);
    }

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let vector_type = IntermediateType::IRef(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU8,
        ))));

        let vec_data = vec![1u8, 2u8, 3u8, 4u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&4u32.to_le_bytes()); // length
        expected.extend(&4u32.to_le_bytes()); // capacity
        expected.extend(&1u8.to_le_bytes()); // first elem
        expected.extend(&2u8.to_le_bytes()); // second elem
        expected.extend(&3u8.to_le_bytes()); // third elem
        expected.extend(&4u8.to_le_bytes()); // fourth elem
        unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_u128() {
        type SolType = sol!((uint128[],));
        let vector_type = IntermediateType::IRef(Rc::new(IntermediateType::IVector(Rc::new(
            IntermediateType::IU128,
        ))));

        let vec_data = vec![1u128, 2u128, 3u128];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&3u32.to_le_bytes()); // length
        expected.extend(&3u32.to_le_bytes()); // capacity
        // pointers to heap elements
        expected.extend(&180u32.to_le_bytes());
        expected.extend(&196u32.to_le_bytes());
        expected.extend(&212u32.to_le_bytes());
        expected.extend(&1u128.to_le_bytes());
        expected.extend(&2u128.to_le_bytes());
        expected.extend(&3u128.to_le_bytes());

        unpack_ref(&data, vector_type.clone(), &expected);
    }
}
