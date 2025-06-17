use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

use super::Unpackable;

impl IStruct {
    pub fn add_unpack_instructions(
        index: usize,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let struct_ = compilation_ctx
            .module_structs
            .iter()
            .find(|s| s.index() == index as u16)
            .unwrap_or_else(|| panic!("struct that with index {index} not found"));

        if struct_.solidity_abi_encode_is_static(compilation_ctx) {
            Self::add_unpack_instructions_static_struct(
                struct_,
                builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );
        } else {
            Self::add_unpack_instructions_dynamic_struct(
                struct_,
                builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );
        }
    }

    fn add_unpack_instructions_static_struct(
        struct_: &IStruct,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let struct_ptr = module.locals.add(ValType::I32);
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let field_ptr = module.locals.add(ValType::I32);

        // Allocate space for the struct
        builder
            .i32_const(struct_.heap_size as i32)
            .call(compilation_ctx.allocator)
            .local_set(struct_ptr);

        let mut offset = 0;
        for field in &struct_.fields {
            // Unpack field
            field.add_unpack_instructions(
                builder,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );

            // If the field is stack type, we need to create the intermediate pointer, otherwise
            // the add_unpack_instructions function leaves the pointer in the stack
            match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64 => {
                    let data_size = field.stack_data_size();
                    let (val, store_kind) = if data_size == 8 {
                        (val_64, StoreKind::I64 { atomic: false })
                    } else {
                        (val_32, StoreKind::I32 { atomic: false })
                    };

                    // Save the actual value
                    builder.local_set(val);

                    // Create a pointer for the value
                    builder
                        .i32_const(data_size as i32)
                        .call(compilation_ctx.allocator)
                        .local_tee(field_ptr);

                    // Store the actual value behind the middle_ptr
                    builder.local_get(val).store(
                        compilation_ctx.memory_id,
                        store_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
                _ => {
                    builder.local_set(field_ptr);
                }
            }

            builder.local_get(struct_ptr).local_get(field_ptr).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg { align: 0, offset },
            );

            offset += 4;
        }

        builder.local_get(struct_ptr);
    }

    fn add_unpack_instructions_dynamic_struct(
        struct_: &IStruct,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let print_memory_from = module.imports.get_func("", "print_memory_from").unwrap();

        let struct_ptr = module.locals.add(ValType::I32);
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let field_ptr = module.locals.add(ValType::I32);

        // In a dynamic struct, the first value is where the values are packed in the calldata
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);
        let data_reader_pointer = module.locals.add(ValType::I32);
        let c_data_reader_pointer = module.locals.add(ValType::I32);

        builder
            .local_get(data_reader_pointer)
            .call(print_memory_from);

        for i in 0..7 {
            builder.block(None, |inner_block| {
                let inner_block_id = inner_block.id();

                inner_block.local_get(reader_pointer);
                inner_block.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        // Abi encoded value is Big endian
                        offset: i * 4,
                    },
                );
                inner_block.i32_const(0);
                inner_block.binop(BinaryOp::I32Eq);
                inner_block.br_if(inner_block_id);
                inner_block.unreachable();
            });
        }
        builder.local_get(reader_pointer);
        builder.load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                // Abi encoded value is Big endian
                offset: 28,
            },
        );
        builder.call(swap_i32_bytes_function);
        builder.local_get(calldata_reader_pointer);
        builder.binop(BinaryOp::I32Add);
        builder.local_tee(data_reader_pointer);
        builder.local_set(calldata_reader_pointer);

        // Allocate space for the struct
        builder
            .i32_const(struct_.heap_size as i32)
            .call(compilation_ctx.allocator)
            .local_set(struct_ptr);

        let mut offset = 0;
        for field in &struct_.fields {
            builder
                .local_get(data_reader_pointer)
                .call(print_memory_from);

            // Before unpacking a field we need to check if it is dynamically encoded in the abi.
            // If that's the case, we first need to read the value that points to the actual data.
            match field {
                IntermediateType::IVector(_) => {
                    /*
                    builder
                        .local_get(data_reader_pointer)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                // Abi encoded value is Big endian
                                offset: 28,
                            },
                        )
                        .call(swap_i32_bytes_function)
                        .local_get(data_reader_pointer)
                        .binop(BinaryOp::I32Add)
                        .local_set(data_reader_pointer);
                    */

                    /*
                    builder
                        .local_get(data_reader_pointer)
                        .local_set(c_data_reader_pointer);*/
                }
                // TODO: Check struct
                _ => {}
            }

            // Unpack field
            field.add_unpack_instructions(
                builder,
                module,
                data_reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );

            // If the field is stack type, we need to create the intermediate pointer, otherwise
            // the add_unpack_instructions function leaves the pointer in the stack
            match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64 => {
                    let data_size = field.stack_data_size();
                    let (val, store_kind) = if data_size == 8 {
                        (val_64, StoreKind::I64 { atomic: false })
                    } else {
                        (val_32, StoreKind::I32 { atomic: false })
                    };

                    // Save the actual value
                    builder.local_set(val);

                    // Create a pointer for the value
                    builder
                        .i32_const(data_size as i32)
                        .call(compilation_ctx.allocator)
                        .local_tee(field_ptr);

                    // Store the actual value behind the middle_ptr
                    builder.local_get(val).store(
                        compilation_ctx.memory_id,
                        store_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
                _ => {
                    builder.local_set(field_ptr);
                }
            }

            builder.local_get(struct_ptr).local_get(field_ptr).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg { align: 0, offset },
            );

            offset += 4;
        }

        builder.local_get(struct_ptr);
    }
}
