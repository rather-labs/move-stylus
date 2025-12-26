use super::RuntimeFunction;
use super::error::RuntimeFunctionError;
use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiUnpackError},
    abi_types::unpacking::Unpackable,
    data::DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
    data::DATA_UNPACK_FROZEN_OFFSET,
    translation::intermediate_types::{IntermediateType, vector::IVector},
    wasm_builder_extensions::WasmBuilderExtension,
};

use alloy_sol_types::{SolType, sol_data};

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

pub fn unpack_bytes_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);

    // Advance the reader pointer by 32
    function_body
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(reader_pointer);

    function_builder.name(RuntimeFunction::UnpackBytes.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_u32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size = module.locals.add(ValType::I32);

    // Load the value
    function_body
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        )
        .call(swap_i32_bytes_function);

    // Set the global reader pointer to reader pointer + encoded size
    function_body
        .local_get(reader_pointer)
        .local_get(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_builder.name(RuntimeFunction::UnpackU32.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer, encoded_size], &mut module.funcs))
}

pub fn unpack_u64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None)?;

    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<64>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)?;

    // Load the value
    function_body
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 24,
            },
        )
        .call(swap_i64_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size as i32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_builder.name(RuntimeFunction::UnpackU64.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_u128_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i128_bytes_function =
        RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx))?;
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<128>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    // The data is padded 16 bytes to the right
    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .local_get(reader_pointer)
        .i32_const(16)
        .binop(BinaryOp::I32Add);
    function_body
        .i32_const(16)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i128_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackU128.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_u256_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i256_bytes_function =
        RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx))?;
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<256>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    function_body.local_get(reader_pointer);
    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i256_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackU256.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_address_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Address::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(unpacked_pointer);

    for i in 0..4 {
        function_body
            .local_get(unpacked_pointer)
            .local_get(reader_pointer)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            )
            .store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
    }

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackAddress.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

/// Generates a runtime function that unpacks a vector from ABI-encoded calldata.
///
/// This function:
/// 1. Reads the pointer to the vector data from calldata
/// 2. Reads the vector length
/// 3. Allocates memory for the vector
/// 4. Unpacks each element recursively
/// 5. Returns a pointer to the unpacked vector
///
/// # WASM Function Arguments
/// * `reader_pointer` - (i32): pointer to the current position in the ABI-encoded data
/// * `calldata_base_pointer` - (i32): pointer to the start of the calldata
///
/// # WASM Function Returns
/// * `vector_pointer` - (i32): pointer to the unpacked vector in memory
pub fn unpack_vector_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::UnpackVector.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_base_pointer = module.locals.add(ValType::I32);

    // Runtime functions
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;
    let validate_pointer_fn =
        RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.

    // Validate that the pointer fits in 32 bits
    builder.local_get(reader_pointer).call(validate_pointer_fn);

    // Load the pointer to the data, swap it to little-endian and add that to the calldata reader pointer.
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
        .local_get(calldata_base_pointer)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer); // This references the vector actual data

    // Increment the reader pointer to next argument
    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    // Validate that the data reader pointer fits in 32 bits
    builder
        .local_get(data_reader_pointer)
        .call(validate_pointer_fn);

    // Vector length: current number of elements in the vector
    let length = module.locals.add(ValType::I32);

    builder
        .local_get(data_reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        )
        .call(swap_i32_bytes_function)
        .local_set(length);

    // Increment data reader pointer
    builder
        .local_get(data_reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer);

    let vector_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    let data_size = inner
        .wasm_memory_data_size()
        .map_err(RuntimeFunctionError::from)?;
    IVector::allocate_vector_with_header(
        &mut builder,
        compilation_ctx,
        vector_pointer,
        length,
        length,
        data_size,
    );

    // Set the writer pointer to the start of the vector data
    builder
        .skip_vec_header(vector_pointer)
        .local_set(writer_pointer);

    // Copy elements
    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);

    let calldata_base_pointer_ = module.locals.add(ValType::I32);
    builder
        .local_get(data_reader_pointer)
        .local_set(calldata_base_pointer_);

    let mut inner_result: Result<(), AbiError> = Ok(());
    builder.loop_(None, |loop_block| {
        inner_result = (|| {
            let loop_block_id = loop_block.id();

            loop_block.local_get(writer_pointer);
            // This will leave in the stack [pointer/value i32/i64, length i32]
            inner.add_unpack_instructions(
                loop_block,
                module,
                data_reader_pointer,
                calldata_base_pointer_,
                compilation_ctx,
            )?;

            // Store the value
            loop_block.store(
                compilation_ctx.memory_id,
                inner.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // increment writer pointer
            loop_block.local_get(writer_pointer);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_set(writer_pointer);

            // increment i
            loop_block.local_get(i);
            loop_block.i32_const(1);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_tee(i);

            loop_block.local_get(length);
            loop_block.binop(BinaryOp::I32LtU);
            loop_block.br_if(loop_block_id);

            Ok(())
        })();
    });

    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    builder.local_get(vector_pointer);

    // Check for errors from the loop
    inner_result?;

    Ok(function.finish(
        vec![reader_pointer, calldata_base_pointer],
        &mut module.funcs,
    ))
}

pub fn unpack_enum_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let enum_ = compilation_ctx.get_enum_by_intermediate_type(itype)?;
    if !enum_.is_simple {
        return Err(
            AbiError::from(AbiUnpackError::EnumIsNotSimple(enum_.identifier.to_owned())).into(),
        );
    }
    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<8>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpack_u32_function = RuntimeFunction::UnpackU32.get(module, Some(compilation_ctx))?;

    // Save the variant to check it later
    let variant_number = module.locals.add(ValType::I32);
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .call(unpack_u32_function)
        .local_tee(variant_number);

    // Trap if the variant number is higher that the quantity of variants the enum contains
    function_body
        .i32_const(enum_.variants.len() as i32 - 1)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                then.unreachable();
            },
            |_| {},
        );

    // The enum should occupy only 4 bytes since only the variant number is saved
    let enum_ptr = module.locals.add(ValType::I32);
    function_body
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(enum_ptr)
        .local_get(variant_number)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function_body.local_get(enum_ptr);

    function_builder.name(RuntimeFunction::UnpackEnum.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;
    // Validate that the pointer fits in 32 bits
    let validate_pointer_fn =
        RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;

    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.
    function_body
        .local_get(reader_pointer)
        .call(validate_pointer_fn);

    function_body
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
        .local_set(data_reader_pointer); // This references the vector actual data

    // Advance the reader pointer by 32
    function_body
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    // Validate that the data reader pointer fits in 32 bits
    function_body
        .local_get(data_reader_pointer)
        .call(validate_pointer_fn);

    // Vector length: current number of elements in the vector
    let length = module.locals.add(ValType::I32);

    function_body
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
        .local_set(length);

    // Increment data reader pointer
    function_body
        .local_get(data_reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer);

    let vector_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Allocate space for the vector
    // Each u8 element takes 1 byte
    IVector::allocate_vector_with_header(
        &mut function_body,
        compilation_ctx,
        vector_pointer,
        length,
        length,
        1,
    );
    function_body
        .local_get(vector_pointer)
        .local_set(writer_pointer);

    // Set writer pointer to the start of the vector data
    function_body
        .skip_vec_header(writer_pointer)
        .local_set(writer_pointer);

    // Copy elements
    let i = module.locals.add(ValType::I32);
    function_body.i32_const(0).local_set(i);

    function_body.loop_(None, |loop_block| {
        let loop_block_id = loop_block.id();

        loop_block.local_get(writer_pointer);

        loop_block.local_get(data_reader_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        loop_block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Increment data reader pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(data_reader_pointer)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(data_reader_pointer);

        // Increment writer pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(writer_pointer)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(writer_pointer);

        // Increment i
        loop_block
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_tee(i);

        loop_block
            .local_get(length)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_block_id);
    });

    let struct_ptr = module.locals.add(ValType::I32);
    // Create the struct pointing to the vector
    function_body
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(struct_ptr);

    // Save the vector pointer as the first value
    function_body.local_get(vector_pointer).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Return the String struct
    function_body.local_get(struct_ptr);

    function_builder.name(RuntimeFunction::UnpackString.name().to_owned());
    Ok(function_builder.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

// According to the formal specification of the encoding, a tuple (T1,...,Tk) is dynamic if
// Ti is dynamic for some 1 <= i <= k.
//
// Since structs are encoded as tuples of their fields, a struct is also considered dynamic
// if any of its fields is dynamic.
//
// Based on the ABI specification, the following types are considered dynamic:
// - bytes
// - string
// - T[] for any T
// - T[k] for any dynamic T and any k >= 0
// - (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k
//
// For example, the following Move's struct:
//
// public struct Foo has drop {
//    x: u8,
//    y: vector<u32>,
//    z: vector<u128>,
// }
//
// Is equivalent to the following struct in Solidity:
//
// struct Foo {
//     uint8 x;
//     uint32[] y;
//     uint128[] z;
// }
//
// Given that the struct contains vectors, it becomes a dynamic. This means that the first encoded
// value of this struct will be a number pointing to where the values are packed in the calldata.
//
// If we call a function that have Foo as an argument with:
// Foo {
//     x: 254,
//     y: [1, 2, u32::MAX],
//     z: [1, 2, u128::MAX],
// }
//
// The encoded data will be:
// bytes   0..3      4..35   36..67   68..99   100..131
//       [selector,   32  ,   254   ,   96   ,   224  , [3,1,2,u32::MAX], [3,1,2,u128::MAX]]
//                 ptr_foo  ▲  x       ptr_y    ptr_z   ▲                 ▲
//                    │     │           │         │     │                 │
//                    └─────┘           └─────────┼─────┘                 │
//                                                └───────────────────────┘
// where
//  - selector: the called function selector
//
//  - ptr_foo: where the Foo struct's values are packed in the calldata. It is 32 because it does
//    not take in account the selector.  36 = len(selector) + len(ptr_foo) = 4 + 32,
//    where the packed data starts
//
//  - x: 254 packed as uint8 (32 bytes)
//
//  - ptr_y: where the y's vector values are packed. It does not take in account the selector and
//    ptr_foo. 96 = len(x) + len(ptr_y) + len(ptr_z) = 32 + 32 + 32
//
//  - ptr_z: where the z's vector values are packed. It does not take in account the selector and
//    ptr_foo. 224 = len(x) + len(ptr_y) + len(ptr_z) + y_data = 32 + 32 + 32 + 128.
//    y_data has length 128 because it contains its length (32 bytes) and 3 elements (3 x 32bytes)
//
// If a struct does not contain any dynamic fields, all its fields are encoded inline, packed
// contiguously without any offset or pointer.
//
// For more information:
// https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encoding
pub fn unpack_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    // Moving pointer for reading data of the fields
    let data_reader_pointer = module.locals.add(ValType::I32);

    // Pointer to where the struct is packed
    let calldata_ptr = module.locals.add(ValType::I32);

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    // In a dynamic struct, the first value is where the values are packed in the calldata
    if struct_.solidity_abi_encode_is_dynamic(compilation_ctx)? {
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

        // Validate that the pointer fits in 32 bits
        let validate_pointer_fn =
            RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;
        function_body
            .local_get(reader_pointer)
            .call(validate_pointer_fn);

        function_body
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
        function_body
            .local_get(reader_pointer)
            .local_set(data_reader_pointer)
            .local_get(calldata_reader_pointer)
            .local_set(calldata_ptr);
    }

    // Allocate space for the struct
    let struct_ptr = module.locals.add(ValType::I32);
    function_body
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    let mut offset = 0;
    let field_ptr = module.locals.add(ValType::I32);
    for field in &struct_.fields {
        // Unpack field
        field.add_unpack_instructions(
            &mut function_body,
            module,
            data_reader_pointer,
            calldata_ptr,
            compilation_ctx,
        )?;

        // If the field is stack type, we need to create the intermediate pointer, otherwise
        // the add_unpack_instructions function leaves the pointer in the stack
        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let data_size = field.wasm_memory_data_size()?;
                let val = module.locals.add(ValType::try_from(field)?);
                let store_kind = field.store_kind()?;

                // Save the actual value
                function_body.local_set(val);

                // Create a pointer for the value
                function_body
                    .i32_const(data_size)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Store the actual value behind the middle_ptr
                function_body.local_get(val).store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            _ => {
                function_body.local_set(field_ptr);
            }
        }

        function_body
            .local_get(struct_ptr)
            .local_get(field_ptr)
            .store(
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
    let advancement = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx)? {
        32
    } else {
        struct_.solidity_abi_encode_size(compilation_ctx)? as i32
    };

    function_body
        .local_get(reader_pointer)
        .i32_const(advancement)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(struct_ptr);

    function_builder.name(RuntimeFunction::UnpackStruct.name().to_owned());
    Ok(function_builder.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

pub fn unpack_storage_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);

    // Search for the object in the objects mappings
    let locate_storage_data_fn =
        RuntimeFunction::LocateStorageData.get(module, Some(compilation_ctx))?;

    function_body
        .local_get(uid_ptr)
        .i32_const(DATA_UNPACK_FROZEN_OFFSET)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .call(locate_storage_data_fn);

    // Read the object
    let read_and_decode_from_storage_fn =
        RuntimeFunction::ReadAndDecodeFromStorage.get_generic(module, compilation_ctx, &[itype])?;

    // Copy the slot number into a local to avoid overwriting it later
    let slot_ptr = module.locals.add(ValType::I32);
    function_body
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    function_body
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .call(read_and_decode_from_storage_fn);

    // Reset the unpack frozen flag to false.
    // This is always the default value for the flag.
    function_body
        .i32_const(DATA_UNPACK_FROZEN_OFFSET)
        .i32_const(0)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function_builder.name(RuntimeFunction::UnpackStorageStruct.name().to_owned());
    Ok(function_builder.finish(vec![uid_ptr], &mut module.funcs))
}

pub fn unpack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    match itype {
        // If inner is a heap type, forward the pointer
        IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;
        }
        // For immediates, allocate and store
        IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IBool => {
            let ptr_local = module.locals.add(walrus::ValType::I32);

            let data_size = itype.wasm_memory_data_size()?;
            function_body
                .i32_const(data_size)
                .call(compilation_ctx.allocator)
                .local_tee(ptr_local);

            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;

            function_body.store(
                compilation_ctx.memory_id,
                itype.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            function_body.local_get(ptr_local);
        }

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::RefInsideRef,
            )));
        }
        IntermediateType::ITypeParameter(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::UnpackingGenericTypeParameter,
            )));
        }
    }

    function_builder.name(RuntimeFunction::UnpackReference.name().to_owned());
    Ok(function_builder.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{ConstExpr, FunctionBuilder, ValType, ir::Value};
    use wasmtime::WasmResults;

    use crate::{
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    use super::*;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Test helper for unpacking simple integer types that fit in WASM value types
    fn unpack_uint<T: WasmResults + PartialEq + std::fmt::Debug>(
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                args_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module::<_, T>(&mut raw_module, data.to_vec(), "test_function", None);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    /// Test helper for unpacking heap-allocated types (u128, u256, address)
    fn unpack_heap_uint(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                args_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);

        let global_next_free_memory_pointer = global_next_free_memory_pointer
            .get(&mut store)
            .i32()
            .unwrap();
        assert_eq!(
            global_next_free_memory_pointer,
            (expected_result_bytes.len() + data.len()) as i32
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    /// Test helper for unpacking vector types
    fn unpack_vec(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        int_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    /// Test helper for unpacking reference types
    fn unpack_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let calldata_reader_pointer_global = raw_module.globals.add_local(
            ValType::I32,
            true,
            false,
            ConstExpr::Value(Value::I32(0)),
        );
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                &mut func_body,
                &mut raw_module,
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_memory_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_memory_bytes,
            "Heap memory at returned pointer does not match expected content"
        );
    }

    // ============================================================================
    // Simple Integer Types (u8, u16, u32, u64)
    // ============================================================================

    #[test]
    fn test_unpack_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IU8;

        let data = SolType::abi_encode_params(&(88,));
        unpack_uint(int_type.clone(), &data, 88, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IU16;

        let data = SolType::abi_encode_params(&(1616,));
        unpack_uint(int_type.clone(), &data, 1616, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u32() {
        type IntType = u32;
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IU32;

        let data = SolType::abi_encode_params(&(323232,));
        unpack_uint(int_type.clone(), &data, 323232, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u64() {
        type IntType = u64;
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IU64;

        let data = SolType::abi_encode_params(&(6464646464,));
        unpack_uint(int_type.clone(), &data, 6464646464i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i64,
            ValType::I64,
        );
    }

    // ============================================================================
    // Heap-Allocated Types (u128, u256, address)
    // ============================================================================

    #[test]
    fn test_unpack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let data = SolType::abi_encode_params(&(88,));
        unpack_heap_uint(&data, int_type.clone(), &88u128.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_heap_uint(&data, int_type.clone(), &(IntType::MAX - 1).to_le_bytes());
    }

    #[test]
    fn test_unpack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let data = SolType::abi_encode_params(&(U256::from(88),));
        unpack_heap_uint(&data, int_type.clone(), &U256::from(88).to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        unpack_heap_uint(
            &data,
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
        );
    }

    #[test]
    fn test_unpack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let data = SolType::abi_encode_params(&(Address::ZERO,));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        unpack_heap_uint(&data, int_type.clone(), &data);
    }

    // ============================================================================
    // Vector Types - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u8_empty() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params::<(Vec<u8>,)>(&(vec![],));
        let expected_result_bytes =
            [0u32.to_le_bytes().as_slice(), 0u32.to_le_bytes().as_slice()].concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u8() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(vec![1, 2],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Vector Types - Heap-Allocated Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u256() {
        type SolType = sol!((uint256[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU256));

        let data =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            U256::from(1).to_le_bytes::<32>().as_slice(),
            U256::from(2).to_le_bytes::<32>().as_slice(),
            U256::from(3).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_address() {
        type SolType = sol!((address[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IAddress));

        let data = SolType::abi_encode_params(&(vec![
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
        ],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Nested Vector Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU32,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU128,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),                        // len
            2u32.to_le_bytes().as_slice(),                        // capacity
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),  // first element pointer
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),  // second element pointer
            3u32.to_le_bytes().as_slice(),                        // first element length
            3u32.to_le_bytes().as_slice(),                        // first element capacity
            ((data.len() + 36) as u32).to_le_bytes().as_slice(), // first element - first value pointer
            ((data.len() + 52) as u32).to_le_bytes().as_slice(), // first element - second value pointer
            ((data.len() + 68) as u32).to_le_bytes().as_slice(), // first element - third value pointer
            1u128.to_le_bytes().as_slice(),                      // first element - first value
            2u128.to_le_bytes().as_slice(),                      // first element - second value
            3u128.to_le_bytes().as_slice(),                      // first element - third value
            3u32.to_le_bytes().as_slice(),                       // second element length
            3u32.to_le_bytes().as_slice(),                       // second element capacity
            ((data.len() + 104) as u32).to_le_bytes().as_slice(), // second element - first value pointer
            ((data.len() + 120) as u32).to_le_bytes().as_slice(), // second element - second value pointer
            ((data.len() + 136) as u32).to_le_bytes().as_slice(), // second element - third value pointer
            4u128.to_le_bytes().as_slice(),                       // second element - first value
            5u128.to_le_bytes().as_slice(),                       // second element - second value
            6u128.to_le_bytes().as_slice(),                       // second element - third value
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Reference Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u8() {
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88u8,));
        let expected = 88u8.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u16() {
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(88u16,));
        let expected = 88u16.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u32() {
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        unpack_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        unpack_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u128() {
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123u128,));
        let expected = 123u128.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u256() {
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let expected = value.to_le_bytes::<32>().to_vec();

        let data = SolType::abi_encode_params(&(value,));
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IAddress));

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_ref(&data, ref_type.clone(), &data);
    }

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let vector_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU8,
        ))));

        let vec_data = vec![1u8, 2u8, 3u8, 4u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&4u32.to_le_bytes()); // length
        expected.extend(&4u32.to_le_bytes()); // capacity
        expected.extend(&1u8.to_le_bytes()); // first elem
        expected.extend(&2u8.to_le_bytes()); // second elem
        expected.extend(&3u8.to_le_bytes()); // third elem
        expected.extend(&4u8.to_le_bytes()); // fourth elem
        unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_u128() {
        type SolType = sol!((uint128[],));
        let vector_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU128,
        ))));

        let vec_data = vec![1u128, 2u128, 3u128];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&3u32.to_le_bytes()); // length
        expected.extend(&3u32.to_le_bytes()); // capacity
        // pointers to heap elements
        expected.extend(&180u32.to_le_bytes());
        expected.extend(&196u32.to_le_bytes());
        expected.extend(&212u32.to_le_bytes());
        expected.extend(&1u128.to_le_bytes());
        expected.extend(&2u128.to_le_bytes());
        expected.extend(&3u128.to_le_bytes());

        unpack_ref(&data, vector_type.clone(), &expected);
    }
}
