// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_new_empty_state() {
        let state = State::new();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 0,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_state_alloc() {
        let mut state = State::new();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 0,
                buffer: None,
            }
        );
        state.end = 5;
        state.alloc();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 5,
                buffer: Some(vec![0, 0, 0, 0, 0]),
            }
        );
    }

    #[test]
    fn test_zig_zag_encode() {
        assert_eq!(zig_zag_encode(0), 0);
        assert_eq!(zig_zag_encode(1), 2);
        assert_eq!(zig_zag_encode(2), 4);
        assert_eq!(zig_zag_encode(3), 6);
        assert_eq!(zig_zag_encode(4), 8);
        assert_eq!(zig_zag_encode(5), 10);
        assert_eq!(zig_zag_encode(6), 12);
        assert_eq!(zig_zag_encode(42), 84);
        assert_eq!(zig_zag_encode(-4200), 8399);
    }

    #[test]
    fn test_bool_pre_encode() {
        let mut state = State::new();

        true.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );
        false.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 2,
                buffer: None,
            }
        );
    }

    #[test]
    fn test_bool_encode() {
        let mut state = State::new();

        true.pre_encode(&mut state);
        false.pre_encode(&mut state);

        state.alloc();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 2,
                buffer: Some(vec![0, 0]),
            }
        );

        assert_eq!(true.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 2,
                buffer: Some(vec![1, 0]),
            }
        );

        assert_eq!(false.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 2,
                end: 2,
                buffer: Some(vec![1, 0]),
            }
        );
    }

    #[test]
    fn test_bool_decode() {
        let mut state = State::new();

        true.pre_encode(&mut state);
        false.pre_encode(&mut state);

        state.alloc();

        assert_eq!(true.encode(&mut state), Ok(()));
        assert_eq!(false.encode(&mut state), Ok(()));

        state.start = 0;
        assert_eq!(bool::decode(&mut state), Ok(true));
        assert_eq!(bool::decode(&mut state), Ok(false));
        assert_eq!(state.start, state.end);

        assert_eq!(f32::decode(&mut state), Err(DecodeError::BufferTooSmall));
    }
}
