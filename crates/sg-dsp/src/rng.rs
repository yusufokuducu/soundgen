//! Deterministic pseudo-random numbers (xoshiro256**), seeded per variation.

#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

struct SplitMix64 {
    x: u64,
}

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.x = self.x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = SplitMix64 {
            x: seed ^ 0x9E37_79B9_7F4A_7C15,
        };
        let mut s = [0u64; 4];
        for v in s.iter_mut() {
            *v = sm.next();
        }
        if s.iter().all(|&v| v == 0) {
            s[0] = 1;
        }
        Self { s }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1).
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) * (1.0 / 16_777_216.0)
    }

    /// Uniform in [-1, 1).
    #[inline]
    pub fn bipolar(&mut self) -> f32 {
        self.next_f32() * 2.0 - 1.0
    }

    /// Uniform in [lo, hi).
    #[inline]
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    #[inline]
    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }

    /// Approximately normal, mean 0, stddev 1 (Box–Muller, no cache).
    #[inline]
    pub fn normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-7);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (TAU_ * u2).cos()
    }

    /// Log-uniform in [lo, hi): natural for frequency/Q jitter.
    #[inline]
    pub fn log_range(&mut self, lo: f32, hi: f32) -> f32 {
        let l = lo.max(1e-6).ln();
        let h = hi.max(1e-6).ln();
        (l + (h - l) * self.next_f32()).exp()
    }

    /// Pick a random element.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u64() as usize) % items.len()]
    }
}

const TAU_: f32 = std::f32::consts::TAU;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn uniform_range() {
        let mut r = Rng::new(7);
        let mut sum = 0.0;
        for _ in 0..10_000 {
            let v = r.range(-1.0, 1.0);
            assert!((-1.0..1.0).contains(&v));
            sum += v;
        }
        assert!((sum / 10_000.0).abs() < 0.05);
    }
}
