# SoundGen — API Cheat Sheet (for coding agents)

Crate layout:

```
crates/sg-dsp      low-level DSP (rng, noise, env, filter, osc, modal, fx)
crates/sg-engine   Recipe / Layer model + offline renderer + AudioBuffer
crates/sg-presets  the factory library (one module per category)
crates/sg-io       WAV export + cpal player
apps/soundgen      egui UI
```

## Building a preset

```rust
use crate::prelude::*;   // inside sg-presets/src/<category>.rs

pub fn presets() -> Vec<Recipe> {
    vec![pistol()]
}

fn pistol() -> Recipe {
    let mut r = base("weapon_pistol_fire", "Pistol Fire", Category::Weapon);
    r.duration = 0.45;
    r.fade_out = 0.02;
    r.style = 0.35;
    r.tags = vec!["gun".into(), "small".into()];
    r.description = "Tight 9 mm crack with a short room tail.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        presence_db: 2.0,
        presence_hz: 3200.0,
        comp_threshold_db: -12.0,
        comp_ratio: 3.0,
        drive_db: 3.0,
        reverb_wet: 0.18,
        reverb_size: 0.35,
        stereo_width: 1.1,
        ..Default::default()
    };
    r = r.layer(
        Layer::new("crack", LayerKind::Noise {
            color: NoiseColor::White,
            env: AdEnv { attack: 0.0, hold: 0.0, decay: 0.012, curve: 4.0, attack_curve: 1.0 },
            filter: FilterEnv::new(7000.0, 2500.0, 0.006, 0.9),
            mode: 2,          // 0 = LP, 1 = BP, 2 = HP
            tame_hz: 12000.0,
        })
        .gain(-3.0)
        .pan(0.0)
        .style(1.0, 0.85),
    );
    r
}
```

## Types

```rust
Recipe {
  id: String, name: String, category: Category, tags: Vec<String>, description: String,
  layers: Vec<Layer>, duration: f32, fade_out: f32, fade_in: f32, looped: bool,
  master: MasterChain, style: f32 /* 0 realistic .. 1 cartoon */, sample_rate: u32,
}
Recipe::new(id, name, category) -> Recipe
Recipe::tag(s), .layer(l), .about(s), .randomize(amount 0..1, seed)

Category::{Weapon, Zombie, Gore, Foley, Loop, Crate, Celebration, Ui, Ambience}

Layer::new(name, kind)
  .gain(db) .pan(-1..1) .delay(seconds) .style(realistic_gain, cartoon_gain)
  fields: name, kind, gain_db, pan, delay, style_gain, mute, solo

AdEnv { attack, hold, decay /*seconds*/, curve /*1 linear, >1 percussive*/, attack_curve }
AdEnv::perc(decay) / AdEnv::swelled(attack, decay)

FilterEnv::new(start_hz, end_hz, tau, q)   // exponential sweep of the filter cutoff

MasterChain { highpass_hz, low_shelf_db, low_shelf_hz, presence_db, presence_hz,
  comp_threshold_db, comp_ratio, drive_db, reverb_wet, reverb_size, reverb_damp,
  stereo_width, ceiling_db, target_peak_db }   // ..Default::default()
```

## LayerKind variants (exact fields)

```rust
Noise  { color: NoiseColor, env: AdEnv, filter: FilterEnv, mode: u8 /*0 LP,1 BP,2 HP*/, tame_hz: f32 }
Tone   { shape: WaveShape, start_hz: f32, end_hz: f32, pitch_tau: f32, env: AdEnv,
         fm_hz: f32 /*ratio*/, fm_depth: f32, drive: f32, vibrato: f32 }
Modal  { modes: Vec<Mode>, exciter_ms: f32 /*0 = impulse*/, exciter_color: NoiseColor,
         exciter_hp: f32, exciter_lp: f32, detune_cents: f32 }
Grain  { source_hz: f32, grain_ms: f32, density: f32 /*grains per second*/, spray: f32 0..1,
         color: NoiseColor, env: AdEnv, pitch_jitter: f32 }
Rotor  { blades: f32, rpm: f32, tone_hz: f32, air: f32, flap_depth: f32, env: AdEnv, color: NoiseColor }
Sweep  { start_hz: f32, end_hz: f32, tau: f32, q: f32, color: NoiseColor, env: AdEnv, tone_mix: f32 }

NoiseColor::{White, Pink, Brown, Blue, Velvet}
WaveShape::{Sine, Triangle, Saw, Square}
Mode::new(freq, amp, decay_seconds)
inharmonic_modes(base, count, stretch, brightness, decay, &mut rng) -> Vec<Mode>
```

Material recipes from the research (use these numbers):
- metal bar: 1 / 2.756 / 5.404 / 8.933
- flat plate: 1 / 1.594 / 2.136 / 2.296
- wooden box: 1 / 2.32 / 3.71 / 5.14
- glass: 1 / 2.32 / 4.25 / 6.63 (long decay)
Always mistune ±1-2% per mode.

## Rules

1. Every layer gets a meaningful `name` (`crack`, `body`, `tail`, `debris`).
2. 2-6 layers per recipe. Layer roles: transient, body, tail, sweetener.
3. `style(a, b)` is REQUIRED thinking: which layer should disappear (0.0) or grow (1.6) in cartoon mode?
   Cartoon = deeper pitch envelopes, lower modal Q, added tonal/boing layers, longer tails, wider stereo.
4. `duration` must cover the longest tail; `render_length()` adds 50 ms safety.
5. Loop recipes (`Category::Loop`): set `looped = true` and make `duration` an exact multiple of the
   blade-pass period `60/rpm/blades` (propeller) or the tone period, otherwise the loop clicks.
6. Keep peak sane: renderer normalises to `master.target_peak_db` (-1 dBFS default).
7. Never allocate in hot loops; the renderer is offline but should stay fast (<20 ms per recipe).
8. Do not touch files outside your assignment. Do not create git commits.

## Build

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check -p sg-presets
```
