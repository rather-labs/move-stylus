// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module contains the native implementation for `std::type_name::get<T>()`.
//!
//! Since this is a generic native function, the type `T` is known at compile time
//! (monomorphized). We compute the fully qualified type name string in Rust and emit
//! WASM instructions that write those constant bytes into memory at runtime.
use super::NativeFunction;
use crate::data::RuntimeErrorData;
use crate::translation::intermediate_types::IntermediateType;
use crate::{
    CompilationContext, compilation_context::ModuleId,
    native_functions::error::NativeFunctionError, runtime::RuntimeFunction,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{MemArg, StoreKind},
};

pub fn add_type_name_get_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_TYPE_NAME_GET,
        compilation_ctx,
        &[itype],
        module_id,
    )?;

    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    // Compute the fully qualified type name string at compile time.
    // e.g. "u8", "vector<u8>", "0000...0001::string::String"
    let type_name_string = itype.get_type_name(compilation_ctx)?;
    let type_name_bytes = type_name_string.as_bytes();

    let allocate_vector_with_header_function = RuntimeFunction::AllocateVectorWithHeader.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let type_name_ptr = module.locals.add(ValType::I32);
    let string_ptr = module.locals.add(ValType::I32);
    let vector_ptr = module.locals.add(ValType::I32);

    // TypeName struct memory layout:
    //   TypeName { name: String }       → 4 bytes (pointer to String struct)
    //   String { bytes: vector<u8> }    → 4 bytes (pointer to vector<u8>)
    //   vector<u8>                      → 8 bytes header (len + capacity) + N bytes data

    // 1. Allocate TypeName struct (4 bytes) and save pointer
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_set(type_name_ptr);

    // 2. Allocate String struct (4 bytes) and save pointer
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_set(string_ptr);

    // 3. Store string_ptr at type_name_ptr (TypeName.name → String)
    builder
        .local_get(type_name_ptr)
        .local_get(string_ptr)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // 4. Allocate vector<u8> with the type name bytes
    //    AllocateVectorWithHeader(len, capacity, data_size) → pointer
    let len = type_name_bytes.len() as i32;
    builder
        .i32_const(len) // len
        .i32_const(len) // capacity
        .i32_const(1) // data_size (1 byte per u8 element)
        .call(allocate_vector_with_header_function)
        .local_set(vector_ptr);

    // 5. Store vector_ptr at string_ptr (String.bytes → vector<u8>)
    builder.local_get(string_ptr).local_get(vector_ptr).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // 6. Write each byte of the type name string into the vector data area.
    //    Vector layout: [len(4 bytes)][capacity(4 bytes)][data...]
    //    Data starts at offset 8 from the vector pointer.
    for (i, &byte) in type_name_bytes.iter().enumerate() {
        builder.local_get(vector_ptr);
        builder.i32_const(byte as i32);
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 8 + i as u32,
            },
        );
    }

    // 7. Return type_name_ptr
    builder.local_get(type_name_ptr);

    Ok(function.finish(vec![], &mut module.funcs))
}
