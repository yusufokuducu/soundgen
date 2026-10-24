//! Band-limited-ish oscillators with per-sample frequency control.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveShape {
    Sine,
    Triangle,
    Saw,
    Square,
}

#[derive(Clone, Debug)]
pub struct Osc {
    phase: f32,
    sr: f32,
}

impl Osc {
    pub fn new(sr: f32) -> Self {
        Self { phase: 0.0, sr }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn set_phase(&mut self, p: f32) {
        self.phase = p.rem_euclid(1.0);
    }

    #[inline]
    fn advance(&mut self, freq: f32) -> (f32, f32) {
        let t = self.phase;
        let dt = (freq / self.sr).clamp(0.0, 0.5);
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        (t, dt)
    }

    #[inline]
    pub fn next(&mut self, freq: f32, shape: WaveShape) -> f32 {
        let (t, dt) = self.advance(freq);
        match shape {
            WaveShape::Sine => (t * std::f32::consts::TAU).sin(),
            WaveShape::Triangle => 1.0 - 4.0 * (t - 0.5).abs(),
            WaveShape::Saw => 2.0 * t - 1.0 - poly_blep(t, dt),
            WaveShape::Square => {
                let s = if t < 0.5 { 1.0 } else { -1.0 };
                s + poly_blep(t, dt) - poly_blep((t + 0.5).rem_euclid(1.0), dt)
            }
        }
    }
}

#[inline]
pub fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

/// Exponential pitch envelope: `freq(t) = end + (start-end) * exp(-t/tau)`.
#[derive(Clone, Debug)]
pub struct PitchEnv {
    t: f32,
    sr: f32,
    pub start: f32,
    pub end: f32,
    pub tau: f32,
}

impl PitchEnv {
    pub fn new(sr: f32, start: f32, end: f32, tau: f32) -> Self {
        Self {
            t: 0.0,
            sr,
            start,
            end,
            tau: tau.max(1e-5),
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        let f = self.end + (self.start - self.end) * (-self.t / self.tau).exp();
        self.t += 1.0 / self.sr;
        f
    }

    pub fn value_at(&self, t: f32) -> f32 {
        self.end + (self.start - self.end) * (-t / self.tau).exp()
    }
}
