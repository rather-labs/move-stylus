module test::misc;

#[ext(abi_error)]
public struct ErrorWithKey(String) has copy, drop, key;

#[ext(event)]
public struct EventWithKey(String) has key;

#[ext(abi_error)]
public struct ErrorOk(String) has drop;

// This is invalid because Error (and Panic) are reserved names.
#[ext(abi_error)]
public struct Error(String) has drop;

#[ext(event(indexes = 1))]
public struct EventOk has copy, drop {
    a: u64,
    b: u128,
}

// This is valid
entry fun revert_error(a: String) {
    revert(ErrorOk(a));
}

// This is invalid because Error (and Panic) are reserved names.
entry fun revert_error_invalid_name(s: String) {
    revert(Error(s));
}

// This is invalid because we cannot pass an error struct as an argument to a normal function, only as a revert argument.
entry fun revert_error_bad_arg(error: ErrorOk) {
    revert(error);
}

// This is valid
entry fun emit_event(a: u64, b: u128) {
    emit(EventOk { a, b });
}

// This is invalid because we cannot pass an event struct as an argument to a normal function, only as an emit argument.
entry fun emit_event_bad_arg(event: EventOk) {
    emit(event);
}