# waveform-data

Read, write, and reshape precomputed audio waveform peak data in the BBC
`audiowaveform` formats.

A waveform holds `length` data points. Each point carries a `(min, max)` pair
per channel summarizing `scale` consecutive audio samples. This crate parses
the binary `.dat` layout (versions 1 and 2) and the matching JSON object,
reads header and sample values, reshapes with resample, concat, and slice, and
builds peaks from PCM.

## Installation

```toml
[dependencies]
waveform-data = "0.1"
```

## Usage

Load from a binary buffer or a JSON object:

```rust
use waveform_data::{JsonWaveformData, WaveformData};

let json = JsonWaveformData {
    version: None,
    channels: None,
    sample_rate: 48000,
    samples_per_pixel: 512,
    bits: 8,
    length: 2,
    data: vec![0, 0, -10, 10],
};

let waveform = WaveformData::from_json(&json).unwrap();
assert_eq!(waveform.length(), 2);
assert_eq!(waveform.duration(), 2.0 * 512.0 / 48000.0);
assert_eq!(waveform.channel(0).unwrap().min_array(), vec![0, -10]);
```

Read fields with `sample_rate`, `scale`, `bits`, `length`, `channels`,
`duration`, `pixels_per_second`, and `seconds_per_pixel`. Read samples through
`channel`. Reshape with `resample`, `concat`, and `slice`. Serialize with
`to_json` and `as_bytes`.

## Peak generation

`generate_waveform_data` builds a version-2 buffer from PCM channels. It mixes
down to one channel or splits per channel, and clamps to the int8 or int16
range.

```rust
use waveform_data::{generate_waveform_data, GenerateOptions, WaveformData};

let left = vec![0.0f32; 1024];
let right = vec![0.5f32; 1024];
let channels: Vec<&[f32]> = vec![&left, &right];

let buffer = generate_waveform_data(&GenerateOptions {
    scale: 512,
    bits: 8,
    amplitude_scale: 1.0,
    split_channels: false,
    length: 1024,
    sample_rate: 44100,
    channels: &channels,
});

let waveform = WaveformData::from_binary(buffer).unwrap();
assert_eq!(waveform.length(), 2);
```

## Format

All multi-byte header fields are little-endian. Version 1 has a 20-byte header
and a single implicit channel. Version 2 has a 24-byte header with an explicit
channel count. The bits flag at offset 4 stores 1 for 8-bit and 0 for 16-bit.
Samples are signed `int8` or `int16`, interleaved per data point, per channel,
as `(min, max)` pairs.

## License

Licensed under the [MIT license](LICENSE).
