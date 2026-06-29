//! Slicing by index and time, with clamping and range errors.

mod common;

use common::{make_data, Format};
use waveform_data::{Error, Slice};

#[test]
fn slice_by_index() {
    for format in [Format::Binary, Format::Json] {
        for bits in [8, 16] {
            let wf = make_data(format, 1, bits);
            let out = wf.slice(Slice::Index { start: 1, end: 4 }).unwrap();
            assert_eq!(out.length(), 3);
            assert_eq!(out.bits(), bits);
            assert_eq!(out.channels(), 1);
            assert_eq!(out.channel(0).unwrap().min_array(), vec![-10, 0, -5]);
            assert_eq!(out.channel(0).unwrap().max_array(), vec![10, 0, 7]);
        }
    }
}

#[test]
fn slice_end_beyond_length_clamps() {
    let wf = make_data(Format::Binary, 1, 8);
    let out = wf.slice(Slice::Index { start: 1, end: 12 }).unwrap();
    assert_eq!(out.length(), 9);
    assert_eq!(
        out.channel(0).unwrap().min_array(),
        vec![-10, 0, -5, -5, 0, 0, 0, 0, -2]
    );
    assert_eq!(
        out.channel(0).unwrap().max_array(),
        vec![10, 0, 7, 7, 0, 0, 0, 0, 2]
    );
}

#[test]
fn slice_equal_bounds_empty() {
    let wf = make_data(Format::Binary, 1, 8);
    let out = wf.slice(Slice::Index { start: 1, end: 1 }).unwrap();
    assert_eq!(out.length(), 0);
    assert_eq!(out.channel(0).unwrap().min_array(), Vec::<i32>::new());
    assert_eq!(out.channel(0).unwrap().max_array(), Vec::<i32>::new());
}

#[test]
fn slice_start_greater_than_end_empty() {
    let wf = make_data(Format::Binary, 1, 8);
    let out = wf.slice(Slice::Index { start: 4, end: 1 }).unwrap();
    assert_eq!(out.length(), 0);
}

#[test]
fn slice_both_beyond_length_empty() {
    let wf = make_data(Format::Binary, 1, 8);
    let out = wf.slice(Slice::Index { start: 10, end: 12 }).unwrap();
    assert_eq!(out.length(), 0);
}

#[test]
fn slice_negative_bounds_error() {
    let wf = make_data(Format::Binary, 1, 8);
    assert_eq!(
        wf.slice(Slice::Index { start: -1, end: 4 }).unwrap_err(),
        Error::NegativeStart
    );
    assert_eq!(
        wf.slice(Slice::Index { start: 1, end: -1 }).unwrap_err(),
        Error::NegativeEnd
    );
}

#[test]
fn slice_by_time_matches_index() {
    let wf = make_data(Format::Binary, 1, 8);
    let start = wf.time(1.0);
    let end = wf.time(4.0);
    let out = wf.slice(Slice::Time { start, end }).unwrap();
    assert_eq!(out.length(), 3);
    assert_eq!(out.channel(0).unwrap().min_array(), vec![-10, 0, -5]);
    assert_eq!(out.channel(0).unwrap().max_array(), vec![10, 0, 7]);
}
