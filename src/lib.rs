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

    pub fn dealloc(&mut self) {
        self.start = 0;
        self.end = 0;
        // drop current buffer
        let _ = self.buffer.take();
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
pub trait EnDecoder: PrimitiveEnDecoder {
    //type DecodeResult = DecodeResultT<Self::PrimitiveType>;

    /// allocate the required size in State for current type
    fn pre_encode(state: &mut State, n: Self::PrimitiveType) {
        // size of protocol header
        state.end += Self::PROTOCOL_HEADER_SIZE;
        // data size
        Self::primitive_pre_encode(state, n);
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        Self::add_protocol_header(state)?;
        Self::primitive_encode(state, n)
    }

    /// decode value at current buffer pointer into Self::Base
    fn decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        state.next_u8().map(|u| {
            if Self::protocol_header_matches(u) {
                state.start += Self::PROTOCOL_HEADER_SIZE;
                Self::primitive_decode(state)
            } else {
                Err(DecodeError::TypeMismatch)
            }
        })?
    }
}

/// encoding and decoding of primitive values
/// provides methods which are used for en-/decoding with protocol headers
pub trait PrimitiveEnDecoder {
    /// the underlying primitive type which is en-/decoded
    type PrimitiveType;
    /// the protocol prefix to attach to the stream
    const PROTOCOL_PREFIX: u8;
    /// the size of the protocol header to attached (required for pre_encode() for correct buffer size)
    const PROTOCOL_HEADER_SIZE: usize;
    /// size of the type to en-/decode
    const SIZE: usize = std::mem::size_of::<Self::PrimitiveType>();

    /// check if the protocol header matches
    /// override, if different behavioris required
    fn protocol_header_matches(value: u8) -> bool {
        if Self::PROTOCOL_HEADER_SIZE == 0 {
            true
        } else {
            value == Self::PROTOCOL_PREFIX
        }
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
    fn primitive_pre_encode(state: &mut State, _n: Self::PrimitiveType) {
        state.end += Self::SIZE;
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

impl EnDecoder for Uint8 {}

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

impl EnDecoder for Uint16 {}

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

impl EnDecoder for Uint32 {}

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

impl EnDecoder for Uint64 {}

pub fn zig_zag_encode(value: i64) -> u64 {
    let result = match value {
        n if n < 0 => (2 * -n) - 1,
        n if n == 0 => 0,
        n => 2 * n,
    };
    result as u64
}

pub fn zig_zag_decode(value: u64) -> i64 {
    match value {
        n if n == 0 => n as i64,
        n if (n & 1) == 0 => (n as i64) / 2,
        n => -((n as i64) + 1) / 2,
    }
}

/// encoder/decoder for i8
pub struct Int8();

impl PrimitiveEnDecoder for Int8 {
    type PrimitiveType = i8;
    const PROTOCOL_PREFIX: u8 = Uint8::PROTOCOL_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = Uint8::PROTOCOL_HEADER_SIZE;

    fn protocol_header_matches(value: u8) -> bool {
        Uint8::protocol_header_matches(value)
    }

    fn add_protocol_header(state: &mut State) -> EncodeResult {
        Uint8::add_protocol_header(state)
    }

    fn primitive_pre_encode(state: &mut State, n: Self::PrimitiveType) {
        Uint8::primitive_pre_encode(
            state,
            zig_zag_encode(n as i64) as <Uint8 as PrimitiveEnDecoder>::PrimitiveType,
        );
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        Uint8::primitive_encode(
            state,
            zig_zag_encode(n as i64) as <Uint8 as PrimitiveEnDecoder>::PrimitiveType,
        )
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        Ok(zig_zag_decode(Uint8::primitive_decode(state)? as u64) as Self::PrimitiveType)
    }
}

impl EnDecoder for Int8 {}

/// encoder/decoder for i16
pub struct Int16();

impl PrimitiveEnDecoder for Int16 {
    type PrimitiveType = i16;
    const PROTOCOL_PREFIX: u8 = Uint16::PROTOCOL_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = Uint16::PROTOCOL_HEADER_SIZE;

    fn protocol_header_matches(value: u8) -> bool {
        Uint16::protocol_header_matches(value)
    }

    fn add_protocol_header(state: &mut State) -> EncodeResult {
        Uint16::add_protocol_header(state)
    }

    fn primitive_pre_encode(state: &mut State, n: Self::PrimitiveType) {
        Uint16::primitive_pre_encode(
            state,
            zig_zag_encode(n as i64) as <Uint16 as PrimitiveEnDecoder>::PrimitiveType,
        );
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        Uint16::primitive_encode(
            state,
            zig_zag_encode(n as i64) as <Uint16 as PrimitiveEnDecoder>::PrimitiveType,
        )
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        Ok(zig_zag_decode(Uint16::primitive_decode(state)? as u64) as Self::PrimitiveType)
    }
}

impl EnDecoder for Int16 {}

/// encoder/decoder for i32
pub struct Int32();

impl PrimitiveEnDecoder for Int32 {
    type PrimitiveType = i32;
    const PROTOCOL_PREFIX: u8 = Uint32::PROTOCOL_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = Uint32::PROTOCOL_HEADER_SIZE;

    fn protocol_header_matches(value: u8) -> bool {
        Uint32::protocol_header_matches(value)
    }

    fn add_protocol_header(state: &mut State) -> EncodeResult {
        Uint32::add_protocol_header(state)
    }

    fn primitive_pre_encode(state: &mut State, n: Self::PrimitiveType) {
        Uint32::primitive_pre_encode(
            state,
            zig_zag_encode(n as i64) as <Uint32 as PrimitiveEnDecoder>::PrimitiveType,
        );
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        Uint32::primitive_encode(
            state,
            zig_zag_encode(n as i64) as <Uint32 as PrimitiveEnDecoder>::PrimitiveType,
        )
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        Ok(zig_zag_decode(Uint32::primitive_decode(state)? as u64) as Self::PrimitiveType)
    }
}

impl EnDecoder for Int32 {}

/// encoder/decoder for i64
pub struct Int64();

impl PrimitiveEnDecoder for Int64 {
    type PrimitiveType = i64;
    const PROTOCOL_PREFIX: u8 = Uint64::PROTOCOL_PREFIX;
    const PROTOCOL_HEADER_SIZE: usize = Uint64::PROTOCOL_HEADER_SIZE;

    fn protocol_header_matches(value: u8) -> bool {
        Uint64::protocol_header_matches(value)
    }

    fn add_protocol_header(state: &mut State) -> EncodeResult {
        Uint64::add_protocol_header(state)
    }

    fn primitive_pre_encode(state: &mut State, n: Self::PrimitiveType) {
        Uint64::primitive_pre_encode(
            state,
            zig_zag_encode(n as i64) as <Uint64 as PrimitiveEnDecoder>::PrimitiveType,
        );
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        Uint64::primitive_encode(
            state,
            zig_zag_encode(n as i64) as <Uint64 as PrimitiveEnDecoder>::PrimitiveType,
        )
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        Ok(zig_zag_decode(Uint64::primitive_decode(state)? as u64) as Self::PrimitiveType)
    }
}

impl EnDecoder for Int64 {}

/// encoder/decoder for f32
pub struct Float32();

impl PrimitiveEnDecoder for Float32 {
    type PrimitiveType = f32;
    const PROTOCOL_PREFIX: u8 = 0;
    const PROTOCOL_HEADER_SIZE: usize = 0;

    fn add_protocol_header(_state: &mut State) -> EncodeResult {
        // do not add header
        Ok(())
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
        let view = &mut buffer[state.start..state.start + Self::SIZE];
        view.copy_from_slice(&n.to_le_bytes());
        state.start += Self::SIZE;
        Ok(())
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let view = &buffer[state.start..state.start + Self::SIZE];
        let value: Self::PrimitiveType = Self::PrimitiveType::from_le_bytes(
            view.try_into().map_err(|_| DecodeError::OutOfBounds)?,
        );
        state.start += Self::SIZE;
        Ok(value)
    }
}

impl EnDecoder for Float32 {}

/// encoder/decoder for f64
pub struct Float64();

impl PrimitiveEnDecoder for Float64 {
    type PrimitiveType = f64;
    const PROTOCOL_PREFIX: u8 = 0;
    const PROTOCOL_HEADER_SIZE: usize = 0;

    fn add_protocol_header(_state: &mut State) -> EncodeResult {
        // do not add header
        Ok(())
    }

    fn primitive_encode(state: &mut State, n: Self::PrimitiveType) -> EncodeResult {
        let buffer: &mut [u8] = &mut state
            .buffer
            .as_deref_mut()
            .ok_or_else(|| -> EncodeError { EncodeError::NoBuffer })?;
        let view = &mut buffer[state.start..state.start + Self::SIZE];
        view.copy_from_slice(&n.to_le_bytes());
        state.start += Self::SIZE;
        Ok(())
    }

    fn primitive_decode(state: &mut State) -> DecodeResultT<Self::PrimitiveType> {
        if (state.end - state.start) < Self::SIZE {
            return Err(DecodeError::OutOfBounds);
        }
        let buffer = &state.buffer.as_ref().ok_or_else(|| DecodeError::NoBuffer)?;
        let view = &buffer[state.start..state.start + Self::SIZE];
        let value: Self::PrimitiveType = Self::PrimitiveType::from_le_bytes(
            view.try_into().map_err(|_| DecodeError::OutOfBounds)?,
        );
        state.start += Self::SIZE;
        Ok(value)
    }
}

impl EnDecoder for Float64 {}

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

    #[test]
    fn test_int() {
        let mut state = State::new();

        Int8::pre_encode(&mut state, 42);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 1,
                buffer: None,
            }
        );

        Int16::pre_encode(&mut state, -4200);
        assert_eq!(
            state,
            State {
                start: 0,
                end: 4,
                buffer: None,
            }
        );

        state.alloc();

        assert_eq!(Int8::encode(&mut state, 42), Ok(()));
        assert_eq!(
            state,
            State {
                start: 1,
                end: 4,
                buffer: Some(vec![84, 0, 0, 0]),
            }
        );

        assert_eq!(Int16::encode(&mut state, -4200), Ok(()));
        assert_eq!(
            state,
            State {
                start: 4,
                end: 4,
                buffer: Some(vec![84, 0xFD, 207, 32]),
            }
        );

        state.start = 0;
        assert_eq!(Int8::decode(&mut state), Ok(42));
        assert_eq!(state.start, 1);
        assert_eq!(state.end, 4);

        assert_eq!(Int16::decode(&mut state), Ok(-4200));
        assert_eq!(state.start, 4);
        assert_eq!(state.end, 4);

        assert_eq!(state.start, state.end);

        assert_eq!(Int8::decode(&mut state), Err(DecodeError::OutOfBounds));
    }

    #[test]
    fn test_float32() {
        let mut state = State::new();
        const NUM: f32 = 162.2377294;

        Float32::pre_encode(&mut state, NUM);
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

        assert_eq!(Float32::encode(&mut state, NUM), Ok(()));
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
        assert_eq!(Float32::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(Float32::decode(&mut state), Err(DecodeError::OutOfBounds));
    }

    #[test]
    fn test_float64() {
        let mut state = State::new();
        const NUM: f64 = 162.2377294;

        Float64::pre_encode(&mut state, NUM);
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

        assert_eq!(Float64::encode(&mut state, NUM), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
            }
        );

        state.start = 0;
        assert_eq!(Float64::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(Float64::decode(&mut state), Err(DecodeError::OutOfBounds));

        // alignment
        state.dealloc();

        Uint8::pre_encode(&mut state, 0);
        Float64::pre_encode(&mut state, NUM);
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

        assert_eq!(Uint8::encode(&mut state, 0), Ok(()));
        assert_eq!(Float64::encode(&mut state, NUM), Ok(()));
        assert_eq!(
            state,
            State {
                start: 9,
                end: 9,
                buffer: Some(vec![0, 0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
            }
        );

        state.start = 0;
        assert_eq!(Uint8::decode(&mut state), Ok(0));
        assert_eq!(Float64::decode(&mut state), Ok(NUM));
        assert_eq!(state.start, state.end);

        assert_eq!(Float64::decode(&mut state), Err(DecodeError::OutOfBounds));

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

        // assert_eq!(Uint8::encode(&mut state, 0), Ok(()));
        // assert_eq!(Float64::encode(&mut state, NUM), Ok(()));
        // assert_eq!(
        //     state,
        //     State {
        //         start: 9,
        //         end: 9,
        //         buffer: Some(vec![0, 0x87, 0xC9, 0xAF, 0x7A, 0x9B, 0x47, 0x64, 0x40]),
        //     }
        // );

        // state.start = 0;
        // assert_eq!(Uint8::decode(&mut state), Ok(0));
        // assert_eq!(Float64::decode(&mut state), Ok(NUM));
        // assert_eq!(state.start, state.end);

        // 0
        state.dealloc();
        Float64::pre_encode(&mut state, NUM);

        state.alloc();
        assert_eq!(Float64::encode(&mut state, 0.), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0, 0]),
            }
        );

        state.start = 0;
        assert_eq!(Float64::decode(&mut state), Ok(0.));
        assert_eq!(state.start, state.end);

        // infinity
        state.dealloc();
        Float64::pre_encode(&mut state, f64::INFINITY);

        state.alloc();
        assert_eq!(Float64::encode(&mut state, f64::INFINITY), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0, 0, 0, 0, 0, 0, 0xF0, 0x7F]),
            }
        );

        state.start = 0;
        assert_eq!(Float64::decode(&mut state), Ok(f64::INFINITY));
        assert_eq!(state.start, state.end);

        // edge cases
        state.dealloc();
        Float64::pre_encode(&mut state, 0.1 + 0.2);

        state.alloc();
        assert_eq!(Float64::encode(&mut state, 0.1 + 0.2), Ok(()));
        assert_eq!(
            state,
            State {
                start: 8,
                end: 8,
                buffer: Some(vec![0x34, 0x33, 0x33, 0x33, 0x33, 0x33, 0xD3, 0x3F]),
            }
        );

        state.start = 0;
        assert_eq!(Float64::decode(&mut state), Ok(0.1 + 0.2));
        assert_eq!(state.start, state.end);
    }
}
