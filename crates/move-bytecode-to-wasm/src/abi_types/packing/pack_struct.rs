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
//! If we call a function that returns Foo with the following fields:
//! Foo {
//!     x: 254,
//!     y: [1, 2, u32::MAX],
//!     z: [1, 2, u128::MAX],
//! }
//!
//! If the function only returns Foo, the struct will be statically exapnded as a tuple. The
//! encoded data will be:
//!
//! bytes   0..31    32..63   64..95
//!       [  254  ,    96   ,   224  , [3,1,2,u32::MAX], [3,1,2,u128::MAX]]
//!           x       ptr_y    ptr_z   ▲                 ▲
//!                    │         │     │                 │
//!                    └─────────┼─────┘                 │
//!                              └───────────────────────┘
//! where
//!  - x: 254 packed as uint8 (32 bytes)
//!
//!  - ptr_y: where the y's vector values are packed. It is relative to where the tuple starts
//!    96 = len(x) + len(ptr_y) + len(ptr_z) = 32 + 32 + 32
//!
//!  - ptr_z: where the z's vector values are packed. It is relative to where the tuple starts.
//!    224 = len(x) + len(ptr_y) + len(ptr_z) + y_data = 32 + 32 + 32 + 128.
//!    y_data has length 128 because it contains its length (32 bytes) and 3 elements (3 x 32bytes)
//!
//!
//! If the function returns Foo along with other values, for example, (u16, Foo, vector<128>) where
//! the u16 is 42 and the vector<u128> is [1,2,3]. The encoded data will be:
//!
//! The encoded data will be:
//! bytes   0..31   32..63   64..95       96..n             n..
//!       [  42   ,   96  ,    n   ,  foo_packed_data , vector_packed_data ]
//!          u16    ptr_foo  ptr_vec        ▲                  ▲
//!                    │        │           │                  │
//!                    └────────┼───────────┘                  │
//!                             └──────────────────────────────┘
//!
//! where:
//!
//!  - u16: The actual value of the u16 field
//!
//!  - ptr_foo: where Foo's values are packed. It is relative to where the tuple starts
//!    96 = len(x) + len(ptr_foo) + len(ptr_vec) = 32 + 32 + 32
//!
//!  - ptr_vec: where vector<u128> values are packed
//!
//! If a struct does not contain any dynamic fields, all its fields are encoded inline, packed
//! contiguously without any offset or pointer.
//!
//! For more information:
//! https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encodinguse walrus::{
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    abi_types::packing::pack_native_int::pack_i32_type_instructions,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

use super::Packable;

impl IStruct {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pack_instructions(
        &self,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
        base_calldata_reference_pointer: Option<LocalId>,
    ) {
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let struct_ptr = local;
        let reference_value = module.locals.add(ValType::I32);

        let data_ptr = module.locals.add(ValType::I32);
        let inner_data_reference = module.locals.add(ValType::I32);

        // If base_calldata_reference_ptr is Some(_), means we are packing an struct inside a
        // struct and that the struct is dynamic.
        // base_calldata_reference_pointer is the reference pointer to the original value, and it
        // is used to calculate the offset where the struct will be allocated in the parent struct.
        // The calculated offset will be written in the place where the struct should be.
        if let Some(base_calldata_reference_ptr) = base_calldata_reference_pointer {
            // Allocate memory for the packed value. Set the data_ptr the beginning, since
            // we are going to pack the values from there
            block
                .i32_const(self.solidity_abi_encode_size(compilation_ctx) as i32)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr)
                .local_tee(inner_data_reference);

            // The pointer in the packed data must be relative to the calldata_reference_pointer,
            // so we substract calldata_reference_pointer from the writer_pointer
            block
                .local_get(base_calldata_reference_ptr)
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
        } else {
            block.local_get(writer_pointer).local_set(data_ptr);
        }

        // Load the value to be written in the calldata, if it is a stack value we need to double
        // reference a pointer, otherwise we read the pointer and leave the stack value in the
        // stack
        for (index, field) in self.fields.iter().enumerate() {
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

            // If base_calldata_reference_pointer is none, means we are not packing this struct
            // dynamically, so, we can set inner_data_reference as the root reference pointer
            if base_calldata_reference_pointer.is_none() {
                block
                    .local_get(calldata_reference_pointer)
                    .local_set(inner_data_reference);
            }

            // If the field to pack is a struct, it will be packed dynamically, that means, in the
            // current offset of writer pointer, we are going to write the offset where we can find
            // the struct
            let advancement = match field {
                IntermediateType::IStruct(index) => {
                    let child_struct = compilation_ctx
                        .root_module_data
                        .structs
                        .get_by_index(*index)
                        .unwrap();
                    if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx) {
                        child_struct.add_pack_instructions(
                            block,
                            module,
                            field_local,
                            data_ptr,
                            inner_data_reference,
                            compilation_ctx,
                            Some(inner_data_reference),
                        );
                        32
                    } else {
                        child_struct.add_pack_instructions(
                            block,
                            module,
                            field_local,
                            data_ptr,
                            inner_data_reference,
                            compilation_ctx,
                            None,
                        );
                        field.encoded_size(compilation_ctx)
                    }
                }
                IntermediateType::IGenericStructInstance(index, types) => {
                    let child_struct = compilation_ctx
                        .root_module_data
                        .structs
                        .get_by_index(*index)
                        .unwrap();
                    let child_struct_instance = child_struct.instantiate(types);

                    if child_struct_instance.solidity_abi_encode_is_dynamic(compilation_ctx) {
                        child_struct_instance.add_pack_instructions(
                            block,
                            module,
                            field_local,
                            data_ptr,
                            inner_data_reference,
                            compilation_ctx,
                            Some(inner_data_reference),
                        );
                        32
                    } else {
                        child_struct_instance.add_pack_instructions(
                            block,
                            module,
                            field_local,
                            data_ptr,
                            inner_data_reference,
                            compilation_ctx,
                            None,
                        );
                        field.encoded_size(compilation_ctx)
                    }
                }
                _ => {
                    field.add_pack_instructions(
                        block,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                    );
                    32
                }
            };

            // The value of advacement depends on the following conditions:
            // - If the field we are encoding is a static struct, the pointer must be advanced the size
            //   of the tuple that represents the struct.
            // - If the field we are encoding is a dynamic struct, we just need to advance the pointer
            //   32 bytes because in the argument's place there is only a pointer to where the
            //   struct's values are packed
            // - If it is not a struct:
            //   - If it is a static field it will occupy 32 bytes,
            //   - if it is a dynamic field, the offset pointing to where to find the values will be
            //     written, also occuping 32 bytes.
            block
                .i32_const(advancement as i32)
                .local_get(data_ptr)
                .binop(BinaryOp::I32Add)
                .local_set(data_ptr);
        }
    }
}
