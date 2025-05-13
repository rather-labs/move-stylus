use functions::{
    MappedFunction, add_unpack_function_return_values_instructions, prepare_function_return,
};
use intermediate_types::vector::IVector;
use intermediate_types::{IntermediateType, SignatureTokenToIntermediateType};
use move_binary_format::file_format::{Bytecode, Constant, SignatureIndex};
use walrus::{
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
    ir::{MemArg, StoreKind},
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
    allocator: FunctionId,
    memory: MemoryId,
) {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            let mut data = constant.data.clone().into_iter();
            constant
                .type_
                .to_intermediate_type()
                .load_constant_instructions(module_locals, builder, &mut data, allocator, memory);
            assert!(
                data.next().is_none(),
                "Constant data not consumed: {:?}",
                data
            );
        }
        // Load literals
        Bytecode::LdFalse => {
            builder.i32_const(0);
        }
        Bytecode::LdTrue => {
            builder.i32_const(1);
        }
        Bytecode::LdU8(literal) => {
            builder.i32_const(*literal as i32);
        }
        Bytecode::LdU16(literal) => {
            builder.i32_const(*literal as i32);
        }
        Bytecode::LdU32(literal) => {
            builder.i32_const(*literal as i32);
        }
        Bytecode::LdU64(literal) => {
            builder.i64_const(*literal as i64);
        }
        Bytecode::LdU128(literal) => {
            add_load_literal_heap_type_to_memory_instructions(
                module_locals,
                builder,
                memory,
                allocator,
                &literal.to_le_bytes(),
            );
        }
        Bytecode::LdU256(literal) => {
            add_load_literal_heap_type_to_memory_instructions(
                module_locals,
                builder,
                memory,
                allocator,
                &literal.to_le_bytes(),
            );
        }
        // Function calls
        Bytecode::Call(function_handle_index) => {
            builder.call(function_ids[function_handle_index.0 as usize]);
            add_unpack_function_return_values_instructions(
                builder,
                module_locals,
                &mapped_function.signature.returns,
                memory,
            );
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            builder.local_set(mapped_function.local_variables[*local_id as usize]);
        }
        Bytecode::MoveLoc(local_id) => {
            // Values and references are loaded into a new variable
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            builder.local_get(mapped_function.local_variables[*local_id as usize]);
        }
        Bytecode::CopyLoc(local_id) => {
            mapped_function.local_variables_type[*local_id as usize].copy_loc_instructions(
                module_locals,
                builder,
                allocator,
                memory,
                mapped_function.local_variables[*local_id as usize],
            );
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
        }
        Bytecode::Pop => {
            builder.drop();
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

pub fn get_intermediate_type_for_signature_index(
    mapped_function: &MappedFunction,
    signature_index: SignatureIndex,
) -> IntermediateType {
    let tokens = &mapped_function.move_module_signatures[signature_index.0 as usize].0;
    assert_eq!(
        tokens.len(),
        1,
        "Expected signature to have exactly 1 token for VecPack"
    );
    tokens[0].to_intermediate_type()
}
