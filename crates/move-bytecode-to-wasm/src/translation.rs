use functions::MappedFunction;
use intermediate_types::SignatureTokenToIntermediateType;
use move_binary_format::file_format::{Bytecode, Constant};
use walrus::{
    FunctionId, MemoryId, Module, ValType,
    ir::{MemArg, StoreKind},
};

pub mod functions;
/// The types in this module represent an intermediate Rust representation of Move types
/// that is used to generate the WASM code.
pub mod intermediate_types;

fn map_bytecode_instruction(
    instruction: &Bytecode,
    constants: &[Constant],
    function_ids: &[FunctionId],
    mapped_function: &MappedFunction,
    module: &mut Module,
    allocator: FunctionId,
    memory: MemoryId,
) {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &constants[global_index.0 as usize];
            constant
                .type_
                .to_intermediate_type()
                .load_constant_instructions(
                    module,
                    mapped_function.id,
                    &constant.data,
                    allocator,
                    memory,
                );
        }
        // Load literals
        Bytecode::LdFalse => mapped_function.add_i32_const(module, 0),
        Bytecode::LdTrue => mapped_function.add_i32_const(module, 1),
        Bytecode::LdU8(literal) => mapped_function.add_i32_const(module, *literal as i32),
        Bytecode::LdU16(literal) => mapped_function.add_i32_const(module, *literal as i32),
        Bytecode::LdU32(literal) => mapped_function.add_i32_const(module, *literal as i32),
        Bytecode::LdU64(literal) => mapped_function.add_i64_const(module, *literal as i64),
        Bytecode::LdU128(literal) => mapped_function
            .add_load_literal_heap_type_to_memory_instructions(
                module,
                memory,
                allocator,
                &literal.to_le_bytes(),
            ),
        Bytecode::LdU256(literal) => mapped_function
            .add_load_literal_heap_type_to_memory_instructions(
                module,
                memory,
                allocator,
                &literal.to_le_bytes(),
            ),
        // Function calls
        Bytecode::Call(function_handle_index) => {
            mapped_function
                .get_function_body_builder(module)
                .call(function_ids[function_handle_index.0 as usize]);
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            mapped_function
                .get_function_body_builder(module)
                .local_set(mapped_function.local_variables[*local_id as usize]);
        }
        Bytecode::MoveLoc(local_id) => {
            // Values and references are loaded into a new variable
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            mapped_function
                .get_function_body_builder(module)
                .local_get(mapped_function.local_variables[*local_id as usize]);
        }
        Bytecode::CopyLoc(local_id) => {
            // Values or references from the stack are copied to the local variable
            // This works for stack and heap types
            // Note: This is valid because heap types are currently immutable, this may change in the future
            mapped_function
                .get_function_body_builder(module)
                .local_get(mapped_function.local_variables[*local_id as usize]);
        }
        Bytecode::Pop => {
            mapped_function.get_function_body_builder(module).drop();
        }
        // TODO: ensure this is the last instruction
        Bytecode::Ret => {
            mapped_function.get_function_body_builder(module).return_();
        }
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}

impl MappedFunction {
    fn add_i32_const(&self, module: &mut Module, value: i32) {
        self.get_function_body_builder(module).i32_const(value);
    }

    fn add_i64_const(&self, module: &mut Module, value: i64) {
        self.get_function_body_builder(module).i64_const(value);
    }

    fn add_load_literal_heap_type_to_memory_instructions(
        &self,
        module: &mut Module,
        memory: MemoryId,
        allocator: FunctionId,
        bytes: &[u8],
    ) {
        let pointer = module.locals.add(ValType::I32);

        let mut builder = self.get_function_body_builder(module);

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
}
