//! Peak generation goldens from a real 4-channel recording.
//!
//! The source is a 44100 Hz, 4-channel, 88200-sample WAV. Decoding it to f32
//! and running the generator reproduces the published peak values.

mod common;

use common::{decode_wav, fixture_path, DecodedPcm};
use waveform_data::{generate_waveform_data, GenerateOptions, WaveformData};

fn load_pcm() -> DecodedPcm {
    let bytes = std::fs::read(fixture_path("4channel.wav")).unwrap();
    decode_wav(&bytes)
}

fn channel_refs(pcm: &DecodedPcm) -> Vec<&[f32]> {
    pcm.channels.iter().map(|c| c.as_slice()).collect()
}

#[test]
fn default_mono_mix() {
    let pcm = load_pcm();
    let channels = channel_refs(&pcm);
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 8,
        amplitude_scale: 1.0,
        split_channels: false,
        length: pcm.length,
        sample_rate: pcm.sample_rate,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(wf.channels(), 1);
    assert_eq!(wf.bits(), 8);
    // 88200 / 512 = 172 remainder 136 -> 173 points.
    assert_eq!(wf.length(), 173);

    let ch = wf.channel(0).unwrap();
    assert_eq!(ch.min_sample(0).unwrap(), -23);
    assert_eq!(ch.max_sample(0).unwrap(), 22);
    let last = wf.length() as i32 - 1;
    assert_eq!(ch.min_sample(last).unwrap(), -23);
    assert_eq!(ch.max_sample(last).unwrap(), 22);
}

#[test]
fn scale_128_length() {
    let pcm = load_pcm();
    let channels = channel_refs(&pcm);
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 128,
        bits: 8,
        amplitude_scale: 1.0,
        split_channels: false,
        length: pcm.length,
        sample_rate: pcm.sample_rate,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    // 88200 / 128 = 689 remainder 8 -> 690 points.
    assert_eq!(wf.length(), 690);
}

#[test]
fn amplitude_scale_doubles_peaks() {
    let pcm = load_pcm();
    let channels = channel_refs(&pcm);
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 8,
        amplitude_scale: 2.0,
        split_channels: false,
        length: pcm.length,
        sample_rate: pcm.sample_rate,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    let ch = wf.channel(0).unwrap();
    assert_eq!(ch.min_sample(0).unwrap(), -45);
    assert_eq!(ch.max_sample(0).unwrap(), 44);
    let last = wf.length() as i32 - 1;
    assert_eq!(ch.min_sample(last).unwrap(), -45);
    assert_eq!(ch.max_sample(last).unwrap(), 44);
}

#[test]
fn split_channels() {
    let pcm = load_pcm();
    let channels = channel_refs(&pcm);
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 8,
        amplitude_scale: 1.0,
        split_channels: true,
        length: pcm.length,
        sample_rate: pcm.sample_rate,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(wf.channels(), 4);

    let expected = [(-1, 0), (-1, 0), (-90, 89), (-1, 0)];
    let last = wf.length() as i32 - 1;
    for (c, (min, max)) in expected.iter().enumerate() {
        let ch = wf.channel(c as i32).unwrap();
        assert_eq!(ch.min_sample(0).unwrap(), *min);
        assert_eq!(ch.max_sample(0).unwrap(), *max);
        assert_eq!(ch.min_sample(last).unwrap(), *min);
        assert_eq!(ch.max_sample(last).unwrap(), *max);
    }
}

#[test]
fn sixteen_bit_length() {
    let pcm = load_pcm();
    let channels = channel_refs(&pcm);
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 16,
        amplitude_scale: 1.0,
        split_channels: false,
        length: pcm.length,
        sample_rate: pcm.sample_rate,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    assert_eq!(wf.channels(), 1);
    assert_eq!(wf.bits(), 16);
    assert_eq!(wf.length(), 173);
}

#[test]
fn clamp_is_asymmetric_8bit() {
    // The min accumulator clamps only when it drops below range_min, the max
    // accumulator only when it rises above range_max. A high positive value
    // becomes both running extremes on the first sample: max clamps to 127, min
    // keeps the raw 508 and wraps to -4 on write (508 as i8).
    let samples = vec![1.0f32; 1024];
    let channels: Vec<&[f32]> = vec![&samples];
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 8,
        amplitude_scale: 4.0,
        split_channels: false,
        length: 1024,
        sample_rate: 44100,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    let ch = wf.channel(0).unwrap();
    // floor(127 * 1 * 4) = 508. max clamps to 127. min stays 508 -> wraps to -4.
    assert_eq!(ch.max_sample(0).unwrap(), 127);
    assert_eq!(ch.min_sample(0).unwrap(), -4);
}

#[test]
fn clamp_is_asymmetric_16bit() {
    // A high negative value: min clamps to -32768, max keeps the raw -131068 and
    // wraps to 4 on write (-131068 as i16).
    let samples = vec![-1.0f32; 1024];
    let channels: Vec<&[f32]> = vec![&samples];
    let buffer = generate_waveform_data(&GenerateOptions {
        scale: 512,
        bits: 16,
        amplitude_scale: 4.0,
        split_channels: false,
        length: 1024,
        sample_rate: 44100,
        channels: &channels,
    });
    let wf = WaveformData::from_binary(buffer).unwrap();
    let ch = wf.channel(0).unwrap();
    // floor(32767 * -1 * 4) = -131068. min clamps to -32768. max stays -131068 -> wraps to 4.
    assert_eq!(ch.min_sample(0).unwrap(), -32768);
    assert_eq!(ch.max_sample(0).unwrap(), 4);
}
