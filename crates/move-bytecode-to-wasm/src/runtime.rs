use walrus::{FunctionId, Module};

use crate::CompilationContext;

mod integers;
mod integers_bitwise;
mod swap;

#[derive(PartialEq)]
pub enum RuntimeFunction {
    // Integer operations
    HeapIntSum,
    HeapIntShiftLeft,
    HeapIntShiftRight,
    AddU32,
    AddU64,
    CheckOverflowU8U16,
    DowncastU64ToU32,
    DowncastU128U256ToU32,
    DowncastU128U256ToU64,
    SubU32,
    SubU64,
    HeapIntSub,
    // Swap bytes
    SwapI32Bytes,
    SwapI64Bytes,
}

impl RuntimeFunction {
    pub fn name(&self) -> &'static str {
        match self {
            // Integer operations
            Self::HeapIntSum => "heap_integer_add",
            Self::HeapIntSub => "heap_integer_sub",
            Self::AddU32 => "add_u32",
            Self::AddU64 => "add_u64",
            Self::CheckOverflowU8U16 => "check_overflow_u8_u16",
            Self::DowncastU64ToU32 => "downcast_u64_to_u32",
            Self::DowncastU128U256ToU32 => "downcast_u128_u256_to_u32",
            Self::DowncastU128U256ToU64 => "downcast_u128_u256_to_u64",
            Self::SubU32 => "sub_u32",
            Self::SubU64 => "sub_u64",
            // Bitwise
            Self::HeapIntShiftLeft => "heap_integer_shift_left",
            Self::HeapIntShiftRight => "heap_integer_shift_right",
            // Swap bytes
            Self::SwapI32Bytes => "swap_i32_bytes",
            Self::SwapI64Bytes => "swap_i64_bytes",
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
                (Self::HeapIntSub, Some(ctx)) => integers::heap_integers_sub(module, ctx),
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
                (Self::SubU32, _) => integers::sub_u32(module),
                (Self::SubU64, _) => integers::sub_u64(module),
                // Swap
                (Self::SwapI32Bytes, _) => swap::swap_i32_bytes_function(module),
                (Self::SwapI64Bytes, _) => {
                    let swap_i32_f = Self::SwapI32Bytes.get(module, compilation_ctx);
                    swap::swap_i64_bytes_function(module, swap_i32_f)
                }
                // Bitwise
                (Self::HeapIntShiftLeft, Some(ctx)) => {
                    integers_bitwise::heap_int_shift_left(module, ctx)
                }
                (Self::HeapIntShiftRight, Some(ctx)) => {
                    integers_bitwise::heap_int_shift_right(module, ctx)
                }
                _ => panic!(
                    r#"there was an error linking "{}" function, missing compilation context?"#,
                    self.name()
                ),
            }
        }
    }
}
