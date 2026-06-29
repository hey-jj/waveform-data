//! Shared fixtures and helpers for the conformance tests.
//!
//! `json_data` and `binary_data` build the toy length-10 waveform in both
//! layouts, matching the values the format readers must agree on.

#![allow(dead_code)]

use waveform_data::{JsonWaveformData, WaveformData};

/// Input layout to exercise: a JSON object or a binary buffer.
#[derive(Clone, Copy, Debug)]
pub enum Format {
    /// JSON object input.
    Json,
    /// Binary `.dat` buffer input.
    Binary,
}

/// Toy single- or two-channel waveform values, by channel count.
///
/// Single channel: 20 interleaved `(min, max)` values.
/// Two channels: 40 values, per channel per point.
fn sample_data(channels: i32) -> Vec<i32> {
    if channels == 1 {
        vec![
            0, 0, -10, 10, 0, 0, -5, 7, -5, 7, 0, 0, 0, 0, 0, 0, 0, 0, -2, 2,
        ]
    } else {
        vec![
            0, 0, 0, 0, -10, 10, -8, 8, 0, 0, -2, 2, -5, 7, -6, 3, -5, 7, -6, 3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -2, 2, -3, 3,
        ]
    }
}

/// Builds the JSON fixture for the given channel count and bits.
///
/// Single-channel fixtures omit version and channels, matching version-1 JSON.
pub fn json_data(channels: i32, bits: i32) -> JsonWaveformData {
    let data = sample_data(channels);
    if channels == 1 {
        JsonWaveformData {
            version: None,
            channels: None,
            sample_rate: 48000,
            samples_per_pixel: 512,
            bits,
            length: 10,
            data,
        }
    } else {
        JsonWaveformData {
            version: Some(2),
            channels: Some(channels),
            sample_rate: 48000,
            samples_per_pixel: 512,
            bits,
            length: 10,
            data,
        }
    }
}

/// Builds the binary fixture for the given channel count and bits.
///
/// Single channel defaults to version 1 (20-byte header), two channels to
/// version 2 (24-byte header). Pass an explicit `version` to override.
pub fn binary_data(channels: i32, bits: i32, version_override: Option<i32>) -> Vec<u8> {
    let data = sample_data(channels);
    let version = version_override.unwrap_or(if channels == 1 { 1 } else { 2 });
    let header_size = if version == 2 { 24usize } else { 20 };
    let eight_bit = bits == 8;
    let bytes_per_sample = if eight_bit { 1usize } else { 2 };
    let total = header_size + data.len() * bytes_per_sample;
    let mut buffer = vec![0u8; total];

    put_i32(&mut buffer, 0, version);
    put_u32(&mut buffer, 4, if eight_bit { 1 } else { 0 });
    put_i32(&mut buffer, 8, 48000);
    put_i32(&mut buffer, 12, 512);
    put_u32(
        &mut buffer,
        16,
        (data.len() / (2 * channels as usize)) as u32,
    );
    if version == 2 {
        put_i32(&mut buffer, 20, channels);
    }

    let mut index = header_size;
    if eight_bit {
        for &value in &data {
            buffer[index] = value as i8 as u8;
            index += 1;
        }
    } else {
        for &value in &data {
            let bytes = (value as i16).to_le_bytes();
            buffer[index] = bytes[0];
            buffer[index + 1] = bytes[1];
            index += 2;
        }
    }

    buffer
}

/// Builds a [`WaveformData`] from the toy fixture in the given layout.
///
/// Drives the `[binary, json] x [8, 16] bits` matrix the conformance tests run.
pub fn make_data(format: Format, channels: i32, bits: i32) -> WaveformData {
    match format {
        Format::Json => WaveformData::from_json(&json_data(channels, bits)).unwrap(),
        Format::Binary => WaveformData::from_binary(binary_data(channels, bits, None)).unwrap(),
    }
}

fn put_i32(buffer: &mut [u8], offset: usize, value: i32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Path to a copied binary fixture file under `tests/fixtures/`.
pub fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Parses the flat demo JSON object into a [`JsonWaveformData`].
///
/// The demo file is a single object with scalar header fields and one integer
/// `data` array. This reads those fields without a JSON dependency.
pub fn parse_demo_json(text: &str) -> JsonWaveformData {
    let version = scalar_field(text, "version");
    let channels = scalar_field(text, "channels");
    let sample_rate = scalar_field(text, "sample_rate").expect("sample_rate present");
    let samples_per_pixel = scalar_field(text, "samples_per_pixel").expect("samples_per_pixel");
    let bits = scalar_field(text, "bits").expect("bits present");
    let length = scalar_field(text, "length").expect("length present");
    let data = array_field(text, "data");

    JsonWaveformData {
        version,
        channels,
        sample_rate,
        samples_per_pixel,
        bits,
        length,
        data,
    }
}

/// Decoded PCM: one f32 buffer per channel, values in `[-1, 1)`.
pub struct DecodedPcm {
    /// Samples per second.
    pub sample_rate: i32,
    /// Frames per channel.
    pub length: i32,
    /// Planar channel data.
    pub channels: Vec<Vec<f32>>,
}

/// Decodes a 16-bit integer PCM WAV file into normalized f32 channels.
///
/// Walks the RIFF chunks, reads the `fmt ` and `data` chunks, and normalizes
/// each `i16` sample by 32768. This matches how the Web Audio API presents
/// 16-bit PCM, so the generator goldens line up.
pub fn decode_wav(bytes: &[u8]) -> DecodedPcm {
    assert_eq!(&bytes[0..4], b"RIFF", "RIFF header");
    assert_eq!(&bytes[8..12], b"WAVE", "WAVE form");

    let mut channels_count = 0u16;
    let mut sample_rate = 0u32;
    let mut bits_per_sample = 0u16;
    let mut data: &[u8] = &[];

    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        let body = pos + 8;
        match id {
            b"fmt " => {
                channels_count = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]);
                sample_rate = u32::from_le_bytes([
                    bytes[body + 4],
                    bytes[body + 5],
                    bytes[body + 6],
                    bytes[body + 7],
                ]);
                bits_per_sample = u16::from_le_bytes([bytes[body + 14], bytes[body + 15]]);
            }
            b"data" => {
                data = &bytes[body..body + size];
            }
            _ => {}
        }
        // Chunks are padded to even sizes.
        pos = body + size + (size & 1);
    }

    assert_eq!(bits_per_sample, 16, "expected 16-bit PCM");
    let channels = channels_count as usize;
    let frames = data.len() / 2 / channels;
    let mut buffers = vec![Vec::with_capacity(frames); channels];
    for frame in 0..frames {
        for (ch, buffer) in buffers.iter_mut().enumerate() {
            let off = (frame * channels + ch) * 2;
            let sample = i16::from_le_bytes([data[off], data[off + 1]]);
            buffer.push(sample as f32 / 32768.0);
        }
    }

    DecodedPcm {
        sample_rate: sample_rate as i32,
        length: frames as i32,
        channels: buffers,
    }
}

fn scalar_field(text: &str, key: &str) -> Option<i32> {
    let needle = format!("\"{key}\":");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    rest[..end].trim().parse::<i32>().ok()
}

fn array_field(text: &str, key: &str) -> Vec<i32> {
    let needle = format!("\"{key}\":[");
    let start = text.find(&needle).expect("array key present") + needle.len();
    let rest = &text[start..];
    let end = rest.find(']').expect("array close");
    rest[..end]
        .split(',')
        .map(|piece| piece.trim().parse::<i32>().expect("integer element"))
        .collect()
}
