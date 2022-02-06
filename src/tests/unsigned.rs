// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_uint8_pre_encode() {
        let mut state = State::new();

        42u8.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );
        state.end = 0;
        U8_MAX_VALUE.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );
        state.end = 0;
        (U8_MAX_VALUE + 1).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 3,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_uint16_pre_encode() {
        let mut state = State::new();

        (u16::MAX - 2).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 3,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_uint32_pre_encode() {
        let mut state = State::new();

        (u32::MAX - 2).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 5,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_uint64_pre_encode() {
        let mut state = State::new();

        (u64::MAX - 2).pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 9,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_uint8_encode() {
        let mut state = State::new();

        42u8.pre_encode(&mut state);

        state.alloc();

        assert_eq!(42u8.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 1,
                buffer: Some(vec![42]),
            }
        );
        state.start = 0;
        assert_eq!(U8_MAX_VALUE.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 1,
                buffer: Some(vec![0xFC]),
            }
        );

        state.dealloc();

        0xFDu8.pre_encode(&mut state);
        state.alloc();
        assert_eq!(0xFDu8.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 3,
                end: 3,
                buffer: Some(vec![U16_PREFIX, 0xFD, 0]),
            }
        );
    }

    #[test]
    fn test_uint16_encode() {
        let mut state = State::new();

        (i16::MAX - 2).pre_encode(&mut state);

        state.alloc();

        assert_eq!((u16::MAX - 2).encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 3,
                end: 3,
                buffer: Some(vec![
                    U16_PREFIX, 0xFD, 0xFF, // u16
                ]),
            }
        );
    }

    #[test]
    fn test_uint32_encode() {
        let mut state = State::new();

        (u32::MAX - 2).pre_encode(&mut state);

        state.alloc();

        assert_eq!((u32::MAX - 2).encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 5,
                end: 5,
                buffer: Some(vec![
                    U32_PREFIX, 0xFD, 0xFF, 0xFF, 0xFF, // u32
                ]),
            }
        );
    }

    #[test]
    fn test_uint64_encode() {
        let mut state = State::new();

        (u64::MAX - 2).pre_encode(&mut state);

        state.alloc();

        assert_eq!((u64::MAX - 2).encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 9,
                end: 9,
                buffer: Some(vec![
                    U64_PREFIX, 0xFD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
                ]),
            }
        );
    }
}
