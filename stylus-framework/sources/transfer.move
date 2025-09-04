module stylus::transfer;

/// Transfer ownership of `obj` to `recipient`. `obj` must have the `key` attribute,
/// which (in turn) ensures that `obj` has a globally unique ID.
public native fun transfer<T: key>(obj: T, recipient: address);

/// Freezes `obj`. After freezing `obj` becomes immutable and can no longer be transferred or
/// mutated.
public native fun freeze_object<T: key>(obj: T);

/// Turns the given object into a mutable shared object that everyone can access and mutate.
/// This is irreversible, i.e. once an object is shared, it will stay shared forever.
public native fun share_object<T: key>(obj: T);
