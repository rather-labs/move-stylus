use walrus::{
    ConstExpr, FunctionBuilder, FunctionId, MemoryId, Module, ValType,
    ir::{BinaryOp, Value},
};

const MEMORY_PAGE_SIZE: i32 = 65536;

/// Setup the module memory
/// This function adds the following components to the module:
/// - memory export
/// - global variables
/// - memory allocator function
///
/// This simple implementation assumes that memory is never freed,
/// As contract execution is short lived and we can afford memory leaks, as runtime will be restarted
///
/// Notes:
///     - Alignment is assumed to be 1 byte (no alignment)
///     - Alignment is not implemented in the current function
///     - Memory is allocated in pages of 64KiB
///     - Memory starts at offset 0
pub fn setup_module_memory(
    module: &mut Module,
    initial_offset: Option<i32>,
) -> (FunctionId, MemoryId) {
    let memory_id = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory_id);

    let global_next_free_memory_pointer = module.globals.add_local(
        ValType::I32,
        true,
        false,
        ConstExpr::Value(Value::I32(initial_offset.unwrap_or(0))),
    );

    let global_available_memory = module.globals.add_local(
        ValType::I32,
        true,
        false,
        ConstExpr::Value(Value::I32(MEMORY_PAGE_SIZE)),
    );

    let mut func_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let requested_size = module.locals.add(ValType::I32);
    let memory_delta = module.locals.add(ValType::I32);
    let grow_pages = module.locals.add(ValType::I32);
    let memory_pointer = module.locals.add(ValType::I32);
    let mut body = func_builder.func_body();

    // If there is not enough memory, grow the memory
    body.block(None, |block| {
        let block_label = block.id();

        block.local_get(requested_size);
        block.global_get(global_available_memory);
        block.binop(BinaryOp::I32Sub);
        // Memory delta (requested_size - available_memory)
        block.local_tee(memory_delta);
        // If memory delta is greater than 0, grow the memory
        block.i32_const(0);
        block.binop(BinaryOp::I32LeS);
        block.br_if(block_label);
        block.block(None, |block| {
            // Calculate grow pages
            block.local_get(memory_delta);
            block.i32_const(MEMORY_PAGE_SIZE);
            block.binop(BinaryOp::I32DivU);
            // Round up
            block.i32_const(1);
            block.binop(BinaryOp::I32Add);
            block.local_tee(grow_pages);

            // Grow the memory
            block.memory_grow(memory_id);
            // Panic if memory growth failed
            block.i32_const(0);
            block.binop(BinaryOp::I32GtS);
            block.if_else(
                None,
                |block| {
                    // Update the global available memory
                    block.local_get(grow_pages);
                    block.i32_const(MEMORY_PAGE_SIZE);
                    block.binop(BinaryOp::I32Mul);
                    block.global_get(global_available_memory);
                    block.binop(BinaryOp::I32Add);
                    block.global_set(global_available_memory);
                },
                |block| {
                    // Panic
                    block.unreachable();
                },
            );
        });
    });

    // Return the pointer to the allocated memory
    body.global_get(global_next_free_memory_pointer);
    body.local_tee(memory_pointer);
    body.local_get(requested_size);
    body.binop(BinaryOp::I32Add);
    body.global_set(global_next_free_memory_pointer);

    // Reduce the available memory
    body.global_get(global_available_memory);
    body.local_get(requested_size);
    body.binop(BinaryOp::I32Sub);
    body.global_set(global_available_memory);

    body.local_get(memory_pointer);

    // Finish the function and add it to the module
    let func = func_builder.finish(vec![requested_size], &mut module.funcs);

    // export globals only for testing
    if cfg!(test) {
        module
            .exports
            .add("available_memory", global_available_memory);

        module.exports.add("allocator", func);
        module.exports.add(
            "global_next_free_memory_pointer",
            global_next_free_memory_pointer,
        );
    }

    (func, memory_id)
}

#[cfg(test)]
mod tests {
    use crate::test_tools::build_module;

    use super::*;

    use wasmtime::{Engine, Instance, Module as WasmModule, Store};

    #[test]
    fn test_memory_allocator() {
        let (mut raw_module, _, _) = build_module(None);

        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &raw_module.emit_wasm()).unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let allocator = instance
            .get_typed_func::<i32, i32>(&mut store, "allocator")
            .unwrap();

        let memory_size = instance.get_memory(&mut store, "memory").unwrap();
        let available_memory = instance.get_global(&mut store, "available_memory").unwrap();

        let result = allocator.call(&mut store, 2).unwrap();
        assert_eq!(result, 0);
        assert_eq!(memory_size.size(&mut store), 1);
        assert_eq!(
            available_memory.get(&mut store).i32().unwrap(),
            MEMORY_PAGE_SIZE - 2
        );

        let result = allocator.call(&mut store, 2).unwrap();
        assert_eq!(result, 2);
        assert_eq!(memory_size.size(&mut store), 1);
        assert_eq!(
            available_memory.get(&mut store).i32().unwrap(),
            MEMORY_PAGE_SIZE - 4
        );

        let result = allocator.call(&mut store, MEMORY_PAGE_SIZE - 4).unwrap();
        assert_eq!(result, 4);
        assert_eq!(memory_size.size(&mut store), 1);
        assert_eq!(available_memory.get(&mut store).i32().unwrap(), 0);

        let result = allocator.call(&mut store, 2).unwrap();
        assert_eq!(result, 65536);
        assert_eq!(memory_size.size(&mut store), 2);
        assert_eq!(
            available_memory.get(&mut store).i32().unwrap(),
            MEMORY_PAGE_SIZE - 2
        );
    }
}
