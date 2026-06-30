//! Peak generation from PCM samples.
//!
//! Adapted from Audacity's `BlockFile::CalcSummary`. Produces a version-2
//! binary buffer of `(min, max)` peaks.

use crate::{write_i32_le, write_u32_le};

const INT8_MAX: f64 = 127.0;
const INT8_MIN: f64 = -128.0;
const INT16_MAX: f64 = 32767.0;
const INT16_MIN: f64 = -32768.0;

/// Options for [`generate_waveform_data`].
///
/// `channels` holds one PCM buffer per input channel. PCM values are normally
/// in `[-1.0, 1.0]`. With `split_channels` false, all input channels are mixed
/// down to one output channel. With it true, each input channel maps to its own
/// output channel.
pub struct GenerateOptions<'a> {
    /// Audio samples summarized per output data point.
    pub scale: i32,
    /// Output sample width. Only exactly 8 selects 8-bit storage.
    pub bits: i32,
    /// Linear gain applied to each sample before flooring.
    pub amplitude_scale: f64,
    /// One output channel per input channel when true, else mix to one.
    pub split_channels: bool,
    /// Number of audio samples per channel.
    pub length: i32,
    /// Audio samples per second, written to the header.
    pub sample_rate: i32,
    /// PCM data, one slice per input channel.
    pub channels: &'a [&'a [f32]],
}

fn calculate_waveform_data_length(audio_sample_count: i32, scale: i32) -> i32 {
    let mut data_length = (audio_sample_count as f64 / scale as f64).floor() as i32;
    let samples_remaining = audio_sample_count - data_length * scale;
    if samples_remaining > 0 {
        data_length += 1;
    }
    data_length
}

/// Generates a version-2 binary waveform buffer from PCM samples.
///
/// For each block of `scale` consecutive samples, this records the per-channel
/// `(min, max)` of `floor(range_max * value * amplitude_scale)` (divided by the
/// input channel count in mixdown), clamped to the sample range. A trailing
/// partial block is flushed with whatever accumulated.
///
/// Pass the result to [`WaveformData::from_binary`](crate::WaveformData::from_binary).
pub fn generate_waveform_data(options: &GenerateOptions<'_>) -> Vec<u8> {
    let scale = options.scale;
    let amplitude_scale = options.amplitude_scale;
    let length = options.length;
    let sample_rate = options.sample_rate;
    let channels = options.channels;
    let input_channels = channels.len();
    let output_channels: usize = if options.split_channels {
        input_channels
    } else {
        1
    };
    let header_size = 24usize;
    let data_length = calculate_waveform_data_length(length, scale);
    let eight_bit = options.bits == 8;
    let bytes_per_sample = if eight_bit { 1usize } else { 2 };
    let total_size = header_size + data_length as usize * 2 * bytes_per_sample * output_channels;
    let mut buffer = vec![0u8; total_size];

    let mut scale_counter = 0i32;
    let mut offset = header_size;

    let mut min_value = vec![f64::INFINITY; output_channels];
    let mut max_value = vec![f64::NEG_INFINITY; output_channels];

    let range_min = if eight_bit { INT8_MIN } else { INT16_MIN };
    let range_max = if eight_bit { INT8_MAX } else { INT16_MAX };

    write_i32_le(&mut buffer, 0, 2);
    write_u32_le(&mut buffer, 4, if eight_bit { 1 } else { 0 });
    write_i32_le(&mut buffer, 8, sample_rate);
    write_i32_le(&mut buffer, 12, scale);
    write_i32_le(&mut buffer, 16, data_length);
    write_i32_le(&mut buffer, 20, output_channels as i32);

    for i in 0..length as usize {
        if output_channels == 1 {
            let mut sample = 0.0f64;
            for channel in channels.iter() {
                sample += channel[i] as f64;
            }
            let sample = (range_max * sample * amplitude_scale / input_channels as f64).floor();

            if sample < min_value[0] {
                min_value[0] = sample;
                if min_value[0] < range_min {
                    min_value[0] = range_min;
                }
            }
            if sample > max_value[0] {
                max_value[0] = sample;
                if max_value[0] > range_max {
                    max_value[0] = range_max;
                }
            }
        } else {
            for channel in 0..output_channels {
                let sample = (range_max * channels[channel][i] as f64 * amplitude_scale).floor();
                if sample < min_value[channel] {
                    min_value[channel] = sample;
                    if min_value[channel] < range_min {
                        min_value[channel] = range_min;
                    }
                }
                if sample > max_value[channel] {
                    max_value[channel] = sample;
                    if max_value[channel] > range_max {
                        max_value[channel] = range_max;
                    }
                }
            }
        }

        scale_counter += 1;
        if scale_counter == scale {
            for channel in 0..output_channels {
                write_block(
                    &mut buffer,
                    &mut offset,
                    eight_bit,
                    min_value[channel],
                    max_value[channel],
                );
                min_value[channel] = f64::INFINITY;
                max_value[channel] = f64::NEG_INFINITY;
            }
            scale_counter = 0;
        }
    }

    if scale_counter > 0 {
        for channel in 0..output_channels {
            write_block(
                &mut buffer,
                &mut offset,
                eight_bit,
                min_value[channel],
                max_value[channel],
            );
        }
    }

    buffer
}

fn write_block(buffer: &mut [u8], offset: &mut usize, eight_bit: bool, min: f64, max: f64) {
    if eight_bit {
        buffer[*offset] = min as i64 as i8 as u8;
        buffer[*offset + 1] = max as i64 as i8 as u8;
        *offset += 2;
    } else {
        let min16 = (min as i64 as i16).to_le_bytes();
        let max16 = (max as i64 as i16).to_le_bytes();
        buffer[*offset..*offset + 2].copy_from_slice(&min16);
        buffer[*offset + 2..*offset + 4].copy_from_slice(&max16);
        *offset += 4;
    }
}
