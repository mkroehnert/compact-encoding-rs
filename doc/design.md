# compact_encoding Rust design ideas

Use features instead of dedicated crates for base encoding, -net, and -struct libraries.

Make the original tests pass + add more tests.

## Follow original API

Trait which en-/decodes the primitive types without headers.
Trait which en-/decodes the types with headers.

dedicated unit-structs which implement the traits for the individual types

### Pros

### Cons

* uint type is difficult, since it needs to return a variant of all the possible subtypes (may not be ergonomic to use)
* does not seem very rust-y

## Implement pre_encode(), encode(), decode() on supported types?


### Pros

* approach is also used by other crates (e.g. [bincode](https://github.com/bincode-org/bincode))
* feels more rust-y
* no extra types just for serialization

### Cons

## implement as a serde backend?


### Pros

* Can be done on top of previous example

### Cons
