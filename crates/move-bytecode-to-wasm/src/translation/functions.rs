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
    /// This field maps the types of the values that are currently in the stack
    pub stack_types: Vec<IntermediateType>,
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
            stack_types: vec![],
            move_module_signatures,
        }
    }

    pub fn translate_function(
        // TODO: Maybe use interior mutability
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

        for instruction in &self.move_code_unit.code {
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

            //self.process_stack_type();
        }

        Ok(())
    }

    fn process_stack_type(&self, instruction: &Bytecode, constant_pool: &[Constant]) {
        todo!();
        match instruction {
            // Load a fixed constant
            Bytecode::LdConst(global_index) => {
                if let Some(constant) = constant_pool.get(global_index.0 as usize) {
                    match constant.type_ {
                        move_binary_format::file_format::SignatureToken::Bool => todo!(),
                        move_binary_format::file_format::SignatureToken::U8 => todo!(),
                        move_binary_format::file_format::SignatureToken::U64 => todo!(),
                        move_binary_format::file_format::SignatureToken::U128 => todo!(),
                        move_binary_format::file_format::SignatureToken::Address => todo!(),
                        move_binary_format::file_format::SignatureToken::Signer => todo!(),
                        move_binary_format::file_format::SignatureToken::Vector(
                            signature_token,
                        ) => todo!(),
                        move_binary_format::file_format::SignatureToken::Datatype(
                            datatype_handle_index,
                        ) => todo!(),
                        move_binary_format::file_format::SignatureToken::DatatypeInstantiation(
                            _,
                        ) => todo!(),
                        move_binary_format::file_format::SignatureToken::Reference(
                            signature_token,
                        ) => todo!(),
                        move_binary_format::file_format::SignatureToken::MutableReference(
                            signature_token,
                        ) => todo!(),
                        move_binary_format::file_format::SignatureToken::TypeParameter(_) => {
                            todo!()
                        }
                        move_binary_format::file_format::SignatureToken::U16 => todo!(),
                        move_binary_format::file_format::SignatureToken::U32 => todo!(),
                        move_binary_format::file_format::SignatureToken::U256 => todo!(),
                    }
                }
            }
            // Load literals
            Bytecode::LdFalse => {}
            Bytecode::LdTrue => {}
            Bytecode::LdU8(literal) => {}
            Bytecode::LdU16(literal) => {}
            Bytecode::LdU32(literal) => {}
            Bytecode::LdU64(literal) => {}
            Bytecode::LdU128(literal) => {}
            Bytecode::LdU256(literal) => {}
            // Function calls
            Bytecode::Call(function_handle_index) => {}
            // Locals
            Bytecode::StLoc(local_id) => {}
            Bytecode::MoveLoc(local_id) => {}
            Bytecode::CopyLoc(local_id) => {}
            Bytecode::VecPack(signature_index, num_elements) => {}
            Bytecode::Pop => {}
            Bytecode::Ret => {}
            Bytecode::Add => {}
            _ => panic!("Unsupported instruction: {:?}", instruction),
        }
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
