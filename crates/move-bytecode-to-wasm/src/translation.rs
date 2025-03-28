use anyhow::Result;
use move_binary_format::file_format::{
    Bytecode, Constant, FunctionDefinition, Signature, SignatureToken, Visibility,
};
use walrus::{FunctionBuilder, FunctionId, InstrSeqBuilder, Module, ValType};

pub fn translate_function(
    function_def: &FunctionDefinition,
    function_arguments: &[ValType],
    function_return: &[ValType],
    constant_pool: &[Constant],
    module: &mut Module,
) -> Result<FunctionId> {
    anyhow::ensure!(
        function_def.acquires_global_resources.is_empty(),
        "Acquiring global resources is not supported yet"
    );

    anyhow::ensure!(
        function_def.visibility == Visibility::Public,
        "Only public functions are supported"
    );

    let code = function_def
        .code
        .as_ref()
        .ok_or(anyhow::anyhow!("Function has no code"))?;

    anyhow::ensure!(
        code.jump_tables.is_empty(),
        "Jump tables are not supported yet"
    );

    anyhow::ensure!(
        function_arguments.is_empty(),
        "Function arguments are not supported yet"
    );

    let mut function_builder =
        FunctionBuilder::new(&mut module.types, function_arguments, function_return);

    let mut function_body = function_builder.func_body();

    for instruction in code.code.iter() {
        map_bytecode_instruction(instruction, constant_pool, &mut function_body);
    }

    let function = function_builder.finish(vec![], &mut module.funcs);

    Ok(function)
}

pub fn map_signatures(signatures: &[Signature]) -> Vec<Vec<ValType>> {
    signatures
        .iter()
        .map(|s| s.0.iter().map(map_signature_token).collect())
        .collect()
}

fn map_signature_token(signature_token: &SignatureToken) -> ValType {
    match signature_token {
        SignatureToken::Bool => ValType::I32,
        SignatureToken::U8 => ValType::I32,
        SignatureToken::U16 => ValType::I32,
        SignatureToken::U32 => ValType::I32,
        SignatureToken::U64 => ValType::I64,
        SignatureToken::U128 => panic!("U128 is not supported"),
        SignatureToken::U256 => panic!("U256 is not supported"),
        SignatureToken::Address => panic!("Address is not supported"),
        SignatureToken::Signer => panic!("Signer is not supported"),
        SignatureToken::Vector(_) => panic!("Vector is not supported"),
        SignatureToken::Datatype(_) => panic!("Datatype is not supported"),
        SignatureToken::Reference(_) => panic!("Reference is not supported"),
        SignatureToken::MutableReference(_) => panic!("MutableReference is not supported"),
        SignatureToken::TypeParameter(_) => panic!("TypeParameter is not supported"),
        SignatureToken::DatatypeInstantiation(_) => {
            panic!("DatatypeInstantiation is not supported")
        }
    }
}

fn map_bytecode_instruction<'a, 'b>(
    instruction: &Bytecode,
    constants: &[Constant],
    builder: &'a mut InstrSeqBuilder<'b>,
) -> &'a mut InstrSeqBuilder<'b> {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            match constant.type_ {
                SignatureToken::U8 => builder.i32_const(i32::from_le_bytes(
                    constant
                        .data
                        .clone()
                        .try_into()
                        .expect("Constant is not a u8"),
                )),
                SignatureToken::U16 => builder.i32_const(i32::from_le_bytes(
                    constant
                        .data
                        .clone()
                        .try_into()
                        .expect("Constant is not a u16"),
                )),
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
        // TODO: ensure this is the last instruction
        Bytecode::Ret => builder.return_().unreachable(),
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}
