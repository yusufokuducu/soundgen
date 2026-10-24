//! sg-dsp — allocation-free DSP primitives for SoundGen.
//!
//! Every block here is designed to run either in a fast offline bounce
//! (the primary path) or in a real-time callback. No global state, no
//! randomness unless a `Rng` is passed in, deterministic for a given seed.

pub mod env;
pub mod filter;
pub mod fx;
pub mod modal;
pub mod noise;
pub mod osc;
pub mod rng;

pub const TAU: f32 = std::f32::consts::TAU;

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db * 0.05)
}

#[inline]
pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.abs().max(1e-9).log10()
}

/// Equal-power pan. `pan` in [-1, 1]; returns (left, right).
#[inline]
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let p = (pan.clamp(-1.0, 1.0) + 1.0) * 0.25 * std::f32::consts::PI;
    (p.cos(), p.sin())
}

/// Soft clipper with a musical knee; hard-clips outside [-1, 1].
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    if x <= -3.0 {
        -1.0
    } else if x >= 3.0 {
        1.0
    } else {
        x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
    }
}
