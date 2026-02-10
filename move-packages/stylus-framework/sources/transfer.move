// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module defines the core 'storage functions' used to manage object ownership 
/// and accessibility. 
/// 
/// All functions are generic over the type `T`, which must possess the `key` 
/// ability to be recognized as a persistent object in global storage.
module stylus::transfer;

/// Transfers the ownership of `obj` to the specified `recipient`.
///
/// Once transferred, only the recipient address has the authority to access 
/// or further manipulate the object.
public native fun transfer<T: key>(obj: T, recipient: address);

/// Converts `obj` into an immutable "frozen" object.
///
/// Frozen objects are read-only and globally accessible. After freezing, 
/// an object can no longer be transferred, mutated, or deleted. 
/// Any attempt to perform a state-changing operation on a frozen object 
/// will result in a runtime error.
public native fun freeze_object<T: key>(obj: T);

/// Converts `obj` into a mutable shared object that is accessible by any user.
///
/// Shared objects serve as global state that multiple participants can 
/// read and mutate. While a shared object cannot be frozen or transferred 
/// to a single owner, it remains eligible for deletion if permitted by 
/// the module logic. Any attempt to transfer or freeze a shared object 
/// will result in a runtime error.
public native fun share_object<T: key>(obj: T);