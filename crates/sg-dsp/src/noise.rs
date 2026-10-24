//! Coloured noise generators.

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseColor {
    /// Flat spectrum, -3 dB/oct energy.
    White,
    /// -3 dB/oct amplitude, the "natural" hiss.
    Pink,
    /// -6 dB/oct, rumble / wind / engine body.
    Brown,
    /// +3 dB/oct, thin and fizzy (metallic scratch, air).
    Blue,
    /// Sparse impulses: crackle, dust, vinyl, gravel.
    Velvet,
}

#[derive(Clone, Debug)]
pub struct NoiseGen {
    pub color: NoiseColor,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    brown: f32,
    lp: f32,
    /// Probability of an impulse per sample for `Velvet`.
    pub density: f32,
}

impl NoiseGen {
    pub fn new(color: NoiseColor) -> Self {
        Self {
            color,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            brown: 0.0,
            lp: 0.0,
            density: 0.01,
        }
    }

    #[inline]
    pub fn next(&mut self, rng: &mut Rng) -> f32 {
        let w = rng.bipolar();
        match self.color {
            NoiseColor::White => w,
            NoiseColor::Pink => {
                // Paul Kellet's refined method.
                self.b0 = 0.99886 * self.b0 + w * 0.0555179;
                self.b1 = 0.99332 * self.b1 + w * 0.0750759;
                self.b2 = 0.96900 * self.b2 + w * 0.1538520;
                self.b3 = 0.86650 * self.b3 + w * 0.3104856;
                self.b4 = 0.55000 * self.b4 + w * 0.5329522;
                self.b5 = -0.7616 * self.b5 - w * 0.0168980;
                let pink = self.b0
                    + self.b1
                    + self.b2
                    + self.b3
                    + self.b4
                    + self.b5
                    + self.b6
                    + w * 0.5362;
                self.b6 = w * 0.115926;
                pink * 0.11
            }
            NoiseColor::Brown => {
                self.brown = (self.brown + w * 0.02).clamp(-1.0, 1.0);
                self.brown * 3.5
            }
            NoiseColor::Blue => {
                self.lp += 0.05 * (w - self.lp);
                (w - self.lp) * 0.5
            }
            NoiseColor::Velvet => {
                if rng.next_f32() < self.density {
                    rng.bipolar()
                } else {
                    0.0
                }
            }
        }
    }
}
