module hello_world::stack;

/// Very simple stack implementation using the wrapper type pattern. Does not allow
/// accessing the elements unless they are popped.
public struct Stack<T>(vector<T>) has copy, store, drop;

/// Create a new instance by wrapping the value.
public fun new<T>(value: vector<T>): Stack<T> {
    Stack(value)
}

/// Push an element to the stack.
public fun push_back<T>(v: &mut Stack<T>, el: T) {
    v.0.push_back(el);
}

/// Pop an element from the stack. Unlike `vector`, this function won't
/// fail if the stack is empty and will return `None` instead.
public fun pop_back<T>(v: &mut Stack<T>): Option<T> {
    if (v.0.length() == 0) option::none()
    else option::some(v.0.pop_back())
}

/// Get the size of the stack.
public fun size<T>(v: &Stack<T>): u64 {
    v.0.length()
}
