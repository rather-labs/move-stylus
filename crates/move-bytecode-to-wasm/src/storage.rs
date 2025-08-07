use crate::hostio::host_functions;
use crate::translation::intermediate_types::{heap_integers::IU256, simple_integers::IU32};
use crate::{CompilationContext, runtime::RuntimeFunction};
use alloy_primitives::keccak256;
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

// TODO
// - Revisit h(k), more important for string and bytes arrays
// - Reuse pointers

// The value corresponding to a mapping key k is located at keccak256(h(k) . p) where . is concatenation
// and h is a function that is applied to the key depending on its type:
// - for value types, h pads the value to 32 bytes in the same way as when storing the value in memory.
// - for strings and byte arrays, h(k) is just the unpadded data.
//
// If the mapping value is a non-value type, the computed slot marks the start of the data.
// If the value is of struct type, for example, you have to add an offset corresponding to the struct member to reach the member.

fn derive_mapping_slot_for_key(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    mapping_slot_ptr: LocalId, // pointer to the mapping slot (32 bytes)
    key_ptr: LocalId,          // pointer to the key (32 bytes)
    derived_slot_ptr: LocalId, // pointer to the derived slot (32 bytes)
) {
    let (native_keccak, _) = host_functions::native_keccak256(module);
    // let (storage_load_fn, _) = host_functions::storage_load_bytes32(module);
    // let (storage_cache_fn, _) = host_functions::storage_cache_bytes32(module);
    // let (storage_flush_cache_fn, _) = host_functions::storage_flush_cache(module);

    // Allocate memory for the hash data
    let data_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(64)
        .call(compilation_ctx.allocator)
        .local_set(data_ptr);

    // Concatenate the key and slot at #data_ptr -> data = h(k) . p
    let offset = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(offset);
    builder.block(None, |block| {
        let block_id = block.id();
        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            // if offset >= 32, break
            loop_
                .local_get(offset)
                .i32_const(32)
                .binop(BinaryOp::I32GeU)
                .br_if(block_id);

            loop_
                .local_get(data_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add) // data_ptr + offset
                .local_get(key_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add) // key_ptr + offset
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Load and store a slot chunk at #data_ptr + (i + 4) * 8
            loop_
                .local_get(data_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add) // data_ptr + offset
                .i32_const(32)
                .binop(BinaryOp::I32Add) // data_ptr + offset + 32
                .local_get(mapping_slot_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add) // mapping_slot_ptr + offset
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            loop_
                .local_get(offset)
                .i32_const(8)
                .binop(BinaryOp::I32Add)
                .local_set(offset);

            loop_.br(loop_id);
        });
    });

    // Hash the data, this is the mapping slot we are looking for -> v = keccak256(h(k) . p)
    builder
        .local_get(data_ptr)
        .i32_const(64)
        .local_get(derived_slot_ptr)
        .call(native_keccak);
}

/// Computes the storage slot for an element at a given index in a dynamic array,
/// where the array is stored at the slot pointed to by `array_slot_ptr`.
/// The derived element slot is stored at `derived_slot_ptr`.
///
/// Follows Solidity layout:
///   base = keccak256(p)
///   element_slot = base + i * elem_size
pub fn derive_dyn_array_slot_for_index(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    array_slot_ptr: LocalId, // pointer to the dynamic array's base slot (32 bytes)
    elem_index_ptr: LocalId, // pointer to the index of the element (u32)
    elem_size_ptr: LocalId,  // pointer to the size of the element in bytes (u32)
    derived_elem_slot_ptr: LocalId, // pointer to the derived slot of the element (32 bytes)
) {
    let (native_keccak, _) = host_functions::native_keccak256(module);
    let swap_i32_bytes_fn = RuntimeFunction::SwapI32Bytes.get(module, None);

    let base_slot_ptr = module.locals.add(ValType::I32);
    // Array data is located starting at keccak256(p)
    builder
        .local_get(array_slot_ptr)
        .i32_const(32)
        .local_get(base_slot_ptr)
        .call(native_keccak);

    // The amount of slots occupied by an element is equal to the element size in bytes divided by the slot size (32 bytes)
    // We add 1 to the result, because the amount of slots used cannot be 0
    builder
        .local_get(elem_size_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(32)
        .binop(BinaryOp::I32DivU)
        .i32_const(1)
        .binop(BinaryOp::I32Add);

    // Multiply the index by the amount of slots occupied by an element
    builder.local_get(elem_index_ptr).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );
    IU32::mul(builder, module);
    builder.call(swap_i32_bytes_fn); // swap the result to big-endian

    // Allocate memory for index * elem_size_in_slots as u256
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_index_ptr) // repurpose the index pointer to store the result
        .store(
            // store the result at the last 4 bytes
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        );

    // Add the base slot to the result
    builder.local_get(elem_index_ptr);
    builder.local_get(base_slot_ptr);
    IU256::add(builder, module, compilation_ctx);
    builder.local_set(derived_elem_slot_ptr);
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use rstest::rstest;
    use std::str::FromStr;
    use walrus::FunctionBuilder;

    use super::*;

    #[rstest]
    #[case(
        U256::from(1),
        U256::from(2),
        U256::from_str(
            "98521912898304110675870976153671229506380941016514884467413255631823579132687"
        ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(3),
        U256::from_str(
            "56988696150268759067033853745049141362335364605175666696514897554729450063371"
    ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(123456789),
        U256::from_str(
            "66492595055558910473828628519319372113473818625668867548228543292688569385097"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(2),
        U256::from_str(
            "46856049987324987851654180578118835177937932377897439695260177228387632849548"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(3),
        U256::from_str(
            "61684305963762951884865369267618438865725240706238913880678826931473020346819"
    ).unwrap()
    )]
    fn test_derive_mapping_slot_for_key(
        #[case] slot: U256,
        #[case] key: U256,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id) = build_module(Some(64));

        let slot_ptr = module.locals.add(ValType::I32);
        let key_ptr = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        let ctx = test_compilation_context!(memory_id, allocator_func);
        derive_mapping_slot_for_key(
            &mut func_body,
            &ctx,
            &mut module,
            slot_ptr,
            key_ptr,
            result_ptr,
        );

        func_body.local_get(result_ptr);
        let function = builder.finish(vec![slot_ptr, key_ptr], &mut module.funcs);
        module.exports.add("test_fn", function);

        // Create a linker with the required host functions
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::Linker::new(&engine);

        // Define the native_keccak256 function
        linker
            .func_wrap(
                "vm_hooks",
                "native_keccak256",
                |mut caller: wasmtime::Caller<'_, ()>,
                 input_data_ptr: u32,
                 data_length: u32,
                 return_data_ptr: u32| {
                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut input_data = vec![0; data_length as usize];
                    memory
                        .read(&caller, input_data_ptr as usize, &mut input_data)
                        .unwrap();

                    let hash = alloy_primitives::keccak256(input_data);

                    memory
                        .write(&mut caller, return_data_ptr as usize, hash.as_slice())
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        let data = vec![slot.to_be_bytes::<32>(), key.to_be_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        U256::from(1),
        U256::from(2),
        U256::from(4),
        U256::from_str(
            "23991499908108302765562531213920885141500505546388542086856722761454457053429"
        ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(5),
        U256::from(21),
        U256::from_str(
            "67151859839340103677100435873946963192465517128770968255452291644285690915775"
        ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(7),
        U256::from(28),
        U256::from_str(
            "70122961159721460691158963782174993504655102344268525554192115423808014779926"
        ).unwrap()
    )]
    fn test_derive_nested_mapping_slot_for_key(
        #[case] slot: U256,
        #[case] outer_key: U256,
        #[case] inner_key: U256,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id) = build_module(Some(96));

        let slot_ptr = module.locals.add(ValType::I32);
        let outer_key_ptr = module.locals.add(ValType::I32);
        let inner_key_ptr = module.locals.add(ValType::I32);

        // Allocate memory for the result
        let nested_mapping_slot_ptr = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        let ctx = test_compilation_context!(memory_id, allocator_func);
        derive_mapping_slot_for_key(
            &mut func_body,
            &ctx,
            &mut module,
            slot_ptr,
            outer_key_ptr,
            nested_mapping_slot_ptr,
        );

        derive_mapping_slot_for_key(
            &mut func_body,
            &ctx,
            &mut module,
            nested_mapping_slot_ptr,
            inner_key_ptr,
            result_ptr,
        );

        func_body.local_get(result_ptr);
        let function = builder.finish(vec![slot_ptr, outer_key_ptr, inner_key_ptr], &mut module.funcs);
        module.exports.add("test_fn", function);

        // Create a linker with the required host functions
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::Linker::new(&engine);

        // Define the native_keccak256 function
        linker
            .func_wrap(
                "vm_hooks",
                "native_keccak256",
                |mut caller: wasmtime::Caller<'_, ()>,
                 input_data_ptr: u32,
                 data_length: u32,
                 return_data_ptr: u32| {
                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut input_data = vec![0; data_length as usize];
                    memory
                        .read(&caller, input_data_ptr as usize, &mut input_data)
                        .unwrap();

                    let hash = alloy_primitives::keccak256(input_data);

                    memory
                        .write(&mut caller, return_data_ptr as usize, hash.as_slice())
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        let data = vec![slot.to_be_bytes::<32>(), outer_key.to_be_bytes::<32>(), inner_key.to_be_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 64)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }
}
