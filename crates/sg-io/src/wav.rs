//! WAV writing with game-ready presets (48 kHz, 16/24-bit, mono option).

use anyhow::{Context, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use sg_engine::AudioBuffer;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    pub bits: u16,
    pub mono: bool,
    pub trim_silence: bool,
    pub normalize: bool,
    pub target_peak_db: f32,
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            bits: 16,
            mono: false,
            trim_silence: true,
            normalize: true,
            target_peak_db: -1.0,
            fade_in_ms: 0.5,
            fade_out_ms: 20.0,
        }
    }
}

impl ExportOptions {
    pub fn unity_mobile() -> Self {
        Self {
            bits: 16,
            mono: true,
            ..Default::default()
        }
    }
    pub fn raw() -> Self {
        Self {
            bits: 24,
            mono: false,
            trim_silence: false,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 0.0,
            ..Default::default()
        }
    }
}

pub fn write_wav(path: &Path, buffer: &AudioBuffer, opts: &ExportOptions) -> Result<()> {
    let mut buf = buffer.clone();
    if opts.trim_silence {
        buf.trim_silence(-72.0, 1.0);
    }
    if opts.fade_in_ms > 0.0 {
        buf.fade_in(opts.fade_in_ms);
    }
    if opts.fade_out_ms > 0.0 {
        buf.fade_out(opts.fade_out_ms);
    }
    if opts.normalize {
        buf.normalize_peak(opts.target_peak_db);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create dir {parent:?}"))?;
    }

    let channels = if opts.mono { 1 } else { 2 };
    let sample_format = if opts.bits <= 16 {
        SampleFormat::Int
    } else {
        SampleFormat::Int
    };
    let spec = WavSpec {
        channels,
        sample_rate: buf.sample_rate,
        bits_per_sample: opts.bits,
        sample_format,
    };
    let mut writer = WavWriter::create(path, spec).with_context(|| format!("create {path:?}"))?;

    match opts.bits {
        16 => {
            for frame in 0..buf.frames() {
                let l = buf.samples[frame * 2];
                let r = buf.samples[frame * 2 + 1];
                if opts.mono {
                    writer.write_sample(to_i16((l + r) * 0.5))?;
                } else {
                    writer.write_sample(to_i16(l))?;
                    writer.write_sample(to_i16(r))?;
                }
            }
        }
        24 => {
            for frame in 0..buf.frames() {
                let l = buf.samples[frame * 2];
                let r = buf.samples[frame * 2 + 1];
                if opts.mono {
                    writer.write_sample(to_i32_24((l + r) * 0.5))?;
                } else {
                    writer.write_sample(to_i32_24(l))?;
                    writer.write_sample(to_i32_24(r))?;
                }
            }
        }
        _ => {
            for frame in 0..buf.frames() {
                let l = buf.samples[frame * 2];
                let r = buf.samples[frame * 2 + 1];
                if opts.mono {
                    writer.write_sample((l + r) * 0.5)?;
                } else {
                    writer.write_sample(l)?;
                    writer.write_sample(r)?;
                }
            }
        }
    }
    writer.finalize().context("finalize wav")?;
    Ok(())
}

#[inline]
fn to_i16(x: f32) -> i16 {
    (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

#[inline]
fn to_i32_24(x: f32) -> i32 {
    (x.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32
}
