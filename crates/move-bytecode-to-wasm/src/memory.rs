use walrus::{
    ConstExpr, FunctionBuilder, FunctionId, MemoryId, Module, ValType,
    ir::{BinaryOp, Value},
};

const MEMORY_PAGE_SIZE: i32 = 65536;

/// Setup the module memory
/// This function adds the following components to the module:
/// * memory export
/// * global variables
/// * memory allocator function
///
/// This simple implementation assumes that memory is never freed,
/// As contract execution is short lived and we can afford memory leaks, as runtime will be restarted
///
/// Notes:
///     * Alignment is assumed to be 1 byte (no alignment)
///     * Alignment is not implemented in the current function
///     * Memory is allocated in pages of 64KiB
///     * Memory starts at offset 0
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

        block
            .local_get(requested_size)
            .global_get(global_available_memory)
            .binop(BinaryOp::I32Sub);

        // Memory delta (requested_size - available_memory)
        block.local_tee(memory_delta);

        // If memory delta is greater than 0, grow the memory
        block
            .i32_const(0)
            .binop(BinaryOp::I32LeS)
            .br_if(block_label);

        block.block(None, |block| {
            // Calculate grow pages
            block
                .local_get(memory_delta)
                .i32_const(MEMORY_PAGE_SIZE)
                .binop(BinaryOp::I32DivU);

            // Round up
            block
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_tee(grow_pages);

            // Grow the memory
            block.memory_grow(memory_id);

            // Panic if memory growth failed
            block.i32_const(0).binop(BinaryOp::I32GtS).if_else(
                None,
                |then_| {
                    // Update the global available memory
                    then_
                        .local_get(grow_pages)
                        .i32_const(MEMORY_PAGE_SIZE)
                        .binop(BinaryOp::I32Mul)
                        .global_get(global_available_memory)
                        .binop(BinaryOp::I32Add)
                        .global_set(global_available_memory);
                },
                |else_| {
                    // Panic
                    else_.unreachable();
                },
            );
        });
    });

    // Return the pointer to the allocated memory
    body.global_get(global_next_free_memory_pointer)
        .local_tee(memory_pointer)
        .local_get(requested_size)
        .binop(BinaryOp::I32Add)
        .global_set(global_next_free_memory_pointer);

    // Reduce the available memory
    body.global_get(global_available_memory)
        .local_get(requested_size)
        .binop(BinaryOp::I32Sub)
        .global_set(global_available_memory);

    body.local_get(memory_pointer);

    // Finish the function and add it to the module
    let func = func_builder.finish(vec![requested_size], &mut module.funcs);

    // export globals only for testing
    if cfg!(test) {
        // Function that resets memory
        let mut func_builder = FunctionBuilder::new(&mut module.types, &[], &[]);

        let mut builder = func_builder.name("reset_memory".to_owned()).func_body();

        // Wipe memory
        builder
            .i32_const(initial_offset.unwrap_or(0))
            .i32_const(0)
            .i32_const(MEMORY_PAGE_SIZE)
            .i32_const(initial_offset.unwrap_or(0))
            .binop(BinaryOp::I32Sub)
            .memory_fill(memory_id);

        // Reset globals
        builder
            .i32_const(initial_offset.unwrap_or(0))
            .global_set(global_next_free_memory_pointer);

        let reset_memoryn_func = func_builder.finish(vec![], &mut module.funcs);

        module.exports.add("reset_memory", reset_memoryn_func);

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
        let (mut raw_module, _, _, _) = build_module(None);

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
