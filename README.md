# compact-encoding

Rust implementation of `<https://github.com/compact-encoding/compact-encoding>`

## Usage

```
use compact_encoding::*;

let mut state = State::new();

let unsigned8: u8 = 42;
// use pre_encode to determine the required buffer size
unsigned8.pre_encode(&mut state);
//String::pre_encode(&mut state, "hi");

println!("State: {:#?}", state);

// allocate the buffer based on previous pre_encode() calls
state.alloc();

// actually encode the values into the buffer
unsigned8.encode(&mut state);
//String::encode(&mut state, "hi");

// for decoding, the decode() method is used (state.start should point to start of buffer)
state.start = 0;

let uint8 = u8::decode(&mut state).expect("could not decode");
println!("{:?}", uint8);
//let string = String::decode(&mut state).expect("could not decode"));
//println!("{:?}", string);
```

## Encoder API

### `State`

See [State]

### `val.preencode(state);`

Does a fast preencode dry-run that only sets state.end.
Use this to figure out the required buffer size.
Afterwards call `state.alloc();` to create `state.buffer`.

### `val.encode(state).expect("could not encode");`

Encodes `val` into `state.buffer` at position `state.start`.
Updates `state.start` to point after the encoded value when done.

### `let val = <type>::decode(state).expect("could not decode");`

Decodes a value from `state.buffer` at position `state.start`.
Updates `state.start` to point after the decoded value in the buffer when done.

## Helpers

If you are just encoding to a buffer or decoding from one you can use the `encode` and `decode` helpers
to reduce your boilerplate

```
//const buf = cenc.encode(cenc.bool, true)
//const bool = cenc.decode(cenc.bool, buf)
```

## Spec

Details on the encoding are documented in [spec.md](./doc/spec.md).

## Design

Details on design ideas are documented in [design.md](./doc/design.md).

## License

MIT
