use walrus::{FunctionId, Module};

use crate::CompilationContext;

mod integers;
mod swap;

#[derive(PartialEq)]
pub enum RuntimeFunction {
    // Integer operations
    HeapIntSum,
    AddU32,
    AddU64,
    CheckOverflowU8U16,
    DowncastU64ToU32,
    DowncastU128U256ToU32,
    DowncastU128U256ToU64,
    // Swap bytes
    SwapI32Bytes,
    SwapI64Bytes,
}

impl RuntimeFunction {
    pub fn name(&self) -> &'static str {
        match self {
            RuntimeFunction::HeapIntSum => "heap_integer_add",
            RuntimeFunction::AddU32 => "add_u32",
            RuntimeFunction::AddU64 => "add_u64",
            RuntimeFunction::CheckOverflowU8U16 => "check_overflow_u8_u16",
            RuntimeFunction::DowncastU64ToU32 => "downcast_u64_to_u32",
            RuntimeFunction::DowncastU128U256ToU32 => "downcast_u128_u256_to_u32",
            RuntimeFunction::DowncastU128U256ToU64 => "downcast_u128_u256_to_u64",
            RuntimeFunction::SwapI32Bytes => "swap_i32_bytes",
            RuntimeFunction::SwapI64Bytes => "swap_i64_bytes",
        }
    }

    /// Links the function into the module and returns its id. If the function is already present
    /// it just resunts the id.
    ///
    /// This funciton is idempotent.
    pub fn get(
        &self,
        module: &mut Module,
        compilation_ctx: Option<&CompilationContext>,
    ) -> FunctionId {
        if let Some(function) = module.funcs.by_name(self.name()) {
            function
        } else {
            match (self, compilation_ctx) {
                // Integers
                (Self::HeapIntSum, Some(ctx)) => integers::heap_integers_add(module, ctx),
                (Self::AddU32, _) => integers::add_u32(module),
                (Self::AddU64, _) => integers::add_u64(module),
                (Self::CheckOverflowU8U16, _) => integers::check_overflow_u8_u16(module),
                (Self::DowncastU64ToU32, _) => integers::downcast_u64_to_u32(module),
                (Self::DowncastU128U256ToU32, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u32(module, ctx)
                }
                (Self::DowncastU128U256ToU64, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u64(module, ctx)
                }
                // Swap
                (Self::SwapI32Bytes, _) => swap::swap_i32_bytes_function(module),
                (Self::SwapI64Bytes, _) => {
                    let swap_i32_f = Self::SwapI32Bytes.get(module, compilation_ctx);
                    swap::swap_i64_bytes_function(module, swap_i32_f)
                }
                _ => panic!(
                    r#"there was an error linking "{}" function, missing compilation context?"#,
                    self.name()
                ),
            }
        }
    }
}
