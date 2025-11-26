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

use crate::{
    CompilationContext,
    abi_types::{
        error::{AbiEncodingError, AbiError},
        packing::Packable,
    },
    compilation_context::ModuleData,
    generics::replace_type_parameters,
    vm_handled_types::{VmHandledType, string::String_},
};

use super::{
    IntermediateType,
    error::IntermediateTypeError,
    user_type_fields::{FieldsErrorContext, UserTypeFields},
};
use move_binary_format::{
    file_format::{FieldHandleIndex, StructDefinitionIndex},
    internals::ModuleIndex,
};
use walrus::{InstrSeqBuilder, Module, ValType, ir::BinaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IStructType {
    OneTimeWitness,
    Event { indexes: u8, is_anonymous: bool },
    AbiError,
    Common,
}

#[derive(Debug, Clone)]
pub struct IStruct {
    /// Struct identifier
    pub identifier: String,

    /// Field's types ordered by index
    pub fields: Vec<IntermediateType>,

    /// Map between handles and fields types
    pub fields_types: HashMap<FieldHandleIndex, IntermediateType>,

    /// Map between handles and fields offset
    pub field_offsets: HashMap<FieldHandleIndex, u32>,

    /// Move's struct index
    pub struct_definition_index: StructDefinitionIndex,

    /// How much memory this struct occupies (in bytes).
    ///
    /// This will be the quantity of fields * 4 because we save pointers for all data types (stack
    /// or heap).
    ///
    /// This does not take in account how much space the actual data occupies because we can't know
    /// it (if the struct contains dynamic data such as vector, the size can change depending on how
    /// many elements the vector has), just the pointers to them.
    pub heap_size: u32,

    pub has_key: bool,

    pub type_: IStructType,
}

impl IStruct {
    pub fn new(
        index: StructDefinitionIndex,
        identifier: String,
        fields: Vec<(Option<FieldHandleIndex>, IntermediateType)>,
        fields_types: HashMap<FieldHandleIndex, IntermediateType>,
        has_key: bool,
        type_: IStructType,
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
            identifier,
            heap_size,
            field_offsets,
            fields_types,
            fields: ir_fields,
            has_key,
            type_,
        }
    }

    pub fn equality(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) -> Result<(), IntermediateTypeError> {
        let s1_ptr = module.locals.add(ValType::I32);
        let s2_ptr = module.locals.add(ValType::I32);

        builder.local_set(s1_ptr).local_set(s2_ptr);

        UserTypeFields::compare_fields(
            &self.fields,
            builder,
            module,
            compilation_ctx,
            module_data,
            s1_ptr,
            s2_ptr,
        )?;

        Ok(())
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) -> Result<(), IntermediateTypeError> {
        let src_ptr = module.locals.add(ValType::I32);
        let ptr = module.locals.add(ValType::I32);

        builder.local_set(src_ptr);

        // If the struct has the key ability, we need to copy the owner id too,
        // which is prepended 32 bytes before the struct in memory.
        if self.has_key {
            builder
                .i32_const(32)
                .call(compilation_ctx.allocator)
                .local_get(src_ptr)
                .i32_const(32)
                .binop(BinaryOp::I32Sub)
                .i32_const(32)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }

        // Allocate space for the new struct
        builder
            .i32_const(self.heap_size as i32)
            .call(compilation_ctx.allocator)
            .local_set(ptr);

        UserTypeFields::copy_fields(
            &self.fields,
            builder,
            module,
            compilation_ctx,
            module_data,
            src_ptr,
            ptr,
            0,
            FieldsErrorContext::Struct {
                struct_index: self.index(),
            },
        )?;

        builder.local_get(ptr);

        Ok(())
    }

    pub fn index(&self) -> u16 {
        self.struct_definition_index.into_index() as u16
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
    pub fn solidity_abi_encode_is_dynamic(
        &self,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, AbiError> {
        for field in &self.fields {
            match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64
                | IntermediateType::IU128
                | IntermediateType::IU256
                | IntermediateType::IAddress
                | IntermediateType::IEnum { .. }
                | IntermediateType::IGenericEnumInstance { .. } => continue,
                IntermediateType::IVector(_) => return Ok(true),
                IntermediateType::IStruct {
                    module_id, index, ..
                } if String_::is_vm_type(module_id, *index, compilation_ctx)? => return Ok(true),
                IntermediateType::IStruct { .. }
                | IntermediateType::IGenericStructInstance { .. } => {
                    let child_struct = compilation_ctx.get_struct_by_intermediate_type(field)?;

                    if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx)? {
                        return Ok(true);
                    }
                }
                IntermediateType::ISigner => return Err(AbiEncodingError::FoundSignerType)?,
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                    return Err(AbiEncodingError::RefInsideRef)?;
                }
                IntermediateType::ITypeParameter(_) => {
                    return Err(AbiEncodingError::GenericTypeParameterIsDynamic)?;
                }
            }
        }

        Ok(false)
    }

    /// Returns the size of the struct when encoded in Solidity ABI format.
    pub fn solidity_abi_encode_size(
        &self,
        compilation_ctx: &CompilationContext,
    ) -> Result<usize, AbiError> {
        let mut size = 0;
        for field in &self.fields {
            match field {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU64
                | IntermediateType::IU128
                | IntermediateType::IU256
                | IntermediateType::IAddress
                | IntermediateType::IVector(_)
                | IntermediateType::IEnum { .. }
                | IntermediateType::IGenericEnumInstance { .. } => {
                    size += (field as &dyn Packable).encoded_size(compilation_ctx)?;
                }
                IntermediateType::IStruct { .. }
                | IntermediateType::IGenericStructInstance { .. } => {
                    let child_struct = compilation_ctx.get_struct_by_intermediate_type(field)?;

                    if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx)? {
                        size += 32;
                    } else {
                        size += field.encoded_size(compilation_ctx)?;
                    }
                }
                IntermediateType::ISigner => return Err(AbiEncodingError::FoundSignerType)?,
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                    return Err(AbiEncodingError::RefInsideRef)?;
                }
                IntermediateType::ITypeParameter(_) => {
                    return Err(AbiEncodingError::FoundGenericTypeParameter)?;
                }
            }
        }

        Ok(size)
    }

    /// Replaces all type parameters in the struct with the provided types.
    pub fn instantiate(&self, types: &[IntermediateType]) -> Self {
        let fields = self
            .fields
            .iter()
            .map(|itype| replace_type_parameters(itype, types))
            .collect();

        let fields_types = self
            .fields_types
            .iter()
            .map(|(k, v)| {
                let key = FieldHandleIndex::new(k.into_index() as u16);
                let value = replace_type_parameters(v, types);
                (key, value)
            })
            .collect();

        let field_offsets = self
            .field_offsets
            .iter()
            .map(|(k, v)| (FieldHandleIndex::new(k.into_index() as u16), *v))
            .collect();

        Self {
            fields,
            identifier: self.identifier.clone(),
            fields_types,
            field_offsets,
            struct_definition_index: StructDefinitionIndex::new(
                self.struct_definition_index.into_index() as u16,
            ),
            ..*self
        }
    }
}
