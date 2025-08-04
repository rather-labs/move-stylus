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

/// Stores the keccak256 hash of the input string into the memory at the given pointer.
pub fn keccak_string_to_memory(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    key: &str,
    ptr: LocalId,
) {
    let binding = keccak256(key.as_bytes());
    let counter_key = binding.as_slice();
    for (i, byte) in counter_key.iter().enumerate() {
        builder.local_get(ptr); // base ptr
        builder.i32_const(*byte as i32); // byte value
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: i as u32,
            },
        );
    }
}
