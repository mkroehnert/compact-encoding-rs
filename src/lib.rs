// SPDX-License-Identifier: MIT
#![doc = include_str!("../README.md")]
#![doc(test(no_crate_inject))]
#![doc(html_no_source)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod error;

use crate::error::*;

/// State contains the en-/decode buffer and index metadata.
///
/// State is an object that looks like this `{ start, end, buffer }`.
///
/// A blank state object is created using `State::new()`.
///
/// * `start` is the byte offset to start encoding/decoding at.
/// * `end` is the byte offset indicating the end of the buffer.
/// * `buffer` is a Vec<u8>.
#[derive(Debug, PartialEq)]
pub struct State {
    pub start: usize,
    end: usize,
    buffer: Option<Vec<u8>>,
}

impl State {
    /// create a new and empty State instance
    pub fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            buffer: None,
        }
    }

    /// allocate an internal buffer based on self.end
    pub fn alloc(&mut self) {
        // TODO: throw error if alloc fails?
        self.buffer = Some(vec![0; self.end]);
    }


    /// get reference to next u8 value if one exists
    pub fn next_u8(&self) -> Option<&u8> {
        if self.start >= self.end {
            None
        } else if let Some(ref buffer) = self.buffer {
            Some(&buffer[self.start])
        } else {
            None
        }
    }
}

/// Trait which defines the required encoding and decoding functions
pub trait EnDecoder {
    /// the type to be en-/decoded
    type Base;
    //type DecodeResult = DecodeResultT<Self::Base>;

    /// size of the type to en-/decode
    const SIZE: usize = std::mem::size_of::<Self::Base>();

    /// allocate the required size in State for current type
    fn pre_encode(state: &mut State, n: Self::Base);
    /// encode n with state
    /// requires state.buffer to be allocated first
    fn encode(state: &mut State, n: Self::Base) -> EncodeResult;
    /// decode value at current buffer pointer into Self::Base
    fn decode(state: &mut State) -> DecodeResultT<Self::Base>;
}

/// encoder/decoder for u8
pub struct Uint8();

// implement directly on u8?
impl EnDecoder for Uint8 {
    type Base = u8;

    fn pre_encode(state: &mut State, _: Self::Base) {
        state.end += Self::SIZE;
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
        buffer[state.start] = n;
        state.start += 1;
        Ok(())
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let value: Self::Base = buffer[state.start];
        state.start += 1;
        Ok(value)
    }
}

pub struct Uint16();

impl EnDecoder for Uint16 {
    type Base = u16;

    fn pre_encode(state: &mut State, _: Self::Base) {
        state.end += Self::SIZE;
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        let buffer = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| EncodeError::NoBuffer)?;
        buffer[state.start] = n as u8;
        state.start += 1;
        buffer[state.start] = (n >> 8) as u8;
        state.start += 1;
        Ok(())
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let mut value: Self::Base = buffer[state.start] as Self::Base;
        state.start += 1;
        value += buffer[state.start] as Self::Base * 256;
        state.start += 1;
        Ok(value)
    }
}

pub struct Uint32();

impl EnDecoder for Uint32 {
    type Base = u32;

    fn pre_encode(state: &mut State, _: Self::Base) {
        state.end += Self::SIZE;
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        let buffer = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| EncodeError::NoBuffer)?;
        buffer[state.start] = n as u8;
        state.start += 1;
        buffer[state.start] = (n >> 8) as u8;
        state.start += 1;
        buffer[state.start] = (n >> (8 * 2)) as u8;
        state.start += 1;
        buffer[state.start] = (n >> (8 * 3)) as u8;
        state.start += 1;
        Ok(())
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let mut value: Self::Base = buffer[state.start] as Self::Base;
        state.start += 1;
        value += buffer[state.start] as Self::Base * 256;
        state.start += 1;
        value += buffer[state.start] as Self::Base * ((256 as Self::Base).pow(2));
        state.start += 1;
        value += buffer[state.start] as Self::Base * ((256 as Self::Base).pow(3));
        state.start += 1;
        Ok(value)
    }
}

pub struct Uint64();

impl EnDecoder for Uint64 {
    type Base = u64;

    fn pre_encode(state: &mut State, _: Self::Base) {
        state.end += Self::SIZE;
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        let r = n / (2 as Self::Base).pow(32);
        // encode lower 32 bits
        Uint32::encode(state, n as u32)?;
        // encode upper 32 bits
        Uint32::encode(state, r as u32)?;
        Ok(())
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let mut value: Self::Base = Uint32::decode(state)? as Self::Base;
        value += Uint32::decode(state)? as Self::Base * (2 as Self::Base).pow(32);
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_uint() {
        let mut state = State::new();

        Uint8::pre_encode(&mut state, 42);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );

        Uint16::pre_encode(&mut state, 4200);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 3,
                buffer: None,
            }
        );

        state.alloc();

        assert_eq!(Uint8::encode(&mut state, 42), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 3,
                buffer: Some(vec![42, 0, 0]),
            }
        );

        assert_eq!(Uint16::encode(&mut state, 4200), Ok(()));
        assert_eq!(
            state,
            State {
                start: 3,
                end: 3,
                buffer: Some(vec![42, 104, 16]),
            }
        );

        state.start = 0;
        assert_eq!(Uint8::decode(&mut state), Ok(42));
        assert_eq!(state.start, 1);
        assert_eq!(state.end, 3);

        assert_eq!(Uint16::decode(&mut state), Ok(4200));

        assert_eq!(state.start, state.end);

        assert_eq!(Uint8::decode(&mut state), Err(DecodeError::OutOfBounds));
    }
}
