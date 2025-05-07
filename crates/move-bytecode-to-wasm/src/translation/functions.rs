use anyhow::Result;
use move_binary_format::file_format::{CodeUnit, Constant, FunctionDefinition, Signature};
use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ModuleLocals, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::translation::{intermediate_types::ISignature, map_bytecode_instruction};

use super::intermediate_types::{IntermediateType, SignatureTokenToIntermediateType};

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

        assert!(
            function_returns.len() <= 1,
            "Multiple return values is not enabled in Stylus VM"
        );

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
                self,
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

/// Adds the instructions to unpack the return values from memory
///
/// The returns values are read from memory and pushed to the stack
pub fn add_unpack_function_return_values_instructions(
    builder: &mut InstrSeqBuilder,
    module_locals: &mut ModuleLocals,
    returns: &[IntermediateType],
    memory: MemoryId,
) {
    if returns.is_empty() {
        return;
    }

    let pointer = module_locals.add(ValType::I32);
    builder.local_set(pointer);

    let mut offset = 0;
    for return_ty in returns.iter() {
        builder.local_get(pointer);
        if return_ty.stack_data_size() == 4 {
            builder.load(
                memory,
                LoadKind::I32 { atomic: false },
                MemArg { align: 0, offset },
            );
        } else if return_ty.stack_data_size() == 8 {
            builder.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg { align: 0, offset },
            );
        } else {
            unreachable!("Unsupported type size");
        }
        offset += return_ty.stack_data_size();
    }
}

/// Packs the return values into a tuple if the function has return values
///
/// This is necessary because the Stylus VM does not support multiple return values
/// Values are written to memory and a pointer to the first value is returned
pub fn prepare_function_return(
    module_locals: &mut ModuleLocals,
    builder: &mut InstrSeqBuilder,
    returns: &[IntermediateType],
    memory: MemoryId,
    allocator: FunctionId,
) {
    if !returns.is_empty() {
        let mut locals = Vec::new();
        let mut total_size = 0;
        for return_ty in returns.iter().rev() {
            let local = return_ty.add_stack_to_local_instructions(module_locals, builder);
            locals.push(local);
            total_size += return_ty.stack_data_size();
        }
        locals.reverse();

        let pointer = module_locals.add(ValType::I32);

        builder.i32_const(total_size as i32);
        builder.call(allocator);
        builder.local_set(pointer);

        let mut offset = 0;
        for (return_ty, local) in returns.iter().zip(locals.iter()) {
            builder.local_get(pointer);
            builder.local_get(*local);
            if return_ty.stack_data_size() == 4 {
                builder.store(
                    memory,
                    StoreKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );
            } else if return_ty.stack_data_size() == 8 {
                builder.store(
                    memory,
                    StoreKind::I64 { atomic: false },
                    MemArg { align: 0, offset },
                );
            } else {
                unreachable!("Unsupported type size");
            }
            offset += return_ty.stack_data_size();
        }

        builder.local_get(pointer);
    }

    builder.return_();
}
