//! Rejection of truncated and inflated binary buffers at construction.
//!
//! A binary buffer carries its data point count in the header `length` field.
//! `from_binary` checks that count against the bytes that follow the header, so
//! a buffer that claims more data than it holds is rejected up front instead of
//! panicking later inside a sample accessor.

use waveform_data::{Error, WaveformData};

/// Builds a version-2 header with an explicit `length` field and no data bytes.
fn header_only(length: i32, channels: i32, eight_bit: bool) -> Vec<u8> {
    let mut b = vec![0u8; 24];
    b[0..4].copy_from_slice(&2i32.to_le_bytes());
    b[4..8].copy_from_slice(&(if eight_bit { 1u32 } else { 0 }).to_le_bytes());
    b[8..12].copy_from_slice(&48000i32.to_le_bytes());
    b[12..16].copy_from_slice(&512i32.to_le_bytes());
    b[16..20].copy_from_slice(&length.to_le_bytes());
    b[20..24].copy_from_slice(&channels.to_le_bytes());
    b
}

#[test]
fn inflated_length_is_rejected() {
    // A header that claims 100 points but carries zero data bytes.
    let buffer = header_only(100, 1, true);
    assert_eq!(
        WaveformData::from_binary(buffer).unwrap_err(),
        Error::DataLengthMismatch
    );
}

#[test]
fn overflowing_data_length_is_rejected() {
    let buffer = header_only(-1, i32::MAX, false);
    assert_eq!(
        WaveformData::from_binary(buffer).unwrap_err(),
        Error::DataLengthMismatch
    );
}

#[test]
fn exact_data_section_is_accepted() {
    // length 2, 1 channel, 8-bit: 4 data bytes after the 24-byte header.
    let mut buffer = header_only(2, 1, true);
    buffer.extend_from_slice(&[0, 0, 0, 0]);
    let wf = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(wf.length(), 2);
    // The accessors do not panic on an accepted buffer.
    assert_eq!(wf.channel(0).unwrap().min_array(), vec![0, 0]);
    let _ = wf.to_json();
}

#[test]
fn short_data_section_is_rejected() {
    // length 2 needs 4 data bytes; supply 3.
    let mut buffer = header_only(2, 1, true);
    buffer.extend_from_slice(&[0, 0, 0]);
    assert_eq!(
        WaveformData::from_binary(buffer).unwrap_err(),
        Error::DataLengthMismatch
    );
}

#[test]
fn truncated_v2_header_is_unknown_format() {
    // Eight bytes hold the version field but not the full 24-byte v2 header.
    let mut buffer = vec![0u8; 8];
    buffer[0..4].copy_from_slice(&2i32.to_le_bytes());
    assert_eq!(
        WaveformData::from_binary(buffer).unwrap_err(),
        Error::UnknownDataFormat
    );
}

#[test]
fn truncated_v1_header_is_unknown_format() {
    // Eight bytes hold the version field but not the full 20-byte v1 header.
    let mut buffer = vec![0u8; 8];
    buffer[0..4].copy_from_slice(&1i32.to_le_bytes());
    assert_eq!(
        WaveformData::from_binary(buffer).unwrap_err(),
        Error::UnknownDataFormat
    );
}

#[test]
fn sixteen_bit_data_section_is_checked() {
    // length 1, 1 channel, 16-bit: 4 data bytes (2 samples x 2 bytes).
    let mut buffer = header_only(1, 1, false);
    buffer.extend_from_slice(&[0, 0, 0, 0]);
    assert!(WaveformData::from_binary(buffer).is_ok());

    // Same header with only 2 data bytes is rejected.
    let mut short = header_only(1, 1, false);
    short.extend_from_slice(&[0, 0]);
    assert_eq!(
        WaveformData::from_binary(short).unwrap_err(),
        Error::DataLengthMismatch
    );
}
