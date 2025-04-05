use move_binary_format::file_format::{Bytecode, Constant, SignatureToken};
use walrus::{FunctionId, InstrSeqBuilder, LocalId};

pub mod functions;

fn map_bytecode_instruction<'a, 'b>(
    instruction: &Bytecode,
    constants: &[Constant],
    function_ids: &[FunctionId],
    builder: &'a mut InstrSeqBuilder<'b>,
    local_variables: &[LocalId],
) -> &'a mut InstrSeqBuilder<'b> {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            match constant.type_ {
                SignatureToken::U8 | SignatureToken::U16 | SignatureToken::U32 => {
                    let mut bytes = constant.data.clone();
                    assert!(bytes.len() <= 4, "Constant is too large to fit in u32");

                    // pad to 4 bytes on the right
                    bytes.resize(4, 0);

                    builder.i32_const(i32::from_le_bytes(bytes.try_into().unwrap()))
                }
                SignatureToken::U64 => builder.i64_const(i64::from_le_bytes(
                    constant
                        .data
                        .clone()
                        .try_into()
                        .expect("Constant is not a u64"),
                )),
                _ => panic!("Unsupported constant: {:?}", constant),
            }
        }
        // Load literals
        Bytecode::LdFalse => builder.i64_const(0),
        Bytecode::LdTrue => builder.i64_const(1),
        Bytecode::LdU8(literal) => builder.i32_const(*literal as i32),
        Bytecode::LdU16(literal) => builder.i32_const(*literal as i32),
        Bytecode::LdU32(literal) => builder.i32_const(*literal as i32),
        Bytecode::LdU64(literal) => builder.i64_const(*literal as i64),
        // Function calls
        Bytecode::Call(function_handle_index) => {
            builder.call(function_ids[function_handle_index.0 as usize])
        }
        // Locals
        Bytecode::StLoc(local_id) => builder.local_set(local_variables[*local_id as usize]),
        Bytecode::MoveLoc(local_id) => builder.local_get(local_variables[*local_id as usize]),
        Bytecode::CopyLoc(local_id) => builder.local_get(local_variables[*local_id as usize]),
        // TODO: ensure this is the last instruction
        Bytecode::Pop => builder.drop(),
        Bytecode::Ret => builder.return_(),
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}
