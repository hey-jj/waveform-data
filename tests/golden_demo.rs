//! Cross-format parity on a real 5221-point stereo waveform.
//!
//! The `.dat` and `.json` files describe the same waveform. Parsing both must
//! yield equal metadata and per-channel arrays.

mod common;

use common::{fixture_path, parse_demo_json};
use waveform_data::WaveformData;

fn load_pair() -> (WaveformData, WaveformData) {
    let dat_bytes = std::fs::read(fixture_path("07023003-2channel.dat")).unwrap();
    let json_text = std::fs::read_to_string(fixture_path("07023003-2channel.json")).unwrap();
    let from_dat = WaveformData::from_binary(dat_bytes).unwrap();
    let from_json = WaveformData::from_json(&parse_demo_json(&json_text)).unwrap();
    (from_dat, from_json)
}

#[test]
fn metadata_matches() {
    let (dat, json) = load_pair();
    assert_eq!(dat.length(), 5221);
    assert_eq!(dat.channels(), 2);
    assert_eq!(dat.sample_rate(), 44100);
    assert_eq!(dat.scale(), 128);
    assert_eq!(dat.bits(), 8);

    assert_eq!(dat.length(), json.length());
    assert_eq!(dat.channels(), json.channels());
    assert_eq!(dat.sample_rate(), json.sample_rate());
    assert_eq!(dat.scale(), json.scale());
    assert_eq!(dat.bits(), json.bits());
}

#[test]
fn per_channel_arrays_match() {
    let (dat, json) = load_pair();
    for c in 0..2 {
        assert_eq!(
            dat.channel(c).unwrap().min_array(),
            json.channel(c).unwrap().min_array()
        );
        assert_eq!(
            dat.channel(c).unwrap().max_array(),
            json.channel(c).unwrap().max_array()
        );
    }
}

#[test]
fn to_json_of_dat_matches_parsed_json() {
    let (dat, json) = load_pair();
    let produced = dat.to_json();
    let parsed = json.to_json();
    assert_eq!(produced.version, parsed.version);
    assert_eq!(produced.channels, parsed.channels);
    assert_eq!(produced.sample_rate, parsed.sample_rate);
    assert_eq!(produced.samples_per_pixel, parsed.samples_per_pixel);
    assert_eq!(produced.bits, parsed.bits);
    assert_eq!(produced.length, parsed.length);
    assert_eq!(produced.data, parsed.data);
}
