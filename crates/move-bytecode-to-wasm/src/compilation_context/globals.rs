use walrus::GlobalId;

#[derive(Debug, Clone, Copy)]
pub struct CompilationContextGlobals {
    pub calldata_reader_pointer: GlobalId,
    pub next_free_memory_pointer: GlobalId,
    pub available_memory: GlobalId,
}

impl CompilationContextGlobals {
    pub fn new(
        calldata_reader_pointer: GlobalId,
        next_free_memory_pointer: GlobalId,
        available_memory: GlobalId,
    ) -> Self {
        Self {
            calldata_reader_pointer,
            next_free_memory_pointer,
            available_memory,
        }
    }
}
