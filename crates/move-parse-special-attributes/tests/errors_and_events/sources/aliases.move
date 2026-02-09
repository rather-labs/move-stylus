module test::aliases;


use stylus::error::revert as revert_alias;
use stylus::event::emit as emit_alias;

#[ext(abi_error)]
public struct ErrorWithKey(String) has copy, drop, key;

#[ext(event)]
public struct EventWithKey(String) has key;

#[ext(abi_error)]
public struct ErrorOk(String) has drop;

#[ext(abi_error)]
public struct ErrorOk2{
    a: u64,
    b: u128,
} has drop;

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
    revert_alias(ErrorOk(a));
}

// This is valid too
entry fun revert_error_2(a: String) {
    let e = ErrorOk(a);
    revert_alias(e);
}

// This is valid
entry fun revert_error_conditional(s1: String, s2: String, b: bool) {
    if (b) {
        revert_alias(ErrorOk(s1));
    } else {
        revert_alias(ErrorOk(s2));
    }
}

// This is invalid (revert expects an abi_error struct)
// The error should be caught by parsing the function body and validating the calls to the native revert function.
entry fun revert_error_check_body(s: String) {
    let e = EventOk(s);
    revert_alias(e);
}

// This is invalid too as one of the branches has a revert call with an event struct, not an error struct.
entry fun revert_error_check_body_2(b: bool) {
    if (b) {
        revert_alias(ErrorOk2 { a: 1, b: 1 });
    } else {
        revert_alias(EventOk { a: 2, b: 2 });
    }
}

// This is invalid because Error (and Panic) are reserved names, hence they are not included in the abi_error structs.
// When validating the call to the revert, the argument is not found in the abi_error structs, hence the error is raised.
entry fun revert_error_invalid_name(s: String) {
    revert_alias(Error(s));
}

// This is invalid because we cannot pass an error struct as an argument to a normal function, only as a revert argument.
entry fun revert_error_bad_arg(error: ErrorOk) {
    revert_alias(error);
}

// This is valid
entry fun emit_event(a: u64, b: u128) {
    emit_alias(EventOk { a, b });
}

// This is invalid because we cannot pass an event struct as an argument to a normal function, only as an emit argument.
entry fun emit_event_bad_arg(event: EventOk) {
    emit_alias(event);
}

// This is invalid because the emit expects an event struct, not an error struct.
entry fun emit_event(s: String) {
    let e = ErrorOk(s);
    emit_alias(e);
}
