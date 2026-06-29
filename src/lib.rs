//! Read, write, and reshape precomputed audio waveform peak data.
//!
//! This crate handles the BBC `audiowaveform` data formats: the little-endian
//! binary `.dat` layout (versions 1 and 2) and the matching JSON object. A
//! waveform holds `length` data points. Each point carries a `(min, max)` pair
//! per channel summarizing `scale` consecutive audio samples.
//!
//! # Loading
//!
//! Build a [`WaveformData`] from a binary buffer with
//! [`WaveformData::from_binary`], or from a [`JsonWaveformData`] object with
//! [`WaveformData::from_json`]. JSON input is converted to version-2 binary on
//! load.
//!
//! ```
//! use waveform_data::{JsonWaveformData, WaveformData};
//!
//! let json = JsonWaveformData {
//!     version: None,
//!     channels: None,
//!     sample_rate: 48000,
//!     samples_per_pixel: 512,
//!     bits: 8,
//!     length: 2,
//!     data: vec![0, 0, -10, 10],
//! };
//! let waveform = WaveformData::from_json(&json).unwrap();
//! assert_eq!(waveform.length(), 2);
//! assert_eq!(waveform.channel(0).unwrap().min_array(), vec![0, -10]);
//! ```
//!
//! # Accessors and transforms
//!
//! Read header fields with [`WaveformData::sample_rate`], [`WaveformData::scale`],
//! [`WaveformData::bits`], [`WaveformData::length`], [`WaveformData::channels`],
//! and the derived [`WaveformData::duration`],
//! [`WaveformData::pixels_per_second`], [`WaveformData::seconds_per_pixel`].
//! Read samples through [`WaveformData::channel`]. Reshape with
//! [`WaveformData::resample`], [`WaveformData::concat`], and
//! [`WaveformData::slice`]. Serialize with [`WaveformData::to_json`] and
//! [`WaveformData::as_bytes`].
//!
//! # Peak generation
//!
//! Build peaks from PCM with
//! [`generate_waveform_data`](generator::generate_waveform_data).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod channel;
mod error;
pub mod generator;
mod json;

pub use channel::Channel;
pub use error::Error;
pub use generator::{generate_waveform_data, GenerateOptions};
pub use json::JsonWaveformData;

/// Selection passed to [`WaveformData::resample`].
pub enum Resample {
    /// Target output width in data points. The scale is derived from it.
    Width(f64),
    /// Target audio samples per pixel.
    Scale(f64),
}

/// Selection passed to [`WaveformData::slice`].
pub enum Slice {
    /// Half-open data point range `[start, end)`.
    Index {
        /// First data point to keep.
        start: i64,
        /// First data point to drop.
        end: i64,
    },
    /// Time range in seconds, converted with [`WaveformData::at_time`].
    Time {
        /// Start time in seconds.
        start: f64,
        /// End time in seconds.
        end: f64,
    },
}

/// A parsed waveform: header fields plus interleaved `(min, max)` samples.
///
/// Internally the data is held as a version-2 or version-1 binary buffer. JSON
/// input is converted to version-2 on load. Transforms ([`resample`](Self::resample),
/// [`concat`](Self::concat), [`slice`](Self::slice)) and generated buffers are
/// always version 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformData {
    data: Vec<u8>,
    offset: usize,
}

impl WaveformData {
    /// Builds a waveform from a binary `.dat` buffer.
    ///
    /// Accepts version 1 (20-byte header, single channel) and version 2
    /// (24-byte header).
    ///
    /// # Errors
    ///
    /// - [`Error::UnknownDataFormat`] when the buffer is shorter than the version
    ///   field or shorter than the full header for its version.
    /// - [`Error::UnsupportedVersion`] for any version other than 1 or 2.
    /// - [`Error::DataLengthMismatch`] when the `length` header field claims more
    ///   data points than the buffer can hold. This rejects a truncated or
    ///   inflated buffer up front so the sample accessors never read past the end.
    pub fn from_binary(data: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let data = data.into();
        if data.len() < 4 {
            return Err(Error::UnknownDataFormat);
        }
        let version = read_i32_le(&data, 0);
        if version != 1 && version != 2 {
            return Err(Error::UnsupportedVersion);
        }
        let offset = if version == 2 { 24 } else { 20 };
        if data.len() < offset {
            return Err(Error::UnknownDataFormat);
        }

        // The header is present. Check the claimed data section against the bytes
        // that follow it. length, channels, and the bit flag all live in the
        // header now that the buffer is at least `offset` long.
        let length = read_u32_le(&data, 16) as u64;
        let channels = if version == 2 {
            read_i32_le(&data, 20)
        } else {
            1
        };
        if channels <= 0 {
            return Err(Error::DataLengthMismatch);
        }
        let bytes_per_sample = if read_u32_le(&data, 4) != 0 { 1u64 } else { 2 };
        let data_bytes = length * channels as u64 * 2 * bytes_per_sample;
        if (data.len() - offset) as u64 != data_bytes {
            return Err(Error::DataLengthMismatch);
        }

        Ok(WaveformData { data, offset })
    }

    /// Builds a waveform from a [`JsonWaveformData`] object.
    ///
    /// The object is converted to a version-2 binary buffer. Returns
    /// [`Error::LengthMismatch`] when `data.len()` is not
    /// `length * 2 * channels`.
    pub fn from_json(data: &JsonWaveformData) -> Result<Self, Error> {
        let buffer = convert_json_to_binary(data)?;
        // The converter always writes a valid version-2 header.
        WaveformData::from_binary(buffer)
    }

    fn version(&self) -> i32 {
        read_i32_le(&self.data, 0)
    }

    /// Returns the number of data points.
    pub fn length(&self) -> u32 {
        read_u32_le(&self.data, 16)
    }

    /// Returns the bits per stored sample, either 8 or 16.
    ///
    /// The header stores a flag, not a width. A nonzero flag means 8-bit.
    pub fn bits(&self) -> i32 {
        if read_u32_le(&self.data, 4) != 0 {
            8
        } else {
            16
        }
    }

    /// Returns the approximate audio duration in seconds.
    ///
    /// Computed as `length * scale / sample_rate` in `f64`.
    pub fn duration(&self) -> f64 {
        self.length() as f64 * self.scale() as f64 / self.sample_rate() as f64
    }

    /// Returns data points per second, `sample_rate / scale`.
    pub fn pixels_per_second(&self) -> f64 {
        self.sample_rate() as f64 / self.scale() as f64
    }

    /// Returns seconds per data point, `scale / sample_rate`.
    pub fn seconds_per_pixel(&self) -> f64 {
        self.scale() as f64 / self.sample_rate() as f64
    }

    /// Returns the channel count. Version 1 is always 1.
    pub fn channels(&self) -> i32 {
        if self.version() == 2 {
            read_i32_le(&self.data, 20)
        } else {
            1
        }
    }

    /// Returns the audio samples per second from the header.
    pub fn sample_rate(&self) -> i32 {
        read_i32_le(&self.data, 8)
    }

    /// Returns the audio samples summarized per data point.
    pub fn scale(&self) -> i32 {
        read_i32_le(&self.data, 12)
    }

    /// Returns a reader for channel `index`.
    ///
    /// Returns [`Error::InvalidChannel`] when `index` is negative or at/above
    /// the channel count.
    pub fn channel(&self, index: i32) -> Result<Channel<'_>, Error> {
        if index >= 0 && index < self.channels() {
            Ok(Channel::new(self, index))
        } else {
            Err(Error::InvalidChannel(index))
        }
    }

    /// Returns the data point index for a time in seconds.
    ///
    /// `floor(time * sample_rate / scale)`. Floors toward negative infinity.
    pub fn at_time(&self, time: f64) -> i64 {
        (time * self.sample_rate() as f64 / self.scale() as f64).floor() as i64
    }

    /// Returns the time in seconds for a data point index.
    ///
    /// `index * scale / sample_rate`. This is the inverse of [`at_time`](Self::at_time),
    /// which returns an `i64` index, so the parameter is an `i64` too. The
    /// round-trip `at_time(time(n)) == n` holds for any non-negative `n` in range.
    pub fn time(&self, index: i64) -> f64 {
        index as f64 * self.scale() as f64 / self.sample_rate() as f64
    }

    /// Reads flat sample slot `index`.
    ///
    /// Returns [`Error::IndexOutOfRange`] when the slot falls outside the data
    /// section, matching the bounds check of a `DataView` read.
    fn at(&self, index: i64) -> Result<i32, Error> {
        if self.bits() == 8 {
            let byte = self.offset as i64 + index;
            if byte < 0 || byte as usize >= self.data.len() {
                return Err(Error::IndexOutOfRange);
            }
            Ok(self.data[byte as usize] as i8 as i32)
        } else {
            let byte = self.offset as i64 + index * 2;
            if byte < 0 || (byte as usize) + 2 > self.data.len() {
                return Err(Error::IndexOutOfRange);
            }
            let b = byte as usize;
            Ok(i16::from_le_bytes([self.data[b], self.data[b + 1]]) as i32)
        }
    }

    /// Returns the waveform as a JSON object.
    ///
    /// Always reports `version = 2`. The `samples_per_pixel` field carries the
    /// scale. The `data` array is interleaved by data point then channel then
    /// `(min, max)`.
    pub fn to_json(&self) -> JsonWaveformData {
        let channels = self.channels();
        let length = self.length();
        let mut data = Vec::with_capacity((length as usize) * (channels as usize) * 2);
        for i in 0..length as i32 {
            for channel in 0..channels {
                let ch = self.channel(channel).expect("channel in range");
                data.push(ch.min_sample(i).expect("index in range"));
                data.push(ch.max_sample(i).expect("index in range"));
            }
        }
        JsonWaveformData {
            version: Some(2),
            channels: Some(channels),
            sample_rate: self.sample_rate(),
            samples_per_pixel: self.scale(),
            bits: self.bits(),
            length: length as i32,
            data,
        }
    }

    /// Returns the underlying binary buffer as a borrowed slice.
    ///
    /// For a binary input this is the original buffer with its original header,
    /// so a version-1 input keeps its 20-byte header. For a JSON input it is the
    /// converted version-2 buffer.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Resamples to a lower resolution and returns a new waveform.
    ///
    /// With [`Resample::Width`] the target scale is
    /// `floor(duration * sample_rate / width)`. The output is always version 2.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidWidth`] / [`Error::InvalidScale`] when the value is not
    ///   positive.
    /// - [`Error::ZoomTooLow`] when the target scale is below the source scale.
    pub fn resample(&self, options: Resample) -> Result<WaveformData, Error> {
        let target_scale = self.resample_target_scale(options)?;
        let mut resampler = WaveformResampler::new(self, target_scale);
        while !resampler.next() {}
        WaveformData::from_binary(resampler.output_data)
    }

    fn resample_target_scale(&self, options: Resample) -> Result<i32, Error> {
        // Compute the target scale in i64. With a wide source the width divisor
        // can floor to a value past i32::MAX, so the range check runs in i64
        // before the result narrows.
        let target = match options {
            Resample::Width(width) => {
                if width <= 0.0 {
                    return Err(Error::InvalidWidth);
                }
                (self.duration() * self.sample_rate() as f64 / width).floor() as i64
            }
            Resample::Scale(scale) => {
                if scale <= 0.0 {
                    return Err(Error::InvalidScale);
                }
                scale as i64
            }
        };

        if target < self.scale() as i64 {
            return Err(Error::ZoomTooLow {
                target,
                minimum: self.scale(),
            });
        }

        Ok(target as i32)
    }

    /// Concatenates one or more waveforms onto this one.
    ///
    /// Every operand must match this waveform's channels, sample rate, bits, and
    /// scale, else [`Error::IncompatibleWaveforms`]. The result copies this
    /// waveform's header, sums the lengths, and appends each data section. The
    /// header version follows this waveform.
    ///
    /// `others` accepts anything that iterates over `&WaveformData`, so a slice,
    /// an array, a `Vec`, or an iterator all work. Concatenating one waveform
    /// reads as `a.concat([&b])`.
    pub fn concat<'a, I>(&self, others: I) -> Result<WaveformData, Error>
    where
        I: IntoIterator<Item = &'a WaveformData>,
    {
        let others: Vec<&WaveformData> = others.into_iter().collect();
        for other in &others {
            if self.channels() != other.channels()
                || self.sample_rate() != other.sample_rate()
                || self.bits() != other.bits()
                || self.scale() != other.scale()
            {
                return Err(Error::IncompatibleWaveforms);
            }
        }

        let header_size = self.offset;
        let mut buffers: Vec<&[u8]> = Vec::with_capacity(others.len() + 1);
        buffers.push(&self.data);
        for other in &others {
            buffers.push(&other.data);
        }

        let mut total_size = header_size;
        // Sum the length fields in i64. Each field is a 32-bit value, so several
        // operands can sum past i32::MAX before the result narrows back into the
        // 32-bit length field.
        let mut total_data_length: i64 = 0;
        for buffer in &buffers {
            // concat reuses the receiver's header offset for every operand. An
            // operand shorter than that offset has no data region to copy, so
            // treat it as incompatible instead of underflowing the subtraction.
            if buffer.len() < header_size {
                return Err(Error::IncompatibleWaveforms);
            }
            let data_size = read_i32_le(buffer, 16) as i64;
            total_size += buffer.len() - header_size;
            total_data_length += data_size;
        }

        let mut total_buffer = vec![0u8; total_size];
        total_buffer[..header_size].copy_from_slice(&self.data[..header_size]);
        write_i32_le(&mut total_buffer, 16, total_data_length as i32);

        let mut offset = header_size;
        for buffer in &buffers {
            let chunk = &buffer[header_size..];
            total_buffer[offset..offset + chunk.len()].copy_from_slice(chunk);
            offset += chunk.len();
        }

        WaveformData::from_binary(total_buffer)
    }

    /// Returns a sub-range as a new waveform.
    ///
    /// Index pairs apply directly. Time pairs convert with
    /// [`at_time`](Self::at_time). Indices clamp to `[_, length]`, and if start
    /// exceeds end the result is empty. The output is always version 2.
    ///
    /// Returns [`Error::NegativeStart`] / [`Error::NegativeEnd`] for negative
    /// bounds.
    pub fn slice(&self, options: Slice) -> Result<WaveformData, Error> {
        let (mut start_index, mut end_index) = match options {
            Slice::Index { start, end } => (start, end),
            Slice::Time { start, end } => (self.at_time(start), self.at_time(end)),
        };

        if start_index < 0 {
            return Err(Error::NegativeStart);
        }
        if end_index < 0 {
            return Err(Error::NegativeEnd);
        }

        let length_field = self.length() as i64;
        if start_index > length_field {
            start_index = length_field;
        }
        if end_index > length_field {
            end_index = length_field;
        }
        if start_index > end_index {
            start_index = end_index;
        }

        let length = end_index - start_index;
        let channels = self.channels() as i64;
        let header_size = 24usize;
        let eight_bit = self.bits() == 8;
        let bytes_per_sample = if eight_bit { 1usize } else { 2 };
        let total_size = header_size + (length * 2 * channels) as usize * bytes_per_sample;

        let mut output = vec![0u8; total_size];
        write_i32_le(&mut output, 0, 2);
        write_u32_le(&mut output, 4, if eight_bit { 1 } else { 0 });
        write_i32_le(&mut output, 8, self.sample_rate());
        write_i32_le(&mut output, 12, self.scale());
        write_i32_le(&mut output, 16, length as i32);
        write_i32_le(&mut output, 20, self.channels());

        let slots = length * channels * 2;
        for i in 0..slots {
            let sample = self.at(start_index * channels * 2 + i)?;
            if eight_bit {
                output[header_size + i as usize] = sample as i8 as u8;
            } else {
                let bytes = (sample as i16).to_le_bytes();
                let pos = header_size + i as usize * 2;
                output[pos] = bytes[0];
                output[pos + 1] = bytes[1];
            }
        }

        WaveformData::from_binary(output)
    }
}

/// Audacity-derived min/max downsampler driving [`WaveformData::resample`].
struct WaveformResampler {
    output_data: Vec<u8>,
    output_channels: i32,
    output_samples_per_pixel: i32,
    scale: i32,
    input_buffer_size: i32,
    input_min: Vec<Vec<i32>>,
    input_max: Vec<Vec<i32>>,
    input_index: i32,
    output_index: i32,
    min: Vec<i32>,
    max: Vec<i32>,
    min_value: i32,
    max_value: i32,
    last_input_index: i32,
    eight_bit: bool,
}

impl WaveformResampler {
    fn new(input: &WaveformData, target_scale: i32) -> Self {
        let channels = input.channels();
        let scale = input.scale();
        let input_buffer_size = input.length() as i32;

        let input_buffer_length_samples = input_buffer_size as i64 * scale as i64;
        let output_buffer_length_samples =
            (input_buffer_length_samples as f64 / target_scale as f64).ceil() as i32;

        let eight_bit = input.bits() == 8;
        let bytes_per_sample = if eight_bit { 1usize } else { 2 };
        let header_size = 24usize;
        let total_size = header_size
            + output_buffer_length_samples as usize * 2 * channels as usize * bytes_per_sample;

        let mut output_data = vec![0u8; total_size];
        write_i32_le(&mut output_data, 0, 2);
        write_u32_le(&mut output_data, 4, if eight_bit { 1 } else { 0 });
        write_i32_le(&mut output_data, 8, input.sample_rate());
        write_i32_le(&mut output_data, 12, target_scale);
        write_i32_le(&mut output_data, 16, output_buffer_length_samples);
        write_i32_le(&mut output_data, 20, channels);

        // Cache the input samples per channel up front so the resampler does not
        // hold a borrow on the input while writing the output.
        let mut input_min = Vec::with_capacity(channels as usize);
        let mut input_max = Vec::with_capacity(channels as usize);
        for c in 0..channels {
            let ch = input.channel(c).expect("channel in range");
            input_min.push(ch.min_array());
            input_max.push(ch.max_array());
        }

        let mut min = vec![0i32; channels as usize];
        let mut max = vec![0i32; channels as usize];
        for c in 0..channels as usize {
            if input_buffer_size > 0 {
                min[c] = input_min[c][0];
                max[c] = input_max[c][0];
            } else {
                min[c] = 0;
                max[c] = 0;
            }
        }

        let (min_value, max_value) = if eight_bit {
            (-128, 127)
        } else {
            (-32768, 32767)
        };

        WaveformResampler {
            output_data,
            output_channels: channels,
            output_samples_per_pixel: target_scale,
            scale,
            input_buffer_size,
            input_min,
            input_max,
            input_index: 0,
            output_index: 0,
            min,
            max,
            min_value,
            max_value,
            last_input_index: 0,
            eight_bit,
        }
    }

    fn sample_at_pixel(&self, x: i32) -> i64 {
        // Keep the floored product in i64. A wide output makes
        // `x * output_samples_per_pixel` exceed i32::MAX, and an i32 cast would
        // saturate or wrap so the pixel-boundary checks diverge from the f64
        // reference. The downstream stop index narrows back to i32 when indexing.
        (x as f64 * self.output_samples_per_pixel as f64).floor() as i64
    }

    fn write_output(&mut self, point: i32, channel: i32, min: i32, max: i32) {
        let slot_min = (point as i64 * self.output_channels as i64 + channel as i64) * 2;
        let slot_max = slot_min + 1;
        let header = 24usize;
        if self.eight_bit {
            self.output_data[header + slot_min as usize] = min as i8 as u8;
            self.output_data[header + slot_max as usize] = max as i8 as u8;
        } else {
            let mn = (min as i16).to_le_bytes();
            let mx = (max as i16).to_le_bytes();
            let pmin = header + slot_min as usize * 2;
            let pmax = header + slot_max as usize * 2;
            self.output_data[pmin..pmin + 2].copy_from_slice(&mn);
            self.output_data[pmax..pmax + 2].copy_from_slice(&mx);
        }
    }

    fn next(&mut self) -> bool {
        let mut count = 0;
        let total = 1000;
        let channels = self.output_channels;

        while self.input_index < self.input_buffer_size && count < total {
            while (self.sample_at_pixel(self.output_index) as f64 / self.scale as f64).floor()
                as i64
                == self.input_index as i64
            {
                if self.output_index > 0 {
                    for i in 0..channels {
                        let point = self.output_index - 1;
                        let min = self.min[i as usize];
                        let max = self.max[i as usize];
                        self.write_output(point, i, min, max);
                    }
                }

                self.last_input_index = self.input_index;
                self.output_index += 1;

                let where_ = self.sample_at_pixel(self.output_index);
                let prev_where = self.sample_at_pixel(self.output_index - 1);

                if where_ != prev_where {
                    for i in 0..channels as usize {
                        self.min[i] = self.max_value;
                        self.max[i] = self.min_value;
                    }
                }
            }

            let where_ = self.sample_at_pixel(self.output_index);
            let mut stop = (where_ as f64 / self.scale as f64).floor() as i64;
            if stop > self.input_buffer_size as i64 {
                stop = self.input_buffer_size as i64;
            }
            let stop = stop as i32;

            while self.input_index < stop {
                for i in 0..channels as usize {
                    let value = self.input_min[i][self.input_index as usize];
                    if value < self.min[i] {
                        self.min[i] = value;
                    }
                    let value = self.input_max[i][self.input_index as usize];
                    if value > self.max[i] {
                        self.max[i] = value;
                    }
                }
                self.input_index += 1;
            }

            count += 1;
        }

        if self.input_index < self.input_buffer_size {
            false
        } else {
            if self.input_index != self.last_input_index {
                for i in 0..channels {
                    let point = self.output_index - 1;
                    let min = self.min[i as usize];
                    let max = self.max[i as usize];
                    self.write_output(point, i, min, max);
                }
            }
            true
        }
    }
}

fn convert_json_to_binary(data: &JsonWaveformData) -> Result<Vec<u8>, Error> {
    let channels = match data.channels {
        Some(c) if c != 0 => c,
        _ => 1,
    };
    let header_size = 24usize;
    let eight_bit = data.bits == 8;
    let bytes_per_sample = if eight_bit { 1usize } else { 2 };
    let expected_length = data.length as i64 * 2 * channels as i64;

    if data.data.len() as i64 != expected_length {
        return Err(Error::LengthMismatch);
    }

    let total_size = header_size + data.data.len() * bytes_per_sample;
    let mut buffer = vec![0u8; total_size];

    write_i32_le(&mut buffer, 0, 2);
    write_u32_le(&mut buffer, 4, if eight_bit { 1 } else { 0 });
    write_i32_le(&mut buffer, 8, data.sample_rate);
    write_i32_le(&mut buffer, 12, data.samples_per_pixel);
    write_i32_le(&mut buffer, 16, data.length);
    write_i32_le(&mut buffer, 20, channels);

    let mut index = header_size;
    if eight_bit {
        for &value in &data.data {
            buffer[index] = value as i8 as u8;
            index += 1;
        }
    } else {
        for &value in &data.data {
            let bytes = (value as i16).to_le_bytes();
            buffer[index] = bytes[0];
            buffer[index + 1] = bytes[1];
            index += 2;
        }
    }

    Ok(buffer)
}

fn read_i32_le(buffer: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

fn read_u32_le(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

fn write_i32_le(buffer: &mut [u8], offset: usize, value: i32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u32_le(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
