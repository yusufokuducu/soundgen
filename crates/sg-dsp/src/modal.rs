//! Modal synthesis: a bank of exponentially decaying sinusoidal resonators.
//! This is the workhorse for wood, metal, plastic, glass and bone impacts.
//!
//! Two ways in:
//! * `Mode::new(hz, amp, decay)` / `inharmonic_modes` — the old, hand-tuned path.
//! * `table_to_modes(&STEEL, base, ...)` — build the set from a measured
//!   material table (`Material`), which carries the mode ratios, the
//!   frequency-dependent damping law and a strike/listen pickup comb.
//!
//! The tables live at the bottom of this file; see
//! `docs/research/v2/02-modal-malzeme.md` for where the numbers come from.

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
        // A unit impulse into this two-pole resonator leaves a sine whose
        // amplitude is 1/sin(w). Left uncorrected, every mode gets an implicit
        // +6 dB/oct tilt towards the top, which is a large part of why the old
        // bodies sounded synthetic. Scaling by sin(w) makes the requested
        // amplitude mean what it says.
        let amp = mode.amp * w.sin();
        Self {
            y1: 0.0,
            y2: 0.0,
            a1,
            a2,
            amp,
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

// ---------------------------------------------------------------------------
// material tables
// ---------------------------------------------------------------------------

/// One row of a material table: where the mode sits relative to the
/// fundamental, how loud it is, and optionally a hand-set T60.
#[derive(Clone, Copy, Debug)]
pub struct MatMode {
    /// `f_k / f_0`.
    pub ratio: f32,
    /// Linear amplitude, max 1.0, before the pickup comb.
    pub amp: f32,
    /// Absolute T60 in seconds. `0.0` = take it from the damping law
    /// (`t60_0 * ratio^-alpha`); use this for Helmholtz modes, which do not
    /// follow a bending-wave law.
    pub t60: f32,
}

/// Table row with the damping law doing the work.
pub const fn m(ratio: f32, amp: f32) -> MatMode {
    MatMode {
        ratio,
        amp,
        t60: 0.0,
    }
}

/// Table row with an explicit T60 (Helmholtz resonators, air cavities).
pub const fn mt(ratio: f32, amp: f32, t60: f32) -> MatMode {
    MatMode { ratio, amp, t60 }
}

/// Where the object is hit and where it is listened to, as fractions of its
/// length (0 = one end, 1 = the other). Modes with a node at either point are
/// attenuated — this comb is what makes a struck object sound struck instead
/// of sounding like filtered noise.
#[derive(Clone, Copy, Debug)]
pub struct Pickup {
    pub strike: f32,
    pub listen: f32,
    /// 0 = no comb (flesh, cartoon), 1 = full nulls.
    pub depth: f32,
}

impl Pickup {
    pub const fn new(strike: f32, listen: f32, depth: f32) -> Self {
        Self {
            strike,
            listen,
            depth,
        }
    }
    /// Plank, barrel, gun frame.
    pub const fn bar() -> Self {
        Self::new(0.18, 0.31, 0.85)
    }
    /// Sheet metal, ceramic tile, glass shard.
    pub const fn plate() -> Self {
        Self::new(0.22, 0.38, 0.80)
    }
    /// Cylinder shell — walls ring, ends barely move.
    pub const fn shell() -> Self {
        Self::new(0.25, 0.50, 0.70)
    }
    /// Small bucket / can: strong, almost wooden nulls.
    pub const fn bucket() -> Self {
        Self::new(0.30, 0.45, 0.90)
    }
    /// Flesh is a mass, not a bar: no comb.
    pub const fn flesh() -> Self {
        Self::new(0.50, 0.50, 0.15)
    }
    /// Stylised: damp the comb so the body reads as one object.
    pub const fn cartoon() -> Self {
        Self::new(0.50, 0.50, 0.10)
    }
}

/// A material, as measured on a reference-sized object.
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub id: &'static str,
    /// Fundamental at the reference size.
    pub f0_hz: f32,
    /// T60 of the fundamental at the reference size, seconds.
    pub t60_0: f32,
    /// Damping law exponent: `T60_k = t60_0 * ratio_k^-alpha`.
    /// 0.0 makes every mode ring equally long (organ-like, wrong);
    /// 0.5–1.0 is what metal, wood and bone actually do.
    pub alpha: f32,
    pub modes: &'static [MatMode],
    /// How much of the impact is noise rather than modal body.
    pub noise_mix: f32,
    /// Suggested contact time in seconds (see the excitation side).
    pub contact: f32,
    pub pickup: Pickup,
}

/// `|sin(k*pi*x_s) * sin(k*pi*x_l)|`, softened by `depth` so no mode is ever
/// exactly zero — real objects are asymmetric.
#[inline]
pub fn comb(k: usize, strike: f32, listen: f32, depth: f32) -> f32 {
    let a = (PI * k as f32 * strike).sin().abs();
    let b = (PI * k as f32 * listen).sin().abs();
    1.0 - depth + depth * a * b
}

impl Material {
    /// Build a playable mode set for this material at `base_hz`.
    ///
    /// * `base_hz` — frequency of the first mode (so the caller keeps control
    ///   of size, exactly like the old `Mode::new(hz, ..)` tables).
    /// * `decay_scale` — multiplies the table's T60; pass
    ///   `wanted_t60 / mat.t60_0` to keep old preset tunings.
    /// * `brightness` — 0.5 neutral, <0.5 darker, >0.5 brighter.
    pub fn modes_at(
        &self,
        base_hz: f32,
        decay_scale: f32,
        brightness: f32,
    ) -> Vec<Mode> {
        self.modes_at_pickup(base_hz, decay_scale, brightness, self.pickup)
    }

    pub fn modes_at_pickup(
        &self,
        base_hz: f32,
        decay_scale: f32,
        brightness: f32,
        pickup: Pickup,
    ) -> Vec<Mode> {
        let base = base_hz.max(20.0);
        // Bigger object (lower f0) rings longer.
        let size = (self.f0_hz / base).clamp(0.25, 4.0);
        let t60_ref = self.t60_0 * decay_scale * size.powf(0.35);
        self.build(base, t60_ref, brightness, pickup)
    }

    /// Same table, but the fundamental gets exactly `first_t60`. Presets use
    /// this so their existing hand-tuned decay stays meaningful.
    pub fn modes_with_t60(&self, base_hz: f32, first_t60: f32, brightness: f32) -> Vec<Mode> {
        self.build(
            base_hz.max(20.0),
            first_t60.max(1e-4),
            brightness,
            self.pickup,
        )
    }

    pub fn modes_with_t60_pickup(
        &self,
        base_hz: f32,
        first_t60: f32,
        brightness: f32,
        pickup: Pickup,
    ) -> Vec<Mode> {
        self.build(base_hz.max(20.0), first_t60.max(1e-4), brightness, pickup)
    }

    fn build(&self, base: f32, t60_ref: f32, brightness: f32, pickup: Pickup) -> Vec<Mode> {
        // 0.5 is neutral: the table amplitudes already describe the spectrum.
        // Below 0.5 tilts the top down, above it lifts the top.
        let tilt = (0.5 - brightness).clamp(-1.0, 1.0) * 1.2;

        let mut modes: Vec<Mode> = self
            .modes
            .iter()
            .enumerate()
            .map(|(i, mm)| {
                let ratio = mm.ratio.max(0.01);
                let f = base * ratio;
                let g = mm.amp * comb(i + 1, pickup.strike, pickup.listen, pickup.depth);
                let g = g * ratio.powf(-tilt);
                let t60 = if mm.t60 > 0.0 {
                    mm.t60 * (t60_ref / self.t60_0)
                } else {
                    t60_ref * ratio.powf(-self.alpha)
                };
                Mode::new(f, g, t60.max(1e-4))
            })
            // Anything above 20 kHz would fold at 44.1 kHz.
            .filter(|m| m.freq < 20_000.0)
            .collect();

        // Normalise the sum of the mode amplitudes so a 7-mode table and a
        // 12-mode table hit the same peak. Without this the library drifts by
        // several dB from preset to preset just because the tables are
        // different sizes.
        let sum: f32 = modes.iter().map(|m| m.amp.abs()).sum();
        if sum > 1e-6 {
            let scale = TABLE_AMP_SUM / sum;
            for m in &mut modes {
                m.amp *= scale;
            }
        }
        modes
    }
}

/// Target sum of mode amplitudes — calibrated against the hand-tuned
/// five-mode sets this replaced, so presets keep their balance against the
/// noise layers.
const TABLE_AMP_SUM: f32 = 2.2;

pub static STEEL: Material = Material {
    id: "steel",
    f0_hz: 480.0,
    t60_0: 1.90,
    alpha: 0.72,
    noise_mix: 0.10,
    contact: 0.000_15,
    pickup: Pickup::shell(),
    modes: &[
        m(1.000, 1.00),
        m(1.009, 0.50),
        m(2.756, 0.62),
        m(2.790, 0.34),
        m(5.404, 0.40),
        m(5.470, 0.22),
        m(8.933, 0.25),
        m(9.062, 0.14),
        m(13.340, 0.16),
        m(17.710, 0.10),
        m(23.530, 0.07),
        m(30.100, 0.04),
    ],
};

pub static ALUMINIUM: Material = Material {
    id: "aluminium",
    f0_hz: 1180.0,
    t60_0: 0.34,
    alpha: 0.85,
    noise_mix: 0.12,
    contact: 0.000_20,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 0.72),
        m(1.729, 0.44),
        m(2.328, 1.00),
        m(3.906, 0.30),
        m(4.044, 0.18),
        m(6.137, 0.20),
        m(6.688, 0.12),
        m(8.612, 0.09),
        m(10.900, 0.06),
        m(13.200, 0.04),
    ],
};

/// Oak plank — crate walls, stocks, pallets.
pub static OAK: Material = Material {
    id: "oak",
    f0_hz: 158.0,
    t60_0: 0.26,
    alpha: 0.55,
    noise_mix: 0.25,
    contact: 0.000_80,
    pickup: Pickup::bar(),
    modes: &[
        m(1.000, 0.85),
        m(1.630, 0.55),
        m(2.712, 1.00),
        m(5.240, 0.42),
        m(8.550, 0.24),
        m(12.620, 0.14),
        m(17.200, 0.08),
        m(22.300, 0.05),
        m(28.300, 0.03),
    ],
};

/// Pine board — cheaper, shorter, softer than oak.
pub static PINE: Material = Material {
    id: "pine",
    f0_hz: 124.0,
    t60_0: 0.185,
    alpha: 0.62,
    noise_mix: 0.30,
    contact: 0.000_80,
    pickup: Pickup::bar(),
    modes: &[
        m(1.000, 0.80),
        m(1.580, 0.45),
        m(2.690, 1.00),
        m(5.200, 0.36),
        m(8.440, 0.18),
        m(12.400, 0.10),
        m(16.900, 0.05),
        m(21.900, 0.03),
    ],
};

/// Glass bottle: bending modes plus a Helmholtz air resonance below them.
pub static GLASS_BOTTLE: Material = Material {
    id: "glass_bottle",
    f0_hz: 520.0,
    t60_0: 1.90,
    alpha: 0.42,
    noise_mix: 0.08,
    contact: 0.000_20,
    pickup: Pickup::shell(),
    modes: &[
        mt(0.420, 0.30, 0.22),
        m(1.000, 1.00),
        m(1.006, 0.62),
        m(2.830, 0.55),
        m(2.860, 0.35),
        m(5.420, 0.28),
        m(8.770, 0.18),
        m(12.900, 0.10),
        m(17.700, 0.06),
    ],
};

/// A free glass shard: plate-like, very short, very bright.
pub static GLASS: Material = Material {
    id: "glass",
    f0_hz: 1800.0,
    t60_0: 0.095,
    alpha: 0.50,
    noise_mix: 0.30,
    contact: 0.000_15,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 0.75),
        m(1.729, 1.00),
        m(2.328, 0.85),
        m(3.906, 0.42),
        m(4.044, 0.30),
        m(6.137, 0.22),
        m(6.688, 0.14),
        m(8.612, 0.09),
    ],
};

pub static CERAMIC: Material = Material {
    id: "ceramic",
    f0_hz: 1080.0,
    t60_0: 0.235,
    alpha: 0.68,
    noise_mix: 0.18,
    contact: 0.000_20,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 0.85),
        m(1.660, 1.00),
        m(2.280, 0.78),
        m(3.720, 0.40),
        m(4.150, 0.32),
        m(6.020, 0.20),
        m(6.550, 0.13),
        m(8.400, 0.09),
        m(10.500, 0.05),
        m(12.800, 0.03),
    ],
};

/// Hollow bone (femur-like). The plate table previously used here was in fact
/// a membrane series, which is why bone read as a timpani.
pub static BONE: Material = Material {
    id: "bone",
    f0_hz: 860.0,
    t60_0: 0.105,
    alpha: 0.60,
    noise_mix: 0.35,
    contact: 0.000_25,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 0.80),
        m(1.620, 1.00),
        m(2.720, 0.55),
        m(2.780, 0.35),
        m(3.940, 0.45),
        m(5.310, 0.28),
        m(5.440, 0.18),
        m(7.900, 0.12),
        m(9.200, 0.07),
        m(11.600, 0.04),
    ],
};

/// Muscle and fat: a damped mass, not a resonator.
pub static FLESH: Material = Material {
    id: "flesh",
    f0_hz: 92.0,
    t60_0: 0.048,
    alpha: 0.50,
    noise_mix: 0.70,
    contact: 0.004_00,
    pickup: Pickup::flesh(),
    modes: &[
        m(1.000, 1.00),
        m(1.850, 0.55),
        m(3.200, 0.30),
        m(5.100, 0.16),
        m(7.400, 0.09),
        m(10.500, 0.05),
        m(14.200, 0.03),
    ],
};

pub static CONCRETE: Material = Material {
    id: "concrete",
    f0_hz: 155.0,
    t60_0: 0.38,
    alpha: 0.90,
    noise_mix: 0.45,
    contact: 0.000_60,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 1.00),
        m(1.520, 0.62),
        m(2.310, 0.48),
        m(3.140, 0.30),
        m(4.420, 0.22),
        m(6.050, 0.14),
        m(8.100, 0.09),
        m(10.600, 0.05),
        m(13.800, 0.03),
    ],
};

pub static CARDBOARD: Material = Material {
    id: "cardboard",
    f0_hz: 430.0,
    t60_0: 0.013,
    alpha: 1.00,
    noise_mix: 0.60,
    contact: 0.003_00,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 1.00),
        m(1.900, 0.45),
        m(3.300, 0.28),
        m(5.200, 0.16),
        m(7.600, 0.09),
        m(10.400, 0.05),
        m(14.000, 0.03),
    ],
};

pub static PLASTIC: Material = Material {
    id: "plastic",
    f0_hz: 720.0,
    t60_0: 0.125,
    alpha: 0.80,
    noise_mix: 0.20,
    contact: 0.000_40,
    pickup: Pickup::plate(),
    modes: &[
        m(1.000, 0.85),
        m(1.720, 1.00),
        m(2.350, 0.70),
        m(3.880, 0.35),
        m(4.100, 0.25),
        m(6.100, 0.16),
        m(6.720, 0.10),
        m(8.500, 0.07),
        m(10.800, 0.04),
        m(13.500, 0.03),
    ],
};

/// Convenience: the table for a category's default body material.
pub fn material(id: &str) -> Option<&'static Material> {
    const ALL: [&Material; 12] = [
        &STEEL,
        &ALUMINIUM,
        &OAK,
        &PINE,
        &GLASS_BOTTLE,
        &GLASS,
        &CERAMIC,
        &BONE,
        &FLESH,
        &CONCRETE,
        &CARDBOARD,
        &PLASTIC,
    ];
    ALL.iter().copied().find(|m| m.id == id)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn all_materials() -> Vec<&'static Material> {
        vec![
            &STEEL,
            &ALUMINIUM,
            &OAK,
            &PINE,
            &GLASS_BOTTLE,
            &GLASS,
            &CERAMIC,
            &BONE,
            &FLESH,
            &CONCRETE,
            &CARDBOARD,
            &PLASTIC,
        ]
    }

    #[test]
    fn damping_follows_the_table_law() {
        for mat in all_materials() {
            let modes = mat.modes_at(mat.f0_hz, 1.0, 0.5);
            // Skip leading Helmholtz rows (explicit T60) — find the fundamental.
            let base = modes
                .iter()
                .zip(mat.modes)
                .find(|(_, raw)| raw.t60 == 0.0)
                .map(|(mode, _)| mode)
                .expect("material has a mode governed by the damping law");
            assert!(
                (base.decay - mat.t60_0).abs() < 1e-3,
                "{} fundamental T60 {} != {}",
                mat.id,
                base.decay,
                mat.t60_0
            );
            // Log-log slope of T60 against frequency must be -alpha.
            let hi = modes.iter().max_by(|a, b| a.freq.total_cmp(&b.freq)).unwrap();
            let slope = (hi.decay / base.decay).ln() / (hi.freq / base.freq).ln();
            assert!(
                (slope + mat.alpha).abs() < 0.02,
                "{} damping slope {slope:.3}, expected -{:.2}",
                mat.id,
                mat.alpha
            );
        }
    }

    #[test]
    fn pickup_comb_leaves_no_mode_silent() {
        for mat in all_materials() {
            let modes = mat.modes_at(mat.f0_hz, 1.0, 0.5);
            for (i, mode) in modes.iter().enumerate() {
                assert!(
                    mode.amp > 1e-3,
                    "{} mode {} is silent (comb depth leaves no nulls)",
                    mat.id,
                    i + 1
                );
                assert!(mode.amp.is_finite() && mode.decay.is_finite(), "{} NaN", mat.id);
            }
        }
    }

    #[test]
    fn pickup_comb_places_nulls() {
        // A bar struck at 0.18 and heard at 0.31: the product has nulls near
        // k = 3, 5-6, 9, 12. At least three of twelve modes must be torn down
        // to less than half the table amplitude, otherwise there is no comb.
        let modes = OAK.modes_at(OAK.f0_hz, 1.0, 0.5);
        let notched = OAK
            .modes
            .iter()
            .zip(&modes)
            .filter(|(raw, mode)| mode.amp < raw.amp * 0.5)
            .count();
        assert!(notched >= 2, "comb found only {notched} nulls");
    }

    #[test]
    fn modes_stay_below_nyquist() {
        let sr = 48_000.0;
        for mat in all_materials() {
            for mode in mat.modes_at(mat.f0_hz, 1.0, 0.5) {
                assert!(
                    mode.freq < sr * 0.48,
                    "{} mode at {:.0} Hz is above Nyquist",
                    mat.id,
                    mode.freq
                );
            }
        }
    }

    #[test]
    fn comb_never_returns_zero() {
        for k in 1..=32 {
            let g = comb(k, 0.5, 0.5, 1.0);
            assert!(g >= 0.0);
            let g = comb(k, 0.19, 0.34, 0.85);
            assert!(g > 0.0, "mode {k} nulled out at depth 0.85");
        }
    }
}

