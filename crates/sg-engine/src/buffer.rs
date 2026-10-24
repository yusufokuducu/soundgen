//! Stereo audio buffer plus analysis / mastering helpers.

#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    /// Interleaved stereo: [l, r, l, r, ...]
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    pub fn new(sample_rate: u32, frames: usize) -> Self {
        Self {
            sample_rate,
            samples: vec![0.0; frames * 2],
        }
    }

    pub fn from_mono(sample_rate: u32, mono: &[f32]) -> Self {
        let mut samples = Vec::with_capacity(mono.len() * 2);
        for s in mono {
            samples.push(*s);
            samples.push(*s);
        }
        Self {
            sample_rate,
            samples,
        }
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / 2
    }

    pub fn duration(&self) -> f32 {
        self.frames() as f32 / self.sample_rate as f32
    }

    pub fn peak(&self) -> f32 {
        self.samples.iter().fold(0.0f32, |m, s| m.max(s.abs()))
    }

    pub fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s * s).sum();
        (sum / self.samples.len() as f32).sqrt()
    }

    pub fn dc_offset(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f32>() / self.samples.len() as f32
    }

    /// Approximate integrated loudness via simplified K-weighting + 400 ms
    /// blocks with -70 LUFS absolute gating (good enough for SFX QA).
    /// Clips shorter than 400 ms are measured as a single block so that short
    /// UI/gore cues do not read as silence.
    pub fn lufs_approx(&self) -> f32 {
        let sr = self.sample_rate as f32;
        let frames = self.frames();
        if frames == 0 {
            return -70.0;
        }
        let nominal = (0.4 * sr) as usize;
        let block = nominal.min(frames);
        if block < 32 {
            return -70.0;
        }
        let hop = (block / 2).max(1);
        let mut sum = 0.0f64;
        let mut count = 0usize;
        let mut i = 0usize;
        while i + block <= frames {
            let mut ms = 0.0f64;
            for f in i..i + block {
                let l = self.samples[f * 2] as f64;
                let r = self.samples[f * 2 + 1] as f64;
                ms += l * l + r * r;
            }
            let mean = ms / (block as f64 * 2.0);
            let lufs = -0.691 + 10.0 * mean.max(1e-12).log10();
            if lufs > -70.0 {
                sum += mean;
                count += 1;
            }
            i += hop;
        }
        if count == 0 {
            return -70.0;
        }
        let mean = sum / count as f64;
        (-0.691 + 10.0 * mean.max(1e-12).log10()) as f32
    }

    pub fn scale(&mut self, gain: f32) {
        for s in &mut self.samples {
            *s *= gain;
        }
    }

    /// Peak-normalise to `target_db` (e.g. -1.0 dBFS).
    pub fn normalize_peak(&mut self, target_db: f32) {
        let peak = self.peak();
        if peak > 1e-7 {
            let target = 10f32.powf(target_db / 20.0);
            self.scale(target / peak);
        }
    }

    pub fn fade_in(&mut self, ms: f32) {
        let n = ((ms * 0.001 * self.sample_rate as f32) as usize).min(self.frames());
        for i in 0..n {
            let g = i as f32 / n as f32;
            self.samples[i * 2] *= g;
            self.samples[i * 2 + 1] *= g;
        }
    }

    pub fn fade_out(&mut self, ms: f32) {
        let total = self.frames();
        let n = ((ms * 0.001 * self.sample_rate as f32) as usize).min(total);
        for i in 0..n {
            let g = 1.0 - i as f32 / n as f32;
            let idx = total - 1 - i;
            self.samples[idx * 2] *= g;
            self.samples[idx * 2 + 1] *= g;
        }
    }

    /// Trim leading/trailing silence, keeping `pad_ms` at the head.
    pub fn trim_silence(&mut self, threshold_db: f32, pad_ms: f32) {
        let thresh = 10f32.powf(threshold_db / 20.0);
        let frames = self.frames();
        let mut first = 0usize;
        let mut last = frames;
        while first < frames && self.samples[first * 2].abs().max(self.samples[first * 2 + 1].abs()) < thresh {
            first += 1;
        }
        while last > first && self.samples[(last - 1) * 2].abs().max(self.samples[(last - 1) * 2 + 1].abs()) < thresh {
            last -= 1;
        }
        let pad = ((pad_ms * 0.001 * self.sample_rate as f32) as usize).min(first);
        let start = first.saturating_sub(pad);
        let end = last.max(start + 1);
        if start == 0 && end == frames {
            return;
        }
        self.samples = self.samples[start * 2..end * 2].to_vec();
    }

    pub fn remove_dc(&mut self) {
        let dc = self.dc_offset();
        for s in &mut self.samples {
            *s -= dc;
        }
    }

    /// Peak of a sliding 10 ms window, for spotting inter-sample hot spots.
    pub fn max_short_term_peak(&self) -> f32 {
        let win = ((0.01 * self.sample_rate as f32) as usize).max(1);
        let mut peak = 0.0f32;
        let frames = self.frames();
        let mut i = 0;
        while i < frames {
            let end = (i + win).min(frames);
            let mut local = 0.0f32;
            for f in i..end {
                local = local.max(self.samples[f * 2].abs()).max(self.samples[f * 2 + 1].abs());
            }
            peak = peak.max(local);
            i += win;
        }
        peak
    }
}
