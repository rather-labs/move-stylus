use alloy_sol_types::{SolType, sol_data};
use pack_native_int::{pack_i32_type_instructions, pack_i64_type_instructions};
use walrus::{FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType, ir::BinaryOp};

use crate::translation::intermediate_types::{
    IntermediateType,
    address::IAddress,
    heap_integers::{IU128, IU256},
    vector::IVector,
};

mod pack_heap_int;
mod pack_native_int;
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
        memory: MemoryId,
        alloc_function: FunctionId,
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
    fn encoded_size(&self) -> usize;
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
    memory: MemoryId,
    alloc_function: FunctionId,
) {
    if function_return_signature.is_empty() {
        builder.i32_const(0);
        builder.i32_const(0);
        return;
    }

    // We need to load all return types into locals in order to reverse the read order
    // Otherwise they would be popped in reverse order
    let mut locals = Vec::new();
    let mut args_size = 0;
    for signature_token in function_return_signature.iter().rev() {
        let local = signature_token.add_load_local_instructions(builder, module);
        locals.push(local);
        args_size += signature_token.encoded_size();
    }
    locals.reverse();

    let pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Allocate memory for the first level arguments
    builder.i32_const(args_size as i32);
    builder.call(alloc_function);
    builder.local_tee(pointer);

    // Store the writer pointer
    builder.local_set(writer_pointer);

    for (local, signature_token) in locals.iter().zip(function_return_signature.iter()) {
        // Copy the reference just to be safe in case in internal function modifies it
        let calldata_reference_pointer = module.locals.add(ValType::I32);
        builder.local_get(pointer);
        builder.local_set(calldata_reference_pointer);

        signature_token.add_pack_instructions(
            builder,
            module,
            *local,
            writer_pointer,
            calldata_reference_pointer,
            memory,
            alloc_function,
        );

        builder.local_get(writer_pointer);
        builder.i32_const(signature_token.encoded_size() as i32);
        builder.binop(BinaryOp::I32Add);
        builder.local_set(writer_pointer);
    }

    builder.local_get(pointer); // This will remain in the stack as return value

    // use the allocator to get a pointer to the end of the calldata
    builder.i32_const(0);
    builder.call(alloc_function);
    builder.local_get(pointer);
    builder.binop(BinaryOp::I32Sub);
    // The value remaining in the stack is the length of the encoded data
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
            | IntermediateType::IVector(_)
            | IntermediateType::IAddress => {
                let local = module.locals.add(ValType::I32);
                builder.local_set(local);
                local
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);
                builder.local_set(local);
                local
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
        memory: MemoryId,
        alloc_function: FunctionId,
    ) {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
                pack_i32_type_instructions(builder, module, memory, local, writer_pointer);
            }
            IntermediateType::IU64 => {
                pack_i64_type_instructions(builder, module, memory, local, writer_pointer);
            }
            IntermediateType::IU128 => {
                IU128::add_pack_instructions(builder, module, local, writer_pointer, memory)
            }
            IntermediateType::IU256 => {
                IU256::add_pack_instructions(builder, module, local, writer_pointer, memory)
            }
            IntermediateType::IAddress => {
                IAddress::add_pack_instructions(builder, module, local, writer_pointer, memory)
            }
            IntermediateType::IVector(inner) => IVector::add_pack_instructions(
                inner,
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                memory,
                alloc_function,
            ),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            IntermediateType::IBool => sol_data::Bool::ENCODED_SIZE.unwrap(),
            IntermediateType::IU8 => sol_data::Uint::<8>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU16 => sol_data::Uint::<16>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU32 => sol_data::Uint::<32>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU64 => sol_data::Uint::<64>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU128 => sol_data::Uint::<128>::ENCODED_SIZE.unwrap(),
            IntermediateType::IU256 => sol_data::Uint::<256>::ENCODED_SIZE.unwrap(),
            IntermediateType::IAddress => sol_data::Address::ENCODED_SIZE.unwrap(),
            IntermediateType::IVector(_) => 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, primitives::U256, sol};
    use walrus::{FunctionBuilder, FunctionId, MemoryId, ModuleConfig, ValType};
    use wasmtime::{
        Caller, Engine, Extern, IntoFunc, Linker, Module as WasmModule, Store, TypedFunc,
        WasmParams,
    };

    use crate::{memory::setup_module_memory, utils::display_module};

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module<A: WasmParams, V>(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        function_name: &str,
        validator_func: impl IntoFunc<(), V, ()>,
    ) -> (Linker<()>, Store<()>, TypedFunc<A, ()>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        linker.func_wrap("", "validator", validator_func).unwrap();

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<A, ()>(&mut store, function_name)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        memory.write(&mut store, 0, &initial_memory_data).unwrap();
        // Print current memory
        let memory_data = memory.data(&mut store);
        println!(
            "Current memory: {:?}",
            memory_data.iter().take(64).collect::<Vec<_>>()
        );

        (linker, store, entrypoint)
    }

    #[test]
    fn test_build_pack_instructions() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

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

        build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            &mut raw_module,
            memory_id,
            allocator_func,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            vec![],
            "test_function",
            move |mut caller: Caller<()>, pointer: u32, length: u32| {
                println!("validator: {}, {}", pointer, length);

                assert_eq!(pointer, 0);
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
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_pack_instructions_memory_offset() {
        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate some memory just to increase the offset
        func_body.i32_const(100);
        func_body.call(allocator_func);
        func_body.drop();

        // Load arguments to stack
        func_body.i32_const(1);
        func_body.i32_const(1234);
        func_body.i64_const(123456789012345);

        build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IBool,
                IntermediateType::IU16,
                IntermediateType::IU64,
            ],
            &mut raw_module,
            memory_id,
            allocator_func,
        );

        // validation
        func_body.call(validator_func);

        let function = function_builder.finish(vec![args_pointer, args_len], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let data =
            <sol!((bool, uint16, uint64))>::abi_encode_params(&(true, 1234, 123456789012345));
        println!("data: {:?}", data);
        let data_len = data.len() as i32;

        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            vec![],
            "test_function",
            move |mut caller: Caller<()>, pointer: u32, length: u32| {
                println!("validator: {}, {}", pointer, length);
                assert_eq!(pointer, 100);
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
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }

    #[test]
    fn test_build_pack_instructions_dynamic_types() {
        let data = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            76u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            92u32.to_le_bytes().as_slice(),
            108u32.to_le_bytes().as_slice(),
            124u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
            U256::from(123456789012345u128)
                .to_le_bytes::<32>()
                .as_slice(),
        ]
        .concat();
        let data_len = data.len() as i32;

        let (mut raw_module, allocator_func, memory_id) = build_module();

        let validator_func_type = raw_module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (validator_func, _) = raw_module.add_import_func("", "validator", validator_func_type);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32, ValType::I32], &[]);

        let args_len = raw_module.locals.add(ValType::I32);
        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // allocate memory to match the expected data length
        func_body.i32_const(data_len);
        func_body.call(allocator_func);
        func_body.drop();

        // Load arguments to stack
        func_body.i32_const(1234);
        func_body.i32_const(0); // vector pointer
        func_body.i32_const(140); // u256 pointer

        build_pack_instructions(
            &mut func_body,
            &[
                IntermediateType::IU16,
                IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
                    IntermediateType::IU128,
                )))),
                IntermediateType::IU256,
            ],
            &mut raw_module,
            memory_id,
            allocator_func,
        );

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
        let (_, mut store, entrypoint) = setup_wasmtime_module::<(i32, i32), _>(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            move |mut caller: Caller<()>, pointer: u32, length: u32| {
                println!("validator: {}, {}", pointer, length);

                assert_eq!(pointer, data_len as u32);
                assert_eq!(length, expected_data.len() as u32);

                let memory = caller.get_export("memory").unwrap();
                let memory = match memory {
                    Extern::Memory(memory) => memory,
                    _ => panic!("memory not found"),
                };

                let mut buffer = vec![0; length as usize];
                memory
                    .read(&mut caller, pointer as usize, &mut buffer)
                    .unwrap();
                assert_eq!(buffer, expected_data);
            },
        );

        entrypoint.call(&mut store, (0, data_len)).unwrap();
    }
}
