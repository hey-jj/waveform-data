//! Per-channel sample access.

use crate::error::Error;
use crate::WaveformData;

/// Reads the `(min, max)` samples of a single channel.
///
/// Get one with [`WaveformData::channel`]. Index positions run from 0 to
/// `length - 1`. Indexing at or beyond `length` returns
/// [`Error::IndexOutOfRange`].
#[derive(Debug)]
pub struct Channel<'a> {
    waveform: &'a WaveformData,
    channel_index: i32,
}

impl<'a> Channel<'a> {
    pub(crate) fn new(waveform: &'a WaveformData, channel_index: i32) -> Self {
        Channel {
            waveform,
            channel_index,
        }
    }

    fn min_offset(&self, index: i32) -> i64 {
        (index as i64 * self.waveform.channels() as i64 + self.channel_index as i64) * 2
    }

    fn max_offset(&self, index: i32) -> i64 {
        (index as i64 * self.waveform.channels() as i64 + self.channel_index as i64) * 2 + 1
    }

    /// Returns the minimum value at `index`.
    ///
    /// Returns [`Error::IndexOutOfRange`] when `index` is at or beyond the
    /// waveform length.
    pub fn min_sample(&self, index: i32) -> Result<i32, Error> {
        self.waveform.at(self.min_offset(index))
    }

    /// Returns the maximum value at `index`.
    ///
    /// Returns [`Error::IndexOutOfRange`] when `index` is at or beyond the
    /// waveform length.
    pub fn max_sample(&self, index: i32) -> Result<i32, Error> {
        self.waveform.at(self.max_offset(index))
    }

    /// Returns every minimum value, one per data point, in order.
    pub fn min_samples(&self) -> impl Iterator<Item = i32> + '_ {
        let length = self.waveform.length() as i32;
        (0..length).map(move |i| self.min_sample(i).expect("index within length"))
    }

    /// Returns every maximum value, one per data point, in order.
    pub fn max_samples(&self) -> impl Iterator<Item = i32> + '_ {
        let length = self.waveform.length() as i32;
        (0..length).map(move |i| self.max_sample(i).expect("index within length"))
    }

    /// Collects every minimum value into a `Vec`, one per data point, in order.
    pub fn min_array(&self) -> Vec<i32> {
        self.min_samples().collect()
    }

    /// Collects every maximum value into a `Vec`, one per data point, in order.
    pub fn max_array(&self) -> Vec<i32> {
        self.max_samples().collect()
    }
}
