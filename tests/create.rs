//! Format detection and construction.

mod common;

use common::{binary_data, json_data};
use waveform_data::{Error, WaveformData};

#[test]
fn rejects_short_binary_as_unknown_format() {
    // An empty or sub-header buffer has no version field.
    assert_eq!(
        WaveformData::from_binary(Vec::new()).unwrap_err(),
        Error::UnknownDataFormat
    );
    assert_eq!(
        WaveformData::from_binary(vec![0u8, 0, 0]).unwrap_err(),
        Error::UnknownDataFormat
    );
}

#[test]
fn creates_from_json_object() {
    let data = json_data(1, 8);
    let waveform = WaveformData::from_json(&data).unwrap();
    assert_eq!(waveform.length(), 10);
}

#[test]
fn creates_from_binary_buffer() {
    let data = binary_data(2, 8, None);
    let waveform = WaveformData::from_binary(data).unwrap();
    assert_eq!(waveform.channels(), 2);
}

#[test]
fn rejects_unknown_version() {
    let data = binary_data(1, 8, Some(3));
    assert_eq!(
        WaveformData::from_binary(data).unwrap_err(),
        Error::UnsupportedVersion
    );
}

#[test]
fn unsupported_version_message() {
    for version in [0, 3, 99] {
        let data = binary_data(1, 8, Some(version));
        let err = WaveformData::from_binary(data).unwrap_err();
        assert_eq!(
            err.to_string(),
            "WaveformData.create(): This waveform data version not supported"
        );
    }
}

#[test]
fn json_length_mismatch() {
    let mut data = json_data(1, 8);
    data.data.pop();
    assert_eq!(
        WaveformData::from_json(&data).unwrap_err(),
        Error::LengthMismatch
    );
}

#[test]
fn json_channels_defaults_to_one() {
    // Missing channels means a single channel of 20 interleaved values.
    let mut data = json_data(1, 8);
    data.channels = None;
    let waveform = WaveformData::from_json(&data).unwrap();
    assert_eq!(waveform.channels(), 1);

    data.channels = Some(0);
    let waveform = WaveformData::from_json(&data).unwrap();
    assert_eq!(waveform.channels(), 1);
}
