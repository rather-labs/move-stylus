# Pattern: Hot Potato

Within the abilities system, a struct that has no abilities is referred to as a *hot potato*. Such a struct cannot be stored (neither as an object nor as a field within another struct), and it cannot be copied or discarded. Therefore, once constructed, it must be properly unpacked by its defining module. If left unused, the compiler will throw an error due to the presence of a value without the `drop` ability.

The term originates from the children’s game in which a ball is passed rapidly among players, and no one wants to be the last to hold it when the music stops—otherwise, they are out of the game. This serves as the perfect analogy for the pattern: an instance of a hot-potato struct is passed between calls, and no module is allowed to retain it.

## Defining a Hot Potato

A hot potato can be any struct without abilities. For instance, the following struct qualifies as a hot potato:

```move
public struct Request {}
```

Since `Request` has no abilities and cannot be stored or ignored, the module must provide a function to unpack it. For example

```move
/// Constructs a new `Request`.
public fun new_request(): Request {
    Request {}
}

/// Unpacks the `Request`. Because of the hot potato pattern, this function
/// must be called to prevent the compiler from throwing an error due to an
/// unused value.
public fun confirm_request(request: Request) {
    let Request {} = request;
}
```


