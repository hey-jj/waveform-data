//! Concatenation: same format, cross format, multiple, incompatible.

mod common;

use common::{make_data, Format};
use waveform_data::Error;

#[test]
fn concat_single_channel_doubles() {
    for format in [Format::Binary, Format::Json] {
        let a = make_data(format, 1, 8);
        let b = make_data(format, 1, 8);
        let out = a.concat(&[&b]).unwrap();
        assert_eq!(out.channels(), 1);
        assert_eq!(out.length(), 20);
        assert_eq!(out.duration(), 0.21333333333333335);
        assert_eq!(
            out.channel(0).unwrap().min_array(),
            vec![0, -10, 0, -5, -5, 0, 0, 0, 0, -2, 0, -10, 0, -5, -5, 0, 0, 0, 0, -2]
        );
    }
}

#[test]
fn concat_two_channel_cross_format() {
    let a = make_data(Format::Binary, 2, 8);
    let b = make_data(Format::Json, 2, 8);
    let out = a.concat(&[&b]).unwrap();
    assert_eq!(out.channels(), 2);
    assert_eq!(out.length(), 20);
    assert_eq!(out.duration(), 0.21333333333333335);
    assert_eq!(
        out.channel(0).unwrap().min_array(),
        vec![0, -10, 0, -5, -5, 0, 0, 0, 0, -2, 0, -10, 0, -5, -5, 0, 0, 0, 0, -2]
    );
}

#[test]
fn concat_three_way() {
    let a = make_data(Format::Binary, 1, 8);
    let b = make_data(Format::Binary, 1, 8);
    let c = make_data(Format::Binary, 1, 8);
    let out = a.concat(&[&b, &c]).unwrap();
    assert_eq!(out.length(), 30);
    assert_eq!(out.duration(), 0.32);
}

#[test]
fn concat_incompatible_channels() {
    let mono = make_data(Format::Binary, 1, 8);
    let stereo = make_data(Format::Binary, 2, 8);
    assert_eq!(
        mono.concat(&[&stereo]).unwrap_err(),
        Error::IncompatibleWaveforms
    );
    assert_eq!(
        stereo.concat(&[&mono]).unwrap_err(),
        Error::IncompatibleWaveforms
    );
}

#[test]
fn concat_incompatible_bits() {
    let eight = make_data(Format::Binary, 1, 8);
    let sixteen = make_data(Format::Binary, 1, 16);
    assert_eq!(
        eight.concat(&[&sixteen]).unwrap_err(),
        Error::IncompatibleWaveforms
    );
}
