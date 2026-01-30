// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module defines the error handling mechanism for the Stylus framework.
///
/// It provides a native function to revert the current transaction with a given error.
module stylus::error;

/// Reverts the current transaction with a an error of type `T`.
/// The error is abi-encoded as expected by the EVM.
public native fun revert<T: copy + drop>(error: T);
