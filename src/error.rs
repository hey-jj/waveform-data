//! Error type for waveform parsing and manipulation.

use std::error::Error as StdError;
use std::fmt;

/// Errors returned when parsing, accessing, or transforming waveform data.
///
/// The variants map to the failures the format and operations can raise. Each
/// carries the message text the operation reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Input was neither JSON waveform data nor a binary waveform buffer.
    UnknownDataFormat,
    /// Binary header carried a version other than 1 or 2.
    UnsupportedVersion,
    /// JSON `data` length did not match `length * 2 * channels`.
    LengthMismatch,
    /// Binary buffer's data section did not match its header `length` and
    /// `channels`. The buffer is truncated or its `length` field is inflated.
    DataLengthMismatch,
    /// Channel index was negative or at/above the channel count.
    InvalidChannel(i32),
    /// Sample index fell outside the stored data section.
    IndexOutOfRange,
    /// `resample` got a `width` that was not a positive value.
    InvalidWidth,
    /// `resample` got a `scale` that was not a positive value.
    InvalidScale,
    /// `resample` target scale was below the source scale.
    ZoomTooLow {
        /// Requested target scale.
        target: i64,
        /// Source scale, the minimum allowed.
        minimum: i32,
    },
    /// Negative `startIndex` or `startTime` passed to `slice`.
    NegativeStart,
    /// Negative `endIndex` or `endTime` passed to `slice`.
    NegativeEnd,
    /// `concat` operands disagreed on channels, sample rate, bits, or scale.
    IncompatibleWaveforms,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownDataFormat => {
                write!(f, "WaveformData.create(): Unknown data format")
            }
            Error::UnsupportedVersion => {
                write!(
                    f,
                    "WaveformData.create(): This waveform data version not supported"
                )
            }
            Error::LengthMismatch => {
                write!(
                    f,
                    "WaveformData.create(): Length mismatch in JSON waveform data"
                )
            }
            Error::DataLengthMismatch => {
                write!(
                    f,
                    "WaveformData.create(): Binary data section does not match the header length"
                )
            }
            Error::InvalidChannel(index) => write!(f, "Invalid channel: {index}"),
            Error::IndexOutOfRange => write!(f, "Index out of range"),
            Error::InvalidWidth => write!(
                f,
                "WaveformData.resample(): width should be a positive integer value"
            ),
            Error::InvalidScale => write!(
                f,
                "WaveformData.resample(): scale should be a positive integer value"
            ),
            Error::ZoomTooLow { target, minimum } => write!(
                f,
                "WaveformData.resample(): Zoom level {target} too low, minimum: {minimum}"
            ),
            Error::NegativeStart => {
                write!(f, "startIndex or startTime must not be negative")
            }
            Error::NegativeEnd => write!(f, "endIndex or endTime must not be negative"),
            Error::IncompatibleWaveforms => {
                write!(f, "WaveformData.concat(): Waveforms are incompatible")
            }
        }
    }
}

impl StdError for Error {}
