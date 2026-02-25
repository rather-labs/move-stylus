use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn unpack_bytes_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::UnpackBytes.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);

    // Advance the reader pointer by 32
    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(reader_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
    };

    #[test]
    fn test_unpack_bytes_fuzz() {
        type SolType = sol!((bytes,));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(1024));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        // Call unpack_bytes
        let unpack_bytes = crate::runtime::RuntimeFunction::UnpackBytes
            .get(&mut raw_module, Some(&compilation_ctx), None)
            .unwrap();

        func_body.local_get(calldata_reader_pointer);
        func_body.call(unpack_bytes);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 1024], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u8>>()
            .cloned()
            .for_each(|byte_vec: Vec<u8>| {
                // In Solidity ABI, bytes is encoded as a dynamic array
                let data = SolType::abi_encode_params(&(byte_vec.clone(),));
                if data.len() > 1024 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                // Read the 32-byte offset word
                let mut offset_bytes = [0u8; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut offset_bytes,
                    )
                    .unwrap();

                // The offset is stored as big-endian u32 in the last 4 bytes
                let offset = u32::from_be_bytes([
                    offset_bytes[28],
                    offset_bytes[29],
                    offset_bytes[30],
                    offset_bytes[31],
                ]) as usize;

                // Follow the offset to read the length
                let length_offset = INITIAL_MEMORY_OFFSET as usize + offset;
                let mut length_bytes = [0u8; 32];
                memory
                    .read(&mut *store.0.borrow_mut(), length_offset, &mut length_bytes)
                    .unwrap();

                let length = u32::from_be_bytes([
                    length_bytes[28],
                    length_bytes[29],
                    length_bytes[30],
                    length_bytes[31],
                ]) as usize;

                assert_eq!(
                    length,
                    byte_vec.len(),
                    "Unpacked bytes length did not match"
                );

                // Read the actual bytes data
                let data_offset = length_offset + 32;
                let mut bytes_data = vec![0u8; length];
                memory
                    .read(&mut *store.0.borrow_mut(), data_offset, &mut bytes_data)
                    .unwrap();

                assert_eq!(
                    bytes_data, byte_vec,
                    "Unpacked bytes data did not match original byte vector"
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}
