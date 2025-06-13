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
    ir::{BinaryOp, LoadKind, MemArg},
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
        for (index, _) in fields.iter().rev() {
            if let Some(idx) = index {
                field_offsets.insert(*idx, heap_size);
            }
            heap_size += 4;
        }

        let fields = fields.into_iter().map(|(_, t)| t).collect();

        Self {
            struct_definition_index: index,
            heap_size,
            field_offsets,
            fields_types,
            fields,
        }
    }

    pub fn equality(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        index: usize,
    ) {
        let struct_ = compilation_ctx
            .module_structs
            .iter()
            .find(|s| s.index() == index as u16)
            .unwrap_or_else(|| panic!("struct with index {index} not found"));

        let print_memory_from = module.imports.get_func("", "print_memory_from").unwrap();

        let s1_ptr = module.locals.add(ValType::I32);
        let s2_ptr = module.locals.add(ValType::I32);
        builder
            /*
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )*/
            .local_set(s1_ptr)
            /*
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )*/
            .local_set(s2_ptr);

        builder.local_get(s1_ptr).call(print_memory_from);
        builder.local_get(s2_ptr).call(print_memory_from);

        for (index, field) in struct_.fields.iter().rev().enumerate() {
            builder
                .local_get(s1_ptr)
                .i32_const(index as i32 * 4)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            if field.is_stack_type() {
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
            }

            builder
                .local_get(s2_ptr)
                .i32_const(index as i32 * 4)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            if field.is_stack_type() {
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
            }

            field.load_equality_instructions(module, builder, compilation_ctx);
        }
    }

    pub fn index(&self) -> u16 {
        self.struct_definition_index.0
    }
}
