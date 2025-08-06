use crate::CompilationContext;
use alloy_primitives::keccak256;
use walrus::{
    InstrSeqBuilder, LocalId,
    ir::{MemArg, StoreKind},
};

#[cfg(test)]
use walrus::Module;

#[cfg(test)]
pub fn display_module(module: &mut Module) {
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    // print with line breaks
    println!("{}", wat.replace("\\n", "\n"));
}

/// Converts the input string to camel case.
pub fn snake_to_camel(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    // .len returns byte count but ok in this case!

    #[derive(PartialEq)]
    enum ChIs {
        FirstOfStr,
        NextOfSepMark,
        Other,
    }

    let mut flag = ChIs::FirstOfStr;

    for ch in input.chars() {
        if flag == ChIs::FirstOfStr {
            result.push(ch.to_ascii_lowercase());
            flag = ChIs::Other;
        } else if ch == '_' {
            flag = ChIs::NextOfSepMark;
        } else if flag == ChIs::NextOfSepMark {
            result.push(ch.to_ascii_uppercase());
            flag = ChIs::Other;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Stores the keccak256 hash of the input string into the memory at the given pointer
pub fn keccak_string_to_memory(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    key: &str,
    ptr: LocalId,
) {
    let hash = keccak256(key.as_bytes());

    // Break the 32-byte hash into 4 × 8-byte chunks
    for (i, chunk) in hash.chunks_exact(8).enumerate() {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(chunk);

        // Interpret chunk as little-endian u64 (matches WASM memory layout)
        let value = u64::from_le_bytes(arr);

        builder
            .local_get(ptr) // base pointer
            .i64_const(value as i64) // push 8-byte value
            .store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: (i * 8) as u32, // offset = 0, 8, 16, 24
                },
            );
    }
}
