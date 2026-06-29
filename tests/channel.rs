//! Channel access: arrays, indexed samples, range errors, direct setters.

mod common;

use common::{make_data, Format};
use waveform_data::Error;

const FORMATS: [Format; 2] = [Format::Binary, Format::Json];
const BITS: [i32; 2] = [8, 16];

#[test]
fn single_channel_arrays() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            assert_eq!(
                wf.channel(0).unwrap().min_array(),
                vec![0, -10, 0, -5, -5, 0, 0, 0, 0, -2]
            );
            assert_eq!(
                wf.channel(0).unwrap().max_array(),
                vec![0, 10, 0, 7, 7, 0, 0, 0, 0, 2]
            );
        }
    }
}

#[test]
fn single_channel_indexed_samples() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            let ch = wf.channel(0).unwrap();
            assert_eq!(ch.min_sample(0).unwrap(), 0);
            assert_eq!(ch.min_sample(4).unwrap(), -5);
            assert_eq!(ch.min_sample(9).unwrap(), -2);
            assert_eq!(ch.max_sample(0).unwrap(), 0);
            assert_eq!(ch.max_sample(4).unwrap(), 7);
            assert_eq!(ch.max_sample(9).unwrap(), 2);
        }
    }
}

#[test]
fn single_channel_out_of_range() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            let ch = wf.channel(0).unwrap();
            assert_eq!(ch.min_sample(10).unwrap_err(), Error::IndexOutOfRange);
            assert_eq!(ch.max_sample(10).unwrap_err(), Error::IndexOutOfRange);
        }
    }
}

#[test]
fn single_channel_invalid_channel() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            assert!(wf.channel(0).is_ok());
            assert_eq!(wf.channel(1).unwrap_err(), Error::InvalidChannel(1));
            assert_eq!(wf.channel(-1).unwrap_err(), Error::InvalidChannel(-1));
        }
    }
}

#[test]
fn two_channel_arrays() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 2, bits);
            assert_eq!(
                wf.channel(0).unwrap().min_array(),
                vec![0, -10, 0, -5, -5, 0, 0, 0, 0, -2]
            );
            assert_eq!(
                wf.channel(0).unwrap().max_array(),
                vec![0, 10, 0, 7, 7, 0, 0, 0, 0, 2]
            );
            assert_eq!(
                wf.channel(1).unwrap().min_array(),
                vec![0, -8, -2, -6, -6, 0, 0, 0, 0, -3]
            );
            assert_eq!(
                wf.channel(1).unwrap().max_array(),
                vec![0, 8, 2, 3, 3, 0, 0, 0, 0, 3]
            );
        }
    }
}

#[test]
fn two_channel_invalid_channel() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 2, bits);
            assert!(wf.channel(0).is_ok());
            assert!(wf.channel(1).is_ok());
            assert_eq!(wf.channel(2).unwrap_err(), Error::InvalidChannel(2));
            assert_eq!(wf.channel(-1).unwrap_err(), Error::InvalidChannel(-1));
        }
    }
}

#[test]
fn two_channel_out_of_range() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 2, bits);
            assert_eq!(
                wf.channel(0).unwrap().min_sample(10).unwrap_err(),
                Error::IndexOutOfRange
            );
            assert_eq!(
                wf.channel(1).unwrap().max_sample(10).unwrap_err(),
                Error::IndexOutOfRange
            );
        }
    }
}

#[test]
fn samples_iterators_match_arrays() {
    for format in FORMATS {
        for bits in BITS {
            let wf = make_data(format, 1, bits);
            let ch = wf.channel(0).unwrap();
            assert_eq!(ch.min_samples().collect::<Vec<_>>(), ch.min_array());
            assert_eq!(ch.max_samples().collect::<Vec<_>>(), ch.max_array());
            // The iterator can fold without allocating a Vec.
            assert_eq!(ch.min_samples().min(), Some(-10));
            assert_eq!(ch.max_samples().max(), Some(10));
        }
    }
}
