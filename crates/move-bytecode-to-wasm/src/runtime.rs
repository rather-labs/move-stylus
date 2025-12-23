use std::hash::Hasher;

use error::RuntimeFunctionError;
use walrus::{FunctionId, GlobalId, Module};

use crate::{
    CompilationContext,
    hasher::get_hasher,
    translation::intermediate_types::{
        IntermediateType,
        heap_integers::{IU128, IU256},
    },
};

mod abi;
mod copy;
mod enums;
mod equality;
pub mod error;
mod integers;
mod storage;
mod swap;
mod unpacking;
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
    VecSwap,
    VecPopBack,
    VecBorrow,
    VecIncrementLen,
    VecDecrementLen,
    VecUpdateMutRef,
    BytesToVec,
    // Storage
    StorageNextSlot,
    DeriveMappingSlot,
    DeriveDynArraySlot,
    WriteObjectSlot,
    LocateStorageData,
    LocateStructSlot,
    GetIdBytesPtr,
    EncodeAndSaveInStorage,
    ReadAndDecodeFromStorage,
    DeleteFromStorage,
    CheckAndDeleteStructTtoFields,
    DeleteTtoObject,
    GetStructOwner,
    CommitChangesToStorage,
    AccumulateOrAdvanceSlotDelete,
    AccumulateOrAdvanceSlotRead,
    AccumulateOrAdvanceSlotWrite,
    CacheStorageObjectChanges,
    // Enums
    GetStorageSizeByOffset,
    ComputeEnumStorageTailPosition,
    // ASCII conversion
    U64ToAsciiBase10,
    // ABI validation
    ValidatePointer32Bit,
    // ABI unpacking
    UnpackVector,
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
            Self::VecSwap => "vec_swap",
            Self::VecPopBack => "vec_pop_back",
            Self::VecBorrow => "vec_borrow",
            Self::VecIncrementLen => "vec_increment_len",
            Self::VecDecrementLen => "vec_decrement_len",
            Self::VecUpdateMutRef => "vec_update_mut_ref",
            Self::BytesToVec => "bytes_to_vec",
            // Storage
            Self::StorageNextSlot => "storage_next_slot",
            Self::DeriveMappingSlot => "derive_mapping_slot",
            Self::DeriveDynArraySlot => "derive_dyn_array_slot",
            Self::LocateStorageData => "locate_storage_data",
            Self::WriteObjectSlot => "write_object_slot",
            Self::LocateStructSlot => "locate_struct_slot",
            Self::GetIdBytesPtr => "get_id_bytes_ptr",
            Self::EncodeAndSaveInStorage => "encode_and_save_in_storage",
            Self::ReadAndDecodeFromStorage => "read_and_decode_from_storage",
            Self::DeleteFromStorage => "delete_from_storage",
            Self::CheckAndDeleteStructTtoFields => "check_and_delete_struct_tto_fields",
            Self::DeleteTtoObject => "delete_tto_object",
            Self::GetStructOwner => "get_struct_owner",
            Self::U64ToAsciiBase10 => "u64_to_ascii_base_10",
            Self::CommitChangesToStorage => "commit_changes_to_storage",
            Self::AccumulateOrAdvanceSlotDelete => "accumulate_or_advance_slot_delete",
            Self::AccumulateOrAdvanceSlotRead => "accumulate_or_advance_slot_read",
            Self::AccumulateOrAdvanceSlotWrite => "accumulate_or_advance_slot_write",
            Self::CacheStorageObjectChanges => "cache_storage_object_changes",
            // Enums
            Self::GetStorageSizeByOffset => "get_storage_size_by_offset",
            Self::ComputeEnumStorageTailPosition => "compute_enum_storage_tail_position",
            // ABI validation
            Self::ValidatePointer32Bit => "validate_pointer_32_bit",
            // ABI unpacking
            Self::UnpackVector => "unpack_vector",
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
    ) -> Result<FunctionId, RuntimeFunctionError> {
        if let Some(function) = module.funcs.by_name(self.name()) {
            Ok(function)
        } else {
            let function_id = match (self, compilation_ctx) {
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
                    integers::div::heap_integers_div_mod(module, ctx)?
                }
                (Self::LessThan, Some(ctx)) => integers::check_if_a_less_than_b(module, ctx),
                // Swap
                (Self::SwapI32Bytes, _) => swap::swap_i32_bytes_function(module),
                (Self::SwapI64Bytes, _) => {
                    let swap_i32_f = Self::SwapI32Bytes.get(module, compilation_ctx)?;
                    swap::swap_i64_bytes_function(module, swap_i32_f)
                }
                (Self::SwapI128Bytes, Some(ctx)) => swap::swap_bytes_function::<2>(
                    module,
                    ctx,
                    Self::SwapI128Bytes.name().to_owned(),
                )?,
                (Self::SwapI256Bytes, Some(ctx)) => swap::swap_bytes_function::<4>(
                    module,
                    ctx,
                    Self::SwapI256Bytes.name().to_owned(),
                )?,
                // Bitwise
                (Self::HeapIntShiftLeft, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_left(module, ctx)?
                }
                (Self::HeapIntShiftRight, Some(ctx)) => {
                    integers::bitwise::heap_int_shift_right(module, ctx)?
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
                    equality::vec_equality_heap_type(module, ctx)?
                }
                (Self::IsZero, Some(ctx)) => equality::is_zero(module, ctx),
                // Vector
                (Self::VecBorrow, Some(ctx)) => vector::vec_borrow_function(module, ctx),
                (Self::VecIncrementLen, Some(ctx)) => {
                    vector::increment_vec_len_function(module, ctx)
                }
                (Self::VecDecrementLen, Some(ctx)) => {
                    vector::decrement_vec_len_function(module, ctx)
                }
                (Self::VecUpdateMutRef, Some(ctx)) => {
                    vector::vec_update_mut_ref_function(module, ctx)
                }
                (Self::BytesToVec, Some(ctx)) => vector::bytes_to_vec_function(module, ctx),
                // Storage
                (Self::StorageNextSlot, Some(ctx)) => {
                    storage::storage_next_slot_function(module, ctx)?
                }
                (Self::DeriveMappingSlot, Some(ctx)) => storage::derive_mapping_slot(module, ctx),
                (Self::DeriveDynArraySlot, Some(ctx)) => {
                    storage::derive_dyn_array_slot(module, ctx)?
                }
                (Self::WriteObjectSlot, Some(ctx)) => storage::write_object_slot(module, ctx)?,
                (Self::LocateStorageData, Some(ctx)) => storage::locate_storage_data(module, ctx)?,
                (Self::LocateStructSlot, Some(ctx)) => storage::locate_struct_slot(module, ctx)?,
                (Self::GetIdBytesPtr, Some(ctx)) => storage::get_id_bytes_ptr(module, ctx),
                (Self::GetStructOwner, None) => storage::get_struct_owner_fn(module),
                (Self::AccumulateOrAdvanceSlotDelete, Some(ctx)) => {
                    storage::accumulate_or_advance_slot_delete(module, ctx)?
                }
                (Self::AccumulateOrAdvanceSlotRead, Some(ctx)) => {
                    storage::accumulate_or_advance_slot_read(module, ctx)?
                }
                (Self::AccumulateOrAdvanceSlotWrite, Some(ctx)) => {
                    storage::accumulate_or_advance_slot_write(module, ctx)?
                }
                // ASCII conversion
                (Self::U64ToAsciiBase10, Some(ctx)) => {
                    integers::ascii::u64_to_ascii_base_10(module, ctx)
                }
                // ABI validation
                (Self::ValidatePointer32Bit, Some(ctx)) => {
                    abi::validate_pointer_32_bit(module, ctx)
                }
                // Error
                _ => return Err(RuntimeFunctionError::CouldNotLink(self.name().to_owned())),
            };

            Ok(function_id)
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
    ) -> Result<FunctionId, RuntimeFunctionError> {
        let function_id = match self {
            Self::EncodeAndSaveInStorage => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::add_encode_and_save_into_storage_fn(module, compilation_ctx, generics[0])?
            }
            Self::ReadAndDecodeFromStorage => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::add_read_and_decode_from_storage_fn(module, compilation_ctx, generics[0])?
            }
            Self::DeleteFromStorage => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::add_delete_struct_from_storage_fn(module, compilation_ctx, generics[0])?
            }
            Self::CheckAndDeleteStructTtoFields => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::add_check_and_delete_struct_tto_fields_fn(
                    module,
                    compilation_ctx,
                    generics[0],
                )?
            }
            Self::DeleteTtoObject => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::add_delete_tto_object_fn(module, compilation_ctx, generics[0])?
            }
            Self::CacheStorageObjectChanges => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                storage::cache_storage_object_changes(module, compilation_ctx, generics[0])?
            }
            Self::GetStorageSizeByOffset => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                enums::get_storage_size_by_offset(module, compilation_ctx, generics[0])?
            }
            Self::ComputeEnumStorageTailPosition => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                enums::compute_enum_storage_tail_position(module, compilation_ctx, generics[0])?
            }
            Self::VecSwap => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                vector::vec_swap_function(module, compilation_ctx, generics[0])?
            }
            Self::VecPopBack => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                vector::vec_pop_back_function(module, compilation_ctx, generics[0])?
            }
            Self::UnpackVector => {
                Self::assert_generics_length(generics.len(), 1, self.name())?;
                unpacking::add_unpack_vector_fn(module, compilation_ctx, generics[0])?
            }
            _ => {
                return Err(RuntimeFunctionError::CouldNotLinkGeneric(
                    self.name().to_owned(),
                ));
            }
        };

        Ok(function_id)
    }

    /// Links the function `commit_changes_to_storage` into the module and returns its id.
    ///
    /// This funciton is idempotent.
    ///
    /// It is not possible to obtain this function with the `get` method because it need the extra
    /// parameter `dynamic_fields_global_variables`
    pub fn get_commit_changes_to_storage_fn(
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        dynamic_fields_global_variables: &Vec<(GlobalId, IntermediateType)>,
    ) -> Result<FunctionId, RuntimeFunctionError> {
        if let Some(function) = module.funcs.by_name(Self::CommitChangesToStorage.name()) {
            Ok(function)
        } else {
            storage::add_commit_changes_to_storage_fn(
                module,
                compilation_ctx,
                dynamic_fields_global_variables,
            )
        }
    }

    fn assert_generics_length(
        len: usize,
        expected: usize,
        name: &str,
    ) -> Result<(), RuntimeFunctionError> {
        if expected != len {
            return Err(RuntimeFunctionError::WrongNumberOfTypeParameters {
                function_name: name.to_owned(),
                expected,
                found: len,
            });
        }

        Ok(())
    }

    pub fn get_generic_function_name(
        &self,
        compilation_ctx: &CompilationContext,
        generics: &[&IntermediateType],
    ) -> Result<String, RuntimeFunctionError> {
        if generics.is_empty() {
            return Err(RuntimeFunctionError::GenericFunctionNameNoGenerics);
        }

        let mut hasher = get_hasher();
        for t in generics {
            t.process_hash(&mut hasher, compilation_ctx)?;
        }
        let hash = format!("{:x}", hasher.finish());

        Ok(format!("runtime_{}_{hash}", self.name()))
    }
}
