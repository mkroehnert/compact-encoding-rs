// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt
#![doc = include_str!("../README.md")]
#![doc(test(no_crate_inject))]
#![doc(html_no_source)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod error;

#[cfg(test)]
mod tests;

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
}

const U8_MAX_VALUE: u8 = 0xFC;
const U16_PREFIX: u8 = 0xFD;
const U32_PREFIX: u8 = 0xFE;
const U64_PREFIX: u8 = 0xFF;

/// encode value from signed i64 into u64
pub fn zig_zag_encode(value: i64) -> u64 {
    let result = match value {
        n if n < 0 => (2 * -n) - 1,
        n if n == 0 => 0,
        n => 2 * n,
    };
    result as u64
}

/// decode value from u64 to i64
pub fn zig_zag_decode(value: u64) -> i64 {
    match value {
        n if n == 0 => n as i64,
        n if (n & 1) == 0 => (n as i64) / 2,
        n => -((n as i64) + 1) / 2,
    }
}

/// Trait that indicates that a struct can be used as a destination to encode data too.
/// Used by [Encode].
pub trait Writer {
    /// Write `bytes` to the underlying writer.
    /// Exactly `bytes.len()` bytes must be written, or else an error should be returned.
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;
}

/// State implements the Writer trait for writing to a binary buffer
impl Writer for State {
    /// write `bytes` to self.buffer
    //#[inline(always)]
    fn write(&mut self, bytes: &[u8]) -> EncodeResult {
        if (self.end - self.start) < bytes.len() {
            return Err(EncodeError::BufferTooSmall);
        };

        if let Some(buffer) = &mut self.buffer {
            let view = &mut buffer[self.start..self.start + bytes.len()];
            view.copy_from_slice(bytes);
            self.start += bytes.len();
            Ok(())
        } else {
            Err(EncodeError::NoBuffer)
        }
    }
}

/// Trait that is used for reading from a buffer.
/// Used by [Decode]
pub trait Reader {
    /// Return a slice into the underlying buffer.
    /// If remaining buffer is smaller than `size` an error must be returned.
    fn read_next<'a>(&'a mut self, size: usize) -> DecodeResultT<&'a [u8]>;

    fn peek_u8(&self) -> DecodeResultT<u8>;
}

/// State implements Reader for extracting data from its binary buffer
impl Reader for State {
    fn read_next<'a>(&'a mut self, size: usize) -> DecodeResultT<&'a [u8]> {
        if self.start >= self.end {
            return Err(DecodeError::BufferTooSmall);
        };
        match &self.buffer {
            // TODO: may not be very performant
            Some(buffer) if (buffer.len() - self.start) < size => Err(DecodeError::BufferTooSmall),
            Some(buffer) => {
                let view = &buffer[self.start..self.start + size];
                self.start += size;
                Ok(view)
            }
            None => Err(DecodeError::NoBuffer),
        }
    }
    /// get reference to next u8 value if one exists
    fn peek_u8(&self) -> DecodeResultT<u8> {
        if self.start >= self.end {
            return Err(DecodeError::BufferTooSmall);
        };
        match &self.buffer {
            Some(buffer) => Ok(buffer[self.start]),
            None => Err(DecodeError::NoBuffer),
        }
    }
}

/// Trait which defines the required encoding functions
pub trait Encode {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State);

    /// encode n into state.buffer
    /// return an error if state.buffer is not allocated or the buffer is too small
    fn encode(&self, state: &mut State) -> EncodeResult;
}

/// Trait which defines the required decoding functions
pub trait Decode: Sized {
    /// return decode value at current buffer pointer
    /// return an error if buffer size does not match or if header information is wrong
    fn decode(state: &mut State) -> DecodeResultT<Self>;
}

//
// bool
//

/// compact encoding for bool
impl Encode for bool {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += std::mem::size_of::<u8>();
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        // encode true as 1u8 and false as 0u8
        state.write(&[if *self { 1u8 } else { 0u8 }])
    }
}

/// compact decoding for u8
impl Decode for bool {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let value = state.read_next(std::mem::size_of::<u8>())?[0];
        Ok(if value == 1 { true } else { false })
    }
}

//
// unsigned integers
//

/// compact encoding for u8
impl Encode for u8 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        if *self <= U8_MAX_VALUE {
            state.end += std::mem::size_of::<u8>();
        } else {
            (*self as u16).pre_encode(state);
        }
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        if *self <= U8_MAX_VALUE {
            state.write(&[*self])
        } else {
            (*self as u16).encode(state)
        }
    }
}

/// compact decoding for u8
impl Decode for u8 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let header = state.peek_u8()?;
        if header <= U8_MAX_VALUE {
            let value = state.read_next(std::mem::size_of::<u8>())?[0];
            Ok(value)
        } else {
            u16::decode(state).map(|v| v as u8)
        }
    }
}

/// compact encoding for u16
impl Encode for u16 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += 1 + (std::mem::size_of::<Self>());
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&[U16_PREFIX, *self as u8, (*self >> 8) as u8])
    }
}

/// compact decoding for u16
impl Decode for u16 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.read_next(1 + std::mem::size_of::<Self>())?;
        if buffer[0] == U16_PREFIX {
            let mut value: Self = buffer[1] as u16;
            value += buffer[2] as u16 * 256;
            Ok(value)
        } else {
            Err(DecodeError::TypeMismatch)
        }
    }
}

/// compact encoding for u32
impl Encode for u32 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += 1 + (std::mem::size_of::<Self>());
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&[U32_PREFIX])?;
        state.write(&encode_u32(*self))
    }
}

/// compact encoding for u32 number to byte array
#[inline(always)]
fn encode_u32(value: u32) -> [u8; 4] {
    [
        value as u8,
        (value >> 8) as u8,
        (value >> (8 * 2)) as u8,
        (value >> (8 * 3)) as u8,
    ]
}

/// compact decoding for u32
impl Decode for u32 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.read_next(1 + std::mem::size_of::<Self>())?;
        if buffer[0] == U32_PREFIX {
            let value = decode_u32(&buffer[1..5])?;
            Ok(value)
        } else {
            Err(DecodeError::TypeMismatch)
        }
    }
}

/// compact encoding for u32 number to byte array
#[inline(always)]
fn decode_u32(buffer: &[u8]) -> DecodeResultT<u32> {
    if buffer.len() < 4 {
        return Err(DecodeError::BufferTooSmall);
    }
    let mut value: u32 = buffer[0] as u32;
    value += buffer[1] as u32 * 256;
    value += buffer[2] as u32 * (256 as u32).pow(2);
    value += buffer[3] as u32 * (256 as u32).pow(3);
    Ok(value)
}

/// compact encoding for u64
impl Encode for u64 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += 1 + (std::mem::size_of::<Self>());
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&[U64_PREFIX])?;
        let r = self / (2 as Self).pow(32);
        // encode lower 32 bits
        state.write(&encode_u32((*self) as u32))?;
        // encode upper 32 bits
        state.write(&encode_u32(r as u32))
    }
}

/// compact decoding for u64
impl Decode for u64 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.read_next(1 + std::mem::size_of::<Self>())?;
        if buffer[0] == U64_PREFIX {
            let mut value = decode_u32(&buffer[1..6])? as u64;
            value += decode_u32(&buffer[5..])? as u64 * (2 as u64).pow(32);
            Ok(value)
        } else {
            Err(DecodeError::TypeMismatch)
        }
    }
}

// compact encoding for usize
impl Encode for usize {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        match *self as u128 {
            x if x <= (U8_MAX_VALUE as u128) => (x as u8).pre_encode(state),
            x if x <= (u16::MAX as u128) => (x as u16).pre_encode(state),
            x if x <= (u32::MAX as u128) => (x as u32).pre_encode(state),
            x if x <= (u64::MAX as u128) => (x as u64).pre_encode(state),
            _ => unimplemented!(),
        };
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        match *self as u128 {
            x if x <= (U8_MAX_VALUE as u128) => (x as u8).encode(state),
            x if x <= (u16::MAX as u128) => (x as u16).encode(state),
            x if x <= (u32::MAX as u128) => (x as u32).encode(state),
            x if x <= (u64::MAX as u128) => (x as u64).encode(state),
            _ => unimplemented!(),
        }
    }
}

/// compact decoding for usize
impl Decode for usize {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.peek_u8()?;
        match buffer as u16 {
            x if x <= U8_MAX_VALUE as u16 => u8::decode(state).map(|value| value as usize),
            x if x <= U16_PREFIX as u16 => u16::decode(state).map(|value| value as usize),
            x if x <= U32_PREFIX as u16 => u32::decode(state).map(|value| value as usize),
            x if x <= U64_PREFIX as u16 => u64::decode(state).map(|value| value as usize),
            _ => unimplemented!(),
        }
    }
}

//
// signed integers
//

/// compact encoding for i8
impl Encode for i8 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        (zig_zag_encode(*self as i64) as u8).pre_encode(state);
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        (zig_zag_encode(*self as i64) as u8).encode(state)
    }
}

/// compact decoding for i8
impl Decode for i8 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        Ok(zig_zag_decode(u8::decode(state)? as u64) as Self)
    }
}

/// compact encoding for i16
impl Encode for i16 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        (zig_zag_encode(*self as i64) as u16).pre_encode(state);
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        (zig_zag_encode(*self as i64) as u16).encode(state)
    }
}

/// compact decoding for i16
impl Decode for i16 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        Ok(zig_zag_decode(u16::decode(state)? as u64) as Self)
    }
}

/// compact encoding for i32
impl Encode for i32 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        (zig_zag_encode(*self as i64) as u32).pre_encode(state);
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        (zig_zag_encode(*self as i64) as u32).encode(state)
    }
}

/// compact decoding for i32
impl Decode for i32 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        Ok(zig_zag_decode(u32::decode(state)? as u64) as Self)
    }
}

/// compact encoding for i64
impl Encode for i64 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        (zig_zag_encode(*self) as u64).pre_encode(state);
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        (zig_zag_encode(*self) as u64).encode(state)
    }
}

/// compact decoding for i64
impl Decode for i64 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        Ok(zig_zag_decode(u32::decode(state)? as u64) as Self)
    }
}

//
// float
//

/// compact encoding for f32
impl Encode for f32 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += std::mem::size_of::<Self>();
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&self.to_le_bytes())
    }
}

/// compact decoding for f32
impl Decode for f32 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.read_next(std::mem::size_of::<Self>())?;
        let value: Self =
            Self::from_le_bytes(buffer.try_into().map_err(|_| DecodeError::BufferTooSmall)?);
        Ok(value)
    }
}

/// compact encoding for f64
impl Encode for f64 {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += std::mem::size_of::<Self>();
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&self.to_le_bytes())
    }
}

/// compact decoding for f64
impl Decode for f64 {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer = state.read_next(std::mem::size_of::<Self>())?;
        let value: Self =
            Self::from_le_bytes(buffer.try_into().map_err(|_| DecodeError::BufferTooSmall)?);
        Ok(value)
    }
}

//
// buffers, arrays
//

/// compact encoding for Option<&[u8]>
impl Encode for Option<&[u8]> {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        match self {
            Some(buffer) => {
                buffer.len().pre_encode(state);
                state.end += buffer.len();
            }
            None => state.end += 1,
        };
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        match self {
            Some(buffer) => {
                buffer.len().encode(state)?;
                state.write(buffer)
            }
            None => state.write(&[0]),
        }
    }
}

/// compact decoding for Option<&[u8]>
impl Decode for Option<Box<Vec<u8>>> {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_size = usize::decode(state)?;
        if buffer_size == 0 {
            return Ok(None);
        };
        let buffer_ref = state.read_next(buffer_size)?;

        println!(
            "buffer: {} - {} {:?}",
            buffer_size,
            &buffer_ref.len(),
            &buffer_ref
        );
        if buffer_ref.len() == buffer_size {
            Ok(Some(Box::new(Vec::from(buffer_ref))))
        } else {
            Err(DecodeError::TypeMismatch)
        }
    }
}
