use anyhow::Result;
use move_binary_format::file_format::{
    Bytecode, CodeUnit, Constant, FunctionDefinition, Signature,
};
use walrus::{
    ir::{LoadKind, MemArg, StoreKind},
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ModuleLocals, ValType,
};

use crate::translation::{intermediate_types::ISignature, map_bytecode_instruction};

use super::intermediate_types::IntermediateType;

pub struct MappedFunction<'a> {
    pub id: FunctionId,
    pub name: String,
    pub signature: ISignature,
    pub move_definition: FunctionDefinition,
    pub move_code_unit: CodeUnit,
    pub local_variables: Vec<LocalId>,
    pub local_variables_type: Vec<IntermediateType>,
    pub move_module_signatures: &'a [Signature],
}

impl<'a> MappedFunction<'a> {
    pub fn new(
        name: String,
        move_arguments: &Signature,
        move_returns: &Signature,
        move_definition: &FunctionDefinition,
        module: &mut Module,
        move_module_signatures: &'a [Signature],
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

        // === Handle argument locals ===
        let arg_local_ids = function_arguments
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect::<Vec<LocalId>>();

        let arg_intermediate_types = move_arguments
            .0
            .iter()
            .map(|token| token.try_into())
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

        // === Create the function ===
        let function_builder =
            FunctionBuilder::new(&mut module.types, &function_arguments, &function_returns);

        let id = function_builder.finish(arg_local_ids.clone(), &mut module.funcs);

        // === Handle declared locals ===
        let move_locals = &code.locals;
        let signature_tokens = &move_module_signatures[move_locals.0 as usize].0;

        let local_intermediate_types = signature_tokens
            .iter()
            .map(|token| token.try_into())
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

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
            move_module_signatures,
        }
    }

    pub fn translate_function(
        &self,
        module: &mut Module,
        constant_pool: &[Constant],
        function_ids: &[FunctionId],
        function_returns: &[Vec<IntermediateType>],
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

        let mut stack_types = Vec::new();

        println!("translating function {:?}", self.id);
        for instruction in &self.move_code_unit.code {
            map_bytecode_instruction(
                instruction,
                constant_pool,
                function_ids,
                &mut builder,
                self,
                &mut module.locals,
                &stack_types,
                allocator,
                memory,
            );

            self.process_stack_type(
                &mut stack_types,
                instruction,
                constant_pool,
                function_returns,
            )
            // TODO: unwrap
            .unwrap();

            println!("Stack types: {instruction:?} -> {stack_types:?}");
        }

        Ok(())
    }

    fn process_stack_type(
        &self,
        stack_types: &mut Vec<IntermediateType>,
        instruction: &Bytecode,
        constant_pool: &[Constant],
        function_returns: &[Vec<IntermediateType>],
    ) -> Result<(), anyhow::Error> {
        match instruction {
            Bytecode::LdConst(global_index) => {
                if let Some(constant) = constant_pool.get(global_index.0 as usize) {
                    let constant_type: IntermediateType = (&constant.type_).try_into()?;
                    stack_types.push(constant_type);
                } else {
                    return Err(anyhow::anyhow!(
                        "unable to find constant with global index: {global_index:?}"
                    ));
                }
            }
            Bytecode::LdFalse | Bytecode::LdTrue => stack_types.push(IntermediateType::IBool),
            Bytecode::LdU8(_) => stack_types.push(IntermediateType::IU8),
            Bytecode::LdU16(_) => stack_types.push(IntermediateType::IU16),
            Bytecode::LdU32(_) => stack_types.push(IntermediateType::IU32),
            Bytecode::LdU64(_) => stack_types.push(IntermediateType::IU64),
            Bytecode::LdU128(_) => stack_types.push(IntermediateType::IU128),
            Bytecode::LdU256(_) => stack_types.push(IntermediateType::IU256),
            Bytecode::Call(function_handle_index) => {
                if let Some(return_types) = function_returns.get(function_handle_index.0 as usize) {
                    for return_type in return_types {
                        stack_types.push(return_type.clone());
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "unable to find return types for function handle index: {function_handle_index:?}"
                    ));
                }
            }
            Bytecode::StLoc(_) | Bytecode::Pop => {
                stack_types.pop();
            }
            Bytecode::CopyLoc(local_id) | Bytecode::MoveLoc(local_id) => {
                if let Some(local_var_type) = self.local_variables_type.get(*local_id as usize) {
                    stack_types.push(local_var_type.clone());
                } else {
                    return Err(anyhow::anyhow!(
                        "unable to find local variable type: {local_id:?}"
                    ));
                }
            }
            Bytecode::VecPack(_signature_index, num_elements) => {
                // TODO: Maybe check that every element is of type _signature_index?
                stack_types.truncate(stack_types.len() - *num_elements as usize);
            }
            Bytecode::Ret => {}
            _ => panic!("Unsupported instruction: {:?}", instruction),
        }

        Ok(())
    }
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
