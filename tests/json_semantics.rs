//! Two format quirks of JSON conversion: value wrap and the bits flag.
//!
//! Out-of-range `data` values wrap two's-complement on conversion, matching the
//! integer write semantics of the format. The `bits` field is a flag, not a
//! width: only an exact 8 selects 8-bit storage, and every other value falls to
//! 16-bit.

use waveform_data::{JsonWaveformData, WaveformData};

#[test]
fn eight_bit_values_wrap_on_conversion() {
    // One point, one channel: min 200, max -200.
    let json = JsonWaveformData {
        version: None,
        channels: None,
        sample_rate: 48000,
        samples_per_pixel: 512,
        bits: 8,
        length: 1,
        data: vec![200, -200],
    };
    let wf = WaveformData::from_json(&json).unwrap();
    let ch = wf.channel(0).unwrap();
    assert_eq!(ch.min_sample(0).unwrap(), -56);
    assert_eq!(ch.max_sample(0).unwrap(), 56);
}

#[test]
fn sixteen_bit_value_wraps_on_conversion() {
    let json = JsonWaveformData {
        version: None,
        channels: None,
        sample_rate: 48000,
        samples_per_pixel: 512,
        bits: 16,
        length: 1,
        data: vec![40000, 0],
    };
    let wf = WaveformData::from_json(&json).unwrap();
    assert_eq!(wf.channel(0).unwrap().min_sample(0).unwrap(), -25536);
}

#[test]
fn non_eight_bits_is_treated_as_sixteen() {
    // bits 24 is not 8, so it takes the 16-bit path: 2 bytes per sample.
    let json = JsonWaveformData {
        version: None,
        channels: None,
        sample_rate: 48000,
        samples_per_pixel: 512,
        bits: 24,
        length: 1,
        data: vec![-5, 7],
    };
    let wf = WaveformData::from_json(&json).unwrap();
    assert_eq!(wf.bits(), 16);
    assert_eq!(wf.channel(0).unwrap().min_sample(0).unwrap(), -5);
    assert_eq!(wf.channel(0).unwrap().max_sample(0).unwrap(), 7);
    // 24-byte header plus 2 samples at 2 bytes each.
    assert_eq!(wf.as_bytes().len(), 28);
}

#[test]
fn zero_bits_is_treated_as_sixteen() {
    let json = JsonWaveformData {
        version: None,
        channels: None,
        sample_rate: 48000,
        samples_per_pixel: 512,
        bits: 0,
        length: 1,
        data: vec![-5, 7],
    };
    let wf = WaveformData::from_json(&json).unwrap();
    assert_eq!(wf.bits(), 16);
    assert_eq!(wf.as_bytes().len(), 28);
}
