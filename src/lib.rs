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
const MAX_ARRAY_DECODE_SIZE: usize = 1048576;

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

        if buffer_ref.len() == buffer_size {
            Ok(Some(Box::new(Vec::from(buffer_ref))))
        } else {
            Err(DecodeError::TypeMismatch)
        }
    }
}

/// wrapper struct for encoding plain buffers without length information
#[derive(Debug, PartialEq)]
pub enum Raw<'a> {
    /// enum variant which is returned by Decode
    Vec(Vec<u8>), // TODO: Replace Vec with Box<Vec>?
    VecRef(&'a Vec<u8>),
    Slice(&'a [u8]),
}

/// compact encoding for raw buffer
impl<'a> Encode for Raw<'a> {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        match *self {
            Raw::Vec(ref buffer) => state.end += buffer.len(),
            Raw::VecRef(buffer) => state.end += buffer.len(),
            Raw::Slice(slice) => state.end += slice.len(),
        }
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        match *self {
            Raw::Vec(ref buffer) => state.write(&buffer),
            Raw::VecRef(buffer) => state.write(&buffer),
            Raw::Slice(slice) => state.write(slice),
        }
    }
}

/// compact decoding for Option<&[u8]>
impl<'a> Decode for Raw<'a> {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_size = state.end - state.start;
        if buffer_size == 0 {
            Ok(Raw::Vec(vec![]))
        } else {
            let buffer_ref = state.read_next(buffer_size)?;

            Ok(Raw::Vec(buffer_ref.into()))
        }
    }
}

/// compact encoding for &str
/// TODO: implement for Into<&str> instead?
impl Encode for &str {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        // len always returns number of bytes
        self.len().pre_encode(state);
        state.end += self.len();
    }

    /// encode self into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        self.len().encode(state)?;
        state.write(self.as_bytes())
    }
}

/// compact decoding into String
impl Decode for String {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_size = usize::decode(state)?;
        if buffer_size == 0 {
            return Ok("".into());
        } else if (state.start + buffer_size) > state.end {
            return Err(DecodeError::BufferTooSmall);
        }
        let buffer_ref = state.read_next(buffer_size)?;
        /*
            const s = b.toString(state.buffer, 'utf8', state.start, state.start += len)
            if (b.byteLength(s) !== len || state.start > state.end) throw new Error('Out of bounds')
        */
        if buffer_ref.len() != buffer_size {
            Err(DecodeError::BufferTooSmall)
        } else {
            Ok(String::from_utf8(buffer_ref.into()).map_err(|_| DecodeError::InvalidUtf8)?)
        }
    }
}

/// compact encoding for arrays [T; N]
impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        N.pre_encode(state);
        // TODO check for MAX_ARRAY_DECODE_SIZE -> not implemented in JS
        for element in self.iter() {
            element.pre_encode(state);
        }
    }

    /// encode self into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        N.encode(state)?;
        // TODO check for MAX_ARRAY_DECODE_SIZE -> not implemented in JS
        for element in self.iter() {
            element.encode(state)?;
        }
        Ok(())
    }
}

/// compact encoding for Vec<T>
impl<T> Encode for Vec<T>
where
    T: Encode,
{
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        self.len().pre_encode(state);
        // TODO check for MAX_ARRAY_DECODE_SIZE
        for element in self.iter() {
            element.pre_encode(state);
        }
    }

    /// encode self into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        self.len().encode(state)?;
        // TODO check for MAX_ARRAY_DECODE_SIZE
        for element in self.iter() {
            element.encode(state)?;
        }
        Ok(())
    }
}

/// compact decoding into Vec<T>
impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_size = usize::decode(state)?;
        if buffer_size == 0 {
            return Ok(vec![]);
        } else if (state.start + buffer_size) > state.end {
            return Err(DecodeError::BufferTooSmall);
        } else if buffer_size > MAX_ARRAY_DECODE_SIZE {
            return Err(DecodeError::ArrayTooLarge);
        }
        let mut vec: Vec<T> = Vec::with_capacity(buffer_size);
        for _ in 0..buffer_size {
            vec.push(T::decode(state)?);
        }
        Ok(vec)
    }
}

#[derive(Debug, PartialEq)]
pub enum U32Array<'a> {
    Vec(Vec<u32>),
    VecRef(&'a Vec<u32>),
    Slice(&'a [u32]),
}

/// compact encoding for U32Array
/// MDN: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint32Array
impl Encode for U32Array<'_> {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        let vec = match self {
            U32Array::Vec(vec) => vec.as_slice(),
            U32Array::VecRef(vec) => vec.as_slice(),
            U32Array::Slice(slice) => slice,
        };
        vec.len().pre_encode(state);
        // u32 has 4 bytes length
        state.end += vec.len() * 4;
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        let vec = match self {
            U32Array::Vec(vec) => vec.as_slice(),
            U32Array::VecRef(vec) => vec.as_slice(),
            U32Array::Slice(slice) => slice,
        };
        vec.len().encode(state)?;
        for num in vec {
            state.write(&num.to_le_bytes())?;
        }
        Ok(())
    }
}

/// compact decoding for U32Array
/// returns U32Array::Vec(_)
impl Decode for U32Array<'_> {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_size = usize::decode(state)?;
        if buffer_size == 0 {
            return Ok(U32Array::Vec(vec![]));
        };
        /* JS Implementation contains this part as well
         * TODO: clarify functionality with original author
            // const byteOffset = state.buffer.byteOffset + state.start
            // const s = state.start

            // state.start += len * 4

            // if ((byteOffset & 3) === 0) {
            //   const arr = new Uint32Array(state.buffer.buffer, byteOffset, len)
            //   if (BE) LEToHost32(arr, len)
            //   return arr
            // }
        */
        // align mismatch
        let mut vec: Vec<u32> = Vec::with_capacity(buffer_size);
        // read all u32 values and decode them from little endian
        // difference to JS implementation: decode each value instead of reading buffer and then decoding buffer
        for _ in 1..(buffer_size + 1) {
            let buffer_ref = state.read_next(4)?;
            vec.push(u32::from_le_bytes(
                buffer_ref
                    .try_into()
                    .map_err(|_| DecodeError::TypeMismatch)?,
            ));
        }
        Ok(U32Array::Vec(vec))
    }
}

/// compact encoding for fixed size buffers
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Fixed<const N: usize>([u8; N]);

pub type Fixed32 = Fixed<32>;
pub type Fixed64 = Fixed<64>;

/// compact encoding for Fixed<N>
impl<const N: usize> Encode for Fixed<N> {
    /// allocate the required size in State for current type
    fn pre_encode(&self, state: &mut State) {
        state.end += N;
    }

    /// encode n into state.buffer
    /// requires state.buffer to be allocated first
    fn encode(&self, state: &mut State) -> EncodeResult {
        state.write(&self.0[..])
    }
}

/// compact decoding for Fixed<N>
impl<const N: usize> Decode for Fixed<N> {
    fn decode(state: &mut State) -> DecodeResultT<Self> {
        let buffer_ref = state.read_next(N)?;
        let mut fixed = Self([0; N]);
        fixed.0.copy_from_slice(buffer_ref);
        Ok(fixed)
    }
}
