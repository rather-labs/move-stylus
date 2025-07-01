//! According to the formal specification of the encoding, a tuple (T1,...,Tk) is dynamic if
//! Ti is dynamic for some 1 <= i <= k.
//!
//! Since structs are encoded as tuples of their fields, a struct is also considered dynamic
//! if any of its fields is dynamic.
//!
//! Based on the ABI specification, the following types are considered dynamic:
//! - bytes
//! - string
//! - T[] for any T
//! - T[k] for any dynamic T and any k >= 0
//! - (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k
//!
//! For example, the following Move's struct:
//!
//! public struct Foo has drop {
//!    x: u8,
//!    y: vector<u32>,
//!    z: vector<u128>,
//! }
//!
//! Is equivalent to the following struct in Solidity:
//!
//! struct Foo {
//!     uint8 x;
//!     uint32[] y;
//!     uint128[] z;
//! }
//!
//! Given that the struct contains vectors, it becomes a dynamic. This means that the first encoded
//! value of this struct will be a number pointing to where the values are packed in the calldata.
//!
//! If we call a function that have Foo as an argument with:
//! Foo {
//!     x: 254,
//!     y: [1, 2, u32::MAX],
//!     z: [1, 2, u128::MAX],
//! }
//!
//! The encoded data will be:
//! bytes   0..3      4..35   36..67   68..99   100..131
//!       [selector,   32  ,   254   ,   96   ,   224  , [3,1,2,u32::MAX], [3,1,2,u128::MAX]]
//!                 ptr_foo  ▲  x       ptr_y    ptr_z   ▲                 ▲
//!                    │     │           │         │     │                 │
//!                    └─────┘           └─────────┼─────┘                 │
//!                                                └───────────────────────┘
//! where
//!  - selector: the called function selector
//!
//!  - ptr_foo: where the Foo struct's values are packed in the calldata. It is 32 because it does
//!    not take in account the selector.  36 = len(selector) + len(ptr_foo) = 4 + 32,
//!    where the packed data starts
//!
//!  - x: 254 packed as uint8 (32 bytes)
//!
//!  - ptr_y: where the y's vector values are packed. It does not take in account the selector and
//!    ptr_foo. 96 = len(x) + len(ptr_y) + len(ptr_z) = 32 + 32 + 32
//!
//!  - ptr_z: where the z's vector values are packed. It does not take in account the selector and
//!    ptr_foo. 224 = len(x) + len(ptr_y) + len(ptr_z) + y_data = 32 + 32 + 32 + 128.
//!    y_data has length 128 because it contains its length (32 bytes) and 3 elements (3 x 32bytes)
//!
//! If a struct does not contain any dynamic fields, all its fields are encoded inline, packed
//! contiguously without any offset or pointer.
//!
//! For more information:
//! https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encoding
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
        &self,
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

        // Moving pointer for reading data of the fields
        let data_reader_pointer = module.locals.add(ValType::I32);

        // Pointer to where the struct is packed
        let calldata_ptr = module.locals.add(ValType::I32);

        // In a dynamic struct, the first value is where the values are packed in the calldata
        if self.solidity_abi_encode_is_dynamic(compilation_ctx) {
            // Big-endian to Little-endian
            let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);

            // We are just assuming that the max value can fit in 32 bits, otherwise we cannot
            // reference WASM memory. If the value is greater than 32 bits, the WASM program
            // will panic.
            for i in 0..7 {
                builder.block(None, |inner_block| {
                    let inner_block_id = inner_block.id();

                    inner_block
                        .local_get(reader_pointer)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                // Abi encoded value is Big endian
                                offset: i * 4,
                            },
                        )
                        .i32_const(0)
                        .binop(BinaryOp::I32Eq)
                        .br_if(inner_block_id)
                        .unreachable();
                });
            }

            builder
                .local_get(reader_pointer)
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
                .local_get(calldata_reader_pointer)
                .binop(BinaryOp::I32Add)
                .local_tee(data_reader_pointer)
                .local_set(calldata_ptr);
        } else {
            builder
                .local_get(reader_pointer)
                .local_set(data_reader_pointer)
                .local_get(calldata_reader_pointer)
                .local_set(calldata_ptr);
        }

        // Allocate space for the struct
        builder
            .i32_const(self.heap_size as i32)
            .call(compilation_ctx.allocator)
            .local_set(struct_ptr);

        let mut offset = 0;
        for field in &self.fields {
            // Unpack field
            field.add_unpack_instructions(
                builder,
                module,
                data_reader_pointer,
                calldata_ptr,
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

        // Advance reader pointer after processing struct.
        // If it is a static struct, the pointer must be advanced the size of the tuple that
        // represents the struct.
        // If it is a dynamic struct, we just need to advance the pointer 32 bytes because in the
        // argument's place there is only a pointer to where the values of the struct are packed
        let advancement = if self.solidity_abi_encode_is_dynamic(compilation_ctx) {
            32
        } else {
            self.solidity_abi_encode_size(compilation_ctx) as i32
        };

        builder
            .local_get(reader_pointer)
            .i32_const(advancement)
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        builder.local_get(struct_ptr);
    }
}
