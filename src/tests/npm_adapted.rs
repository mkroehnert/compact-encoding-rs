// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

#[cfg(test)]
mod tests {
    use crate::*;

    // mdn: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
    const MAX_SAFE_INTEGER: u64 = (2 as u64).pow(53) - 1;

    #[test]
    fn test_uint() {
        let mut state = State::new();

        (42u8).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );

        (4200u16).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 4,
                buffer: None,
            }
        );

        MAX_SAFE_INTEGER.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 13,
                buffer: None,
            }
        );

        state.alloc();

        assert_eq!(42u8.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 13,
                buffer: Some(vec![42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        assert_eq!(4200u16.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 4,
                end: 13,
                buffer: Some(vec![42, 0xFD, 104, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );
        assert_eq!(MAX_SAFE_INTEGER.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 13,
                end: 13,
                buffer: Some(vec![
                    42, 0xFD, 104, 16, 0xFF, 255, 255, 255, 255, 255, 255, 31, 0
                ]),
            }
        );

        state.start = 0;
        assert_eq!(u8::decode(&mut state), Ok(42));
        assert_eq!(state.start, 1);
        assert_eq!(state.end, 13);

        assert_eq!(u16::decode(&mut state), Ok(4200));
        assert_eq!(state.start, 4);
        assert_eq!(state.end, 13);

        assert_eq!(u64::decode(&mut state), Ok(MAX_SAFE_INTEGER));
        assert_eq!(state.start, 13);
        assert_eq!(state.end, 13);

        assert_eq!(state.start, state.end);

        assert_eq!(u8::decode(&mut state), Err(DecodeError::BufferTooSmall));
    }

    #[test]
    fn test_int() {
        let mut state = State::new();

        42i8.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );

        (-4200i16).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 4,
                buffer: None,
            }
        );

        state.alloc();

        assert_eq!(42i8.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 4,
                buffer: Some(vec![84, 0, 0, 0]),
            }
        );

        assert_eq!((-4200i16).encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 4,
                end: 4,
                buffer: Some(vec![84, 0xFD, 207, 32]),
            }
        );

        state.start = 0;
        assert_eq!(i8::decode(&mut state), Ok(42));
        assert_eq!(state.start, 1);
        assert_eq!(state.end, 4);

        assert_eq!(i16::decode(&mut state), Ok(-4200));
        assert_eq!(state.start, 4);
        assert_eq!(state.end, 4);

        assert_eq!(state.start, state.end);

        assert_eq!(i8::decode(&mut state), Err(DecodeError::BufferTooSmall));
    }

    #[test]
    fn test_float64() {
        let mut state = State::new();
        const NUM: f64 = 162.2377294;

        NUM.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 8,
                buffer: None,
            }
        );

        state.alloc();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 8,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        assert_eq!(NUM.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
            }
        );

        state.start = 0;
        assert_eq!(f64::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(f64::decode(&mut state), Err(DecodeError::BufferTooSmall));

        // alignment
        state.dealloc();

        0u8.pre_encode(&mut state);
        NUM.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 9,
                buffer: None,
            }
        );

        state.alloc();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 9,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        assert_eq!(0u8.encode(&mut state), Ok(()));
        assert_eq!(NUM.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 9,
                end: 9,
                buffer: Some(vec![0, 0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
            }
        );

        state.start = 0;
        assert_eq!(u8::decode(&mut state), Ok(0));
        assert_eq!(f64::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(f64::decode(&mut state), Err(DecodeError::BufferTooSmall));

        // subarray (replace buffer?)
        // TODO: check what this test is about and why it is needed
        //       would require state.buffer to point to a buffer instead of buffer being a member
        // let buffer = vec![0; 10];
        // state.buffer = &buffer[1..];
        // assert_eq!(
        //     state,
        //     State {
        //         start: 0,
        //         end: 9,
        //         buffer: Some(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]),
        //     }
        // );

        // assert_eq!(0u8.encode(&mut state), Ok(()));
        // assert_eq!(NUM.encode(&mut state), Ok(()));
        // assert_eq!(
        //     state,
        //     State {
        //         start: 9,
        //         end: 9,
        //         buffer: Some(vec![0, 0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
        //     }
        // );

        // state.start = 0;
        // assert_eq!(u8::decode(&mut state), Ok(0));
        // assert_eq!(f64::decode(&mut state), Ok(NUM));
        // assert_eq!(state.start, state.end);

        // 0
        state.dealloc();
        NUM.pre_encode(&mut state);

        state.alloc();
        assert_eq!(0f64.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        state.start = 0;
        assert_eq!(f64::decode(&mut state), Ok(0.));
        assert_eq!(state.start, state.end);

        // infinity
        state.dealloc();
        f64::INFINITY.pre_encode(&mut state);

        state.alloc();
        assert_eq!(f64::INFINITY.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0xF0, 0x7F]),
            }
        );

        state.start = 0;
        assert_eq!(f64::decode(&mut state), Ok(f64::INFINITY));
        assert_eq!(state.start, state.end);

        // edge cases
        state.dealloc();
        (0.1 + 0.2).pre_encode(&mut state);

        state.alloc();
        assert_eq!((0.1 + 0.2).encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0x34, 0x33, 0x33, 0x33, 0x33, 0x33, 0xD3, 0x3F]),
            }
        );

        state.start = 0;
        assert_eq!(f64::decode(&mut state), Ok(0.1 + 0.2));
        assert_eq!(state.start, state.end);
    }
}
