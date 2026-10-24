//! Modal synthesis: a bank of exponentially decaying sinusoidal resonators.
//! This is the workhorse for wood, metal, plastic, glass and bone impacts.

use crate::rng::Rng;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Mode {
    /// Resonance frequency in Hz.
    pub freq: f32,
    /// Relative amplitude (0..1).
    pub amp: f32,
    /// Time to -60 dB, in seconds.
    pub decay: f32,
}

impl Mode {
    pub fn new(freq: f32, amp: f32, decay: f32) -> Self {
        Self { freq, amp, decay }
    }
}

struct ModeState {
    y1: f32,
    y2: f32,
    a1: f32,
    a2: f32,
    amp: f32,
}

impl ModeState {
    fn new(mode: Mode, sr: f32) -> Self {
        let freq = mode.freq.clamp(20.0, sr * 0.48);
        let w = 2.0 * PI * freq / sr;
        // -60 dB decay time -> per-sample pole radius.
        let r = (-6.907_755 / (mode.decay.max(1e-4) * sr)).exp().clamp(0.0, 0.999_99);
        let a1 = 2.0 * r * w.cos();
        let a2 = -r * r;
        Self {
            y1: 0.0,
            y2: 0.0,
            a1,
            a2,
            amp: mode.amp,
        }
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let y = self.a1 * self.y1 + self.a2 * self.y2 + x;
        self.y2 = self.y1;
        self.y1 = y;
        y * self.amp
    }

    #[inline]
    fn energy(&self) -> f32 {
        self.y1.abs() + self.y2.abs()
    }
}

pub struct ModalBank {
    modes: Vec<ModeState>,
}

impl ModalBank {
    pub fn new(sr: f32, modes: &[Mode]) -> Self {
        Self {
            modes: modes.iter().map(|m| ModeState::new(*m, sr)).collect(),
        }
    }

    pub fn reset(&mut self) {
        for m in &mut self.modes {
            m.y1 = 0.0;
            m.y2 = 0.0;
        }
    }

    #[inline]
    pub fn process(&mut self, excitation: f32) -> f32 {
        let mut out = 0.0;
        for m in &mut self.modes {
            out += m.process(excitation);
        }
        out
    }

    /// Normalised energy, used to trim the render tail.
    pub fn energy(&self) -> f32 {
        self.modes.iter().map(|m| m.energy()).sum()
    }

    pub fn is_silent(&self, eps: f32) -> bool {
        self.energy() < eps
    }
}

/// Generate a physically-plausible inharmonic mode set.
/// `base` = fundamental, `stretch` in (0.5..1.5) controls inharmonicity,
/// `brightness` in (0..1) controls how much energy the upper modes carry.
pub fn inharmonic_modes(base: f32, count: usize, stretch: f32, brightness: f32, decay: f32, rng: &mut Rng) -> Vec<Mode> {
    let mut modes = Vec::with_capacity(count);
    for i in 0..count {
        let n = (i + 1) as f32;
        let mut f = base * n.powf(stretch);
        // Slight per-mode detune keeps it from sounding synthetic.
        f *= rng.range(0.985, 1.015);
        let amp = (1.0 / n.powf(1.6 - brightness * 0.9)).min(1.0);
        // Higher modes die faster: classic material behaviour.
        let d = decay / (1.0 + n * 0.35);
        modes.push(Mode::new(f, amp, d));
    }
    modes
}

