//! Filters: TPT state-variable filter, RBJ biquad, one-pole smoother.

use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvfMode {
    LowPass,
    BandPass,
    HighPass,
    Notch,
}

/// Topology-preserving-transform state variable filter (Zavalishin / Simper).
/// Stable under fast modulation; cutoff may be swept per sample.
#[derive(Clone, Debug)]
pub struct Svf {
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1: f32,
    ic2: f32,
    sr: f32,
}

impl Svf {
    pub fn new(sr: f32) -> Self {
        let mut s = Self {
            g: 0.0,
            k: 1.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            ic1: 0.0,
            ic2: 0.0,
            sr,
        };
        s.set(1000.0, 0.707);
        s
    }

    /// `cutoff` in Hz, `q` linear (0.5..20).
    #[inline]
    pub fn set(&mut self, cutoff: f32, q: f32) {
        let fc = cutoff.clamp(10.0, self.sr * 0.49);
        self.g = (PI * fc / self.sr).tan();
        self.k = 1.0 / q.clamp(0.05, 40.0);
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }

    pub fn reset(&mut self) {
        self.ic1 = 0.0;
        self.ic2 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32, mode: SvfMode) -> f32 {
        let v3 = x - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        match mode {
            SvfMode::LowPass => v2,
            SvfMode::BandPass => v1,
            SvfMode::HighPass => x - self.k * v1 - v2,
            SvfMode::Notch => x - self.k * v1,
        }
    }
}

/// RBJ biquad (peaking / shelves / high-low pass) for static EQ.
#[derive(Clone, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sr: f32,
}

impl Biquad {
    pub fn new(sr: f32) -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sr,
        }
    }

    pub fn identity(&mut self) {
        self.b0 = 1.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.a1 = 0.0;
        self.a2 = 0.0;
    }

    pub fn set_peak(&mut self, freq: f32, q: f32, gain_db: f32) {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(10.0, self.sr * 0.49) / self.sr;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos = w0.cos();
        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cos) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cos) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    pub fn set_low_shelf(&mut self, freq: f32, gain_db: f32, slope: f32) {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(10.0, self.sr * 0.49) / self.sr;
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / 2.0 * ((a + 1.0 / a) * (1.0 / slope.max(0.01) - 1.0) + 2.0).sqrt();
        let sq = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) + (a - 1.0) * cos + sq;
        self.b0 = a * ((a + 1.0) - (a - 1.0) * cos + sq) / a0;
        self.b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos) / a0;
        self.b2 = a * ((a + 1.0) - (a - 1.0) * cos - sq) / a0;
        self.a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos) / a0;
        self.a2 = ((a + 1.0) + (a - 1.0) * cos - sq) / a0;
    }

    pub fn set_high_shelf(&mut self, freq: f32, gain_db: f32, slope: f32) {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(10.0, self.sr * 0.49) / self.sr;
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / 2.0 * ((a + 1.0 / a) * (1.0 / slope.max(0.01) - 1.0) + 2.0).sqrt();
        let sq = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) - (a - 1.0) * cos + sq;
        self.b0 = a * ((a + 1.0) + (a - 1.0) * cos + sq) / a0;
        self.b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos) / a0;
        self.b2 = a * ((a + 1.0) + (a - 1.0) * cos - sq) / a0;
        self.a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos) / a0;
        self.a2 = ((a + 1.0) - (a - 1.0) * cos - sq) / a0;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// One-pole parameter smoother, click-free knob movement.
#[derive(Clone, Debug)]
pub struct Smoother {
    current: f32,
    target: f32,
    coef: f32,
}

impl Smoother {
    pub fn new(initial: f32, time_sec: f32, sr: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            coef: (-1.0 / (time_sec.max(0.0001) * sr)).exp(),
        }
    }

    pub fn set_target(&mut self, v: f32) {
        self.target = v;
    }

    pub fn snap(&mut self, v: f32) {
        self.current = v;
        self.target = v;
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        self.current = self.target + (self.current - self.target) * self.coef;
        self.current
    }

    pub fn value(&self) -> f32 {
        self.current
    }
}
