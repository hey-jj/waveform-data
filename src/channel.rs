//! Per-channel sample access.

use crate::error::Error;
use crate::WaveformData;

/// Read and write the `(min, max)` samples of a single channel.
///
/// Get one with [`WaveformData::channel`]. Index positions run from 0 to
/// `length - 1`. Indexing at or beyond `length` returns
/// [`Error::IndexOutOfRange`].
#[derive(Debug)]
pub struct WaveformDataChannel<'a> {
    waveform: &'a WaveformData,
    channel_index: i32,
}

impl<'a> WaveformDataChannel<'a> {
    pub(crate) fn new(waveform: &'a WaveformData, channel_index: i32) -> Self {
        WaveformDataChannel {
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
    pub fn min_array(&self) -> Vec<i32> {
        let length = self.waveform.length();
        let mut values = Vec::with_capacity(length as usize);
        for i in 0..length as i32 {
            values.push(self.min_sample(i).expect("index within length"));
        }
        values
    }

    /// Returns every maximum value, one per data point, in order.
    pub fn max_array(&self) -> Vec<i32> {
        let length = self.waveform.length();
        let mut values = Vec::with_capacity(length as usize);
        for i in 0..length as i32 {
            values.push(self.max_sample(i).expect("index within length"));
        }
        values
    }
}

/// Mutable per-channel writer used during resampling.
///
/// Mirrors `set_min_sample` and `set_max_sample`. Held separately from
/// [`WaveformDataChannel`] because writing borrows the waveform mutably.
#[derive(Debug)]
pub struct WaveformDataChannelMut<'a> {
    waveform: &'a mut WaveformData,
    channels: i32,
    channel_index: i32,
}

impl<'a> WaveformDataChannelMut<'a> {
    pub(crate) fn new(waveform: &'a mut WaveformData, channel_index: i32) -> Self {
        let channels = waveform.channels();
        WaveformDataChannelMut {
            waveform,
            channels,
            channel_index,
        }
    }

    /// Sets the minimum value at `index`. Values wrap into the sample width.
    pub fn set_min_sample(&mut self, index: i32, sample: i32) {
        let offset = (index as i64 * self.channels as i64 + self.channel_index as i64) * 2;
        self.waveform.set_at(offset, sample);
    }

    /// Sets the maximum value at `index`. Values wrap into the sample width.
    pub fn set_max_sample(&mut self, index: i32, sample: i32) {
        let offset = (index as i64 * self.channels as i64 + self.channel_index as i64) * 2 + 1;
        self.waveform.set_at(offset, sample);
    }
}
