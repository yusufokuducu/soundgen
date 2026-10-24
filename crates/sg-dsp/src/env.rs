//! Envelopes: attack/hold/decay with curvature, plus helpers.

/// Attack / hold / decay envelope description (seconds, except `curve`).
#[derive(Clone, Copy, Debug)]
pub struct AdEnv {
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    /// 1.0 = linear decay, >1 = fast initial drop (percussive), <1 = slow start.
    pub curve: f32,
    /// Attack curvature; 1.0 = linear ramp.
    pub attack_curve: f32,
}

impl Default for AdEnv {
    fn default() -> Self {
        Self {
            attack: 0.001,
            hold: 0.0,
            decay: 0.3,
            curve: 2.0,
            attack_curve: 1.0,
        }
    }
}

impl AdEnv {
    pub fn perc(decay: f32) -> Self {
        Self {
            attack: 0.0005,
            hold: 0.0,
            decay,
            curve: 2.5,
            attack_curve: 1.0,
        }
    }

    pub fn swelled(attack: f32, decay: f32) -> Self {
        Self {
            attack,
            hold: 0.0,
            decay,
            curve: 1.6,
            attack_curve: 0.6,
        }
    }

    pub fn total(&self) -> f32 {
        self.attack + self.hold + self.decay
    }

    pub fn value_at(&self, t: f32) -> f32 {
        if t < 0.0 {
            return 0.0;
        }
        if t < self.attack {
            let x = if self.attack > 0.0 { t / self.attack } else { 1.0 };
            return x.clamp(0.0, 1.0).powf(self.attack_curve.max(0.01));
        }
        let d = t - self.attack - self.hold;
        if d <= 0.0 {
            return 1.0;
        }
        if self.decay <= 0.0 {
            return 0.0;
        }
        let x = (1.0 - d / self.decay).clamp(0.0, 1.0);
        x.powf(self.curve.max(0.01))
    }
}

/// Stateful sample-accurate envelope follower.
#[derive(Clone, Debug)]
pub struct EnvGen {
    sr: f32,
    t: f32,
    pub env: AdEnv,
}

impl EnvGen {
    pub fn new(sr: f32, env: AdEnv) -> Self {
        Self { sr, t: 0.0, env }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        let v = self.env.value_at(self.t);
        self.t += 1.0 / self.sr;
        v
    }

    #[inline]
    pub fn time(&self) -> f32 {
        self.t
    }

    pub fn finished(&self) -> bool {
        self.t > self.env.total()
    }
}

/// Exponential decay coefficient for a given time-constant in seconds.
#[inline]
pub fn exp_coef(tau_sec: f32, sr: f32) -> f32 {
    if tau_sec <= 0.0 {
        0.0
    } else {
        (-1.0 / (tau_sec * sr)).exp()
    }
}
