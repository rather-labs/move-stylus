use walrus::{FunctionId, Module};

use crate::CompilationContext;

mod copy;
mod equality;
mod integers;
mod swap;
mod vector;

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
    HeapIntDivMod,
    MulU32,
    MulU64,
    HeapIntMul,
    LessThan,
    // Swap bytes
    SwapI32Bytes,
    SwapI64Bytes,
    // Copy
    CopyU128,
    CopyU256,
    // Equality
    HeapTypeEquality,
    VecEqualityHeapType,
    // Vector
    VecSwap32,
    VecSwap64,
    VecPopBack32,
    VecPopBack64,
    VecBorrow,
    VecIncrementLen,
    VecDecrementLen,
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
            Self::MulU32 => "mul_u32",
            Self::MulU64 => "mul_u64",
            Self::HeapIntMul => "heap_integer_mul",
            Self::HeapIntDivMod => "heap_integer_div_mod",
            Self::LessThan => "less_than",
            // Bitwise
            Self::HeapIntShiftLeft => "heap_integer_shift_left",
            Self::HeapIntShiftRight => "heap_integer_shift_right",
            // Swap bytes
            Self::SwapI32Bytes => "swap_i32_bytes",
            Self::SwapI64Bytes => "swap_i64_bytes",
            // Copy
            Self::CopyU128 => "copy_u128",
            Self::CopyU256 => "copy_u256",
            // Equality
            Self::HeapTypeEquality => "heap_type_equality",
            Self::VecEqualityHeapType => "vec_equality_heap_type",
            // Vector
            Self::VecSwap32 => "vec_swap_32",
            Self::VecSwap64 => "vec_swap_64",
            Self::VecPopBack32 => "vec_pop_back_32",
            Self::VecPopBack64 => "vec_pop_back_64",
            Self::VecBorrow => "vec_borrow",
            Self::VecIncrementLen => "vec_increment_len",
            Self::VecDecrementLen => "vec_decrement_len",
        }
    }

    /// Links the function into the module and returns its id. If the function is already present
    /// it just returns the id.
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
                (Self::HeapIntSum, Some(ctx)) => integers::add::heap_integers_add(module, ctx),
                (Self::HeapIntSub, Some(ctx)) => integers::sub::heap_integers_sub(module, ctx),
                (Self::AddU32, _) => integers::add::add_u32(module),
                (Self::AddU64, _) => integers::add::add_u64(module),
                (Self::SubU32, _) => integers::sub::sub_u32(module),
                (Self::SubU64, _) => integers::sub::sub_u64(module),
                (Self::CheckOverflowU8U16, _) => integers::check_overflow_u8_u16(module),
                (Self::DowncastU64ToU32, _) => integers::downcast_u64_to_u32(module),
                (Self::DowncastU128U256ToU32, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u32(module, ctx)
                }
                (Self::DowncastU128U256ToU64, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u64(module, ctx)
                }
                (Self::MulU32, _) => integers::mul::mul_u32(module),
                (Self::MulU64, _) => integers::mul::mul_u64(module),
                (Self::HeapIntMul, Some(ctx)) => integers::mul::heap_integers_mul(module, ctx),
                (Self::HeapIntDivMod, Some(ctx)) => {
                    integers::div::heap_integers_div_mod(module, ctx)
                }
                (Self::LessThan, Some(ctx)) => integers::check_if_a_less_than_b(module, ctx),
                // Swap
                (Self::SwapI32Bytes, _) => swap::swap_i32_bytes_function(module),
                (Self::SwapI64Bytes, _) => {
                    let swap_i32_f = Self::SwapI32Bytes.get(module, compilation_ctx);
                    swap::swap_i64_bytes_function(module, swap_i32_f)
                }
                // Bitwise
                (Self::HeapIntShiftLeft, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_left(module, ctx)
                }
                (Self::HeapIntShiftRight, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_right(module, ctx)
                }
                // Copy
                (Self::CopyU128, Some(ctx)) => copy::copy_u128_function(module, ctx),
                (Self::CopyU256, Some(ctx)) => copy::copy_u256_function(module, ctx),
                // Equality
                (Self::HeapTypeEquality, Some(ctx)) => equality::a_equals_b(module, ctx),
                (Self::VecEqualityHeapType, Some(ctx)) => {
                    equality::vec_equality_heap_type(module, ctx)
                }
                // Vector
                (Self::VecSwap32, Some(ctx)) => vector::vec_swap_32_function(module, ctx),
                (Self::VecSwap64, Some(ctx)) => vector::vec_swap_64_function(module, ctx),
                (Self::VecPopBack32, Some(ctx)) => vector::vec_pop_back_32_function(module, ctx),
                (Self::VecPopBack64, Some(ctx)) => vector::vec_pop_back_64_function(module, ctx),
                (Self::VecBorrow, Some(ctx)) => vector::vec_borrow_function(module, ctx),
                (Self::VecIncrementLen, Some(ctx)) => {
                    vector::increment_vec_len_function(module, ctx)
                }
                (Self::VecDecrementLen, Some(ctx)) => {
                    vector::decrement_vec_len_function(module, ctx)
                }
                // Error
                _ => panic!(
                    r#"there was an error linking "{}" function, missing compilation context?"#,
                    self.name()
                ),
            }
        }
    }
}
