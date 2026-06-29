//! Header and derived getters across both layouts and bit widths.

mod common;

use common::{make_data, Format};

const FORMATS: [Format; 2] = [Format::Binary, Format::Json];
const BITS: [i32; 2] = [8, 16];

#[test]
fn single_channel_header_fields() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            assert_eq!(wf.bits(), bits);
            assert_eq!(wf.channels(), 1);
            assert_eq!(wf.length(), 10);
            assert_eq!(wf.sample_rate(), 48000);
            assert_eq!(wf.scale(), 512);
        }
    }
}

#[test]
fn two_channel_header_fields() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 2, bits);
            assert_eq!(wf.bits(), bits);
            assert_eq!(wf.channels(), 2);
            assert_eq!(wf.length(), 10);
            assert_eq!(wf.sample_rate(), 48000);
            assert_eq!(wf.scale(), 512);
        }
    }
}

#[test]
fn derived_getters_are_exact_f64() {
    for format in FORMATS {
        for bits in BITS {
            for channels in [1, 2] {
                let wf = make_data(format, channels, bits);
                // 10 * 512 / 48000
                assert_eq!(wf.duration(), 0.10666666666666667);
                // 48000 / 512
                assert_eq!(wf.pixels_per_second(), 93.75);
                // 512 / 48000
                assert_eq!(wf.seconds_per_pixel(), 0.010666666666666666);
            }
        }
    }
}
