// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt
//! error types for compact-encoding

/// All possible errors that can occur while encoding
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    /// calling encode() on a state on which alloc() was not called
    NoBuffer,
    /// rest of the buffer is too small to decode the expected type
    BufferTooSmall,
    /// trying to encode a type which is not supported, e.g. u128
    TypeNotSupported,
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBuffer => write!(f, "no buffer allocated in State struct"),
            Self::BufferTooSmall => write!(f, "buffer is too small to decode the expected type"),
            Self::TypeNotSupported => write!(f, "the type is not supported by compact-encoding"),
        }
    }
}

/// shorthand type for encoding results
pub type EncodeResult = Result<(), EncodeError>;

/// All possible errors that can occur while decoding
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// calling decode() on a state on which alloc() was not called
    NoBuffer,
    /// buffer is too small to decode the expected type
    BufferTooSmall,
    /// trying to decode a type which is not supported, e.g. u128
    TypeNotSupported,
    /// type does not match the expected type to decode
    TypeMismatch,
    /// buffer did not contain valid UTF8 during decoding to &str
    InvalidUtf8,
    /// encoded array is too large for decoding
    ArrayTooLarge,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBuffer => write!(f, "no buffer allocated in State struct"),
            Self::BufferTooSmall => write!(f, "buffer is too small to decode the expected type"),
            Self::TypeNotSupported => write!(f, "the type is not supported by compact-encoding"),
            Self::TypeMismatch => {
                write!(f, "the encoding does not match the type to be decoded into")
            }
            Self::InvalidUtf8 => {
                write!(
                    f,
                    "the buffer did not contain a valid UTF8 string when decoding to &str"
                )
            }

            Self::ArrayTooLarge => {
                write!(
                    f,
                    "the encoded array is bigger than the maximum supported array size of {}",
                    crate::MAX_ARRAY_DECODE_SIZE
                )
            }
        }
    }
}

/// shorthand type for decoding results
/// must be parameterized with the type of the returned value
pub type DecodeResultT<T> = Result<T, DecodeError>;
