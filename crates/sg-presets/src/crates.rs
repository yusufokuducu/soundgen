//! Crate category: wooden and metal loot containers — creak, pop, shatter,
//! lock, reward reveal.
//!
//! Modal sets come straight from the research notes: wooden box
//! `1 / 2.32 / 3.71 / 5.14 / 6.90`, metal bar `1 / 2.756 / 5.404 / 8.933`,
//! every mode mistuned ±1.5 % so nothing rings like a test tone.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        wood_creak(),
        open_pop(),
        shatter(),
        lock_click(),
        reward_reveal(),
        metal_open(),
    ]
}

// ---------------------------------------------------------------------------
// material helpers
// ---------------------------------------------------------------------------

/// Wooden box modes, from the oak plank table. `brightness` tilts the top
/// (0.5 neutral, >0.5 brighter).
fn wood_modes(base: f32, decay: f32, brightness: f32) -> Vec<Mode> {
    sg_dsp::modal::OAK.modes_with_t60(base, decay, brightness.clamp(0.0, 1.0))
}

/// Steel shell modes: bright, inharmonic, long ring.
fn metal_modes(base: f32, decay: f32) -> Vec<Mode> {
    sg_dsp::modal::STEEL.modes_with_t60(base, decay, 0.5)
}

// ---------------------------------------------------------------------------
// 1. wood creak
// ---------------------------------------------------------------------------

fn wood_creak() -> Recipe {
    let mut r = base("crate_wood_creak", "Wood Creak", Category::Crate);
    r.duration = 1.35;
    r.fade_out = 0.06;
    r.style = 0.3;
    r.tags = vec!["crate".into(), "wood".into(), "creak".into(), "open".into()];
    r.description = "Slow stick-slip groan of an old plank hinge, rough under the fingers and hollow inside.".into();
    r.master = MasterChain {
        highpass_hz: 90.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 180.0,
        presence_db: 1.5,
        presence_hz: 1800.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 1.0,
        reverb_wet: 0.16,
        reverb_size: 0.35,
        reverb_damp: 0.55,
        stereo_width: 1.1,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "stick slip",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 320.0,
                end_hz: 288.0,
                pitch_tau: 0.9, // drifts as the joint unloads
                env: AdEnv::swelled(0.10, 1.10),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.8, // 5.5 Hz judder = the slip/release cycle
            },
        )
        .gain(-11.0)
        .pan(-0.12)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "grain rub",
            LayerKind::Grain {
                source_hz: 900.0,
                grain_ms: 12.0,
                density: 55.0,
                spray: 0.85,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.15, 1.0),
                pitch_jitter: 0.35,
            },
        )
        .gain(-12.0)
        .pan(-0.2)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "wood body",
            LayerKind::Modal {
                modes: wood_modes(240.0, 0.55, 0.55),
                exciter_ms: 10.0,
                exciter_color: NoiseColor::Pink,
                exciter_hp: 220.0,
                exciter_lp: 3200.0,
                detune_cents: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "hinge groan",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 148.0,
                end_hz: 121.0,
                pitch_tau: 0.7,
                env: AdEnv::swelled(0.22, 0.8),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.4,
            },
        )
        .gain(-17.0)
        .pan(0.18)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.2, 0.9),
                filter: FilterEnv::new(900.0, 1400.0, 0.4, 0.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-19.0)
        .pan(0.22)
        .style(1.0, 1.2),
    )
}

// ---------------------------------------------------------------------------
// 2. crate opens with a pop
// ---------------------------------------------------------------------------

fn open_pop() -> Recipe {
    let mut r = base("crate_open_pop", "Crate Open Pop", Category::Crate);
    r.duration = 0.55;
    r.fade_out = 0.05;
    r.style = 0.35;
    r.tags = vec!["crate".into(), "wood".into(), "open".into(), "latch".into()];
    r.description = "Latch snaps, the lid jumps and splinters of dry wood scatter off the rim.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        presence_db: 2.5,
        presence_hz: 3000.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.0,
        drive_db: 2.0,
        reverb_wet: 0.18,
        reverb_size: 0.4,
        reverb_damp: 0.5,
        stereo_width: 1.15,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "latch snap",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.008),
                filter: FilterEnv::new(9000.0, 3200.0, 0.004, 0.9),
                mode: 2,
                tame_hz: 14_000.0,
            },
        )
        .gain(-1.0)
        .style(1.0, 0.85),
    );
    r = r.layer(
        Layer::new(
            "wood crack",
            LayerKind::Modal {
                modes: wood_modes(190.0, 0.35, 0.7),
                exciter_ms: 1.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 150.0,
                exciter_lp: 4500.0,
                detune_cents: 6.0,
            },
        )
        .gain(-3.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "lid pop",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 230.0,
                end_hz: 70.0,
                pitch_tau: 0.018,
                env: AdEnv::perc(0.10),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.6),
    );
    r = r.layer(
        Layer::new(
            "debris",
            LayerKind::Grain {
                source_hz: 2600.0,
                grain_ms: 8.0,
                density: 110.0,
                spray: 0.85,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.005, 0.28),
                pitch_jitter: 0.5,
            },
        )
        .gain(-13.0)
        .pan(0.2)
        .delay(0.03)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.12),
                filter: FilterEnv::new(1200.0, 400.0, 0.05, 0.9),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-11.0)
        .style(0.8, 1.3),
    )
}

// ---------------------------------------------------------------------------
// 3. crate shatters
// ---------------------------------------------------------------------------

fn shatter() -> Recipe {
    let mut r = base("crate_shatter", "Crate Shatter", Category::Crate);
    r.duration = 1.1;
    r.fade_out = 0.08;
    r.style = 0.4;
    r.tags = vec!["crate".into(), "wood".into(), "shatter".into(), "destroy".into()];
    r.description = "Dry crate exploding into planks: hard crack, a burst of splinters and a long shower of debris.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 140.0,
        presence_db: 2.0,
        presence_hz: 3600.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.5,
        drive_db: 3.0,
        reverb_wet: 0.22,
        reverb_size: 0.5,
        reverb_damp: 0.5,
        stereo_width: 1.2,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.012),
                filter: FilterEnv::new(12_000.0, 2600.0, 0.005, 0.8),
                mode: 2,
                tame_hz: 15_000.0,
            },
        )
        .gain(0.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "splinter burst",
            LayerKind::Modal {
                modes: wood_modes(420.0, 0.10, 0.95),
                exciter_ms: 2.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 300.0,
                exciter_lp: 6000.0,
                detune_cents: 40.0,
            },
        )
        .gain(-4.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "splinter spray",
            LayerKind::Grain {
                source_hz: 1800.0,
                grain_ms: 6.0,
                density: 260.0,
                spray: 0.9,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.004, 0.5),
                pitch_jitter: 0.5,
            },
        )
        .gain(-9.0)
        .pan(-0.25)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "debris",
            LayerKind::Grain {
                source_hz: 900.0,
                grain_ms: 14.0,
                density: 45.0,
                spray: 1.0,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.05, 0.75),
                pitch_jitter: 0.6,
            },
        )
        .gain(-14.0)
        .pan(0.3)
        .delay(0.02)
        .style(1.0, 1.5),
    );
    r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.9),
                filter: FilterEnv::new(3500.0, 300.0, 0.12, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-18.0)
        .style(0.7, 1.6),
    )
}

// ---------------------------------------------------------------------------
// 4. lock click
// ---------------------------------------------------------------------------

fn lock_click() -> Recipe {
    let mut r = base("crate_lock_click", "Lock Click", Category::Crate);
    r.duration = 0.15;
    r.fade_out = 0.02;
    r.style = 0.25;
    r.tags = vec!["crate".into(), "metal".into(), "lock".into(), "click".into()];
    r.description = "Tiny two-stage padlock click: sharp tick, then the tumbler seating 55 ms later.".into();
    r.master = MasterChain {
        highpass_hz: 300.0,
        presence_db: 3.0,
        presence_hz: 5200.0,
        comp_threshold_db: -12.0,
        comp_ratio: 2.5,
        drive_db: 1.0,
        reverb_wet: 0.12,
        reverb_size: 0.25,
        reverb_damp: 0.4,
        stereo_width: 1.05,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.0025),
                filter: FilterEnv::new(6500.0, 4200.0, 0.002, 2.0),
                mode: 1,
                tame_hz: 16_000.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "tumbler",
            LayerKind::Modal {
                modes: metal_modes(3400.0, 0.025),
                exciter_ms: 0.0, // single-sample impulse
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Modal {
                modes: wood_modes(420.0, 0.09, 0.6),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::Pink,
                exciter_hp: 150.0,
                exciter_lp: 3000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-13.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "second click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.003),
                filter: FilterEnv::new(6000.0, 3600.0, 0.002, 1.6),
                mode: 1,
                tame_hz: 15_000.0,
            },
        )
        .gain(-6.0)
        .pan(0.1)
        .delay(0.055)
        .style(1.0, 1.2),
    );
    r.layer(
        Layer::new(
            "spring",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 1850.0,
                end_hz: 1450.0,
                pitch_tau: 0.012,
                env: AdEnv::perc(0.035),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-19.0)
        .style(1.0, 1.5),
    )
}

// ---------------------------------------------------------------------------
// 5. reward reveal
// ---------------------------------------------------------------------------

fn reward_reveal() -> Recipe {
    let mut r = base("crate_reward_reveal", "Crate Reward Reveal", Category::Crate);
    r.duration = 1.5;
    r.fade_out = 0.2;
    r.style = 0.7;
    r.tags = vec!["crate".into(), "reward".into(), "riser".into(), "magic".into()];
    r.description = "Magical riser into a shimmering chime, the sound of the crate handing over something good.".into();
    r.master = MasterChain {
        highpass_hz: 120.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 200.0,
        presence_db: 3.0,
        presence_hz: 5000.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 1.5,
        reverb_wet: 0.32,
        reverb_size: 0.62,
        reverb_damp: 0.45,
        stereo_width: 1.35,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "riser",
            LayerKind::Sweep {
                start_hz: 400.0,
                end_hz: 4200.0,
                tau: 0.85,
                q: 2.5,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.35, 0.55),
                tone_mix: 0.9,
            },
        )
        .gain(-6.0)
        .style(0.9, 1.4),
    );
    r = r.layer(
        Layer::new(
            "shimmer",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 1320.0,
                end_hz: 2640.0,
                pitch_tau: 0.55,
                env: AdEnv::swelled(0.15, 0.9),
                fm_hz: 1.5,
                fm_depth: 0.08,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-14.0)
        .pan(-0.2)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "sparkle",
            LayerKind::Grain {
                source_hz: 5200.0,
                grain_ms: 8.0,
                density: 55.0,
                spray: 0.8,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.1, 0.9),
                pitch_jitter: 0.4,
            },
        )
        .gain(-19.0)
        .pan(0.3)
        .style(0.8, 1.6),
    );
    r = r.layer(
        Layer::new(
            "chime",
            LayerKind::Modal {
                modes: metal_modes(1046.5, 0.9), // C6, bell-ish partials
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 400.0,
                exciter_lp: 14_000.0,
                detune_cents: 4.0,
            },
        )
        .gain(-10.0)
        .delay(0.5) // lands on the top of the riser
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "sub swell",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 60.0,
                end_hz: 55.0,
                pitch_tau: 0.9,
                env: AdEnv::swelled(0.3, 0.6),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.1,
                vibrato: 0.0,
            },
        )
        .gain(-18.0)
        .style(1.0, 1.6),
    )
}

// ---------------------------------------------------------------------------
// 6. metal crate opens
// ---------------------------------------------------------------------------

fn metal_open() -> Recipe {
    let mut r = base("crate_metal_open", "Metal Crate Open", Category::Crate);
    r.duration = 0.9;
    r.fade_out = 0.1;
    r.style = 0.35;
    r.tags = vec!["crate".into(), "metal".into(), "open".into(), "lid".into()];
    r.description = "Steel lid pops its latch, groans on a rusty hinge and clangs back against the rim.".into();
    r.master = MasterChain {
        highpass_hz: 60.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 150.0,
        presence_db: 2.5,
        presence_hz: 4000.0,
        comp_threshold_db: -15.0,
        comp_ratio: 3.0,
        drive_db: 2.0,
        reverb_wet: 0.18,
        reverb_size: 0.45,
        reverb_damp: 0.45,
        stereo_width: 1.15,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "latch",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.004),
                filter: FilterEnv::new(8000.0, 3200.0, 0.003, 1.8),
                mode: 1,
                tame_hz: 15_000.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "lid creak",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 180.0,
                end_hz: 150.0,
                pitch_tau: 0.3,
                env: AdEnv::swelled(0.05, 0.5),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.6,
            },
        )
        .gain(-13.0)
        .pan(-0.15)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "metal ring",
            LayerKind::Modal {
                modes: metal_modes(620.0, 0.7),
                exciter_ms: 3.0,
                exciter_color: NoiseColor::Pink,
                exciter_hp: 300.0,
                exciter_lp: 8000.0,
                detune_cents: 8.0,
            },
        )
        .gain(-7.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "hinge groan",
            LayerKind::Grain {
                source_hz: 500.0,
                grain_ms: 25.0,
                density: 30.0,
                spray: 0.6,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.1, 0.55),
                pitch_jitter: 0.25,
            },
        )
        .gain(-17.0)
        .pan(-0.2)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "clank",
            LayerKind::Modal {
                modes: metal_modes(240.0, 0.28),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 120.0,
                exciter_lp: 6000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-9.0)
        .delay(0.3) // lid hits the stop
        .style(1.0, 1.5),
    )
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crates_render_cleanly() {
        let recipes = presets();
        assert!(recipes.len() >= 6, "expected at least 6 crate presets");

        for r in &recipes {
            assert!(
                (3..=6).contains(&r.layers.len()),
                "{} has {} layers (want 3-6)",
                r.id,
                r.layers.len()
            );
            assert!(!r.tags.is_empty(), "{} has no tags", r.id);
            assert!(!r.description.is_empty(), "{} has no description", r.id);
            assert!(!r.looped, "{} is a one-shot, not a loop", r.id);

            let buf = sg_engine::render(r, 11);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.05, "{} too short", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak < 1.0, "{} clips (peak {peak})", r.id);
            assert!(buf.samples.iter().all(|s| s.is_finite()), "{} has NaN/inf", r.id);

            println!(
                "{:<24} {:.3}s peak {:.2} dBFS rms {:.4} lufs {:.1} layers {}",
                r.id,
                buf.duration(),
                sg_dsp::gain_to_db(peak),
                buf.rms(),
                buf.lufs_approx(),
                r.layers.len()
            );
        }
    }

    #[test]
    fn style_morph_changes_the_voice() {
        // Cartoon versions must actually differ, otherwise `.style()` is dead weight.
        for r in presets() {
            let real = sg_engine::render_with_style(&r, 3, 0.0);
            let toon = sg_engine::render_with_style(&r, 3, 1.0);
            let diff: f32 = real
                .samples
                .iter()
                .zip(toon.samples.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max);
            assert!(diff > 1e-4, "{} sounds identical in both styles", r.id);
        }
    }
}
