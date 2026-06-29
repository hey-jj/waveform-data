//! JSON waveform input shape.

/// Plain waveform data in the JSON layout.
///
/// This mirrors the fields of a parsed `.json` waveform object. Feed it to
/// [`WaveformData::from_json`](crate::WaveformData::from_json) to build a
/// waveform. The `data` array holds interleaved `(min, max)` pairs, ordered by
/// data point then channel.
///
/// `version` and `channels` are optional in single-channel files. A missing or
/// zero `channels` is treated as 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonWaveformData {
    /// Format version (1 or 2). Single-channel JSON may omit it.
    pub version: Option<i32>,
    /// Channel count. Missing or zero means 1.
    pub channels: Option<i32>,
    /// Audio samples per second.
    pub sample_rate: i32,
    /// Audio samples summarized per data point.
    pub samples_per_pixel: i32,
    /// Bits per stored sample. Only exactly 8 selects 8-bit storage.
    pub bits: i32,
    /// Number of data points.
    pub length: i32,
    /// Flat interleaved `(min, max)` values.
    pub data: Vec<i32>,
}
