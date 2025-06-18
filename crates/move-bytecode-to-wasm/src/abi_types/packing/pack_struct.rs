use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::packing::pack_native_int::pack_i32_type_instructions,
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

use super::Packable;

impl IStruct {
    pub fn add_pack_instructions(
        index: u16,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let print_memory_from = module.imports.get_func("", "print_memory_from").unwrap();
        let print_i32 = module.imports.get_func("", "print_i32").unwrap();

        let struct_ = compilation_ctx.get_struct_by_index(index).unwrap();
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let struct_ptr = local;

        let calldata_ptr = module.locals.add(ValType::I32);
        let writer_ptr = module.locals.add(ValType::I32);

        let reference_value = module.locals.add(ValType::I32);

        let tmp = module.locals.add(ValType::I32);
        println!(
            "=== > {}",
            struct_.solidity_abi_encode_size(compilation_ctx)
        );

        /*
        block
            .local_get(calldata_reference_pointer)
            .call(print_memory_from);
        block.local_get(writer_pointer).call(print_memory_from);
        */

        // If the struct is dynamic, the space allocated for the struct is only 32 bytes long and we
        // need to save in calldata_reference_pointer the value pointing to the packed struct.
        //
        // If the struct is static, the space for packing it is already allocated, we just need to
        // pack its values
        if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
            // Allocate memory for the packed value. Set the writer pointer at the beginning, since
            // we are going to pack the values from there
            block
                .i32_const(struct_.solidity_abi_encode_size(compilation_ctx) as i32)
                .call(compilation_ctx.allocator)
                .local_tee(writer_ptr);

            // The pointer in the packed data must be relative to the calldata_reference_pointer,
            // so we substract calldata_reference_pointer from the writer_pointer
            block
                .local_get(calldata_reference_pointer)
                .binop(BinaryOp::I32Sub)
                .local_set(reference_value);

            // The result is saved where calldata_reference_pointer is pointing at, the value will
            // be the address where the struct  values are packed, using as origin
            // calldata_reference_pointer
            pack_i32_type_instructions(
                block,
                module,
                compilation_ctx.memory_id,
                reference_value,
                writer_pointer,
            );
            /*
            let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);
            block
                .local_get(calldata_reference_pointer)
                .local_get(struct_values_ptr)
                .call(swap_i32_bytes_function)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 28,
                    },
                );*/
        }

        for (index, field) in struct_.fields.iter().enumerate() {
            // Load field's intermediate pointer
            block.local_get(struct_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: index as u32 * 4,
                },
            );

            // Load the value
            let field_local = match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64 => {
                    let (val, load_kind) = if field.stack_data_size() == 8 {
                        (val_64, LoadKind::I64 { atomic: false })
                    } else {
                        (val_32, LoadKind::I32 { atomic: false })
                    };

                    block
                        .load(
                            compilation_ctx.memory_id,
                            load_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(val);

                    val
                }
                _ => {
                    block.local_set(val_32);
                    val_32
                }
            };

            if field.solidity_abi_encode_is_dynamic(compilation_ctx) {
                // block.local_get(writer_pointer).local_set(calldata_ptr);
                // block.local_get(writer_pointer).local_set(writer_ptr);

                // TODO: Check this sub...
                block
                    .local_get(writer_pointer)
                    .i32_const(32)
                    .binop(BinaryOp::I32Sub)
                    .local_set(calldata_ptr);
                block.local_get(writer_pointer).local_set(writer_ptr);
            } else {
                block
                    .local_get(calldata_reference_pointer)
                    .local_set(calldata_ptr);
                block.local_get(writer_pointer).local_set(writer_ptr);
            }

            // Pack field
            field.add_pack_instructions(
                block,
                module,
                field_local,
                writer_ptr,
                calldata_ptr,
                compilation_ctx,
            );

            block
                .i32_const(field.encoded_size(compilation_ctx) as i32)
                .local_get(writer_pointer)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        }

        block
            .local_get(calldata_reference_pointer)
            .call(print_memory_from);
    }
}
