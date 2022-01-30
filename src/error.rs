// SPDX-License-Identifier: MIT
//! error types for compact-encoding

/// All possible errors that can occur while encoding
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    /// calling encode() on a state on which alloc() was not called
    NoBuffer,
    /// trying to encode a type which is not supported, e.g. u128
    TypeNotSupported,
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBuffer => write!(f, "no buffer allocated in State struct"),
            Self::TypeNotSupported => write!(f, "the type is not supported by compact-encoding"),
        }
    }
}

/// shorthand type for encoding results
pub type EncodeResult = Result<(), EncodeError>;

/// All possible errors that can occur while decoding
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// rest of the buffer is too small to decode the expected type
    OutOfBounds,
    /// calling decode() on a state on which alloc() was not called
    NoBuffer,
    /// trying to decode a type which is not supported, e.g. u128
    TypeNotSupported,
    /// type does not match the expected type to decode
    TypeMismatch,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OutOfBounds => write!(f, "buffer is too small to decode the expected type"),
            Self::NoBuffer => write!(f, "no buffer allocated in State struct"),
            Self::TypeNotSupported => write!(f, "the type is not supported by compact-encoding"),
            Self::TypeMismatch => {
                write!(f, "the encoding does not match the type to be decoded into")
            }
        }
    }
}

/// shorthand type for decoding results
/// must be parameterized with the type of the returned value
pub type DecodeResultT<T> = Result<T, DecodeError>;
