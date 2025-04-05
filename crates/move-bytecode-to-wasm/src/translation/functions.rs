use anyhow::Result;
use move_binary_format::file_format::{Constant, FunctionDefinition, Signature, SignatureToken};
use walrus::{FunctionBuilder, FunctionId, LocalId, Module, ValType};

use crate::translation::map_bytecode_instruction;

pub struct MappedFunction {
    pub id: FunctionId,
    pub name: String,
    pub move_arguments: Signature,
    pub move_returns: Signature,
    pub move_definition: FunctionDefinition,
    pub input_variables: Vec<LocalId>,
}

impl MappedFunction {
    pub fn new(
        name: String,
        move_arguments: &Signature,
        move_returns: &Signature,
        move_definition: &FunctionDefinition,
        module: &mut Module,
    ) -> Self {
        let function_arguments = map_signature(move_arguments);
        let function_returns = map_signature(move_returns);

        let input_variables: Vec<LocalId> = function_arguments
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect();

        let function_builder =
            FunctionBuilder::new(&mut module.types, &function_arguments, &function_returns);

        // Building an empty function to get the function id
        let id = function_builder.finish(input_variables.clone(), &mut module.funcs);

        Self {
            id,
            name,
            move_arguments: move_arguments.clone(),
            move_returns: move_returns.clone(),
            move_definition: move_definition.clone(),
            input_variables,
        }
    }

    fn get_function_builder<'a>(&self, module: &'a mut Module) -> &'a mut FunctionBuilder {
        let function_builder = module
            .funcs
            .get_mut(self.id)
            .kind
            .unwrap_local_mut()
            .builder_mut();

        function_builder
    }

    pub fn translate_function(
        &self,
        module: &mut Module,
        constant_pool: &[Constant],
        function_ids: &[FunctionId],
    ) -> Result<()> {
        let function_def = &self.move_definition;

        anyhow::ensure!(
            function_def.acquires_global_resources.is_empty(),
            "Acquiring global resources is not supported yet"
        );

        let code = function_def
            .code
            .as_ref()
            .ok_or(anyhow::anyhow!("Function has no code"))?;

        anyhow::ensure!(
            code.jump_tables.is_empty(),
            "Jump tables are not supported yet"
        );

        let function_builder = self.get_function_builder(module);

        let mut function_body = function_builder.func_body();

        for instruction in code.code.iter() {
            map_bytecode_instruction(
                instruction,
                constant_pool,
                function_ids,
                &mut function_body,
                &self.input_variables,
            );
        }

        Ok(())
    }
}

pub fn map_signature(signature: &Signature) -> Vec<ValType> {
    signature.0.iter().map(map_signature_token).collect()
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
