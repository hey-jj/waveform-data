//! Time and index conversion.

mod common;

use common::{make_data, Format};

#[test]
fn at_time_floors() {
    let wf = make_data(Format::Binary, 1, 8);
    assert_eq!(wf.at_time(0.0), 0);
    // floor(0.15 * 48000 / 512) = floor(14.0625)
    assert_eq!(wf.at_time(0.15), 14);
    // floor(93.75)
    assert_eq!(wf.at_time(1.0), 93);
}

#[test]
fn time_is_exact_f64() {
    let wf = make_data(Format::Binary, 1, 8);
    // index * scale / sample_rate, evaluated left to right in f64.
    assert_eq!(wf.time(0), 0.0);
    assert_eq!(wf.time(1), 0.010666666666666666);
    assert_eq!(wf.time(14), 0.14933333333333335);
    assert_eq!(wf.time(93), 0.992);
}

#[test]
fn at_time_time_round_trip() {
    let wf = make_data(Format::Binary, 1, 8);
    for n in [0i64, 14, 93] {
        assert_eq!(wf.at_time(wf.time(n)), n);
    }
}

#[test]
fn at_time_negative_floors_toward_neg_infinity() {
    let wf = make_data(Format::Binary, 1, 8);
    // floor(-0.15 * 48000 / 512) = floor(-14.0625) = -15
    assert_eq!(wf.at_time(-0.15), -15);
}
