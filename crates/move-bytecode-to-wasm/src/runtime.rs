use walrus::{FunctionId, Module};

use crate::{
    CompilationContext,
    translation::intermediate_types::{
        IntermediateType,
        heap_integers::{IU128, IU256},
    },
};

mod copy;
mod equality;
mod integers;
mod storage;
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
    SwapI128Bytes,
    SwapI256Bytes,
    // Copy
    CopyU128,
    CopyU256,
    // Equality
    HeapTypeEquality,
    VecEqualityHeapType,
    IsZero,
    // Vector
    VecSwap32,
    VecSwap64,
    VecPopBack32,
    VecPopBack64,
    VecBorrow,
    VecIncrementLen,
    VecDecrementLen,
    // Storage
    StorageNextSlot,
    DeriveMappingSlot,
    DeriveDynArraySlot,
    WriteObjectSlot,
    LocateStorageData,
    LocateStructSlot,
    GetIdBytesPtr,
    EncodeAndSaveInStorage,
    DecodeAndReadFromStorage,
    DeleteFromStorage,
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
            Self::SwapI128Bytes => "swap_i128_bytes",
            Self::SwapI256Bytes => "swap_i256_bytes",
            // Copy
            Self::CopyU128 => "copy_u128",
            Self::CopyU256 => "copy_u256",
            // Equality
            Self::HeapTypeEquality => "heap_type_equality",
            Self::VecEqualityHeapType => "vec_equality_heap_type",
            Self::IsZero => "is_zero",
            // Vector
            Self::VecSwap32 => "vec_swap_32",
            Self::VecSwap64 => "vec_swap_64",
            Self::VecPopBack32 => "vec_pop_back_32",
            Self::VecPopBack64 => "vec_pop_back_64",
            Self::VecBorrow => "vec_borrow",
            Self::VecIncrementLen => "vec_increment_len",
            Self::VecDecrementLen => "vec_decrement_len",
            // Storage
            Self::StorageNextSlot => "storage_next_slot",
            Self::DeriveMappingSlot => "derive_mapping_slot",
            Self::DeriveDynArraySlot => "derive_dyn_array_slot",
            Self::LocateStorageData => "locate_storage_data",
            Self::WriteObjectSlot => "write_object_slot",
            Self::LocateStructSlot => "locate_struct_slot",
            Self::GetIdBytesPtr => "get_id_bytes_ptr",
            Self::EncodeAndSaveInStorage => "encode_and_save_in_storage",
            Self::DecodeAndReadFromStorage => "decode_and_read_from_storage",
            Self::DeleteFromStorage => "delete_from_storage",
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
                (Self::SwapI128Bytes, Some(ctx)) => swap::swap_bytes_function::<2>(
                    module,
                    ctx,
                    Self::SwapI128Bytes.name().to_owned(),
                ),
                (Self::SwapI256Bytes, Some(ctx)) => swap::swap_bytes_function::<4>(
                    module,
                    ctx,
                    Self::SwapI256Bytes.name().to_owned(),
                ),
                // Bitwise
                (Self::HeapIntShiftLeft, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_left(module, ctx)
                }
                (Self::HeapIntShiftRight, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_right(module, ctx)
                }
                // Copy
                (Self::CopyU128, Some(ctx)) => {
                    copy::copy_heap_int_function::<{ IU128::HEAP_SIZE }>(
                        module,
                        ctx,
                        Self::CopyU128.name().to_owned(),
                    )
                }
                (Self::CopyU256, Some(ctx)) => {
                    copy::copy_heap_int_function::<{ IU256::HEAP_SIZE }>(
                        module,
                        ctx,
                        Self::CopyU256.name().to_owned(),
                    )
                }
                // Equality
                (Self::HeapTypeEquality, Some(ctx)) => equality::a_equals_b(module, ctx),
                (Self::VecEqualityHeapType, Some(ctx)) => {
                    equality::vec_equality_heap_type(module, ctx)
                }
                (Self::IsZero, Some(ctx)) => equality::is_zero(module, ctx),
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
                // Storage
                (Self::StorageNextSlot, Some(ctx)) => {
                    storage::storage_next_slot_function(module, ctx)
                }
                (Self::DeriveMappingSlot, Some(ctx)) => storage::derive_mapping_slot(module, ctx),
                (Self::DeriveDynArraySlot, Some(ctx)) => {
                    storage::derive_dyn_array_slot(module, ctx)
                }
                (Self::WriteObjectSlot, Some(ctx)) => storage::write_object_slot(module, ctx),
                (Self::LocateStorageData, Some(ctx)) => storage::locate_storage_data(module, ctx),
                (Self::LocateStructSlot, Some(ctx)) => storage::locate_struct_slot(module, ctx),
                (Self::GetIdBytesPtr, Some(ctx)) => storage::get_id_bytes_ptr(module, ctx),
                // Error
                _ => panic!(
                    r#"there was an error linking "{}" runtime function, missing compilation context?"#,
                    self.name()
                ),
            }
        }
    }

    /// Links the function into the module and returns its id. The function generated depends on
    /// the types passed in the `generics` parameter.
    ///
    /// The idempotency of this function depends on the generator functions. This is designed this
    /// way to avoid errors when calculating the function name based on the types.
    pub fn get_generic(
        &self,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        generics: &[&IntermediateType],
    ) -> FunctionId {
        match self {
            Self::EncodeAndSaveInStorage => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {} expected 1 type parameter, found {}",
                    self.name(),
                    generics.len(),
                );

                storage::add_save_struct_into_storage_fn(module, compilation_ctx, generics[0])
            }
            Self::DecodeAndReadFromStorage => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {} expected 1 type parameter, found {}",
                    self.name(),
                    generics.len(),
                );

                storage::add_read_struct_from_storage_fn(module, compilation_ctx, generics[0])
            }
            Self::DeleteFromStorage => {
                assert_eq!(
                    1,
                    generics.len(),
                    "there was an error linking {} expected 1 type parameter, found {}",
                    self.name(),
                    generics.len(),
                );

                storage::add_delete_struct_from_storage_fn(module, compilation_ctx, generics[0])
            }
            _ => panic!(
                r#"there was an error linking "{}" runtime function, is this function generic?"#,
                self.name()
            ),
        }
    }
}
