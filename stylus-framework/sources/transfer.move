module stylus::transfer;

/// Transfer ownership of `obj` to `recipient`. `obj` must have the `key` attribute,
/// which (in turn) ensures that `obj` has a globally unique ID. Note that if the recipient
/// address represents an object ID, the `obj` sent will be inaccessible after the transfer
/// (though they will be retrievable at a future date once new features are added).
/// This function has custom rules performed by the Sui Move bytecode verifier that ensures
/// that `T` is an object defined in the module where `transfer` is invoked. Use
/// `public_transfer` to transfer an object with `store` outside of its module.
public native fun transfer<T: key>(obj: T, recipient: address);

/// Freeze `obj`. After freezing `obj` becomes immutable and can no longer be transferred or
/// mutated.
/// This function has custom rules performed by the Sui Move bytecode verifier that ensures
/// that `T` is an object defined in the module where `freeze_object` is invoked. Use
/// `public_freeze_object` to freeze an object with `store` outside of its module.
public native fun freeze_object<T: key>(obj: T);

public native fun share_object<T: key>(obj: T);
