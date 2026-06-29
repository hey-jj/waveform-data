//! Resampling: option guards, header upgrade, and golden values.

mod common;

use common::{json_data, make_data, Format};
use waveform_data::{Error, Resample, WaveformData};

const FORMATS: [Format; 2] = [Format::Binary, Format::Json];
const BITS: [i32; 2] = [8, 16];

#[test]
fn width_above_length_too_low() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            // floor(10 * 512 / 11) = 465 < 512
            let err = wf.resample(Resample::Width(11.0)).unwrap_err();
            assert!(matches!(err, Error::ZoomTooLow { .. }));
        }
    }
}

#[test]
fn invalid_options() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            assert_eq!(
                wf.resample(Resample::Scale(0.0)).unwrap_err(),
                Error::InvalidScale
            );
            assert_eq!(
                wf.resample(Resample::Width(0.0)).unwrap_err(),
                Error::InvalidWidth
            );
        }
    }
}

#[test]
fn width_preserves_metadata_and_upgrades_to_v2() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            let out = wf.resample(Resample::Width(5.0)).unwrap();
            assert_eq!(out.length(), 5);
            assert_eq!(out.sample_rate(), 48000);
            assert_eq!(out.channels(), 1);
            assert_eq!(out.bits(), bits);
            // Resample output is always version 2: a 24-byte header.
            assert_eq!(
                out.to_array_buffer().len() - 24,
                5 * 2 * if bits == 8 { 1 } else { 2 }
            );
        }
    }
}

#[test]
fn scale_golden_values_single_channel() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            let out = wf.resample(Resample::Scale(1024.0)).unwrap();
            assert_eq!(out.length(), 5);
            assert_eq!(out.duration(), 0.10666666666666667);
            assert_eq!(
                out.channel(0).unwrap().min_array(),
                vec![-10, -5, -5, 0, -2]
            );
            assert_eq!(out.channel(0).unwrap().max_array(), vec![10, 7, 7, 0, 2]);
        }
    }
}

#[test]
fn scale_golden_values_two_channel() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 2, bits);
            let out = wf.resample(Resample::Scale(1024.0)).unwrap();
            assert_eq!(out.length(), 5);
            assert_eq!(out.duration(), 0.10666666666666667);
            assert_eq!(out.channels(), 2);
            assert_eq!(
                out.channel(0).unwrap().min_array(),
                vec![-10, -5, -5, 0, -2]
            );
            assert_eq!(out.channel(0).unwrap().max_array(), vec![10, 7, 7, 0, 2]);
            assert_eq!(out.channel(1).unwrap().min_array(), vec![-8, -6, -6, 0, -3]);
            assert_eq!(out.channel(1).unwrap().max_array(), vec![8, 3, 3, 0, 3]);
        }
    }
}

#[test]
fn empty_input_resample() {
    // A length-0 waveform exercises the seed-zero branch of the resampler.
    let json = waveform_data::JsonWaveformData {
        version: Some(2),
        channels: Some(2),
        sample_rate: 48000,
        samples_per_pixel: 512,
        bits: 8,
        length: 0,
        data: vec![],
    };
    let wf = WaveformData::from_json(&json).unwrap();
    let out = wf.resample(Resample::Scale(1024.0)).unwrap();
    assert_eq!(out.length(), 0);
    assert_eq!(out.channels(), 2);
    assert_eq!(out.channel(0).unwrap().min_array(), Vec::<i32>::new());
}

#[test]
fn width_to_scale_matches_json_round_trip() {
    // width 5 -> floor(10 * 512 / 5) = 1024, same output as scale 1024.
    let wf = WaveformData::from_json(&json_data(2, 8)).unwrap();
    let by_width = wf.resample(Resample::Width(5.0)).unwrap();
    let by_scale = wf.resample(Resample::Scale(1024.0)).unwrap();
    assert_eq!(by_width.to_array_buffer(), by_scale.to_array_buffer());
}
