//! Low-latency playback of rendered buffers.
//!
//! Strategy: SoundGen renders the whole effect offline (a few ms) and hands the
//! finished stereo buffer to this player. That keeps the audio callback trivial
//! and click-free, and avoids a real-time graph entirely.

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};
use sg_engine::AudioBuffer;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct PlayState {
    buffer: Option<Arc<AudioBuffer>>,
    /// Fractional read position in source frames.
    pos: f64,
    looped: bool,
}

pub struct Player {
    _stream: cpal::Stream,
    state: Arc<Mutex<PlayState>>,
    playing: Arc<AtomicBool>,
    volume_bits: Arc<AtomicU32>,
    device_name: String,
    output_rate: u32,
    channels: u16,
}

impl Player {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("no default output device"))?;
        let device_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "unknown".into());
        let supported = device
            .default_output_config()
            .context("default output config")?;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let output_rate = config.sample_rate;
        let channels = config.channels;

        let state = Arc::new(Mutex::new(PlayState::default()));
        let playing = Arc::new(AtomicBool::new(false));
        let volume = Arc::new(AtomicU32::new(0.9f32.to_bits()));

        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config,
                output_rate,
                state.clone(),
                playing.clone(),
                volume.clone(),
            )?,
            cpal::SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config,
                output_rate,
                state.clone(),
                playing.clone(),
                volume.clone(),
            )?,
            cpal::SampleFormat::U16 => build_stream::<u16>(
                &device,
                &config,
                output_rate,
                state.clone(),
                playing.clone(),
                volume.clone(),
            )?,
            other => return Err(anyhow!("unsupported sample format {other:?}")),
        };
        stream.play().context("start stream")?;

        Ok(Self {
            _stream: stream,
            state,
            playing,
            volume_bits: volume,
            device_name,
            output_rate,
            channels,
        })
    }

    pub fn play(&self, buffer: Arc<AudioBuffer>, looped: bool) {
        if let Ok(mut s) = self.state.lock() {
            s.buffer = Some(buffer);
            s.pos = 0.0;
            s.looped = looped;
        }
        self.playing.store(true, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.playing.store(false, Ordering::Relaxed);
        if let Ok(mut s) = self.state.lock() {
            s.pos = 0.0;
            s.buffer = None;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, v: f32) {
        self.volume_bits
            .store(v.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn output_rate(&self) -> u32 {
        self.output_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Playback position in seconds, for the UI playhead.
    pub fn position_seconds(&self) -> f32 {
        if let Ok(s) = self.state.lock() {
            if let Some(buf) = &s.buffer {
                return (s.pos / buf.sample_rate as f64) as f32;
            }
        }
        0.0
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    device_rate: u32,
    state: Arc<Mutex<PlayState>>,
    playing: Arc<AtomicBool>,
    volume: Arc<AtomicU32>,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32> + Sample,
{
    let out_channels = config.channels as usize;
    let err_fn = |err| eprintln!("audio stream error: {err}");
    let stream = device
        .build_output_stream(
            *config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let vol = f32::from_bits(volume.load(Ordering::Relaxed));
                let silent = |data: &mut [T]| {
                    for s in data.iter_mut() {
                        *s = T::from_sample(0.0f32);
                    }
                };
                if !playing.load(Ordering::Relaxed) {
                    silent(data);
                    return;
                }
                let Ok(mut st) = state.try_lock() else {
                    silent(data);
                    return;
                };
                let Some(buf) = st.buffer.clone() else {
                    silent(data);
                    return;
                };
                let frames = buf.frames();
                if frames == 0 {
                    silent(data);
                    return;
                }
                let advance = buf.sample_rate as f64 / device_rate.max(1) as f64;

                for frame in data.chunks_mut(out_channels) {
                    if st.pos >= frames as f64 {
                        if st.looped {
                            st.pos -= frames as f64;
                        } else {
                            playing.store(false, Ordering::Relaxed);
                            for s in frame.iter_mut() {
                                *s = T::from_sample(0.0f32);
                            }
                            continue;
                        }
                    }
                    let i = st.pos.floor() as usize;
                    let frac = (st.pos - i as f64) as f32;
                    let i2 = (i + 1).min(frames - 1);
                    let l = buf.samples[i * 2] * (1.0 - frac) + buf.samples[i2 * 2] * frac;
                    let r = buf.samples[i * 2 + 1] * (1.0 - frac) + buf.samples[i2 * 2 + 1] * frac;
                    if out_channels == 1 {
                        frame[0] = T::from_sample(((l + r) * 0.5 * vol).clamp(-1.0, 1.0));
                    } else {
                        frame[0] = T::from_sample((l * vol).clamp(-1.0, 1.0));
                        frame[1] = T::from_sample((r * vol).clamp(-1.0, 1.0));
                        for ch in frame.iter_mut().skip(2) {
                            *ch = T::from_sample(0.0f32);
                        }
                    }
                    st.pos += advance;
                }
            },
            err_fn,
            None,
        )
        .context("build output stream")?;
    Ok(stream)
}

