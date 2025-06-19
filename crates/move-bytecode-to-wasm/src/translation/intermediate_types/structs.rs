//! Represents a struct type in Move.
//!
//! All the fields within a struct are contigously packed in memory. Regardless of whether a
//! field's type is stored on the stack or heap, the struct always stores a pointer to the
//! actual data, not the value itself. For example:
//! ```move
//! public struct Foo { a:u16, b:u128 };
//! ```
//!
//! When packing this struct, the memory layout, starting from address 0 will be:
//!
//! addess:  0..3   4..7   8..12   12..28
//! size  :   4       4      4     16
//! offset:   0       4      8     12
//!         [ptr_a, ptr_b,   a,    b    ]
//!           │      │       ▲     ▲
//!           └──────┼───────┘     │
//!                  └─────────────┘
//!
//! The reason why simple (stack) types are behind pointers is because when accesing fields of a
//! struct, it is always done through a reference:
//!
//! ```move
//! public fun echo(): u16 {
//!    let foo = Foo {
//!        a: 42,
//!        b: 314,
//!    };
//!
//!   foo.a
//! }
//! ```
//!
//! The line `foo.a` generates this Move bytecode:
//! ```text
//! ...
//! ImmBorrowLoc(0),
//! ImmBorrowField(FieldHandleIndex(0)),
//! ReadRef,
//! ...
//! ```
//!
//! Because fields are always accessed via references, using pointers uniformly (even for simple
//! values) simplifies the implementation, reduces special-case logic, and ensures consistent
//! field management across all types.
use std::collections::HashMap;

use crate::CompilationContext;

use super::IntermediateType;
use move_binary_format::file_format::{FieldHandleIndex, StructDefinitionIndex};
use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{LoadKind, MemArg},
};

#[derive(Debug)]
pub struct IStruct {
    /// Field's types ordered by index
    pub fields: Vec<IntermediateType>,

    /// Map between handles and fields types
    pub fields_types: HashMap<FieldHandleIndex, IntermediateType>,

    /// Map between handles and fields offset
    pub field_offsets: HashMap<FieldHandleIndex, u32>,

    /// Move's struct index
    pub struct_definition_index: StructDefinitionIndex,

    /// How much memory this struct occupies (in bytes). This will be the quantity of fields *4
    /// because we save pointers for all data types (stack or heap).
    pub heap_size: u32,
}

impl IStruct {
    pub fn new(
        index: StructDefinitionIndex,
        fields: Vec<(Option<FieldHandleIndex>, IntermediateType)>,
        fields_types: HashMap<FieldHandleIndex, IntermediateType>,
    ) -> Self {
        let mut heap_size = 0;
        let mut field_offsets = HashMap::new();
        let mut ir_fields = vec![];
        for (index, field) in fields {
            if let Some(idx) = index {
                field_offsets.insert(idx, heap_size);
            }
            ir_fields.push(field);
            heap_size += 4;
        }

        Self {
            struct_definition_index: index,
            heap_size,
            field_offsets,
            fields_types,
            fields: ir_fields,
        }
    }

    pub fn equality(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        index: u16,
    ) {
        let struct_ = compilation_ctx
            .module_structs
            .iter()
            .find(|s| s.index() == index)
            .unwrap_or_else(|| panic!("struct with index {index} not found"));

        let s1_ptr = module.locals.add(ValType::I32);
        let s2_ptr = module.locals.add(ValType::I32);
        let result = module.locals.add(ValType::I32);

        builder.local_set(s1_ptr).local_set(s2_ptr);
        builder.i32_const(1).local_set(result);

        let load_value_to_stack = |field: &IntermediateType, builder: &mut InstrSeqBuilder<'_>| {
            if field.stack_data_size() == 8 {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            } else {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
        };

        builder.block(None, |block| {
            let block_id = block.id();
            for (index, field) in struct_.fields.iter().enumerate() {
                // Offset of the field's pointer
                let offset = index as u32 * 4;

                block.local_get(s1_ptr).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );

                if field.is_stack_type() {
                    load_value_to_stack(field, block);
                }

                block.local_get(s2_ptr).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );

                if field.is_stack_type() {
                    load_value_to_stack(field, block);
                }

                field.load_equality_instructions(module, block, compilation_ctx);

                block.if_else(
                    None,
                    |_| {},
                    |else_| {
                        else_.i32_const(0).local_set(result).br(block_id);
                    },
                );
            }
        });

        builder.local_get(result);
    }

    pub fn index(&self) -> u16 {
        self.struct_definition_index.0
    }

    /// According to the formal specification of the encoding, a tuple (T1,...,Tk) is dynamic if
    /// Ti is dynamic for some 1 <= i <= k.
    ///
    /// Structs are encoded as a tuple of its fields, so, if any field is dynamic, then the whole
    /// struct is dynamic.
    ///
    /// According to documentation, dynamic types are:
    /// - bytes
    /// - string
    /// - T[] for any T
    /// - T[k] for any dynamic T and any k >= 0
    /// - (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k
    ///
    /// For more information:
    /// https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encoding
    pub fn solidity_abi_encode_is_dynamic(&self, compilation_ctx: &CompilationContext) -> bool {
        for field in &self.fields {
            match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64
                | IntermediateType::IU128
                | IntermediateType::IU256
                | IntermediateType::IAddress => continue,
                IntermediateType::IVector(_) => return true,
                IntermediateType::IStruct(index) => {
                    let struct_ = compilation_ctx
                        .module_structs
                        .iter()
                        .find(|s| s.index() == *index)
                        .unwrap_or_else(|| panic!("struct that with index {index} not found"));

                    if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                        continue;
                    } else {
                        return false;
                    }
                }
                IntermediateType::ISigner => panic!("signer is not abi econdable"),
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                    panic!("found reference inside struct")
                }
            }
        }

        false
    }
}
