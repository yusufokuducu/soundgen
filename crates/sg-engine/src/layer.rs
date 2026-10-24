//! Layer model: every effect is a stack of layers. Each layer is a small,
//! deterministic synth voice with its own envelope, gain, pan and delay.

use sg_dsp::env::AdEnv;
use sg_dsp::modal::Mode;
use sg_dsp::noise::NoiseColor;
use sg_dsp::osc::WaveShape;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Weapon,
    Zombie,
    Gore,
    Foley,
    Loop,
    Crate,
    Celebration,
    Ui,
    Ambience,
}

impl Category {
    pub fn folder(&self) -> &'static str {
        match self {
            Category::Weapon => "weapons",
            Category::Zombie => "zombies",
            Category::Gore => "gore",
            Category::Foley => "foley",
            Category::Loop => "loops",
            Category::Crate => "crates",
            Category::Celebration => "celebration",
            Category::Ui => "ui",
            Category::Ambience => "ambience",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Category::Weapon => "Weapons",
            Category::Zombie => "Zombies",
            Category::Gore => "Gore",
            Category::Foley => "Foley",
            Category::Loop => "Loops",
            Category::Crate => "Crates",
            Category::Celebration => "Celebration",
            Category::Ui => "UI",
            Category::Ambience => "Ambience",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Weapon,
            Category::Zombie,
            Category::Gore,
            Category::Foley,
            Category::Loop,
            Category::Crate,
            Category::Celebration,
            Category::Ui,
            Category::Ambience,
        ]
    }
}

/// Filter movement over the life of a layer: `freq(t) = end + (start-end) * exp(-t/tau)`.
#[derive(Clone, Copy, Debug)]
pub struct FilterEnv {
    pub start_hz: f32,
    pub end_hz: f32,
    pub tau: f32,
    pub q: f32,
}

impl FilterEnv {
    pub fn new(start_hz: f32, end_hz: f32, tau: f32, q: f32) -> Self {
        Self {
            start_hz,
            end_hz,
            tau: tau.max(1e-4),
            q,
        }
    }

    #[inline]
    pub fn at(&self, t: f32) -> f32 {
        self.end_hz + (self.start_hz - self.end_hz) * (-t / self.tau).exp()
    }
}

#[derive(Clone, Debug)]
pub enum LayerKind {
    /// Coloured noise through a modulated band/high/low filter.
    Noise {
        color: NoiseColor,
        env: AdEnv,
        filter: FilterEnv,
        /// Filter mode index: 0 = low-pass, 1 = band-pass, 2 = high-pass.
        mode: u8,
        /// Extra static low-pass to tame fizz (Hz).
        tame_hz: f32,
    },
    /// Oscillator with pitch envelope, optional FM and drive.
    Tone {
        shape: WaveShape,
        start_hz: f32,
        end_hz: f32,
        pitch_tau: f32,
        env: AdEnv,
        fm_hz: f32,
        fm_depth: f32,
        drive: f32,
        /// 0..1 amount of a slowly decaying vibrato.
        vibrato: f32,
    },
    /// Modal bank excited by a short noise burst or impulse.
    Modal {
        modes: Vec<Mode>,
        /// Excitation length in ms (0 = single-sample impulse).
        exciter_ms: f32,
        exciter_color: NoiseColor,
        exciter_hp: f32,
        exciter_lp: f32,
        /// Extra pitch-shift in cents applied to all modes.
        detune_cents: f32,
    },
    /// Granular cloud: the source is a pitched noise/tone bed cut into grains.
    Grain {
        source_hz: f32,
        grain_ms: f32,
        density: f32,
        spray: f32,
        color: NoiseColor,
        env: AdEnv,
        pitch_jitter: f32,
    },
    /// Continuous rotor: blade-pass AM + air noise + motor tone (propeller, drone).
    Rotor {
        blades: f32,
        rpm: f32,
        tone_hz: f32,
        air: f32,
        flap_depth: f32,
        env: AdEnv,
        color: NoiseColor,
    },
    /// Sine/noise sweep used for whooshes and risers.
    Sweep {
        start_hz: f32,
        end_hz: f32,
        tau: f32,
        q: f32,
        color: NoiseColor,
        env: AdEnv,
        tone_mix: f32,
    },
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub kind: LayerKind,
    pub gain_db: f32,
    pub pan: f32,
    /// Delay in seconds before the layer starts.
    pub delay: f32,
    /// Gain multiplier at style 0 (realistic) and style 1 (cartoon).
    pub style_gain: [f32; 2],
    pub mute: bool,
    pub solo: bool,
}

impl Layer {
    pub fn new(name: impl Into<String>, kind: LayerKind) -> Self {
        Self {
            name: name.into(),
            kind,
            gain_db: -6.0,
            pan: 0.0,
            delay: 0.0,
            style_gain: [1.0, 1.0],
            mute: false,
            solo: false,
        }
    }

    pub fn gain(mut self, db: f32) -> Self {
        self.gain_db = db;
        self
    }

    pub fn pan(mut self, p: f32) -> Self {
        self.pan = p.clamp(-1.0, 1.0);
        self
    }

    pub fn delay(mut self, secs: f32) -> Self {
        self.delay = secs.max(0.0);
        self
    }

    /// How this layer fades between realistic (0) and cartoon (1).
    pub fn style(mut self, realistic: f32, cartoon: f32) -> Self {
        self.style_gain = [realistic, cartoon];
        self
    }

    pub fn style_multiplier(&self, style: f32) -> f32 {
        let s = style.clamp(0.0, 1.0);
        self.style_gain[0] + (self.style_gain[1] - self.style_gain[0]) * s
    }

    /// Longest possible tail this layer can produce, in seconds.
    pub fn tail(&self) -> f32 {
        let base = match &self.kind {
            LayerKind::Noise { env, .. } => env.total(),
            LayerKind::Tone { env, .. } => env.total(),
            LayerKind::Sweep { env, .. } => env.total(),
            LayerKind::Grain { env, .. } => env.total(),
            LayerKind::Rotor { env, .. } => env.total(),
            LayerKind::Modal { modes, .. } => modes.iter().map(|m| m.decay).fold(0.0f32, f32::max),
        };
        self.delay + base
    }
}
