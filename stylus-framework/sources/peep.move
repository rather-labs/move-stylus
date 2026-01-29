// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module provides a function to peek into the storage of another address.
module stylus::peep;

use stylus::object::UID;

/// Peeks into the storage of another address and returns a reference to the object.
/// The owner address is the address of the account that owns the object.
/// The id is the id of the object to peek into.
/// 
/// Note the function is generic over the type `T`. 
/// If the provided type does not match the type hash of the object in storage, the function will abort.
public native fun peep<T: key>(owner_address: address, id: &UID): &T;