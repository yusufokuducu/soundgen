//! Effect blocks: DC blocker, saturator, compressor, reverb, limiter, width.

use crate::db_to_gain;

#[derive(Clone, Debug, Default)]
pub struct DcBlocker {
    x1: f32,
    y1: f32,
}

impl DcBlocker {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + 0.995 * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

#[derive(Clone, Debug)]
pub struct Saturator {
    pub drive: f32,
    pub asym: f32,
    pub mix: f32,
}

impl Default for Saturator {
    fn default() -> Self {
        Self {
            drive: 1.0,
            asym: 0.0,
            mix: 1.0,
        }
    }
}

impl Saturator {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let d = self.drive.max(1e-4);
        let shaped = if self.asym.abs() > 1e-4 {
            // Asymmetric tanh: adds even harmonics (grit, tube-like warmth).
            // Subtract the curve's own value at x = 0 so the stage is DC-free;
            // a partial subtraction used to stamp ~0.06 of DC onto every sound,
            // which the master DC blocker then turned into a low thump.
            let bias = self.asym * 0.35;
            ((x + bias) * d).tanh() - (bias * d).tanh()
        } else {
            (x * d).tanh()
        };
        let comp = 1.0 / d.sqrt().max(1.0);
        x * (1.0 - self.mix) + shaped * comp * self.mix
    }
}

/// Stereo-linked feed-forward compressor.
#[derive(Clone, Debug)]
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    env: f32,
    sr: f32,
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        Self {
            threshold: 0.7,
            ratio: 3.0,
            attack: (-1.0 / (0.005 * sr)).exp(),
            release: (-1.0 / (0.120 * sr)).exp(),
            makeup: 1.0,
            env: 0.0,
            sr,
        }
    }

    pub fn configure(&mut self, threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32) {
        self.threshold = db_to_gain(threshold_db);
        self.ratio = ratio.max(1.0);
        self.attack = (-1.0 / (attack_ms.max(0.05) * 0.001 * self.sr)).exp();
        self.release = (-1.0 / (release_ms.max(1.0) * 0.001 * self.sr)).exp();
    }

    pub fn set_makeup_db(&mut self, db: f32) {
        self.makeup = db_to_gain(db);
    }

    #[inline]
    pub fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        let level = l.abs().max(r.abs());
        let coef = if level > self.env {
            self.attack
        } else {
            self.release
        };
        self.env = level + (self.env - level) * coef;

        let thresh = self.threshold.max(1e-6);
        let gain = if self.env > thresh {
            let over = self.env / thresh;
            let target = over.powf(1.0 / self.ratio - 1.0);
            target
        } else {
            1.0
        };
        (l * gain * self.makeup, r * gain * self.makeup)
    }

    pub fn reset(&mut self) {
        self.env = 0.0;
    }
}

struct Comb {
    buf: Vec<f32>,
    idx: usize,
    store: f32,
    damp1: f32,
    damp2: f32,
    feedback: f32,
}

impl Comb {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len.max(1)],
            idx: 0,
            store: 0.0,
            damp1: 0.2,
            damp2: 0.8,
            feedback: 0.8,
        }
    }
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let out = self.buf[self.idx];
        self.store = out * self.damp2 + self.store * self.damp1;
        self.buf[self.idx] = x + self.store * self.feedback;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        out
    }
}

struct Allpass {
    buf: Vec<f32>,
    idx: usize,
}

impl Allpass {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len.max(1)],
            idx: 0,
        }
    }
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let bufout = self.buf[self.idx];
        let out = -x + bufout;
        self.buf[self.idx] = x + bufout * 0.5;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        out
    }
}

const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;

/// Compact Freeverb-style stereo reverb, mono in / stereo out.
pub struct Reverb {
    combs_l: Vec<Comb>,
    combs_r: Vec<Comb>,
    aps_l: Vec<Allpass>,
    aps_r: Vec<Allpass>,
}

impl Reverb {
    pub fn new(sr: f32) -> Self {
        let scale = sr / 44_100.0;
        let combs_l = COMB_TUNING
            .iter()
            .map(|&l| Comb::new((l as f32 * scale) as usize))
            .collect();
        let combs_r = COMB_TUNING
            .iter()
            .map(|&l| Comb::new(((l + STEREO_SPREAD) as f32 * scale) as usize))
            .collect();
        let aps_l = ALLPASS_TUNING
            .iter()
            .map(|&l| Allpass::new((l as f32 * scale) as usize))
            .collect();
        let aps_r = ALLPASS_TUNING
            .iter()
            .map(|&l| Allpass::new(((l + STEREO_SPREAD) as f32 * scale) as usize))
            .collect();
        Self {
            combs_l,
            combs_r,
            aps_l,
            aps_r,
        }
    }

    /// `size` 0..1 (room size), `damp` 0..1 (high-frequency damping).
    pub fn configure(&mut self, size: f32, damp: f32) {
        let feedback = 0.72 + size.clamp(0.0, 1.0) * 0.26;
        let d1 = damp.clamp(0.0, 1.0) * 0.4;
        for c in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            c.feedback = feedback;
            c.damp1 = d1;
            c.damp2 = 1.0 - d1;
        }
    }

    pub fn reset(&mut self) {
        for c in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            c.buf.fill(0.0);
            c.store = 0.0;
        }
        for a in self.aps_l.iter_mut().chain(self.aps_r.iter_mut()) {
            a.buf.fill(0.0);
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> (f32, f32) {
        let mut l = 0.0;
        let mut r = 0.0;
        for c in self.combs_l.iter_mut() {
            l += c.process(x);
        }
        for c in self.combs_r.iter_mut() {
            r += c.process(x);
        }
        for a in self.aps_l.iter_mut() {
            l = a.process(l);
        }
        for a in self.aps_r.iter_mut() {
            r = a.process(r);
        }
        (l * 0.09, r * 0.09)
    }
}

/// Transparent brickwall-ish limiter with a soft knee.
#[derive(Clone, Debug)]
pub struct Limiter {
    ceiling: f32,
    env: f32,
    attack: f32,
    release: f32,
}

impl Limiter {
    pub fn new(sr: f32, ceiling_db: f32) -> Self {
        Self {
            ceiling: db_to_gain(ceiling_db),
            env: 0.0,
            attack: (-1.0 / (0.0008 * sr)).exp(),
            release: (-1.0 / (0.080 * sr)).exp(),
        }
    }

    pub fn reset(&mut self) {
        self.env = 0.0;
    }

    #[inline]
    pub fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        let peak = l.abs().max(r.abs());
        let coef = if peak > self.env {
            self.attack
        } else {
            self.release
        };
        self.env = peak + (self.env - peak) * coef;
        if self.env > self.ceiling && self.env > 1e-9 {
            let g = self.ceiling / self.env;
            let gl = (l * g).clamp(-1.0, 1.0);
            let gr = (r * g).clamp(-1.0, 1.0);
            (gl, gr)
        } else {
            (l.clamp(-1.0, 1.0), r.clamp(-1.0, 1.0))
        }
    }
}

/// Mid/side width control. `width` 1.0 = unchanged, 0.0 = mono, 2.0 = wide.
#[inline]
pub fn widen(l: f32, r: f32, width: f32) -> (f32, f32) {
    let mid = (l + r) * 0.5;
    let side = (l - r) * 0.5 * width;
    (mid + side, mid - side)
}
