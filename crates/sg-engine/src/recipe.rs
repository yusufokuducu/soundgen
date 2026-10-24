//! Recipe model: a named, layer-based, style-morphing sound definition.

use crate::layer::{Category, Layer};
use sg_dsp::rng::Rng;

#[derive(Clone, Copy, Debug)]
pub struct MasterChain {
    pub highpass_hz: f32,
    pub low_shelf_db: f32,
    pub low_shelf_hz: f32,
    pub presence_db: f32,
    pub presence_hz: f32,
    pub comp_threshold_db: f32,
    pub comp_ratio: f32,
    pub drive_db: f32,
    pub reverb_wet: f32,
    pub reverb_size: f32,
    pub reverb_damp: f32,
    pub stereo_width: f32,
    pub ceiling_db: f32,
    /// Peak normalisation target in dBFS applied after the chain.
    pub target_peak_db: f32,
}

impl Default for MasterChain {
    fn default() -> Self {
        Self {
            highpass_hz: 30.0,
            low_shelf_db: 0.0,
            low_shelf_hz: 120.0,
            presence_db: 0.0,
            presence_hz: 3000.0,
            comp_threshold_db: -14.0,
            comp_ratio: 2.5,
            drive_db: 0.0,
            reverb_wet: 0.0,
            reverb_size: 0.45,
            reverb_damp: 0.5,
            stereo_width: 1.0,
            ceiling_db: -0.3,
            target_peak_db: -1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub tags: Vec<String>,
    pub description: String,
    pub layers: Vec<Layer>,
    /// Render length in seconds (torso, before tail extension).
    pub duration: f32,
    pub fade_out: f32,
    pub fade_in: f32,
    pub looped: bool,
    pub master: MasterChain,
    /// 0.0 = realistic, 1.0 = cartoon/stylised.
    pub style: f32,
    /// Loops only: circular crossfade length in seconds (0 = hard splice).
    pub loop_crossfade: f32,
    pub sample_rate: u32,
}

impl Recipe {
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: Category) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            tags: Vec::new(),
            description: String::new(),
            layers: Vec::new(),
            duration: 1.0,
            fade_out: 0.02,
            fade_in: 0.001,
            looped: false,
            master: MasterChain::default(),
            style: 0.35,
            loop_crossfade: 0.0,
            sample_rate: 48_000,
        }
    }

    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into());
        self
    }

    pub fn layer(mut self, l: Layer) -> Self {
        self.layers.push(l);
        self
    }

    pub fn about(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// How long the render must be to hold every layer plus its tail.
    /// Loops must be exactly `duration` (a tail pad would open a silent gap at
    /// the seam), one-shots get 50 ms of safety.
    pub fn render_length(&self) -> f32 {
        if self.looped {
            return self.duration.clamp(0.01, 30.0);
        }
        let tail = self
            .layers
            .iter()
            .filter(|l| !l.mute)
            .map(|l| l.tail())
            .fold(0.0f32, f32::max);
        (self.duration.max(tail) + 0.05).min(30.0)
    }

    /// Jitter every layer's key parameters within perceptual-safe bounds.
    /// `amount` 0..1 = subtle sibling .. wild cousin.
    pub fn randomize(&mut self, amount: f32, seed: u64) {
        let mut rng = Rng::new(seed);
        let a = amount.clamp(0.0, 1.0);
        for l in &mut self.layers {
            l.gain_db += rng.normal() * (1.5 + 2.5 * a);
            l.pan = (l.pan + rng.normal() * 0.08 * a).clamp(-0.8, 0.8);
            l.delay = (l.delay * (1.0 + rng.normal() * 0.15 * a)).max(0.0);
            randomize_kind(&mut l.kind, &mut rng, a);
        }
    }
}

fn randomize_kind(kind: &mut crate::layer::LayerKind, rng: &mut Rng, amount: f32) {
    use crate::layer::LayerKind::*;
    match kind {
        Noise { filter, env, .. } => {
            filter.start_hz = (filter.start_hz * (1.0 + rng.normal() * 0.18 * amount)).clamp(20.0, 20_000.0);
            filter.end_hz = (filter.end_hz * (1.0 + rng.normal() * 0.18 * amount)).clamp(20.0, 20_000.0);
            env.decay = (env.decay * (1.0 + rng.normal() * 0.22 * amount)).clamp(0.002, 8.0);
        }
        Tone { start_hz, end_hz, .. } => {
            *start_hz = (*start_hz * (1.0 + rng.normal() * 0.12 * amount)).clamp(15.0, 12_000.0);
            *end_hz = (*end_hz * (1.0 + rng.normal() * 0.12 * amount)).clamp(15.0, 12_000.0);
        }
        Modal { modes, detune_cents, .. } => {
            *detune_cents += rng.normal() * 30.0 * amount;
            for m in modes.iter_mut() {
                m.freq *= 1.0 + rng.normal() * 0.03 * amount;
                m.decay = (m.decay * (1.0 + rng.normal() * 0.2 * amount)).clamp(0.005, 6.0);
            }
        }
        Rotor { rpm, blades, .. } => {
            *rpm = (*rpm * (1.0 + rng.normal() * 0.05 * amount)).clamp(120.0, 12_000.0);
            *blades = (*blades * (1.0 + rng.normal() * 0.05 * amount)).clamp(2.0, 8.0);
        }
        Grain { grain_ms, density, spray, .. } => {
            *grain_ms = (*grain_ms * (1.0 + rng.normal() * 0.2 * amount)).clamp(3.0, 400.0);
            *density = (*density * (1.0 + rng.normal() * 0.25 * amount)).clamp(0.5, 800.0);
            *spray = (*spray * (1.0 + rng.normal() * 0.25 * amount)).clamp(0.0, 1.0);
        }
        Sweep { start_hz, end_hz, .. } => {
            *start_hz = (*start_hz * (1.0 + rng.normal() * 0.2 * amount)).clamp(20.0, 18_000.0);
            *end_hz = (*end_hz * (1.0 + rng.normal() * 0.2 * amount)).clamp(20.0, 18_000.0);
        }
    }
}
