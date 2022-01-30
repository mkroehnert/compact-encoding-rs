# compact_encoding Rust design ideas

Use features instead of dedicated crates for base encoding, -net, and -struct libraries.

Make the original tests pass + add more tests.

## Follow original API?

One Trait which en-/decodes the primitive types without headers.
One Trait which en-/decodes the types with headers.

dedicated unit-structs which implement the traits for the individual types

### Pros

### Cons

* uint type is difficult, since it needs to return a variant of all the possible subtypes (may not be ergonomic to use)

## Implement pre_encoce(), encode(), decode() on supported types?

### Pros

### Cons

## implement as a serde backend?

### Pros

### Cons
