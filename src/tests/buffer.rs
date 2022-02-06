// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

use crate::*;

//
// buffer
//

#[test]
fn test_buffer_pre_encode_empty() {
    let mut state = State::new();
    None.pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 1,
            buffer: None,
        }
    );
}

#[test]
fn test_buffer_pre_encode_short() {
    let mut state = State::new();
    let buffer = "content".as_bytes();

    Some(buffer).pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 8,
            buffer: None,
        }
    );
}

#[test]
fn test_buffer_pre_encode_long() {
    let mut state = State::new();

    const BUFFER_LONG_SIZE: usize = u8::MAX as usize + 1;
    let buffer_long: Vec<u8> = vec![3u8; BUFFER_LONG_SIZE];

    Some(buffer_long.as_slice()).pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 3 + BUFFER_LONG_SIZE, // buffer length is encoded as u16, since size is larger than u8::MAX
            buffer: None,
        }
    );
}

#[test]
fn test_buffer_encode_empty() {
    let mut state = State::new();

    None.pre_encode(&mut state);

    state.alloc();

    assert_eq!(None.encode(&mut state), Ok(()));
    assert_eq!(
        state,
        State {
            start: 1,
            end: 1,
            buffer: Some(vec![0]),
        }
    );
}

#[test]
fn test_buffer_encode_short() {
    let mut state = State::new();

    let buffer = "content";

    Some(buffer.as_bytes()).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Some(buffer.as_bytes()).encode(&mut state), Ok(()));

    let mut expected_buffer: Vec<u8> = vec![7; 8];
    expected_buffer[1..].copy_from_slice(buffer.as_bytes());
    assert_eq!(
        state,
        State {
            start: 8,
            end: 8,
            buffer: Some(expected_buffer),
        }
    );
}

#[test]
fn test_buffer_encode_long() {
    let mut state = State::new();

    const BUFFER_LONG_SIZE: usize = u8::MAX as usize + 1;
    let buffer: Vec<u8> = vec![3u8; BUFFER_LONG_SIZE];

    Some(buffer.as_slice()).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Some(buffer.as_slice()).encode(&mut state), Ok(()));
    let mut expected_buffer: Vec<u8> = vec![0; 3 + BUFFER_LONG_SIZE];
    // u16 encoded header size
    expected_buffer[0..3].copy_from_slice(&[0xFD, 0, 1]);
    // buffer content
    expected_buffer[3..].copy_from_slice(buffer.as_slice());
    assert_eq!(
        state,
        State {
            start: 3 + BUFFER_LONG_SIZE,
            end: 3 + BUFFER_LONG_SIZE,
            buffer: Some(expected_buffer),
        }
    );
}

#[test]
fn test_buffer_decode_empty() {
    let mut state = State::new();

    None.pre_encode(&mut state);

    state.alloc();

    assert_eq!(None.encode(&mut state), Ok(()));

    state.start = 0;
    assert_eq!(Option::<Box<Vec<u8>>>::decode(&mut state), Ok(None));
    assert_eq!(state.start, state.end);
}

#[test]
fn test_buffer_decode_short() {
    let mut state = State::new();

    let buffer = "content";

    Some(buffer.as_bytes()).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Some(buffer.as_bytes()).encode(&mut state), Ok(()));

    state.start = 0;
    assert_eq!(
        Option::<Box<Vec<u8>>>::decode(&mut state),
        Ok(Some(Box::new(buffer.into())))
    );
    assert_eq!(state.start, state.end);
}

#[test]
fn test_buffer_decode_long() {
    let mut state = State::new();

    const BUFFER_LONG_SIZE: usize = u8::MAX as usize + 1;
    let buffer = vec![3u8; BUFFER_LONG_SIZE];

    Some(buffer.as_slice()).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Some(buffer.as_slice()).encode(&mut state), Ok(()));

    state.start = 0;
    assert_eq!(
        Option::<Box<Vec<u8>>>::decode(&mut state),
        Ok(Some(Box::new(buffer.into())))
    );
    assert_eq!(state.start, state.end);
}

//
// raw
//

#[test]
fn test_raw_pre_encode_slice_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];
    Raw::Slice(buffer.as_slice()).pre_encode(&mut state);
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
fn test_raw_pre_encode_vec_ref_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];
    Raw::VecRef(&buffer).pre_encode(&mut state);
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
fn test_raw_pre_encode_vec_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];
    Raw::Vec(buffer).pre_encode(&mut state);
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
fn test_raw_pre_encode_slice_non_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = "content".into();
    Raw::Slice(buffer.as_slice()).pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 7,
            buffer: None,
        }
    );
}

#[test]
fn test_raw_pre_encode_vec_ref_non_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = "content".into();
    Raw::VecRef(&buffer).pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 7,
            buffer: None,
        }
    );
}

#[test]
fn test_raw_pre_encode_vec_non_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = "content".into();
    Raw::Vec(buffer).pre_encode(&mut state);
    assert_eq!(
        state,
        State {
            start: 0,
            end: 7,
            buffer: None,
        }
    );
}

#[test]
fn test_raw_encode_slice_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];

    Raw::Slice(&buffer).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::Slice(&buffer).encode(&mut state), Ok(()));
    assert_eq!(
        state,
        State {
            start: 0,
            end: 0,
            buffer: Some(vec![]),
        }
    );
}

#[test]
fn test_raw_encode_vec_ref_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];

    Raw::VecRef(&buffer).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::VecRef(&buffer).encode(&mut state), Ok(()));
    assert_eq!(
        state,
        State {
            start: 0,
            end: 0,
            buffer: Some(vec![]),
        }
    );
}

#[test]
fn test_raw_encode_vec_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];

    Raw::Vec(buffer.clone()).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::Vec(buffer).encode(&mut state), Ok(()));
    assert_eq!(
        state,
        State {
            start: 0,
            end: 0,
            buffer: Some(vec![]),
        }
    );
}

#[test]
fn test_raw_encode_vec_ref_non_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = "content".into();
    Raw::VecRef(&buffer).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::VecRef(&buffer).encode(&mut state), Ok(()));
    assert_eq!(
        state,
        State {
            start: 7,
            end: 7,
            buffer: Some(buffer),
        }
    );
}

#[test]
fn test_raw_decode_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = vec![];
    Raw::VecRef(&buffer).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::VecRef(&buffer).encode(&mut state), Ok(()));

    state.start = 0;

    assert_eq!(Raw::decode(&mut state), Ok(Raw::Vec(vec![])));
}

#[test]
fn test_raw_decode_non_empty() {
    let mut state = State::new();
    let buffer: Vec<u8> = "content".into();
    Raw::VecRef(&buffer).pre_encode(&mut state);

    state.alloc();

    assert_eq!(Raw::VecRef(&buffer).encode(&mut state), Ok(()));

    state.start = 0;

    assert_eq!(Raw::decode(&mut state), Ok(Raw::Vec(buffer)));
}
