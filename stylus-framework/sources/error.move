module stylus::error;

/// Reverts the current transaction.
///
/// This function reverts the current transaction with a given error.
public native fun revert<T: copy + drop>(error: T);
