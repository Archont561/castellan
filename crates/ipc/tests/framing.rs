//! Framing tests for the one native channel.
//!
//! Integration tests, on purpose: `encode_frame`/`decode_frame` are what the
//! host binary and the app's socket writer both consume, so the tests eat
//! the same food as every caller — the public API, nothing private.

use castellan_ipc::{MAX_MESSAGE_BYTES, decode_frame, encode_frame};
use proptest::prelude::*;

#[test]
fn frames_round_trip() {
    let payload = br#"{"kind":"request"}"#;
    let mut buf = Vec::new();
    encode_frame(payload, &mut buf);
    assert_eq!(&buf[..4], &(payload.len() as u32).to_le_bytes());

    let mut cursor = std::io::Cursor::new(buf);
    assert_eq!(decode_frame(&mut cursor).unwrap(), payload.to_vec());
}

#[test]
fn oversize_frames_are_refused() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(MAX_MESSAGE_BYTES + 1).to_le_bytes());
    buf.extend_from_slice(&[0u8; 8]);
    let mut cursor = std::io::Cursor::new(buf);
    let err = decode_frame(&mut cursor).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn empty_frame_is_a_message_not_eof() {
    let mut buf = Vec::new();
    encode_frame(b"", &mut buf);
    let mut cursor = std::io::Cursor::new(buf);
    assert!(decode_frame(&mut cursor).unwrap().is_empty());
}

#[test]
fn non_utf8_frames_are_refused() {
    let mut buf = Vec::new();
    encode_frame(&[0xff, 0xfe, 0x00, 0x41], &mut buf);
    let mut cursor = std::io::Cursor::new(buf);
    let err = decode_frame(&mut cursor).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

/// A payload the channel can actually carry: arbitrary text, as bytes.
/// (Not `any::<u8>` collections — the channel carries JSON and nothing
/// else, and `decode_frame` refuses non-UTF-8 by contract.)
fn text_payload() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<char>(), 0..1024).prop_map(|text| {
        let text: String = text.into_iter().collect();
        text.into_bytes()
    })
}

proptest! {
    /// The framing contract, for any payload the wire will ever carry:
    /// four little-endian length bytes, exact contents, byte for byte.
    #[test]
    fn arbitrary_payloads_round_trip(
        payload in text_payload(),
    ) {
        let mut buf = Vec::new();
        encode_frame(&payload, &mut buf);
        prop_assert_eq!(&buf[..4], &(payload.len() as u32).to_le_bytes());
        let mut cursor = std::io::Cursor::new(buf);
        prop_assert_eq!(decode_frame(&mut cursor).unwrap(), payload);
    }

    /// A stream is frames back to back, with no separator and no
    /// ambiguity — the property the host's pump loops depend on, and
    /// that a partial read in the middle would break.
    #[test]
    fn frames_stream_back_to_back(
        payloads in prop::collection::vec(text_payload(), 0..16),
    ) {
        let mut buf = Vec::new();
        for payload in &payloads {
            encode_frame(payload, &mut buf);
        }
        let mut cursor = std::io::Cursor::new(buf);
        for payload in payloads {
            prop_assert_eq!(decode_frame(&mut cursor).unwrap(), payload);
        }
        // Nothing left: the next read is EOF, not a phantom frame.
        prop_assert_eq!(
            decode_frame(&mut cursor).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof,
        );
    }
}
