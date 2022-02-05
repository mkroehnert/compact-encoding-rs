# compact-encoding spec


(uintX) = [byteN, ..., byte2, byte1, byte0]

least significant bit (LSB) for little endian = byte0

| Type            | Encoding                                                       | Implemented | Tested |
|:----------------|:---------------------------------------------------------------|-------------|--------|
| none            |                                                                | [ ]         | [ ]    |
| bool            | true is encoded as 1u8, false as 0u8                           | [X]         | [ ]    |
|                 |                                                                |             |        |
| uint            | see dedicated uintX                                            | [-]         | [ ]    |
| uint8           | [0xFC, byte0] (byte <= 0xFC)                                   | [X]         | [ ]    |
| uint16          | [0xFD, byte0, byte1]                                           | [X]         | [ ]    |
| uint24          | available in JS, but not called automatically                  | [-]         | [ ]    |
| uint32          | [0xFE, byte0, byte1, byte2, byte3]                             | [X]         | [ ]    |
| uint64          | [0xFF, byte0, byte1, byte2, byte3, byte4, byte5, byte6, byte7] | [x]         | [ ]    |
|                 |                                                                |             |        |
| int             | apply zigzag before encoding as uX or after decoding from uX   | [-]         | [ ]    |
| int8            |                                                                | [X]         | [ ]    |
| int16           |                                                                | [X]         | [ ]    |
| int24           | available in JS, but not called automatically                  | [-]         | [ ]    |
| int32           |                                                                | [X]         | [ ]    |
| int64           |                                                                | [X]         | [ ]    |
|                 |                                                                |             |        |
| float32         | no header, encoded as little endian                            | [X]         | [ ]    |
| float64         | no header, encoded as little endian                            | [X]         | [ ]    |
|                 |                                                                |             |        |
| buffer/uint8[]? | encode length first as uint, then copy buffer                  | [X]         | [ ]    |
| uint32[]        |                                                                | [ ]         | [ ]    |
| array           |                                                                | [ ]         | [ ]    |
|                 |                                                                |             |        |
| raw             |                                                                | [ ]         | [ ]    |
|                 |                                                                |             |        |
| string          |                                                                | [ ]         | [ ]    |
|                 |                                                                |             |        |
| fixed32         |                                                                | [ ]         | [ ]    |
| fixed64         |                                                                | [ ]         | [ ]    |
|                 |                                                                |             |        |

## zigzag en/de-coding

# compact-encoding-struct

# compact-encoding-net
