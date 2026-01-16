use walrus::GlobalId;

#[derive(Debug, Clone, Copy)]
pub struct CompilationContextGlobals {
    /// Tracks current position when reading/unpacking ABI-encoded calldata
    pub calldata_reader_pointer: GlobalId,
    /// Points to the next free memory location for allocation
    pub next_free_memory_pointer: GlobalId,
    /// Tracks available memory remaining for allocation
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
