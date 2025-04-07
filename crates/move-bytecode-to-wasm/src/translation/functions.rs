use anyhow::Result;
use move_binary_format::file_format::{CodeUnit, Constant, FunctionDefinition, Signature};
use walrus::{FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType};

use crate::translation::{intermediate_types::ISignature, map_bytecode_instruction};

use super::intermediate_types::SignatureTokenToIntermediateType;

pub struct MappedFunction {
    pub id: FunctionId,
    pub name: String,
    pub signature: ISignature,
    pub move_definition: FunctionDefinition,
    pub move_code_unit: CodeUnit,
    pub local_variables: Vec<LocalId>,
}

impl MappedFunction {
    pub fn new(
        name: String,
        move_arguments: &Signature,
        move_returns: &Signature,
        move_definition: &FunctionDefinition,
        module: &mut Module,
        move_module_signatures: &[Signature],
    ) -> Self {
        assert!(
            move_definition.acquires_global_resources.is_empty(),
            "Acquiring global resources is not supported yet"
        );

        let code = move_definition.code.clone().expect("Function has no code");

        let signature = ISignature::from_signatures(move_arguments, move_returns);
        let function_arguments = signature.get_argument_wasm_types();
        let function_returns = signature.get_return_wasm_types();

        let mut local_variables: Vec<LocalId> = function_arguments
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect();

        let function_builder =
            FunctionBuilder::new(&mut module.types, &function_arguments, &function_returns);

        // Building an empty function to get the function id
        let id = function_builder.finish(local_variables.clone(), &mut module.funcs);

        let move_locals = &code.locals;
        let mapped_locals = map_signature(&move_module_signatures[move_locals.0 as usize]);
        let mapped_locals: Vec<LocalId> = mapped_locals
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect();

        local_variables.extend(mapped_locals);

        Self {
            id,
            name,
            signature,
            move_definition: move_definition.clone(),
            move_code_unit: code,
            local_variables,
        }
    }

    pub fn get_function_body_builder<'a>(&self, module: &'a mut Module) -> InstrSeqBuilder<'a> {
        module
            .funcs
            .get_mut(self.id)
            .kind
            .unwrap_local_mut()
            .builder_mut()
            .func_body()
    }

    pub fn translate_function(
        &self,
        module: &mut Module,
        constant_pool: &[Constant],
        function_ids: &[FunctionId],
        memory: MemoryId,
        allocator: FunctionId,
    ) -> Result<()> {
        anyhow::ensure!(
            self.move_code_unit.jump_tables.is_empty(),
            "Jump tables are not supported yet"
        );

        for instruction in self.move_code_unit.code.iter() {
            map_bytecode_instruction(
                instruction,
                constant_pool,
                function_ids,
                self,
                module,
                allocator,
                memory,
            );
        }

        Ok(())
    }
}

pub fn get_function_body_builder(module: &mut Module, function_id: FunctionId) -> InstrSeqBuilder {
    module
        .funcs
        .get_mut(function_id)
        .kind
        .unwrap_local_mut()
        .builder_mut()
        .func_body()
}

pub fn map_signature(signature: &Signature) -> Vec<ValType> {
    signature
        .0
        .iter()
        .map(|token| token.to_intermediate_type().to_wasm_type())
        .collect()
}
