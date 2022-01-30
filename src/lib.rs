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
    pub fn next_u8(&self) -> DecodeResultT<u8> {
        if self.start >= self.end {
            Err(DecodeError::OutOfBounds)
        } else if let Some(ref buffer) = self.buffer {
            Ok(buffer[self.start])
        } else {
            Err(DecodeError::NoBuffer)
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

/// encoding and decoding of primitive values
/// provides methods which are used for en-/decoding with protocol headers
pub trait PrimitiveEnDecoder {
    type PrimitiveType;
    const PROTOCOL_PREFIX: u8;
    const PROTOCOL_HEADER_SIZE: usize;

    /// check if the protocol header matches
    /// override, if different behavioris required
    fn protocol_header_matches(value: u8) -> bool {
        value == Self::PROTOCOL_PREFIX
    }

    /// add protocol header to the buffer
    /// override, if different behavioris required
    fn add_protocol_header(state: &mut State) -> EncodeResult {
        let buffer = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| EncodeError::NoBuffer)?;
        buffer[state.start] = Self::PROTOCOL_PREFIX;
        state.start += 1;
        Ok(())
    }

    /// add size of primitive to `state`
    fn primitive_pre_encode(state: &mut State) {
        state.end += std::mem::size_of::<Self::PrimitiveType>();
    }

    /// implement the primitive encoding in this method
    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult;

    /// implement the primitive decoding in this method
    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType>;
}

const U8_PREFIX: u8 = 0xFC;
const U16_PREFIX: u8 = 0xFD;
const U32_PREFIX: u8 = 0xFE;
const U64_PREFIX: u8 = 0xFF;

/// encoder/decoder for u8
pub struct Uint8();

impl PrimitiveEnDecoder for Uint8 {
    type PrimitiveType = u8;
    const PROTOCOL_PREFIX: u8 = U8_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = 0;

    fn protocol_header_matches(value: u8) -> bool {
        // TODO: values between u8::MAX and Self::PROTOCOL_PREFIX must be encoded as u16
        value <= Self::PROTOCOL_PREFIX
    }

    fn add_protocol_header(_state: &mut State) -> EncodeResult {
        // override
        // uint8 does not add a protocol header
        Ok(())
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
        buffer[state.start] = n;
        state.start += 1;
        Ok(())
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let value: Self::PrimitiveType = buffer[state.start];
        state.start += 1;
        Ok(value)
    }
}

impl EnDecoder for Uint8 {
    type Base = u8;

    fn pre_encode(state: &mut State, _: Self::Base) {
        // prefix
        state.end += Self::PROTOCOL_HEADER_SIZE;
        Self::primitive_pre_encode(state);
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        Self::add_protocol_header(state)?;
        Self::primitive_encode(state, n)
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        state.next_u8().map(|u| {
            if Self::protocol_header_matches(u) {
                Self::primitive_decode(state)
            } else {
                Err(DecodeError::TypeMismatch)
            }
        })?
    }
}

/// encoder/decoder for u16
pub struct Uint16();

impl PrimitiveEnDecoder for Uint16 {
    type PrimitiveType = u16;
    const PROTOCOL_PREFIX: u8 = U16_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = 1;

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
        buffer[state.start] = n as u8;
        state.start += 1;
        buffer[state.start] = (n >> 8) as u8;
        state.start += 1;
        Ok(())
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let mut value: Self::PrimitiveType = buffer[state.start] as Self::PrimitiveType;
        state.start += 1;
        value += buffer[state.start] as Self::PrimitiveType * 256;
        state.start += 1;
        Ok(value)
    }
}

impl EnDecoder for Uint16 {
    type Base = u16;

    fn pre_encode(state: &mut State, _: Self::Base) {
        // prefix
        state.end += Self::PROTOCOL_HEADER_SIZE;
        Self::primitive_pre_encode(state);
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        Self::add_protocol_header(state)?;
        Self::primitive_encode(state, n)
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        state.next_u8().map(|u| {
            if Self::protocol_header_matches(u) {
                // ignore header
                state.start += 1;
                Self::primitive_decode(state)
            } else {
                Err(DecodeError::TypeMismatch)
            }
        })?
    }
}

/// encoder/decoder for 32
pub struct Uint32();

impl PrimitiveEnDecoder for Uint32 {
    type PrimitiveType = u32;
    const PROTOCOL_PREFIX: u8 = U32_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = 1;

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
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

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let mut value: Self::PrimitiveType = buffer[state.start] as Self::PrimitiveType;
        state.start += 1;
        value += buffer[state.start] as Self::PrimitiveType * 256;
        state.start += 1;
        value += buffer[state.start] as Self::PrimitiveType * ((256 as Self::PrimitiveType).pow(2));
        state.start += 1;
        value += buffer[state.start] as Self::PrimitiveType * ((256 as Self::PrimitiveType).pow(3));
        state.start += 1;
        Ok(value)
    }
}

impl EnDecoder for Uint32 {
    type Base = u32;

    fn pre_encode(state: &mut State, _: Self::Base) {
        // prefix
        state.end += Self::PROTOCOL_HEADER_SIZE;
        Self::primitive_pre_encode(state);
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        Self::add_protocol_header(state)?;
        Self::primitive_encode(state, n)
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        state.next_u8().map(|u| {
            if Self::protocol_header_matches(u) {
                // ignore header
                state.start += 1;
                Self::primitive_decode(state)
            } else {
                Err(DecodeError::TypeMismatch)
            }
        })?
    }
}

/// encoder/decoder for u64
pub struct Uint64();

impl PrimitiveEnDecoder for Uint64 {
    type PrimitiveType = u64;
    const PROTOCOL_PREFIX: u8 = U64_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = 1;

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let r = n / (2 as Self::PrimitiveType).pow(32);
        // encode lower 32 bits
        Uint32::primitive_encode(state, n as u32)?;
        // encode upper 32 bits
        Uint32::primitive_encode(state, r as u32)
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let mut value: Self::PrimitiveType =
            Uint32::primitive_decode(state)? as Self::PrimitiveType;
        value += Uint32::primitive_decode(state)? as Self::PrimitiveType
            * (2 as Self::PrimitiveType).pow(32);
        Ok(value)
    }
}

impl EnDecoder for Uint64 {
    type Base = u64;

    fn pre_encode(state: &mut State, _: Self::Base) {
        // prefix
        state.end += Self::PROTOCOL_HEADER_SIZE;
        Self::primitive_pre_encode(state);
    }

    fn encode(state: &mut State, n: Self::Base) -> EncodeResult {
        Self::add_protocol_header(state)?;
        Self::primitive_encode(state, n)
    }

    fn decode(state: &mut State) -> DecodeResultT<Self::Base> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        state.next_u8().map(|u| {
            if Self::protocol_header_matches(u) {
                // ignore header
                state.start += 1;
                Self::primitive_decode(state)
            } else {
                Err(DecodeError::TypeMismatch)
            }
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mdn: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
    const MAX_SAFE_INTEGER: u64 = (2 as u64).pow(53) - 1;

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
                end: 4,
                buffer: None,
            }
        );

        Uint64::pre_encode(&mut state, MAX_SAFE_INTEGER);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 13,
                buffer: None,
            }
        );

        state.alloc();

        assert_eq!(Uint8::encode(&mut state, 42), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 13,
                buffer: Some(vec![42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        assert_eq!(Uint16::encode(&mut state, 4200), Ok(()));
        assert_eq!(
            state,
            State {
                start: 4,
                end: 13,
                buffer: Some(vec![42, 0xFD, 104, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );
        assert_eq!(Uint64::encode(&mut state, MAX_SAFE_INTEGER), Ok(()));
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
        assert_eq!(Uint8::decode(&mut state), Ok(42));
        assert_eq!(state.start, 1);
        assert_eq!(state.end, 13);

        assert_eq!(Uint16::decode(&mut state), Ok(4200));
        assert_eq!(state.start, 4);
        assert_eq!(state.end, 13);

        assert_eq!(Uint64::decode(&mut state), Ok(MAX_SAFE_INTEGER));
        assert_eq!(state.start, 13);
        assert_eq!(state.end, 13);

        assert_eq!(state.start, state.end);

        assert_eq!(Uint8::decode(&mut state), Err(DecodeError::OutOfBounds));
    }
}
