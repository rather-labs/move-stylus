use functions::{
    add_unpack_function_return_values_instructions, prepare_function_return, MappedFunction,
};
use intermediate_types::IntermediateType;
use intermediate_types::{simple_integers::IU8, vector::IVector};
use move_binary_format::file_format::{Bytecode, Constant, SignatureIndex};
use walrus::{
    ir::{MemArg, StoreKind},
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
};
pub mod functions;
/// The types in this module represent an intermediate Rust representation of Move types
/// that is used to generate the WASM code.
pub mod intermediate_types;

#[allow(clippy::too_many_arguments)]
fn map_bytecode_instruction(
    instruction: &Bytecode,
    constants: &[Constant],
    function_ids: &[FunctionId],
    builder: &mut InstrSeqBuilder,
    mapped_function: &MappedFunction,
    module_locals: &mut ModuleLocals,
    functions_arguments: &[Vec<IntermediateType>],
    functions_returns: &[Vec<IntermediateType>],
    types_stack: &mut Vec<IntermediateType>,
    allocator: FunctionId,
    memory: MemoryId,
) {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            let mut data = constant.data.clone().into_iter();
            let constant_type = &constant.type_;
            let constant_type: IntermediateType = constant_type
                .try_into()
                // TODO: unwrap
                .unwrap();

            constant_type.load_constant_instructions(
                module_locals,
                builder,
                &mut data,
                allocator,
                memory,
            );

            types_stack.push(constant_type);
            assert!(
                data.next().is_none(),
                "Constant data not consumed: {:?}",
                data
            );
        }
        // Load literals
        Bytecode::LdFalse => {
            builder.i32_const(0);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::LdTrue => {
            builder.i32_const(1);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::LdU8(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU8);
        }
        Bytecode::LdU16(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU16);
        }
        Bytecode::LdU32(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU32);
        }
        Bytecode::LdU64(literal) => {
            builder.i64_const(*literal as i64);
            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::LdU128(literal) => {
            add_load_literal_heap_type_to_memory_instructions(
                module_locals,
                builder,
                memory,
                allocator,
                &literal.to_le_bytes(),
            );
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::LdU256(literal) => {
            add_load_literal_heap_type_to_memory_instructions(
                module_locals,
                builder,
                memory,
                allocator,
                &literal.to_le_bytes(),
            );
            types_stack.push(IntermediateType::IU256);
        }
        // Function calls
        Bytecode::Call(function_handle_index) => {
            // Consume from the types stack the arguments that will be used by the function call
            let arguments = &functions_arguments[function_handle_index.0 as usize];
            for argument in arguments.iter().rev() {
                let arg = types_stack.pop().unwrap_or_else(|| {
                    panic!(
                    "function call argument error, expected {argument:?} but types stack is empty"
                )
                });
                assert_eq!(
                    arg, *argument,
                    "function call argument mismatch, expected {argument:?} and found {arg:?}"
                );
            }

            builder.call(function_ids[function_handle_index.0 as usize]);
            add_unpack_function_return_values_instructions(
                builder,
                module_locals,
                &mapped_function.signature.returns,
                memory,
            );

            // Insert in the stack types the types returned by the function (if any)
            let return_types = &functions_returns[function_handle_index.0 as usize];
            for return_type in return_types {
                types_stack.push(return_type.clone());
            }
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            builder.local_set(mapped_function.local_variables[*local_id as usize]);
            types_stack.pop();
        }
        Bytecode::MoveLoc(local_id) => {
            // Values and references are loaded into a new variable
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            builder.local_get(mapped_function.local_variables[*local_id as usize]);
            types_stack.push(mapped_function.local_variables_type[*local_id as usize].clone());
        }
        Bytecode::CopyLoc(local_id) => {
            mapped_function.local_variables_type[*local_id as usize].copy_loc_instructions(
                module_locals,
                builder,
                allocator,
                memory,
                mapped_function.local_variables[*local_id as usize],
            );
            types_stack.push(mapped_function.local_variables_type[*local_id as usize].clone());
        }
        Bytecode::VecPack(signature_index, num_elements) => {
            let inner =
                get_intermediate_type_for_signature_index(mapped_function, *signature_index);
            IVector::vec_pack_instructions(
                &inner,
                module_locals,
                builder,
                allocator,
                memory,
                *num_elements as i32,
            );

            // Remove the packing values from types stack and check if the types are correct
            let mut n = *num_elements as usize;
            while n > 0 {
                let ty = types_stack
                    .pop()
                    .unwrap_or_else(|| panic!("error unpacking vector: types stack is empty"));
                assert_eq!(
                    ty, inner,
                    "found type {ty:?} unpacking vector of type {inner:?}"
                );

                n -= 1;
            }

            types_stack.push(IntermediateType::IVector(Box::new(inner)));
        }
        Bytecode::Pop => {
            builder.drop();
            types_stack.pop();
        }
        // TODO: ensure this is the last instruction in the move code
        Bytecode::Ret => {
            prepare_function_return(
                module_locals,
                builder,
                &mapped_function.signature.returns,
                memory,
                allocator,
            );
        }
        Bytecode::Add => {
            let sum_type = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two add two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };

            match sum_type {
                IntermediateType::IU8 => IU8::add(builder, module_locals),
                IntermediateType::IU16 => todo!(),
                IntermediateType::IU32 => todo!(),
                IntermediateType::IU64 => todo!(),
                IntermediateType::IU128 => todo!(),
                IntermediateType::IU256 => todo!(),
                t => panic!("type stack error: trying to add two {t:?}"),
            }

            types_stack.push(sum_type);
        }
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}

fn add_load_literal_heap_type_to_memory_instructions(
    module_locals: &mut ModuleLocals,
    builder: &mut InstrSeqBuilder,
    memory: MemoryId,
    allocator: FunctionId,
    bytes: &[u8],
) {
    let pointer = module_locals.add(ValType::I32);

    builder.i32_const(bytes.len() as i32);
    builder.call(allocator);
    builder.local_set(pointer);

    let mut offset = 0;

    while offset < bytes.len() {
        builder.local_get(pointer);
        builder.i64_const(i64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        ));
        builder.store(
            memory,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: offset as u32,
            },
        );

        offset += 8;
    }

    builder.local_get(pointer);
}

fn get_intermediate_type_for_signature_index(
    mapped_function: &MappedFunction,
    signature_index: SignatureIndex,
) -> IntermediateType {
    let tokens = &mapped_function.move_module_signatures[signature_index.0 as usize].0;
    assert_eq!(
        tokens.len(),
        1,
        "Expected signature to have exactly 1 token for VecPack"
    );
    // TODO: unwrap
    (&tokens[0]).try_into().unwrap()
}
