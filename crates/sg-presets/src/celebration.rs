//! Celebration presets: confetti, horns, fanfares, coin showers and level-ups.
//!
//! These sit at the loud end of the library (target peak -1 dBFS) because they
//! are one-shot 2D reward cues that must cut through combat beds. Everything is
//! tuned musical: bright but never fizzy above ~13 kHz.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        confetti_popper(),
        party_horn(),
        confetti_rain(),
        fanfare_sting(),
        coin_burst(),
        level_up(),
    ]
}

/// Compressed-air pop plus a paper cloud whose density *ramps*: a dense short
/// grain layer at t=0 and a sparse long one 200 ms later fake the ramp.
fn confetti_popper() -> Recipe {
    let mut r = base(
        "celebrate_confetti_popper",
        "Confetti Popper",
        Category::Celebration,
    );
    r.duration = 1.3;
    r.fade_in = 0.0;
    r.fade_out = 0.05;
    r.style = 0.55;
    r.tags = vec![
        "celebration".into(),
        "confetti".into(),
        "party".into(),
        "one-shot".into(),
    ];
    r.description =
        "Compressed-air pop with a density-ramped paper flutter and falling confetti.".into();
    r.master = MasterChain {
        highpass_hz: 70.0,
        presence_db: 2.5,
        presence_hz: 3800.0,
        comp_threshold_db: -14.0,
        comp_ratio: 2.5,
        drive_db: 1.5,
        reverb_wet: 0.22,
        reverb_size: 0.5,
        reverb_damp: 0.45,
        stereo_width: 1.3,
        target_peak_db: -1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "pop",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.020),
                filter: FilterEnv::new(9000.0, 3000.0, 0.004, 1.0),
                mode: 2,
                tame_hz: 13_000.0,
            },
        )
        .gain(-1.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "air_burst",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.16),
                filter: FilterEnv::new(5200.0, 900.0, 0.05, 0.9),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-6.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "paper_burst_dense",
            LayerKind::Grain {
                source_hz: 3200.0,
                grain_ms: 8.0,
                density: 520.0,
                spray: 0.6,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.005, 0.30),
                pitch_jitter: 0.35,
            },
        )
        .gain(-11.0)
        .pan(0.18)
        .style(0.8, 1.5),
    );
    r = r.layer(
        Layer::new(
            "paper_rain_sparse",
            LayerKind::Grain {
                source_hz: 2300.0,
                grain_ms: 14.0,
                density: 95.0,
                spray: 0.9,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.06, 0.85),
                pitch_jitter: 0.5,
            },
        )
        .gain(-15.0)
        .pan(-0.32)
        .delay(0.20)
        .style(0.8, 1.6),
    );
    r = r.layer(
        Layer::new(
            "paper_rustle",
            LayerKind::Noise {
                color: NoiseColor::Velvet,
                env: AdEnv::perc(0.75),
                filter: FilterEnv::new(2600.0, 1400.0, 0.20, 1.2),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-19.0)
        .pan(0.4)
        .delay(0.05)
        .style(0.9, 1.4),
    );
    r.layer(
        Layer::new(
            "sparkle",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(3100.0, 1.0, 0.09),
                    Mode::new(4700.0, 0.5, 0.07),
                    Mode::new(6400.0, 0.22, 0.05),
                ],
                exciter_ms: 0.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-24.0)
        .pan(-0.5)
        .delay(0.35)
        .style(0.7, 1.5),
    )
}

/// Plastic party horn: buzzy saw reed, two noise "formant" bands and breath.
fn party_horn() -> Recipe {
    let mut r = base("celebrate_party_horn", "Party Horn", Category::Celebration);
    r.duration = 0.8;
    r.fade_in = 0.0;
    r.fade_out = 0.04;
    r.style = 0.55;
    r.tags = vec![
        "celebration".into(),
        "horn".into(),
        "plastic".into(),
        "one-shot".into(),
    ];
    r.description =
        "Plastic party horn: buzzy saw reed through two formant bands, with breath and a wooden body.".into();
    r.master = MasterChain {
        highpass_hz: 90.0,
        presence_db: 3.0,
        presence_hz: 1800.0,
        comp_threshold_db: -16.0,
        comp_ratio: 3.0,
        drive_db: 3.0,
        reverb_wet: 0.14,
        reverb_size: 0.35,
        reverb_damp: 0.5,
        stereo_width: 1.1,
        target_peak_db: -1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "reed",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 245.0,
                end_hz: 208.0,
                pitch_tau: 0.12,
                env: AdEnv::swelled(0.02, 0.55),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.12,
            },
        )
        .gain(-4.0)
        .style(1.0, 1.35),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 122.5,
                end_hz: 104.0,
                pitch_tau: 0.15,
                env: AdEnv::swelled(0.02, 0.50),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.2,
                vibrato: 0.0,
            },
        )
        .gain(-13.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "formant_low",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.03, 0.50),
                filter: FilterEnv::new(1250.0, 980.0, 0.15, 3.5),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "formant_high",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.03, 0.45),
                filter: FilterEnv::new(2500.0, 2100.0, 0.15, 4.0),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-15.0)
        .style(1.0, 1.1),
    );
    r.layer(
        Layer::new(
            "breath",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.015, 0.40),
                filter: FilterEnv::new(4000.0, 2200.0, 0.10, 0.8),
                mode: 1,
                tame_hz: 11_000.0,
            },
        )
        .gain(-20.0)
        .style(1.0, 0.7),
    )
}

/// Long, sparse paper fall with a wide stereo image and a faint glassy tail.
fn confetti_rain() -> Recipe {
    let mut r = base(
        "celebrate_confetti_rain",
        "Confetti Rain",
        Category::Celebration,
    );
    r.duration = 3.0;
    r.fade_in = 0.02;
    r.fade_out = 0.12;
    r.style = 0.5;
    r.tags = vec![
        "celebration".into(),
        "confetti".into(),
        "long".into(),
        "wide".into(),
    ];
    r.description = "Three seconds of sparse falling paper, panned wide with a soft glassy shimmer.".into();
    r.master = MasterChain {
        highpass_hz: 110.0,
        presence_db: 1.5,
        presence_hz: 3200.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.0,
        drive_db: 0.5,
        reverb_wet: 0.30,
        reverb_size: 0.6,
        reverb_damp: 0.5,
        stereo_width: 1.45,
        target_peak_db: -1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "paper_high",
            LayerKind::Grain {
                source_hz: 3200.0,
                grain_ms: 12.0,
                density: 26.0,
                spray: 0.9,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.25, 2.40),
                pitch_jitter: 0.45,
            },
        )
        .gain(-14.0)
        .pan(-0.5)
        .style(0.8, 1.4),
    );
    r = r.layer(
        Layer::new(
            "paper_mid",
            LayerKind::Grain {
                source_hz: 2200.0,
                grain_ms: 18.0,
                density: 18.0,
                spray: 0.95,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.35, 2.20),
                pitch_jitter: 0.5,
            },
        )
        .gain(-15.0)
        .pan(0.45)
        .delay(0.12)
        .style(0.8, 1.5),
    );
    r = r.layer(
        Layer::new(
            "paper_low",
            LayerKind::Grain {
                source_hz: 1500.0,
                grain_ms: 26.0,
                density: 8.0,
                spray: 0.9,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.50, 2.00),
                pitch_jitter: 0.4,
            },
        )
        .gain(-19.0)
        .pan(0.05)
        .delay(0.35)
        .style(0.8, 1.5),
    );
    r = r.layer(
        Layer::new(
            "air_drift",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.35, 2.40),
                filter: FilterEnv::new(2600.0, 1200.0, 0.9, 1.0),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-24.0)
        .pan(0.2)
        .style(1.0, 0.9),
    );
    r.layer(
        Layer::new(
            "glint",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(4200.0, 1.0, 0.18),
                    Mode::new(6300.0, 0.4, 0.12),
                    Mode::new(8100.0, 0.15, 0.08),
                ],
                exciter_ms: 0.4,
                exciter_color: NoiseColor::White,
                exciter_hp: 3000.0,
                exciter_lp: 11_000.0,
                detune_cents: 4.0,
            },
        )
        .gain(-27.0)
        .pan(0.6)
        .delay(0.8)
        .style(0.7, 1.4),
    )
}

/// Cartoon three-note sting (C-E-G) with a support chord and sub.
fn fanfare_sting() -> Recipe {
    let mut r = base(
        "celebrate_fanfare_sting",
        "Fanfare Sting",
        Category::Celebration,
    );
    r.duration = 1.0;
    r.fade_in = 0.0;
    r.fade_out = 0.05;
    r.style = 0.6;
    r.tags = vec![
        "celebration".into(),
        "sting".into(),
        "fanfare".into(),
        "cartoon".into(),
    ];
    r.description = "Short cartoony C-E-G fanfare with a saw chord bed, sub and bell sparkle.".into();
    r.master = MasterChain {
        highpass_hz: 85.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 160.0,
        presence_db: 2.0,
        presence_hz: 2600.0,
        comp_threshold_db: -15.0,
        comp_ratio: 2.8,
        drive_db: 2.5,
        reverb_wet: 0.24,
        reverb_size: 0.5,
        reverb_damp: 0.45,
        stereo_width: 1.2,
        target_peak_db: -1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "note_c",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 523.25,
                end_hz: 523.25,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.004, 0.26),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-6.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "note_e",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 659.25,
                end_hz: 659.25,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.004, 0.26),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-6.0)
        .delay(0.13)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "note_g",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 783.99,
                end_hz: 783.99,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.005, 0.55),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-4.0)
        .delay(0.26)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "chord_bed",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 392.0,
                end_hz: 392.0,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.03, 0.55),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.1,
            },
        )
        .gain(-12.0)
        .delay(0.26)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "bell",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(1568.0, 1.0, 0.25),
                    Mode::new(2093.0, 0.55, 0.20),
                    Mode::new(3136.0, 0.25, 0.15),
                ],
                exciter_ms: 0.6,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 10_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-19.0)
        .pan(0.25)
        .delay(0.26)
        .style(0.7, 1.5),
    );
    r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 130.81,
                end_hz: 130.81,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.01, 0.40),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-12.0)
        .delay(0.26)
        .style(1.0, 1.4),
    )
}

/// A shower of seven tiny bright modal pings at hand-scattered times.
fn coin_burst() -> Recipe {
    let mut r = base("celebrate_coin_burst", "Coin Burst", Category::Celebration);
    r.duration = 0.9;
    r.fade_in = 0.0;
    r.fade_out = 0.04;
    r.style = 0.5;
    r.tags = vec![
        "celebration".into(),
        "coin".into(),
        "reward".into(),
        "sparkle".into(),
    ];
    r.description =
        "Seven tiny bright modal pings scattered across half a second, like a fistful of coins.".into();
    r.master = MasterChain {
        highpass_hz: 350.0,
        presence_db: 0.5,
        presence_hz: 4000.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.2,
        drive_db: 0.5,
        reverb_wet: 0.16,
        reverb_size: 0.35,
        reverb_damp: 0.5,
        stereo_width: 1.3,
        target_peak_db: -1.0,
        ..Default::default()
    };
    let pings = [
        ("ping_a", 1720.0, 0.16, 0.000, -7.0, -0.55),
        ("ping_b", 1960.0, 0.13, 0.045, -9.0, 0.50),
        ("ping_c", 2340.0, 0.11, 0.090, -8.0, -0.25),
        ("ping_d", 1480.0, 0.18, 0.140, -11.0, 0.30),
        ("ping_e", 2610.0, 0.09, 0.200, -10.0, -0.05),
        ("ping_f", 2090.0, 0.12, 0.270, -12.0, 0.60),
        ("ping_g", 2870.0, 0.08, 0.350, -14.0, -0.40),
    ];
    for (name, f, d, delay, gain, pan) in pings {
        r = r.layer(
            Layer::new(
                name,
                LayerKind::Modal {
                    modes: coin_modes(f, d),
                    exciter_ms: 0.5,
                    exciter_color: NoiseColor::White,
                    exciter_hp: 3000.0,
                    exciter_lp: 12_000.0,
                    detune_cents: 0.0,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(delay)
            .style(0.7, 1.4),
        );
    }
    r
}

fn coin_modes(base: f32, decay: f32) -> Vec<Mode> {
    vec![
        Mode::new(base, 1.0, decay),
        Mode::new(base * 2.32, 0.45, decay * 0.7),
        Mode::new(base * 4.25, 0.20, decay * 0.45),
        Mode::new(base * 6.63, 0.07, decay * 0.3),
    ]
}

/// Riser into a bright major chord: sweep, granular shimmer, three saws, sub.
fn level_up() -> Recipe {
    let mut r = base("celebrate_level_up", "Level Up", Category::Celebration);
    r.duration = 1.7;
    r.fade_in = 0.01;
    r.fade_out = 0.08;
    r.style = 0.55;
    r.tags = vec![
        "celebration".into(),
        "level_up".into(),
        "riser".into(),
        "sting".into(),
    ];
    r.description = "A short noise riser and granular shimmer resolving into a bright C major chord.".into();
    r.master = MasterChain {
        highpass_hz: 70.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 140.0,
        presence_db: 2.0,
        presence_hz: 3000.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.0,
        drive_db: 2.0,
        reverb_wet: 0.26,
        reverb_size: 0.6,
        reverb_damp: 0.45,
        stereo_width: 1.3,
        target_peak_db: -1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "riser",
            LayerKind::Sweep {
                start_hz: 280.0,
                end_hz: 4200.0,
                tau: 0.60,
                q: 2.2,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.50, 0.35),
                tone_mix: 0.45,
            },
        )
        .gain(-10.0)
        .style(0.9, 1.4),
    );
    r = r.layer(
        Layer::new(
            "shimmer",
            LayerKind::Grain {
                source_hz: 5200.0,
                grain_ms: 18.0,
                density: 140.0,
                spray: 0.8,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.35, 1.00),
                pitch_jitter: 0.3,
            },
        )
        .gain(-16.0)
        .pan(0.25)
        .style(0.8, 1.5),
    );
    for (name, hz, gain, pan) in [
        ("chord_c", 523.25_f32, -6.0_f32, -0.3_f32),
        ("chord_e", 659.25, -7.0, 0.0),
        ("chord_g", 783.99, -7.0, 0.3),
    ] {
        r = r.layer(
            Layer::new(
                name,
                LayerKind::Tone {
                    shape: WaveShape::Saw,
                    start_hz: hz,
                    end_hz: hz,
                    pitch_tau: 1.0,
                    env: AdEnv::swelled(0.01, 0.85),
                    fm_hz: 0.0,
                    fm_depth: 0.0,
                    drive: 0.2,
                    vibrato: 0.08,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(0.62)
            .style(1.0, 1.3),
        );
    }
    r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 65.4,
                end_hz: 65.4,
                pitch_tau: 1.0,
                env: AdEnv::swelled(0.01, 0.70),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .delay(0.62)
        .style(1.0, 1.5),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celebration_presets_render_cleanly() {
        for r in presets() {
            let buf = sg_engine::render(&r, 42);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.05, "{} too short", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak < 1.0, "{} clips (peak {peak})", r.id);
            assert!(
                buf.samples.iter().all(|s| s.is_finite()),
                "{} has NaN",
                r.id
            );
            println!(
                "{}: {:.3}s peak {:.2} dBFS rms {:.4} lufs {:.1} layers {}",
                r.id,
                buf.duration(),
                sg_dsp::gain_to_db(peak),
                buf.rms(),
                buf.lufs_approx(),
                r.layers.len()
            );
        }
    }
}
