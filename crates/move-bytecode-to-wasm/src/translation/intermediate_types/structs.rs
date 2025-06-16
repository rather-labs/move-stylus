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

use super::IntermediateType;
use move_binary_format::file_format::{FieldHandleIndex, StructDefinitionIndex};

#[derive(Debug)]
pub struct IStruct {
    /// Struct's identifier
    pub name: String,

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
        name: String,
        index: StructDefinitionIndex,
        fields: Vec<(Option<FieldHandleIndex>, IntermediateType)>,
        fields_types: HashMap<FieldHandleIndex, IntermediateType>,
    ) -> Self {
        let mut heap_size = 0;
        let mut field_offsets = HashMap::new();
        for (index, _) in fields.iter() {
            if let Some(idx) = index {
                field_offsets.insert(*idx, heap_size);
            }
            heap_size += 4;
        }

        let fields = fields.into_iter().map(|(_, t)| t).collect();

        Self {
            name,
            struct_definition_index: index,
            heap_size,
            field_offsets,
            fields_types,
            fields,
        }
    }

    pub fn index(&self) -> u16 {
        self.struct_definition_index.0
    }
}
