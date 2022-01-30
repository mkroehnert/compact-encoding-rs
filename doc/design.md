# compact_encoding Rust design ideas

## Follow original API?

unit structs with associated method?

### Pros

### Cons

* uint type is difficult, since it needs to return a variant of all the possible subtypes (may not be ergonomic to use)

## Implement pre_encoce(), encode(), decode() on supported types?

### Pros

### Cons

## implement as a serde backend?

### Pros

### Cons
