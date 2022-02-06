// SPDX-License-Identifier: MIT
// compact-encoding-rs Authors: see AUTHORS.txt

use crate::*;

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
