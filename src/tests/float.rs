// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_float32() {
        let mut state = State::new();
        const NUM: f32 = 162.2377294;

        NUM.pre_encode(&mut state);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 4,
                buffer: None,
            }
        );

        state.alloc();
        assert_eq!(
            state,
            State {
                start: 0,
                end: 4,
                buffer: Some(vec![0, 0, 0, 0]),
            }
        );

        assert_eq!(NUM.encode(&mut state), Ok(()));
        assert_eq!(
            state,
            State {
                start: 4,
                end: 4,
                // TODO double check expected value
                buffer: Some(vec![0xDC, 0x3C, 0x22, 0x43]),
            }
        );

        state.start = 0;
        assert_eq!(f32::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(f32::decode(&mut state), Err(DecodeError::BufferTooSmall));
    }
}
