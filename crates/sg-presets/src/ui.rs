//! UI presets: short, clean, dry and deliberately quiet in the mix.
//!
//! Contract for this category: no layer is longer than ~0.35 s except the two
//! reward cues, reverb stays below 0.1 wet, presence boosts stay under 4 kHz so
//! nothing fizzes at 8 kHz, and the master sits 3-6 dB below the SFX bus.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        click_soft(),
        click_hard(),
        hover_tick(),
        coin_pickup(),
        purchase(),
        error_buzz(),
        level_up_short(),
        menu_whoosh(),
    ]
}

/// Quiet, rounded click for list rows and secondary buttons.
fn click_soft() -> Recipe {
    let mut r = base("ui_click_soft", "UI Click Soft", Category::Ui);
    r.duration = 0.13;
    r.fade_in = 0.0;
    r.fade_out = 0.008;
    r.style = 0.2;
    r.tags = vec![
        "ui".into(),
        "click".into(),
        "soft".into(),
        "button".into(),
    ];
    r.description = "Soft rounded UI click: a short band-passed tick over a tiny plastic resonance.".into();
    r.master = MasterChain {
        highpass_hz: 180.0,
        presence_db: 1.0,
        presence_hz: 2600.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.04,
        reverb_size: 0.2,
        stereo_width: 1.0,
        target_peak_db: -5.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "tick",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.010),
                filter: FilterEnv::new(2600.0, 1600.0, 0.004, 1.6),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(1750.0, 1.0, 0.030),
                    Mode::new(2900.0, 0.45, 0.020),
                    Mode::new(4100.0, 0.18, 0.012),
                ],
                exciter_ms: 0.2,
                exciter_color: NoiseColor::White,
                exciter_hp: 1500.0,
                exciter_lp: 8000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "thump",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 520.0,
                end_hz: 330.0,
                pitch_tau: 0.010,
                env: AdEnv::perc(0.035),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-18.0)
        .style(1.0, 1.4),
    )
}

/// Firm, snappy click for confirm / primary actions.
fn click_hard() -> Recipe {
    let mut r = base("ui_click_hard", "UI Click Hard", Category::Ui);
    r.duration = 0.20;
    r.fade_in = 0.0;
    r.fade_out = 0.010;
    r.style = 0.25;
    r.tags = vec![
        "ui".into(),
        "click".into(),
        "hard".into(),
        "confirm".into(),
    ];
    r.description = "Firm UI click with a bright snap, short metal ring and a small low body.".into();
    r.master = MasterChain {
        highpass_hz: 90.0,
        presence_db: 1.5,
        presence_hz: 3000.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.0,
        drive_db: 2.0,
        reverb_wet: 0.06,
        reverb_size: 0.25,
        stereo_width: 1.0,
        target_peak_db: -3.5,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "snap",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.006),
                filter: FilterEnv::new(6000.0, 2800.0, 0.003, 1.2),
                mode: 2,
                tame_hz: 12_000.0,
            },
        )
        .gain(-3.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "ring",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(1450.0, 1.0, 0.050),
                    Mode::new(1450.0 * 2.756, 0.55, 0.038),
                    Mode::new(1450.0 * 5.404, 0.28, 0.026),
                    Mode::new(1450.0 * 8.933, 0.12, 0.016),
                ],
                exciter_ms: 0.3,
                exciter_color: NoiseColor::White,
                exciter_hp: 900.0,
                exciter_lp: 10_000.0,
                detune_cents: 3.0,
            },
        )
        .gain(-9.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 320.0,
                end_hz: 170.0,
                pitch_tau: 0.020,
                env: AdEnv::perc(0.055),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.4,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .style(1.0, 1.35),
    );
    r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 95.0,
                end_hz: 60.0,
                pitch_tau: 0.030,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-14.0)
        .style(1.0, 1.4),
    )
}

/// Very short, very quiet tick used on hover.
fn hover_tick() -> Recipe {
    let mut r = base("ui_hover_tick", "UI Hover Tick", Category::Ui);
    r.duration = 0.07;
    r.fade_in = 0.0;
    r.fade_out = 0.005;
    r.style = 0.15;
    r.tags = vec!["ui".into(), "hover".into(), "tick".into(), "subtle".into()];
    r.description = "Barely-there hover tick: 6 ms of high band noise plus a tiny ping.".into();
    r.master = MasterChain {
        highpass_hz: 500.0,
        presence_db: 0.0,
        presence_hz: 3000.0,
        comp_threshold_db: -20.0,
        comp_ratio: 1.8,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.2,
        stereo_width: 1.0,
        target_peak_db: -6.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "tick",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.005),
                filter: FilterEnv::new(3600.0, 3000.0, 0.003, 2.2),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-10.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "ping",
            LayerKind::Modal {
                modes: vec![Mode::new(2800.0, 1.0, 0.015), Mode::new(4300.0, 0.3, 0.010)],
                exciter_ms: 0.1,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-16.0)
        .style(1.0, 1.2),
    );
    r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 900.0,
                end_hz: 700.0,
                pitch_tau: 0.008,
                env: AdEnv::perc(0.012),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-26.0)
        .style(1.0, 1.2),
    )
}

/// Two-note coin reward (B5 -> E6) with a click and a short shimmer.
fn coin_pickup() -> Recipe {
    let mut r = base("ui_coin_pickup", "UI Coin Pickup", Category::Ui);
    r.duration = 0.50;
    r.fade_in = 0.0;
    r.fade_out = 0.015;
    r.style = 0.3;
    r.tags = vec![
        "ui".into(),
        "coin".into(),
        "reward".into(),
        "pickup".into(),
    ];
    r.description = "Two-note B5-E6 coin pickup with a bright click and a short overtone shimmer.".into();
    r.master = MasterChain {
        highpass_hz: 380.0,
        presence_db: 1.0,
        presence_hz: 3500.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 0.5,
        reverb_wet: 0.08,
        reverb_size: 0.3,
        stereo_width: 1.05,
        target_peak_db: -4.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.004),
                filter: FilterEnv::new(6000.0, 4200.0, 0.003, 1.4),
                mode: 2,
                tame_hz: 12_000.0,
            },
        )
        .gain(-16.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "note_b5",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 987.77,
                end_hz: 987.77,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.0,
            },
        )
        .gain(-6.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "note_e6",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 1318.51,
                end_hz: 1318.51,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.280),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.0,
            },
        )
        .gain(-5.0)
        .delay(0.075)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "overtone",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 2637.0,
                end_hz: 2637.0,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.160),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-18.0)
        .delay(0.075)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "shimmer",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(1976.0, 1.0, 0.220),
                    Mode::new(2637.0, 0.45, 0.160),
                    Mode::new(3951.0, 0.18, 0.100),
                ],
                exciter_ms: 0.3,
                exciter_color: NoiseColor::White,
                exciter_hp: 1800.0,
                exciter_lp: 10_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-20.0)
        .delay(0.080)
        .style(1.0, 1.4),
    )
}

/// Shop purchase: low thunk, rising two-note chime and a receipt rustle.
fn purchase() -> Recipe {
    let mut r = base("ui_purchase", "UI Purchase", Category::Ui);
    r.duration = 0.55;
    r.fade_in = 0.0;
    r.fade_out = 0.020;
    r.style = 0.3;
    r.tags = vec![
        "ui".into(),
        "purchase".into(),
        "shop".into(),
        "reward".into(),
    ];
    r.description = "Shop purchase confirm: a soft low thunk under a C6-G6 chime with a paper rustle.".into();
    r.master = MasterChain {
        highpass_hz: 200.0,
        presence_db: 0.8,
        presence_hz: 3200.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 0.5,
        reverb_wet: 0.09,
        reverb_size: 0.3,
        stereo_width: 1.05,
        target_peak_db: -4.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "thunk",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 170.0,
                end_hz: 110.0,
                pitch_tau: 0.040,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.1,
                vibrato: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "chime_c6",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 1046.5,
                end_hz: 1046.5,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.160),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "chime_g6",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 1568.0,
                end_hz: 1568.0,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.300),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-7.0)
        .delay(0.090)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "sparkle",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(3136.0, 1.0, 0.180),
                    Mode::new(4186.0, 0.45, 0.120),
                    Mode::new(5274.0, 0.18, 0.080),
                ],
                exciter_ms: 0.3,
                exciter_color: NoiseColor::White,
                exciter_hp: 2500.0,
                exciter_lp: 11_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-20.0)
        .pan(0.3)
        .delay(0.090)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "receipt",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.050),
                filter: FilterEnv::new(2800.0, 1800.0, 0.02, 1.0),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-22.0)
        .pan(-0.2)
        .delay(0.020)
        .style(1.0, 1.2),
    )
}

/// Dissonant two-tone buzz for invalid actions.
fn error_buzz() -> Recipe {
    let mut r = base("ui_error_buzz", "UI Error Buzz", Category::Ui);
    r.duration = 0.35;
    r.fade_in = 0.0;
    r.fade_out = 0.015;
    r.style = 0.3;
    r.tags = vec![
        "ui".into(),
        "error".into(),
        "buzz".into(),
        "negative".into(),
    ];
    r.description = "Dissonant low buzz with a second detuned tone, a sub and a small click attack.".into();
    r.master = MasterChain {
        highpass_hz: 100.0,
        presence_db: -1.0,
        presence_hz: 2500.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.0,
        drive_db: 1.5,
        reverb_wet: 0.04,
        reverb_size: 0.25,
        stereo_width: 1.0,
        target_peak_db: -4.5,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "buzz",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 150.0,
                end_hz: 120.0,
                pitch_tau: 0.120,
                env: AdEnv::swelled(0.005, 0.240),
                fm_hz: 1.0,
                fm_depth: 0.08,
                drive: 0.5,
                vibrato: 0.0,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "buzz_second",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 106.0,
                end_hz: 95.0,
                pitch_tau: 0.100,
                env: AdEnv::swelled(0.005, 0.200),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.4,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .delay(0.090)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 72.0,
                end_hz: 72.0,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.280),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.5),
    );
    r.layer(
        Layer::new(
            "click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.008),
                filter: FilterEnv::new(1400.0, 900.0, 0.004, 1.2),
                mode: 1,
                tame_hz: 6000.0,
            },
        )
        .gain(-18.0)
        .style(1.0, 1.1),
    )
}

/// Compact three-note level-up for in-game rewards (kept under 0.5 s).
fn level_up_short() -> Recipe {
    let mut r = base("ui_level_up_short", "UI Level Up Short", Category::Ui);
    r.duration = 0.50;
    r.fade_in = 0.0;
    r.fade_out = 0.020;
    r.style = 0.35;
    r.tags = vec![
        "ui".into(),
        "level_up".into(),
        "reward".into(),
        "arpeggio".into(),
    ];
    r.description = "Fast C6-E6-G6 triangle arpeggio with granular shimmer and a soft sub anchor.".into();
    r.master = MasterChain {
        highpass_hz: 250.0,
        presence_db: 0.8,
        presence_hz: 3000.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 0.5,
        reverb_wet: 0.09,
        reverb_size: 0.3,
        stereo_width: 1.0,
        target_peak_db: -4.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "note_c6",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 1046.5,
                end_hz: 1046.5,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.120),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-9.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "note_e6",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 1318.5,
                end_hz: 1318.5,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.120),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-9.0)
        .delay(0.060)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "note_g6",
            LayerKind::Tone {
                shape: WaveShape::Triangle,
                start_hz: 1568.0,
                end_hz: 1568.0,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.280),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-6.0)
        .delay(0.120)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "shimmer",
            LayerKind::Grain {
                source_hz: 4200.0,
                grain_ms: 10.0,
                density: 180.0,
                spray: 0.7,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.020, 0.250),
                pitch_jitter: 0.25,
            },
        )
        .gain(-22.0)
        .pan(0.2)
        .style(0.9, 1.4),
    );
    r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 261.6,
                end_hz: 261.6,
                pitch_tau: 1.0,
                env: AdEnv::perc(0.250),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-18.0)
        .delay(0.120)
        .style(1.0, 1.4),
    )
}

/// Menu transition whoosh: rising band-passed noise with a low bed and air.
fn menu_whoosh() -> Recipe {
    let mut r = base("ui_menu_whoosh", "UI Menu Whoosh", Category::Ui);
    r.duration = 0.28;
    r.fade_in = 0.0;
    r.fade_out = 0.020;
    r.style = 0.25;
    r.tags = vec![
        "ui".into(),
        "whoosh".into(),
        "transition".into(),
        "menu".into(),
    ];
    r.description = "Short menu whoosh: band-passed noise sweeping upward over a soft low bed.".into();
    r.master = MasterChain {
        highpass_hz: 80.0,
        presence_db: 0.5,
        presence_hz: 2800.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.08,
        reverb_size: 0.3,
        stereo_width: 1.1,
        target_peak_db: -5.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "core",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.050, 0.220),
                filter: FilterEnv::new(260.0, 3600.0, 0.280, 1.8),
                mode: 1,
                tame_hz: 11_000.0,
            },
        )
        .gain(-6.0)
        .style(0.9, 1.3),
    );
    r = r.layer(
        Layer::new(
            "low_bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.040, 0.190),
                filter: FilterEnv::new(180.0, 120.0, 0.200, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-14.0)
        .style(1.0, 1.2),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.020, 0.160),
                filter: FilterEnv::new(5500.0, 3000.0, 0.100, 0.8),
                mode: 2,
                tame_hz: 9000.0,
            },
        )
        .gain(-24.0)
        .pan(0.15)
        .style(1.0, 0.8),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_presets_render_cleanly() {
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
