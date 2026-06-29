//! Pinned error message text and structured error fields.
//!
//! The message strings and the `ZoomTooLow` fields are the only way a caller
//! tells failure reasons apart, so each is asserted directly.

mod common;

use common::{make_data, Format};
use waveform_data::{Error, Resample, Slice};

#[test]
fn resample_error_messages() {
    let wf = make_data(Format::Binary, 1, 8);

    // floor(10 * 512 / 11) = 465, below the source scale 512.
    assert_eq!(
        wf.resample(Resample::Width(11.0)).unwrap_err().to_string(),
        "WaveformData.resample(): Zoom level 465 too low, minimum: 512"
    );
    assert_eq!(
        wf.resample(Resample::Scale(0.0)).unwrap_err().to_string(),
        "WaveformData.resample(): scale should be a positive integer value"
    );
    assert_eq!(
        wf.resample(Resample::Width(0.0)).unwrap_err().to_string(),
        "WaveformData.resample(): width should be a positive integer value"
    );
}

#[test]
fn zoom_too_low_carries_target_and_minimum() {
    let wf = make_data(Format::Binary, 1, 8);
    let err = wf.resample(Resample::Width(11.0)).unwrap_err();
    assert_eq!(
        err,
        Error::ZoomTooLow {
            target: 465,
            minimum: 512
        }
    );
}

#[test]
fn channel_error_messages() {
    let wf = make_data(Format::Binary, 1, 8);
    assert_eq!(wf.channel(5).unwrap_err().to_string(), "Invalid channel: 5");
    assert_eq!(
        wf.channel(-1).unwrap_err().to_string(),
        "Invalid channel: -1"
    );
}

#[test]
fn slice_negative_bound_messages() {
    let wf = make_data(Format::Binary, 1, 8);
    assert_eq!(
        wf.slice(Slice::Index { start: -1, end: 4 })
            .unwrap_err()
            .to_string(),
        "startIndex or startTime must not be negative"
    );
    assert_eq!(
        wf.slice(Slice::Index { start: 1, end: -1 })
            .unwrap_err()
            .to_string(),
        "endIndex or endTime must not be negative"
    );
}
