//! Serialization through to_json and as_bytes, with round-trips.

mod common;

use common::{make_data, Format};
use waveform_data::{JsonWaveformData, WaveformData};

#[test]
fn to_json_two_channel_8bit() {
    let wf = make_data(Format::Binary, 2, 8);
    let json = wf.to_json();
    assert_eq!(json.version, Some(2));
    assert_eq!(json.channels, Some(2));
    assert_eq!(json.sample_rate, 48000);
    assert_eq!(json.samples_per_pixel, 512);
    assert_eq!(json.bits, 8);
    assert_eq!(json.length, 10);
    assert_eq!(
        json.data,
        vec![
            0, 0, 0, 0, -10, 10, -8, 8, 0, 0, -2, 2, -5, 7, -6, 3, -5, 7, -6, 3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -2, 2, -3, 3,
        ]
    );
}

#[test]
fn to_json_always_version_two() {
    // A version-1 single-channel input still reports version 2 in JSON.
    let wf = make_data(Format::Binary, 1, 8);
    let json = wf.to_json();
    assert_eq!(json.version, Some(2));
    assert_eq!(json.channels, Some(1));
    assert_eq!(json.length, 10);
    assert_eq!(
        json.data,
        vec![0, 0, -10, 10, 0, 0, -5, 7, -5, 7, 0, 0, 0, 0, 0, 0, 0, 0, -2, 2]
    );
}

#[test]
fn to_json_16bit() {
    let wf = make_data(Format::Binary, 2, 16);
    let json = wf.to_json();
    assert_eq!(json.bits, 16);
    assert_eq!(json.version, Some(2));
    assert_eq!(json.length, 10);
}

#[test]
fn as_bytes_byte_lengths() {
    // JSON 2-channel 8-bit: 24 header + 40 data.
    let json = WaveformData::from_json(&common::json_data(2, 8)).unwrap();
    assert_eq!(json.as_bytes().len(), 64);

    // JSON 2-channel 16-bit: 24 + 80.
    let json16 = WaveformData::from_json(&common::json_data(2, 16)).unwrap();
    assert_eq!(json16.as_bytes().len(), 104);

    // Binary 1-channel 8-bit keeps its 20-byte version-1 header: 20 + 20.
    let binary = make_data(Format::Binary, 1, 8);
    assert_eq!(binary.as_bytes().len(), 40);
}

#[test]
fn round_trip_json_2ch() {
    let original = WaveformData::from_json(&common::json_data(2, 8)).unwrap();
    let buffer = original.as_bytes().to_vec();
    let reparsed = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(reparsed.length(), 10);
    assert_eq!(reparsed.bits(), 8);
    assert_eq!(reparsed.sample_rate(), 48000);
    assert_eq!(reparsed.scale(), 512);
    assert_eq!(reparsed.channels(), 2);
    for c in 0..2 {
        assert_eq!(
            reparsed.channel(c).unwrap().min_array(),
            original.channel(c).unwrap().min_array()
        );
        assert_eq!(
            reparsed.channel(c).unwrap().max_array(),
            original.channel(c).unwrap().max_array()
        );
    }
}

#[test]
fn round_trip_binary_1ch() {
    let original = make_data(Format::Binary, 1, 8);
    let buffer = original.as_bytes().to_vec();
    let reparsed = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(reparsed.length(), 10);
    assert_eq!(reparsed.bits(), 8);
    assert_eq!(reparsed.sample_rate(), 48000);
    assert_eq!(reparsed.scale(), 512);
    assert_eq!(reparsed.channels(), 1);
    assert_eq!(
        reparsed.channel(0).unwrap().min_array(),
        original.channel(0).unwrap().min_array()
    );
    assert_eq!(
        reparsed.channel(0).unwrap().max_array(),
        original.channel(0).unwrap().max_array()
    );
}

#[test]
fn to_json_feeds_from_json() {
    // The JSON object produced is a valid input again.
    let wf = make_data(Format::Binary, 2, 16);
    let json: JsonWaveformData = wf.to_json();
    let rebuilt = WaveformData::from_json(&json).unwrap();
    assert_eq!(
        rebuilt.channel(1).unwrap().max_array(),
        wf.channel(1).unwrap().max_array()
    );
}
