use crate::hostio::host_functions;
use crate::translation::intermediate_types::heap_integers::IU256;
use crate::{CompilationContext, runtime::RuntimeFunction};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

// TODO
// - Revisit h(k), particularly for string and bytes arrays
// - Reuse pointers when possible

// The value corresponding to a mapping key k is located at keccak256(h(k) . p) where . is concatenation
// and h is a function that is applied to the key depending on its type:
// - for value types, h pads the value to 32 bytes in the same way as when storing the value in memory.
// - for strings and byte arrays, h(k) is just the unpadded data.
//
// If the mapping value is a non-value type, the computed slot marks the start of the data.
// If the value is of struct type, for example, you have to add an offset corresponding to the struct member to reach the member.

#[allow(dead_code)]
pub fn derive_mapping_slot_for_key(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    mapping_slot_ptr: LocalId, // pointer to the mapping slot (32 bytes)
    key_ptr: LocalId,          // pointer to the key (32 bytes)
    derived_slot_ptr: LocalId, // pointer to the derived slot (32 bytes)
) {
    let (native_keccak, _) = host_functions::native_keccak256(module);

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
/// following Solidity's storage layout:
///   base = keccak256(p)
///   element_slot = base + index * elem_size_in_slots
///
/// `array_slot_ptr` points to the u256 slot `p`.
/// `elem_index_ptr` points to the u32 element index value (little-endian).
/// `elem_size_ptr` points to the u32 element size in bytes (little-endian).
/// The resulting u256 big-endian slot value is stored at `derived_elem_slot_ptr`.
#[allow(dead_code)]
pub fn derive_dyn_array_slot_for_index(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    array_slot_ptr: LocalId,
    elem_index_ptr: LocalId,
    elem_size_ptr: LocalId,
    derived_elem_slot_ptr: LocalId,
) {
    let (native_keccak, _) = host_functions::native_keccak256(module);
    let swap_i32_bytes_fn = RuntimeFunction::SwapI32Bytes.get(module, None);

    // Allocate a local for the keccak256 result (base slot)
    let base_slot_ptr = module.locals.add(ValType::I32);

    // Compute base = keccak256(p)
    builder
        .local_get(array_slot_ptr)
        .i32_const(32)
        .local_get(base_slot_ptr)
        .call(native_keccak);

    // Check if the element size is greater lower than 32 bytes, i.e. it fits in a storage slot
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
        .binop(BinaryOp::I32LtU);
    builder.if_else(
        ValType::I32,
        |then| {
            // Case: Element fits within a single 32-byte (256-bit) storage slot
            //
            // Solidity packs multiple elements per slot when element size < 32 bytes.
            // We need to compute the slot offset where the element is stored:
            //
            // offset = floor(index / floor(32 / elem_size))
            //
            // Step 1: Load the index (u32)
            then.local_get(elem_index_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Step 2: Load the element size and compute divisor = floor(32 / elem_size)
            then.i32_const(32)
                .local_get(elem_size_ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .binop(BinaryOp::I32DivU);

            // Step 3: Compute offset = floor(index / divisor)
            then.binop(BinaryOp::I32DivU);
        },
        |else_| {
            // Case: Element does NOT fit within a single storage slot (elem_size ≥ 32 bytes)
            //
            // Solidity stores each element in full slots and does NOT pack them.
            // We compute how many slots each element needs:
            //
            // slots_per_element = ceil(elem_size / 32) = (elem_size + 31) / 32
            // offset = index * slots_per_element
            //
            // Step 1: Load the index (u32)
            else_.local_get(elem_index_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Step 2: Compute slots_per_element = (elem_size + 31) / 32
            else_
                .local_get(elem_size_ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .i32_const(31)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32DivU);

            // Step 3: Multiply to get offset = index * slots_per_element
            else_.binop(BinaryOp::I32Mul);
        },
    );

    // Convert to big-endian
    builder.call(swap_i32_bytes_fn);

    // Repurpose elem_size_ptr to hold the result (i.e., offset as I32)
    let elem_offset_32 = elem_size_ptr;
    builder.local_set(elem_offset_32);

    // Repurpose elem_index_ptr to allocate and hold the offset as U256
    let elem_offset_256_ptr = elem_index_ptr;
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_offset_256_ptr)
        .local_get(elem_offset_256_ptr)
        .local_get(elem_offset_32)
        // Store the u32 big-endian offset at the last 4 bytes of the memory to convert it to u256
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        );

    // Add base + offset → final element slot
    builder.local_get(elem_offset_256_ptr);
    builder.local_get(base_slot_ptr);
    IU256::add(builder, module, compilation_ctx);
    builder.local_set(derived_elem_slot_ptr);
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{
        build_module, get_linker_with_native_keccak256, setup_wasmtime_module,
    };
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

        let linker = get_linker_with_native_keccak256();

        let data = [slot.to_be_bytes::<32>(), key.to_be_bytes::<32>()].concat();
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
        let function = builder.finish(
            vec![slot_ptr, outer_key_ptr, inner_key_ptr],
            &mut module.funcs,
        );
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>(),
            outer_key.to_be_bytes::<32>(),
            inner_key.to_be_bytes::<32>(),
        ]
        .concat();
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

    #[rstest]
    #[case(
        U256::from(2),
        0_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        1_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        7_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        8_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403023"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        0_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037083"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        1_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037085"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        2_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037087"
    ).unwrap()
    )]
    fn test_derive_dyn_array_slot_for_index(
        #[case] slot: U256,
        #[case] index: u32,
        #[case] elem_size: u32,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id) = build_module(Some(40)); // slot (32 bytes) + index (4 bytes) + elem_size (4 bytes)

        let slot_ptr = module.locals.add(ValType::I32);
        let index_ptr = module.locals.add(ValType::I32);
        let elem_size_ptr = module.locals.add(ValType::I32);
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
        derive_dyn_array_slot_for_index(
            &mut func_body,
            &ctx,
            &mut module,
            slot_ptr,
            index_ptr,
            elem_size_ptr,
            result_ptr,
        );

        func_body.local_get(result_ptr);
        let function = builder.finish(vec![slot_ptr, index_ptr, elem_size_ptr], &mut module.funcs);
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>().to_vec(),
            index.to_le_bytes().to_vec(),
            elem_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 36)).unwrap();
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
        U256::from(2),
        0_u32,
        1_u32,
        4_u32,
        U256::from_str(
            "12072469696963966767691700411905649679726912322096881580412568241040270596576"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        1_u32,
        1_u32,
        4_u32,
        U256::from_str(
            "21317519515597955722743988462724083255677628835556397468395520694449519796017"
    ).unwrap()
    )]
    fn test_derive_nested_dyn_array_slot_for_index(
        #[case] slot: U256,
        #[case] outer_index: u32,
        #[case] inner_index: u32,
        #[case] elem_size: u32,
        #[case] expected: U256,
    ) {
        // slot (32 bytes) + outer_index (4 bytes) + inner_index (4 bytes) + elem_size (4 bytes)
        let (mut module, allocator_func, memory_id) = build_module(Some(44));

        let slot_ptr = module.locals.add(ValType::I32);
        let outer_index_ptr = module.locals.add(ValType::I32);
        let inner_index_ptr = module.locals.add(ValType::I32);
        let elem_size_ptr = module.locals.add(ValType::I32);
        let array_header_size_ptr = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        func_body // the header of the array occupies exactly 1 slot i.e. 32 bytes
            .i32_const(4)
            .call(allocator_func)
            .local_tee(array_header_size_ptr)
            .i32_const(32)
            .store(
                memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        let ctx = test_compilation_context!(memory_id, allocator_func);
        derive_dyn_array_slot_for_index(
            &mut func_body,
            &ctx,
            &mut module,
            slot_ptr,
            outer_index_ptr,
            array_header_size_ptr,
            result_ptr,
        );

        derive_dyn_array_slot_for_index(
            &mut func_body,
            &ctx,
            &mut module,
            result_ptr,
            inner_index_ptr,
            elem_size_ptr,
            result_ptr,
        );

        func_body.local_get(result_ptr);
        let function = builder.finish(
            vec![slot_ptr, outer_index_ptr, inner_index_ptr, elem_size_ptr],
            &mut module.funcs,
        );
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>().to_vec(),
            outer_index.to_le_bytes().to_vec(),
            inner_index.to_le_bytes().to_vec(),
            elem_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 36, 40)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }
}
