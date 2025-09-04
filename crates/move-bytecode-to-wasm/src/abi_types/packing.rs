use alloy_sol_types::{SolType, sol_data};
use pack_native_int::{pack_i32_type_instructions, pack_i64_type_instructions};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        enums::IEnum,
        heap_integers::{IU128, IU256},
        reference::{IMutRef, IRef},
        vector::IVector,
    },
};

mod pack_enum;
mod pack_heap_int;
mod pack_native_int;
mod pack_reference;
mod pack_struct;
mod pack_vector;

pub trait Packable {
    /// Adds the instructions to pack the value into memory according to Solidity's ABI encoding.
    ///
    /// The writer pointer is the pointer to the memory where the value will be written, should be incremented
    /// on each write.
    ///
    /// The calldata reference pointer is the pointer to the start of the calldata portion
    /// in order to calculate the params offset. Should never be modified internally.
    #[allow(clippy::too_many_arguments)]
    fn add_pack_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    );

    /// Adds the instructions to pack the value into memory according to Solidity's ABI encoding.
    ///
    /// The writer pointer is the pointer to the memory where the value will be written, should be
    /// incremented on each write.
    ///
    /// The calldata reference pointer is the pointer to the start of the calldata portion
    /// in order to calculate the params offset. Should never be modified internally.
    ///
    /// This function forces the dynamic encoding (pointer to the location of packed values +
    /// packed values). It is useful for types that can be encoded as dynamic or static depending
    /// on the context.
    ///
    /// For example, given a struct `Foo` that can be encoded dynamically (because it contains one
    /// or more values that are dynamically encoded).
    /// - If `Foo` is returned in a function that returns multiple values `(v1, v2, .., Foo, .., vn)`,
    ///   `Foo` must be encoded dynamically, because it a tuple member.
    /// - If `Foo` is the only return value in a function, it should be encoded statically.
    #[allow(clippy::too_many_arguments)]
    fn add_pack_instructions_dynamic(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    );

    /// Adds the instructions to load the value into a local variable.
    /// This is used to reverse the order of the stack before packing
    ///
    /// For native types this will load the variable itself.
    /// For heap types this will load the reference to the heap value
    fn add_load_local_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
    ) -> LocalId;

    /// Returns the ABI encoded size of the type
    fn encoded_size(&self, compilation_ctx: &CompilationContext) -> usize;

    /// Returns true if the type to be encoded is dynamic
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
    fn is_dynamic(&self, compilation_ctx: &CompilationContext) -> bool;
}

/// Builds the instructions to pack WASM return values into memory according to Solidity's ABI encoding.
///
/// Each return value is encoded and loaded in memory. Complex data types are copied to
/// have a contiguous memory layout.
///
/// Variables should have been loaded in the WASM stack before calling this function.
///
/// Returns a pointer to the memory holding the return data and the length of the encoded data.
pub fn build_pack_instructions<T: Packable>(
    builder: &mut InstrSeqBuilder,
    function_return_signature: &[T],
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> (LocalId, LocalId) {
    // We need to load all return types into locals in order to reverse the read order
    // Otherwise they would be popped in reverse order
    let mut locals = Vec::new();
    let mut args_size = 0;
    for signature_token in function_return_signature.iter().rev() {
        let local = signature_token.add_load_local_instructions(builder, module);
        locals.push(local);

        // If the function returns multiple values, those values will be encoded as a tuple. By
        // definition, a tuple T is dynamic (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k.
        // The encode size for a dynamically encoded field inside a dynamically encoded tuple is
        // just 32 bytes (the value is the offset to where the values are packed)
        args_size += if signature_token.is_dynamic(compilation_ctx) {
            32
        } else {
            signature_token.encoded_size(compilation_ctx)
        };
    }
    locals.reverse();

    let pointer = module.locals.add(ValType::I32);
    let pointer_end = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    // Allocate memory for the first level arguments
    builder
        .i32_const(args_size as i32)
        .call(compilation_ctx.allocator)
        .local_tee(pointer);

    // Store the writer pointer
    builder.local_set(writer_pointer);

    for (local, signature_token) in locals.iter().zip(function_return_signature.iter()) {
        // Copy the reference just to be safe in case in internal function modifies it
        builder
            .local_get(pointer)
            .local_set(calldata_reference_pointer);

        // If the function returns multiple values, those values will be encoded as a tuple. By
        // definition, a tuple T is dynamic (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k.
        // Given that the return tuple is encoded dynamically, for the values that are dynamic
        // inside the tuple, we must force a dynamic encoding.
        if signature_token.is_dynamic(compilation_ctx) {
            signature_token.add_pack_instructions_dynamic(
                builder,
                module,
                *local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            );

            // A dynamic value will only save the offset to where the values are located, so, we
            // just use 32 bytes
            builder
                .local_get(writer_pointer)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        } else {
            signature_token.add_pack_instructions(
                builder,
                module,
                *local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            );

            builder
                .local_get(writer_pointer)
                .i32_const(signature_token.encoded_size(compilation_ctx) as i32)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        }
    }

    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(pointer)
        .binop(BinaryOp::I32Sub)
        .local_set(pointer_end);

    (pointer, pointer_end)

    // The pointer_end remaining in the stack is the length of the encoded data
}

impl Packable for IntermediateType {
    fn add_load_local_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
    ) -> LocalId {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum(_) => {
                let local = module.locals.add(ValType::I32);
                builder.local_set(local);
                local
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);
                builder.local_set(local);
                local
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot pack generic type parameter");
            }
        }
    }

    fn add_pack_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
                pack_i32_type_instructions(
                    builder,
                    module,
                    compilation_ctx.memory_id,
                    local,
                    writer_pointer,
                );
            }
            IntermediateType::IU64 => {
                pack_i64_type_instructions(
                    builder,
                    module,
                    compilation_ctx.memory_id,
                    local,
                    writer_pointer,
                );
            }
            IntermediateType::IU128 => IU128::add_pack_instructions(
                builder,
                module,
                local,
                writer_pointer,
                compilation_ctx.memory_id,
            ),
            IntermediateType::IU256 => IU256::add_pack_instructions(
                builder,
                module,
                local,
                writer_pointer,
                compilation_ctx.memory_id,
            ),
            IntermediateType::ISigner => {
                panic!("signer type cannot be packed as it has no ABI representation")
            }
            IntermediateType::IAddress => IAddress::add_pack_instructions(
                builder,
                module,
                local,
                writer_pointer,
                compilation_ctx.memory_id,
            ),
            IntermediateType::IVector(inner) => IVector::add_pack_instructions(
                inner,
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            ),
            IntermediateType::IRef(inner) => IRef::add_pack_instructions(
                inner,
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            ),
            IntermediateType::IMutRef(inner) => IMutRef::add_pack_instructions(
                inner,
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            ),
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                struct_.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                    None,
                )
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);
                struct_instance.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                    None,
                )
            }
            IntermediateType::IEnum(enum_index) => {
                let enum_ = compilation_ctx
                    .root_module_data
                    .enums
                    .get_enum_by_index(*enum_index)
                    .unwrap();
                if !enum_.is_simple {
                    panic!(
                        "cannot abi pack enum with index {enum_index}, it contains at least one variant with fields"
                    );
                }
                IEnum::add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    compilation_ctx,
                )
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot pack generic type parameter");
            }
        }
    }

    fn add_pack_instructions_dynamic(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        match self {
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                // Load the intermediate pointer
                // And then pack the inner type dynamically
                builder
                    .local_get(local)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(local);

                inner.add_pack_instructions_dynamic(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
            }
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                struct_.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                    Some(calldata_reference_pointer),
                );
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);
                struct_instance.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                    Some(calldata_reference_pointer),
                );
            }

            _ => self.add_pack_instructions(
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            ),
        }
    }

    fn encoded_size(&self, compilation_ctx: &CompilationContext) -> usize {
        match self {
            IntermediateType::IBool => sol_data::Bool::ENCODED_SIZE.unwrap(),
            // According to the official documentation, enum types are encoded as uint8
            IntermediateType::IU8 | IntermediateType::IEnum(_) => {
                sol_data::Uint::<8>::ENCODED_SIZE.unwrap()
            }
            IntermediateType::IU16 => sol_data::Uint::<16>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU32 => sol_data::Uint::<32>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU64 => sol_data::Uint::<64>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU128 => sol_data::Uint::<128>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU256 => sol_data::Uint::<256>::ENCODED_SIZE.unwrap(),
            IntermediateType::IAddress => sol_data::Address::ENCODED_SIZE.unwrap(),
            IntermediateType::ISigner => sol_data::Address::ENCODED_SIZE.unwrap(),
            IntermediateType::IVector(_) => 32,
            IntermediateType::IRef(inner) => inner.encoded_size(compilation_ctx),
            IntermediateType::IMutRef(inner) => inner.encoded_size(compilation_ctx),
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);
                struct_instance.solidity_abi_encode_size(compilation_ctx)
            }
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                struct_.solidity_abi_encode_size(compilation_ctx)
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("can't know the size of a generic type parameter at compile time");
            }
        }
    }

    fn is_dynamic(&self, compilation_ctx: &CompilationContext) -> bool {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IEnum(_) => false,
            IntermediateType::IVector(_) => true,
            IntermediateType::IStruct { module_id, index } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                struct_.solidity_abi_encode_is_dynamic(compilation_ctx)
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let struct_instance = struct_.instantiate(types);
                struct_instance.solidity_abi_encode_is_dynamic(compilation_ctx)
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot check if generic type parameter is dynamic at compile time");
            }
            // References are dynamic if the inner type is dynamic!
            IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
                inner.is_dynamic(compilation_ctx)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use alloy_sol_types::sol;
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::{Caller, Engine, Extern, Linker};

    use crate::{
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        utils::display_module,
    };

    use super::*;

    fn get_validator(
        pointer_addr: u32,
        data_len: i32,
        data: Vec<u8>,
    ) -> impl Fn(Caller<()>, u32, u32) {
        move |mut caller: Caller<()>, pointer: u32, length: u32| {
            println!("validator: {}, {}", pointer, length);

            assert_eq!(pointer, pointer_addr);
            assert_eq!(length, data_len as u32);

            let memory = caller.get_export("memory").unwrap();
            let memory = match memory {
                Extern::Memory(memory) => memory,
                _ => panic!("memory not found"),
            };

            let mut buffer = vec![0; length as usize];
            memory
                .read(&mut caller, pointer as usize, &mut buffer)
                .unwrap();
            assert_eq!(buffer, data);
        }
    }

    #[test]
    fn test_build_pack_instructions() {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.i32_const(1);
        func_body.i32_const(1234);
        func_body.i64_const(123456789012345);

        let (data_start, data_end) = build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            &mut raw_module,
            &compilation_ctx,
        );

        func_body.local_get(data_start).local_get(data_end);

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data = <sol!((bool, uint16, uint64))>::abi_encode(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker
            .func_wrap("", "validator", get_validator(0, data_len, data))
            .unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            vec![],
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_pack_instructions_memory_offset() {
        // Memory offset starts at 100
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(100));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.i32_const(1);
        func_body.i32_const(1234);
        func_body.i64_const(123456789012345);

        let (data_start, data_end) = build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            &mut raw_module,
            &compilation_ctx,
        );

        func_body.local_get(data_start).local_get(data_end);

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data = <sol!((bool, uint16, uint64))>::abi_encode(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker
            .func_wrap("", "validator", get_validator(100, data_len, data))
            .unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            vec![],
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_pack_instructions_dynamic_types() {
        let data = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            16u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            36u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            68u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
            120u32.to_le_bytes().as_slice(),
            136u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
            U256::from(123456789012345u128)
                .to_le_bytes::<32>()
                .as_slice(),
        ]
        .concat();
        let data_len = data.len() as i32;

        let (mut raw_module, allocator_func, memory_id) = build_module(Some(data_len));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Load arguments to stack
        func_body.i32_const(1234);
        func_body.i32_const(0); // vector pointer
        func_body.i32_const(152); // u256 pointer

        let (data_start, data_end) = build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IU16,
                IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                    IntermediateType::IU128,
                )))),
                IntermediateType::IU256,
            ],
            &mut raw_module,
            &compilation_ctx,
        );

        func_body.local_get(data_start).local_get(data_end);

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let expected_data = <sol!((uint16, uint128[][], uint256))>::abi_encode_params(&(
            1234,
            vec![vec![1, 2, 3], vec![4, 5, 6]],
            U256::from(123456789012345u128),
        ));
        println!("expected_data: {:?}", expected_data);

        // Define validator function
        let mut linker = Linker::new(&Engine::default());
        linker
            .func_wrap(
                "",
                "validator",
                get_validator(
                    data_len as u32,
                    expected_data.len() as i32,
                    expected_data.clone(),
                ),
            )
            .unwrap();

        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), ()>(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }
}
