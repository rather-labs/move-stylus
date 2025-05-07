use walrus::{FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals};

use super::address::IAddress;

#[derive(Clone, Copy)]
pub struct ISigner;

impl ISigner {
    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        IAddress::load_constant_instructions(module_locals, builder, bytes, allocator, memory);
    }
}
