use move_binary_format::file_format::{Bytecode, Constant, SignatureToken};
use walrus::{FunctionId, InstrSeqBuilder, LocalId};

pub mod functions;

fn map_bytecode_instruction<'a, 'b>(
    instruction: &Bytecode,
    constants: &[Constant],
    function_ids: &[FunctionId],
    builder: &'a mut InstrSeqBuilder<'b>,
    input_variables: &[LocalId],
) -> &'a mut InstrSeqBuilder<'b> {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            match constant.type_ {
                SignatureToken::U8 => {
                    let mut bytes = constant.data.clone();
                    // pad to 4 bytes on the right
                    bytes.resize(4, 0);

                    builder.i32_const(i32::from_le_bytes(
                        bytes.try_into().expect("Constant is not a u8"),
                    ))
                }
                SignatureToken::U16 => {
                    let mut bytes = constant.data.clone();
                    // pad to 4 bytes on the right
                    bytes.resize(4, 0);

                    builder.i32_const(i32::from_le_bytes(
                        bytes.try_into().expect("Constant is not a u16"),
                    ))
                }
                SignatureToken::U32 => builder.i32_const(i32::from_le_bytes(
                    constant
                        .data
                        .clone()
                        .try_into()
                        .expect("Constant is not a u32"),
                )),
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
        Bytecode::MoveLoc(local_id) => builder.local_get(input_variables[*local_id as usize]), // Handle reference types
        Bytecode::Call(function_handle_index) => {
            builder.call(function_ids[function_handle_index.0 as usize])
        }
        // TODO: ensure this is the last instruction
        Bytecode::Ret => builder.return_(),
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}
