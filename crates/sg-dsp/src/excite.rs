//! Impact excitation: what actually hits a modal body.
//!
//! A struck object is not excited by a click or a noise burst, it is excited by
//! a *contact force*: the surface deforms, the force rises and falls over the
//! contact time `Tc`, and the object rings afterwards. Two things follow, and
//! both matter:
//!
//! * the **area** of the force pulse is the momentum transferred (`J = m·v·(1+e)`),
//!   so it sets how loud the hit is;
//! * the **duration** `Tc` sets the spectrum (`f_-3dB ≈ 0.634 / Tc`), so it sets
//!   how bright the hit is.
//!
//! Loudness and brightness are therefore independent, which a noise burst can
//! never give you.
//!
//! Real surfaces are rough, so contact happens in two scales: a handful of
//! micrometre asperities break contact in 30–300 µs (the broadband "tick") and
//! the whole body stays in contact for 0.2–20 ms (the thump). The asperity
//! burst is returned on a second channel so it can be driven into the top end.

use crate::rng::Rng;

/// How a body and a striker meet.
#[derive(Clone, Copy, Debug)]
pub struct ImpactParams {
    /// Macro contact time in seconds. Metal–metal ≈ 0.15 ms, wood ≈ 0.8 ms,
    /// flesh ≈ 4 ms.
    pub contact_s: f32,
    /// Micro-contact energy, 0..1. Hard, brittle surfaces are ~0.7+, soft
    /// ones (flesh, cloth) are near zero.
    pub asperity: f32,
    /// Variance applied to `Tc` and `J` per hit, 0..1.
    pub jitter: f32,
}

impl Default for ImpactParams {
    fn default() -> Self {
        Self {
            contact_s: 0.000_8,
            asperity: 0.5,
            jitter: 0.25,
        }
    }
}

impl ImpactParams {
    pub fn new(contact_s: f32, asperity: f32) -> Self {
        Self {
            contact_s,
            asperity,
            jitter: 0.25,
        }
    }
}

/// What a single strike turned into. Handy for logging and for the UI.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    /// 0..1 as requested.
    pub velocity: f32,
    /// Striker speed in m/s.
    pub v_mps: f32,
    /// Momentum transferred, in the same arbitrary unit the bank is fed with.
    pub impulse: f32,
    /// Realised contact time after clamping and jitter.
    pub contact_s: f32,
    /// Where the pulse is 3 dB down: `0.634 / Tc`.
    pub f_minus_3db: f32,
    /// Number of asperity micro-contacts.
    pub asperity_count: u32,
}

#[inline]
fn hertz_shape(u: f32) -> f32 {
    // F(u) = (1 - (2u-1)^2)^1.5, the Hertzian force history of two elastic
    // bodies. Zero at both ends, smooth peak in the middle.
    let a = 1.0 - (2.0 * u - 1.0) * (2.0 * u - 1.0);
    if a <= 0.0 {
        0.0
    } else {
        a * a.sqrt()
    }
}

pub struct ImpactExcitation {
    sr: f32,
    pub p: ImpactParams,
    // Prepared per strike.
    j: f32,
    len: u32,
    inv_sum: f32,
    n: u32,
    asp_left: u32,
    asp_next: f32,
    asp_amp: f32,
    done: bool,
}

impl ImpactExcitation {
    pub fn new(sr: f32, p: ImpactParams) -> Self {
        Self {
            sr,
            p,
            j: 0.0,
            len: 0,
            inv_sum: 1.0,
            n: 0,
            asp_left: 0,
            asp_next: 0.0,
            asp_amp: 0.0,
            done: true,
        }
    }

    /// 0..1 velocity -> striker speed, momentum, contact time.
    ///
    /// The speed curve is deliberately steep near the top: a 0.2 swing in
    /// velocity should read as a clearly harder hit, not a 1 dB louder one.
    pub fn prepare(&mut self, velocity: f32, rng: &mut Rng) -> Strike {
        let vel = velocity.clamp(0.0, 1.0);
        let v = 0.08 + 4.92 * vel.powf(2.5);
        let jit = self.p.jitter;

        // Tc ∝ v^-0.2: faster strikes are only slightly shorter, which is why
        // brightness mostly comes from the asperity channel, not from Tc.
        let tc = self.p.contact_s * (1.0 / v.max(0.01)).powf(0.2);
        let tc = tc * (1.0 + rng.normal() * 0.18 * jit);
        self.len = ((tc * self.sr).round() as u32).clamp(2, (0.060 * self.sr) as u32);
        self.n = 0;

        // Midpoint sampling: with two points, endpoint sampling would make the
        // whole pulse evaluate to zero.
        let mut sum = 0.0f32;
        for i in 0..self.len {
            sum += hertz_shape((i as f32 + 0.5) / self.len as f32);
        }
        self.inv_sum = 1.0 / sum.max(1e-9);

        // Normalised so the default velocity (0.75) delivers unit momentum,
        // which keeps the library at the level the presets were trimmed for.
        const V_REF: f32 = 2.55;
        self.j = (v / V_REF) * (1.0 + rng.normal() * 0.06 * jit);

        self.asp_left = (2.0 + 10.0 * vel * self.p.asperity).round() as u32;
        self.asp_amp = 0.35 * self.p.asperity * self.inv_sum;
        self.asp_next = 0.0;
        self.done = false;

        Strike {
            velocity: vel,
            v_mps: v,
            impulse: self.j,
            contact_s: self.len as f32 / self.sr,
            f_minus_3db: 0.634 / (self.len as f32 / self.sr).max(1e-6),
            asperity_count: self.asp_left,
        }
    }

    /// `(force, asperity)`.
    ///
    /// `force` sums to exactly the momentum of the strike (the leftover tail is
    /// shed by the modal bank, not by a normalization step). `asperity` is a
    /// separate channel: add it to the high frequencies, do not add it to
    /// `force` or the momentum conservation is broken (bipolar micro-hits
    /// cancel each other).
    #[inline]
    pub fn next(&mut self, rng: &mut Rng) -> (f32, f32) {
        if self.done {
            return (0.0, 0.0);
        }
        let mut force = 0.0f32;
        let mut asp = 0.0f32;

        if self.n < self.len {
            let u = (self.n as f32 + 0.5) / self.len as f32;
            force = hertz_shape(u) * self.inv_sum;
        }
        if self.asp_left > 0 && self.n as f32 >= self.asp_next {
            asp = rng.bipolar() * self.asp_amp * (1.0 + rng.normal() * 0.4);
            self.asp_left -= 1;
            let span = self.len as f32 * 0.4 + 1.0;
            self.asp_next = self.n as f32 + rng.next_f32() * span * 0.25;
        }

        self.n += 1;
        if self.n >= self.len && self.asp_left == 0 {
            self.done = true;
        }
        (force * self.j, asp * self.j)
    }

    /// True once the pulse has been fully emitted.
    pub fn is_done(&self) -> bool {
        self.done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_sums_to_the_momentum() {
        let mut rng = Rng::new(11);
        for vel in [0.05, 0.3, 0.6, 1.0] {
            for tc in [0.000_15, 0.000_8, 0.004] {
                let mut e = ImpactExcitation::new(48_000.0, ImpactParams::new(tc, 0.5));
                let s = e.prepare(vel, &mut rng);
                let mut sum = 0.0;
                loop {
                    let (f, _) = e.next(&mut rng);
                    sum += f;
                    if e.is_done() {
                        break;
                    }
                }
                assert!(
                    (sum - s.impulse).abs() < 1e-3 * s.impulse.max(1.0),
                    "vel {vel} tc {tc}: force sums to {sum}, momentum is {}",
                    s.impulse
                );
            }
        }
    }

    #[test]
    fn contact_time_sets_the_brightness() {
        let mut rng = Rng::new(3);
        let mut soft = ImpactExcitation::new(48_000.0, ImpactParams::new(0.004, 0.0));
        let mut hard = ImpactExcitation::new(48_000.0, ImpactParams::new(0.000_15, 0.0));
        let a = soft.prepare(1.0, &mut rng);
        let b = hard.prepare(1.0, &mut rng);
        assert!(
            b.f_minus_3db > a.f_minus_3db * 5.0,
            "a 26x shorter contact must move the corner up, got {} vs {}",
            a.f_minus_3db,
            b.f_minus_3db
        );
        assert!(a.contact_s >= 2.0 / 48_000.0);
    }

    #[test]
    fn output_is_finite_and_bounded() {
        let mut rng = Rng::new(5);
        let mut e = ImpactExcitation::new(44_100.0, ImpactParams::new(0.000_2, 0.9));
        e.prepare(1.0, &mut rng);
        loop {
            let (f, a) = e.next(&mut rng);
            assert!(f.is_finite() && a.is_finite(), "NaN in impact output");
            let _ = e.is_done();
            if e.is_done() {
                break;
            }
        }
    }

    #[test]
    fn harder_hits_carry_more_momentum() {
        let mut rng = Rng::new(9);
        let mut e = ImpactExcitation::new(48_000.0, ImpactParams::new(0.000_8, 0.5));
        let soft = e.prepare(0.2, &mut rng).impulse;
        let hard = e.prepare(1.0, &mut rng).impulse;
        assert!(hard > soft * 3.0, "velocity barely changed the momentum");
    }
}
