# compact-encoding

Rust implementation of `<https://github.com/compact-encoding/compact-encoding>`

## Usage

```
use compact_encoding::*;

let mut state = State::new();

// use pre_encode to determine the required buffer size
Uint8::pre_encode(&mut state, 42);
//String::pre_encode(&mut state, "hi");

println!("State: {:#?}", state);

// allocate the buffer based on previous pre_encode() calls
state.alloc();

// actually encode the values into the buffer
Uint8::encode(&mut state, 42);
//String::encode(&mut state, "hi");

// for decoding, the decode() method is used (state.start should point to start of buffer)
state.start = 0;

let uint8 = Uint8::decode(&mut state).expect("could not decode");
println!("{:?}", uint8);
//let string = String::decode(&mut state).expect("could not decode"));
//println!("{:?}", string);
```

## Encoder API

### `State`

See [State]

### `enc.preencode(state, val)`

Does a fast preencode dry-run that only sets state.end.
Use this to figure out the required buffer size.
Afterwards call `state.alloc();` to create `state.buffer`.

### `enc.encode(state, val)`

Encodes `val` into `state.buffer` at position `state.start`.
Updates `state.start` to point after the encoded value when done.

### `val = enc.decode(state)`

Decodes a value from `state.buffer` at position `state.start`.
Updates `state.start` to point after the decoded value when done in the buffer.

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
