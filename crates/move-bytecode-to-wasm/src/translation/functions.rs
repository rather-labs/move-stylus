use anyhow::Result;
use move_binary_format::file_format::{CodeUnit, Constant, FunctionDefinition, Signature};
use walrus::{FunctionBuilder, FunctionId, LocalId, MemoryId, Module, ValType};

use crate::translation::{
    intermediate_types::{ISignature, IntermediateType},
    map_bytecode_instruction,
};

use super::intermediate_types::SignatureTokenToIntermediateType;

pub struct MappedFunction {
    pub id: FunctionId,
    pub name: String,
    pub signature: ISignature,
    pub move_definition: FunctionDefinition,
    pub move_code_unit: CodeUnit,
    pub local_variables: Vec<LocalId>,
    pub local_variables_type: Vec<IntermediateType>,
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
    
        assert!(
            function_returns.len() <= 1,
            "Multiple return values are not supported yet"
        );

        // === Handle argument locals ===
        let arg_local_ids = function_arguments
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect::<Vec<LocalId>>();

        let arg_intermediate_types = move_arguments
            .0
            .iter()
            .map(|token| token.to_intermediate_type())
            .collect::<Vec<IntermediateType>>();

        // === Create the function ===
        let function_builder =
            FunctionBuilder::new(&mut module.types, &function_arguments, &function_returns);

        let id = function_builder.finish(arg_local_ids.clone(), &mut module.funcs);

        // === Handle declared locals ===
        let move_locals = &code.locals;
        let signature_tokens = &move_module_signatures[move_locals.0 as usize].0;

        let local_intermediate_types = signature_tokens
            .iter()
            .map(|token| token.to_intermediate_type())
            .collect::<Vec<IntermediateType>>();

        let local_ids = local_intermediate_types
            .iter()
            .map(|ty| module.locals.add(ty.to_wasm_type()))
            .collect::<Vec<LocalId>>();

        // === Combine all locals and types ===
        let local_variables = [arg_local_ids, local_ids].concat();
        let local_variables_type = [arg_intermediate_types, local_intermediate_types].concat();

        Self {
            id,
            name,
            signature,
            move_definition: move_definition.clone(),
            move_code_unit: code,
            local_variables,
            local_variables_type,
        }
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

        let mut builder = module
            .funcs
            .get_mut(self.id)
            .kind
            .unwrap_local_mut()
            .builder_mut()
            .func_body();

        for instruction in self.move_code_unit.code.iter() {
            map_bytecode_instruction(
                instruction,
                constant_pool,
                function_ids,
                &mut builder,
                &self.local_variables,
                &self.local_variables_type,
                &mut module.locals,
                allocator,
                memory,
            );
        }

        Ok(())
    }
}

pub fn map_signature(signature: &Signature) -> Vec<ValType> {
    signature
        .0
        .iter()
        .map(|token| token.to_intermediate_type().to_wasm_type())
        .collect()
}
