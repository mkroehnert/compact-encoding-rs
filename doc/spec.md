# compact-encoding spec


(uintX) = [byteN, ..., byte2, byte1, byte0]

least significant bit (LSB) for little endian = byte0

| Type            | Encoding                                                         | Implemented | Tested |
|:----------------|:-----------------------------------------------------------------|-------------|--------|
| none            |                                                                  | [ ]         | [ ]    |
| bool            | true is encoded as 1u8, false as 0u8                             | [X]         | [ ]    |
|                 |                                                                  |             |        |
| uint            | see dedicated uintX                                              | [-]         | [ ]    |
| uint8           | [0xFC, byte0] (byte <= 0xFC)                                     | [X]         | [ ]    |
| uint16          | [0xFD, byte0, byte1]                                             | [X]         | [ ]    |
| uint24          | available in JS, but not called automatically                    | [-]         | [ ]    |
| uint32          | [0xFE, byte0, byte1, byte2, byte3]                               | [X]         | [ ]    |
| uint64          | [0xFF, byte0, byte1, byte2, byte3, byte4, byte5, byte6, byte7]   | [x]         | [ ]    |
|                 |                                                                  |             |        |
| int             | apply zigzag before encoding as uX or after decoding from uX     | [-]         | [ ]    |
| int8            | zigzag then u8                                                   | [X]         | [ ]    |
| int16           | zigzag then u16                                                  | [X]         | [ ]    |
| int24           | available in JS, but not called automatically                    | [-]         | [ ]    |
| int32           | zigzag then u32                                                  | [X]         | [ ]    |
| int64           | zigzag then u64                                                  | [X]         | [ ]    |
|                 |                                                                  |             |        |
| float32         | no header, encoded as little endian                              | [X]         | [ ]    |
| float64         | no header, encoded as little endian                              | [X]         | [ ]    |
|                 |                                                                  |             |        |
| buffer/uint8[]? | encode length first as uint, then copy buffer                    | [X]         | [ ]    |
| uint32[]        | each value encoded as little-endian, prefix is number of u32     | [?]         | [ ]    |
| array           | encode length first, then all members (must be same type)        | [X]         | [ ]    |
|                 |                                                                  |             |        |
| raw             | buffer gets copied, no header, decoding returns remaining buffer | [X]         | [ ]    |
|                 |                                                                  |             |        |
| string          | string size encoded as header, utf8 copied as byte buffer        | [X]         | [ ]    |
|                 |                                                                  |             |        |
| fixed32         |                                                                  | [ ]         | [ ]    |
| fixed64         |                                                                  | [ ]         | [ ]    |
|                 |                                                                  |             |        |

## zigzag en/de-coding

## differences to JS implementation
* buffer: a buffer is regarded as a &[u8] slice, which can not be a nullptr. Therefore, the slice is currently wrapped in an Option for expressing non-existent buffers.
* raw: since the Encoder/Decoder traits are attached to types, en-/decoding a buffer as raw is attached to the Raw enum. This allows encoding different buffer types as raw and return a newly allocated buffer during decoding. (This resembles the JS implementation, where a new Buffer is returned).
* String: first implementation encodes &str and decodes to String (due to new buffer allocation) 
* Array: encoding for [T; N] and Vec<T>, decoding currently only into Vec<T>
* uint32 buffer/array requires separate struct U32Array, since Vec<T> already exists. Should it be possible to serialize Vec<u32> differently then a uint32 buffer/array? If not, rust unstable feature of `default` trait implementation would be required.

# compact-encoding-struct

# compact-encoding-net
