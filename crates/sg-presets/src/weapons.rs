//! Weapon presets: gunshots, suppressed shots, mechanical handling and brass.
//!
//! Layer roles follow the research recipes (docs/research/01-dsp-sentez.md §3):
//! crack = the supersonic transient, body = the low pitch-dropping pressure
//! wave, tail = the room/outdoor reflection, mech/debris = the small stuff that
//! sells scale. Cartoon mode exaggerates the pitch drop and the tail and thins
//! out the mechanical detail.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        pistol_fire(),
        rifle_fire(),
        shotgun_fire(),
        smg_fire(),
        suppressed_fire(),
        dry_fire(),
        reload_mag(),
        casing_drop(),
        bolt_pull(),
        shell_insert(),
    ]
}

// ---------------------------------------------------------------------------
// modal helpers — struck bodies built from measured material tables
// (`sg_dsp::modal::{STEEL, OAK, ...}`). The pickup comb inside the table is
// what makes the body sound struck rather than filtered.
// ---------------------------------------------------------------------------

/// Steel receiver / barrel / casing: 12 modes, 0.72 damping slope.
fn metal_modes(base: f32, decay: f32) -> Vec<Mode> {
    sg_dsp::modal::STEEL.modes_with_t60(base, decay, 0.5)
}

/// Oak plank — crate walls, stocks, pallets.
fn wood_modes(base: f32, decay: f32) -> Vec<Mode> {
    sg_dsp::modal::OAK.modes_with_t60(base, decay, 0.5)
}

// ---------------------------------------------------------------------------
// gunshots
// ---------------------------------------------------------------------------

fn pistol_fire() -> Recipe {
    let mut r = base("weapon_pistol_fire", "Pistol Fire", Category::Weapon);
    r.duration = 0.40;
    r.fade_out = 0.03;
    r.style = 0.30;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "pistol".into(),
        "9mm".into(),
        "shot".into(),
        "small".into(),
    ];
    r.description =
        "Tight 9 mm report: 3 ms supersonic crack over an 80-160 Hz pressure drop and a short room tail.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 120.0,
        presence_db: 2.5,
        presence_hz: 3200.0,
        comp_threshold_db: -12.0,
        comp_ratio: 3.0,
        drive_db: 3.0,
        reverb_wet: 0.16,
        reverb_size: 0.32,
        reverb_damp: 0.45,
        stereo_width: 1.10,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.011),
                filter: FilterEnv::new(7500.0, 2400.0, 0.005, 0.9),
                mode: 2,
                tame_hz: 12_000.0,
            },
        )
        .gain(2.0)
        .style(1.0, 0.75),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 165.0,
                end_hz: 78.0,
                pitch_tau: 0.011,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.0,
            },
        )
        .gain(-4.0)
        .style(1.0, 1.45),
    );
    r = r.layer(
        Layer::new(
            "thump",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.075),
                filter: FilterEnv::new(240.0, 110.0, 0.035, 0.9),
                mode: 0,
                tame_hz: 900.0,
            },
        )
        .gain(-3.0)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "mechanism",
            LayerKind::Modal {
                modes: metal_modes(2800.0, 0.045),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-22.0)
        .pan(0.15)
        .delay(0.004)
        .style(1.0, 0.45),
    );
    r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.32),
                filter: FilterEnv::new(900.0, 320.0, 0.12, 0.7),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-4.0)
        .pan(-0.08)
        .delay(0.012)
        .style(0.7, 1.60),
    )
}

fn rifle_fire() -> Recipe {
    let mut r = base("weapon_rifle_fire", "Rifle Fire", Category::Weapon);
    r.duration = 0.72;
    r.fade_out = 0.04;
    r.style = 0.28;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "rifle".into(),
        "5.56".into(),
        "shot".into(),
        "loud".into(),
    ];
    r.description =
        "High-velocity rifle shot with a bright crack, a 14 ms low-end drop and a 650 ms valley tail.".into();
    r.master = MasterChain {
        highpass_hz: 40.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 110.0,
        presence_db: 3.0,
        presence_hz: 3800.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.5,
        drive_db: 4.0,
        reverb_wet: 0.22,
        reverb_size: 0.55,
        reverb_damp: 0.40,
        stereo_width: 1.15,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.009),
                filter: FilterEnv::new(5200.0, 1800.0, 0.005, 0.65),
                mode: 2,
                tame_hz: 14_000.0,
            },
        )
        .gain(3.0)
        .style(1.0, 0.80),
    );
    r = r.layer(
        Layer::new(
            "snap",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.004),
                filter: FilterEnv::new(9000.0, 6200.0, 0.003, 0.8),
                mode: 2,
                tame_hz: 16_000.0,
            },
        )
        .gain(-4.0)
        .delay(0.001)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 221.0,
                end_hz: 85.0,
                pitch_tau: 0.014,
                env: AdEnv::perc(0.075),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.40,
                vibrato: 0.0,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "blast",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.160),
                filter: FilterEnv::new(1200.0, 400.0, 0.070, 0.55),
                mode: 0,
                tame_hz: 4000.0,
            },
        )
        .gain(3.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.65),
                filter: FilterEnv::new(500.0, 220.0, 0.25, 0.5),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(0.0)
        .delay(0.020)
        .style(0.7, 1.70),
    );
    r.layer(
        Layer::new(
            "shell rattle",
            LayerKind::Modal {
                modes: metal_modes(2200.0, 0.120),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1500.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-30.0)
        .pan(0.35)
        .delay(0.085)
        .style(1.0, 0.40),
    )
}

fn shotgun_fire() -> Recipe {
    let mut r = base("weapon_shotgun_fire", "Shotgun Fire", Category::Weapon);
    r.duration = 0.95;
    r.fade_out = 0.05;
    r.style = 0.35;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "shotgun".into(),
        "12ga".into(),
        "boom".into(),
        "low".into(),
    ];
    r.description =
        "Wide 12-gauge boom: a 62 Hz body, 220 ms of brown-noise blast, pellet debris and a long outdoor tail.".into();
    r.master = MasterChain {
        highpass_hz: 32.0,
        low_shelf_db: 3.0,
        low_shelf_hz: 90.0,
        presence_db: 1.5,
        presence_hz: 2500.0,
        comp_threshold_db: -16.0,
        comp_ratio: 4.0,
        drive_db: 5.0,
        reverb_wet: 0.24,
        reverb_size: 0.60,
        reverb_damp: 0.50,
        stereo_width: 1.20,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "boom",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.220),
                filter: FilterEnv::new(420.0, 150.0, 0.060, 0.75),
                mode: 0,
                tame_hz: 2200.0,
            },
        )
        .gain(7.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.035),
                filter: FilterEnv::new(3000.0, 1200.0, 0.020, 0.55),
                mode: 1,
                tame_hz: 12_000.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 0.85),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 112.0,
                end_hz: 62.0,
                pitch_tau: 0.025,
                env: AdEnv::perc(0.140),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.50,
                vibrato: 0.0,
            },
        )
        .gain(-4.0)
        .style(1.0, 1.50),
    );
    r = r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 58.0,
                end_hz: 40.0,
                pitch_tau: 0.040,
                env: AdEnv::perc(0.280),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "pellet debris",
            LayerKind::Grain {
                source_hz: 1800.0,
                grain_ms: 25.0,
                density: 90.0,
                spray: 0.60,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.005, 0.50),
                pitch_jitter: 0.30,
            },
        )
        .gain(-2.0)
        .pan(-0.25)
        .delay(0.030)
        .style(0.6, 1.50),
    );
    r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.60),
                filter: FilterEnv::new(700.0, 200.0, 0.30, 0.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(4.0)
        .pan(0.10)
        .delay(0.020)
        .style(0.7, 1.60),
    )
}

fn smg_fire() -> Recipe {
    let mut r = base("weapon_smg_fire", "SMG Fire", Category::Weapon);
    r.duration = 0.28;
    r.fade_out = 0.025;
    r.style = 0.30;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "smg".into(),
        "auto".into(),
        "tight".into(),
        "metallic".into(),
    ];
    r.description =
        "Fast, tight submachine-gun shot: hard high crack, metallic receiver ring and almost no room.".into();
    r.master = MasterChain {
        highpass_hz: 60.0,
        low_shelf_db: 0.5,
        low_shelf_hz: 130.0,
        presence_db: 3.5,
        presence_hz: 4200.0,
        comp_threshold_db: -11.0,
        comp_ratio: 2.5,
        drive_db: 3.5,
        reverb_wet: 0.12,
        reverb_size: 0.28,
        reverb_damp: 0.45,
        stereo_width: 1.05,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.007),
                filter: FilterEnv::new(8500.0, 3800.0, 0.004, 0.8),
                mode: 2,
                tame_hz: 15_000.0,
            },
        )
        .gain(3.0)
        .style(1.0, 0.70),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 200.0,
                end_hz: 110.0,
                pitch_tau: 0.008,
                env: AdEnv::perc(0.055),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.45,
                vibrato: 0.0,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "receiver",
            LayerKind::Modal {
                modes: metal_modes(3400.0, 0.035),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-18.0)
        .pan(0.20)
        .style(1.0, 0.45),
    );
    r = r.layer(
        Layer::new(
            "thump",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.060),
                filter: FilterEnv::new(280.0, 120.0, 0.030, 0.9),
                mode: 0,
                tame_hz: 900.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 1.20),
    );
    r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.18),
                filter: FilterEnv::new(1400.0, 500.0, 0.050, 0.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-4.0)
        .pan(-0.12)
        .delay(0.010)
        .style(0.6, 1.40),
    )
}

fn suppressed_fire() -> Recipe {
    let mut r = base(
        "weapon_suppressed_fire",
        "Suppressed Pistol Fire",
        Category::Weapon,
    );
    r.duration = 0.42;
    r.fade_out = 0.035;
    r.style = 0.35;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "suppressed".into(),
        "silencer".into(),
        "stealth".into(),
        "quiet".into(),
    ];
    r.description =
        "Suppressed shot: a low gas thump and mechanical action instead of a crack, with a filtered room tail.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 120.0,
        presence_db: -2.0,
        presence_hz: 3000.0,
        comp_threshold_db: -18.0,
        comp_ratio: 3.0,
        drive_db: 1.5,
        reverb_wet: 0.18,
        reverb_size: 0.40,
        reverb_damp: 0.55,
        stereo_width: 1.05,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "pfft",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.040),
                filter: FilterEnv::new(1600.0, 700.0, 0.030, 0.8),
                mode: 1,
                tame_hz: 6000.0,
            },
        )
        .gain(4.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "thump",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 130.0,
                end_hz: 70.0,
                pitch_tau: 0.020,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.30,
                vibrato: 0.0,
            },
        )
        .gain(-3.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "action",
            LayerKind::Modal {
                modes: metal_modes(1900.0, 0.060),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 900.0,
                exciter_lp: 7000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-21.0)
        .pan(0.18)
        .delay(0.003)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "gas",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.130),
                filter: FilterEnv::new(900.0, 420.0, 0.060, 1.4),
                mode: 1,
                tame_hz: 3000.0,
            },
        )
        .gain(2.0)
        .pan(-0.10)
        .delay(0.008)
        .style(0.7, 1.50),
    );
    r.layer(
        Layer::new(
            "tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.30),
                filter: FilterEnv::new(1200.0, 300.0, 0.12, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-6.0)
        .delay(0.020)
        .style(0.7, 1.50),
    )
}

// ---------------------------------------------------------------------------
// handling
// ---------------------------------------------------------------------------

fn dry_fire() -> Recipe {
    let mut r = base("weapon_dry_fire", "Dry Fire", Category::Weapon);
    r.duration = 0.16;
    r.fade_out = 0.02;
    r.style = 0.40;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "click".into(),
        "empty".into(),
        "firing-pin".into(),
        "mechanical".into(),
    ];
    r.description =
        "Empty-chamber click: a sharp firing-pin tick, a tiny metallic modal and the hammer settling.".into();
    r.master = MasterChain {
        highpass_hz: 120.0,
        presence_db: 3.0,
        presence_hz: 4000.0,
        comp_threshold_db: -10.0,
        comp_ratio: 2.0,
        drive_db: 2.0,
        reverb_wet: 0.10,
        reverb_size: 0.20,
        reverb_damp: 0.50,
        stereo_width: 1.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "firing pin",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.006),
                filter: FilterEnv::new(6000.0, 3500.0, 0.002, 1.2),
                mode: 2,
                tame_hz: 15_000.0,
            },
        )
        .gain(2.0)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "pin ring",
            LayerKind::Modal {
                modes: metal_modes(3100.0, 0.030),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-14.0)
        .style(1.0, 0.50),
    );
    r = r.layer(
        Layer::new(
            "hammer body",
            LayerKind::Modal {
                modes: wood_modes(420.0, 0.045),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 300.0,
                exciter_lp: 4000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-32.0)
        .pan(-0.10)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "action scrape",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.030),
                filter: FilterEnv::new(2200.0, 900.0, 0.015, 1.2),
                mode: 1,
                tame_hz: 8000.0,
            },
        )
        .gain(2.0)
        .delay(0.004)
        .style(1.0, 0.60),
    );
    r.layer(
        Layer::new(
            "spring",
            LayerKind::Modal {
                modes: metal_modes(640.0, 0.090),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 400.0,
                exciter_lp: 5000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-34.0)
        .pan(0.20)
        .delay(0.010)
        .style(1.0, 0.40),
    )
}

fn reload_mag() -> Recipe {
    let mut r = base("weapon_reload_mag", "Magazine Reload", Category::Weapon);
    r.duration = 0.62;
    r.fade_out = 0.03;
    r.style = 0.35;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "reload".into(),
        "magazine".into(),
        "foley".into(),
        "mechanical".into(),
    ];
    r.description =
        "Magazine out then in: release click, slide scrape, a solid seat thud and a latch snap 330 ms later.".into();
    r.master = MasterChain {
        highpass_hz: 70.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 140.0,
        presence_db: 2.0,
        presence_hz: 3500.0,
        comp_threshold_db: -12.0,
        comp_ratio: 2.5,
        drive_db: 2.5,
        reverb_wet: 0.15,
        reverb_size: 0.30,
        reverb_damp: 0.45,
        stereo_width: 1.10,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "release click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.010),
                filter: FilterEnv::new(3200.0, 2000.0, 0.005, 1.2),
                mode: 1,
                tame_hz: 12_000.0,
            },
        )
        .gain(-4.0)
        .style(1.0, 0.65),
    );
    r = r.layer(
        Layer::new(
            "mag out",
            LayerKind::Modal {
                modes: metal_modes(2500.0, 0.050),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1500.0,
                exciter_lp: 10_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-10.0)
        .pan(-0.15)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "slide scrape",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.010, 0.070),
                filter: FilterEnv::new(2200.0, 1200.0, 0.050, 2.2),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-7.0)
        .pan(-0.10)
        .delay(0.020)
        .style(0.7, 1.40),
    );
    r = r.layer(
        Layer::new(
            "mag seat",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 180.0,
                end_hz: 90.0,
                pitch_tau: 0.010,
                env: AdEnv::perc(0.060),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.25,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .delay(0.260)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "mag in",
            LayerKind::Modal {
                modes: metal_modes(2100.0, 0.070),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-11.0)
        .pan(0.10)
        .delay(0.260)
        .style(1.0, 0.60),
    );
    r.layer(
        Layer::new(
            "latch",
            LayerKind::Modal {
                modes: metal_modes(3300.0, 0.030),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 13_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-15.0)
        .pan(-0.20)
        .delay(0.330)
        .style(1.0, 0.40),
    )
}

fn casing_drop() -> Recipe {
    let mut r = base("weapon_casing_drop", "Casing Drop", Category::Weapon);
    r.duration = 0.70;
    r.fade_out = 0.04;
    r.style = 0.40;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "brass".into(),
        "casing".into(),
        "bounce".into(),
        "foley".into(),
    ];
    r.description =
        "Brass casing hitting concrete: a bright tink and four smaller bounces that thin out and spread in stereo.".into();
    r.master = MasterChain {
        highpass_hz: 300.0,
        presence_db: 4.0,
        presence_hz: 5000.0,
        comp_threshold_db: -14.0,
        comp_ratio: 2.0,
        drive_db: 1.5,
        reverb_wet: 0.20,
        reverb_size: 0.35,
        reverb_damp: 0.40,
        stereo_width: 1.30,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "impact tick",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.003),
                filter: FilterEnv::new(6500.0, 4000.0, 0.0015, 1.0),
                mode: 2,
                tame_hz: 16_000.0,
            },
        )
        .gain(-4.0)
        .style(1.0, 0.70),
    );
    r = r.layer(
        Layer::new(
            "tink",
            LayerKind::Modal {
                modes: metal_modes(2600.0, 0.160),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 14_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-8.0)
        .pan(0.10)
        .style(1.0, 0.65),
    );
    r = r.layer(
        Layer::new(
            "bounce 2",
            LayerKind::Modal {
                modes: metal_modes(2900.0, 0.090),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2200.0,
                exciter_lp: 14_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-14.0)
        .pan(-0.20)
        .delay(0.140)
        .style(1.0, 0.70),
    );
    r = r.layer(
        Layer::new(
            "bounce 3",
            LayerKind::Modal {
                modes: metal_modes(2400.0, 0.060),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1800.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-19.0)
        .pan(0.25)
        .delay(0.240)
        .style(1.0, 0.80),
    );
    r = r.layer(
        Layer::new(
            "bounce 4",
            LayerKind::Modal {
                modes: metal_modes(3100.0, 0.040),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2400.0,
                exciter_lp: 14_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-24.0)
        .pan(-0.10)
        .delay(0.310)
        .style(1.0, 0.80),
    );
    r.layer(
        Layer::new(
            "bounce 5",
            LayerKind::Modal {
                modes: metal_modes(2200.0, 0.030),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1800.0,
                exciter_lp: 10_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-28.0)
        .pan(0.05)
        .delay(0.360)
        .style(1.0, 0.90),
    )
}

fn bolt_pull() -> Recipe {
    let mut r = base("weapon_bolt_pull", "Bolt Pull", Category::Weapon);
    r.duration = 0.60;
    r.fade_out = 0.03;
    r.style = 0.35;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "bolt".into(),
        "rifle".into(),
        "foley".into(),
        "mechanical".into(),
    ];
    r.description =
        "Bolt-action cycle: unlock click, metal slide back, a heavy clack, the return scrape and the final lock.".into();
    r.master = MasterChain {
        highpass_hz: 80.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 150.0,
        presence_db: 2.5,
        presence_hz: 3800.0,
        comp_threshold_db: -13.0,
        comp_ratio: 2.5,
        drive_db: 2.5,
        reverb_wet: 0.14,
        reverb_size: 0.30,
        reverb_damp: 0.45,
        stereo_width: 1.15,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "unlock",
            LayerKind::Modal {
                modes: metal_modes(3000.0, 0.030),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2000.0,
                exciter_lp: 13_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-10.0)
        .style(1.0, 0.55),
    );
    r = r.layer(
        Layer::new(
            "slide back",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.010, 0.090),
                filter: FilterEnv::new(2600.0, 1400.0, 0.060, 2.5),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-2.0)
        .pan(-0.20)
        .delay(0.020)
        .style(0.7, 1.40),
    );
    r = r.layer(
        Layer::new(
            "bolt clack",
            LayerKind::Modal {
                modes: metal_modes(1800.0, 0.080),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1000.0,
                exciter_lp: 8000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-9.0)
        .pan(0.20)
        .delay(0.120)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "return",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.010, 0.080),
                filter: FilterEnv::new(1200.0, 2600.0, 0.050, 2.0),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-4.0)
        .pan(0.20)
        .delay(0.280)
        .style(0.7, 1.40),
    );
    r = r.layer(
        Layer::new(
            "lock",
            LayerKind::Modal {
                modes: metal_modes(3400.0, 0.040),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 2200.0,
                exciter_lp: 14_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-12.0)
        .pan(-0.10)
        .delay(0.360)
        .style(1.0, 0.40),
    );
    r.layer(
        Layer::new(
            "spring ring",
            LayerKind::Modal {
                modes: metal_modes(640.0, 0.180),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 400.0,
                exciter_lp: 5000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-26.0)
        .pan(0.05)
        .delay(0.130)
        .style(1.0, 0.40),
    )
}

fn shell_insert() -> Recipe {
    let mut r = base("weapon_shell_insert", "Shell Insert", Category::Weapon);
    r.duration = 0.55;
    r.fade_out = 0.03;
    r.style = 0.35;
    r.fade_in = 0.0;
    r.tags = vec![
        "gun".into(),
        "shotgun".into(),
        "shell".into(),
        "reload".into(),
        "foley".into(),
    ];
    r.description =
        "Two brass shells pushed into a shotgun tube: soft thunk, brass ring and a small click as each one seats.".into();
    r.master = MasterChain {
        highpass_hz: 90.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 160.0,
        presence_db: 2.0,
        presence_hz: 3400.0,
        comp_threshold_db: -13.0,
        comp_ratio: 2.2,
        drive_db: 2.0,
        reverb_wet: 0.12,
        reverb_size: 0.25,
        reverb_damp: 0.45,
        stereo_width: 1.10,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "friction",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.005, 0.060),
                filter: FilterEnv::new(1500.0, 900.0, 0.040, 1.8),
                mode: 1,
                tame_hz: 7000.0,
            },
        )
        .gain(-6.0)
        .pan(-0.10)
        .style(0.8, 1.30),
    );
    r = r.layer(
        Layer::new(
            "shell thunk",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 220.0,
                end_hz: 110.0,
                pitch_tau: 0.012,
                env: AdEnv::perc(0.050),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.20,
                vibrato: 0.0,
            },
        )
        .gain(-9.0)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "brass ring",
            LayerKind::Modal {
                modes: metal_modes(1900.0, 0.090),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-22.0)
        .pan(0.15)
        .style(1.0, 0.65),
    );
    r = r.layer(
        Layer::new(
            "seat click",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.008),
                filter: FilterEnv::new(3000.0, 2000.0, 0.004, 1.5),
                mode: 1,
                tame_hz: 12_000.0,
            },
        )
        .gain(-9.0)
        .delay(0.050)
        .style(1.0, 0.60),
    );
    r = r.layer(
        Layer::new(
            "second shell",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 200.0,
                end_hz: 100.0,
                pitch_tau: 0.012,
                env: AdEnv::perc(0.050),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.20,
                vibrato: 0.0,
            },
        )
        .gain(-11.0)
        .pan(0.10)
        .delay(0.280)
        .style(1.0, 1.20),
    );
    r.layer(
        Layer::new(
            "second ring",
            LayerKind::Modal {
                modes: metal_modes(2100.0, 0.080),
                exciter_ms: 0.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1300.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-23.0)
        .pan(-0.15)
        .delay(0.280)
        .style(1.0, 0.65),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_presets_render_cleanly() {
        let presets = presets();
        assert!(presets.len() >= 8, "need at least 8 weapon presets");
        for r in presets {
            let buf = sg_engine::render(&r, 42);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.05, "{} too short", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak <= 1.0, "{} clips (peak {peak})", r.id);
            assert!(
                buf.samples.iter().all(|s| s.is_finite()),
                "{} has NaN",
                r.id
            );
            assert!(r.layers.len() >= 3, "{} needs 3+ layers", r.id);
            println!(
                "{}: {:.3}s peak {:.2} dBFS rms {:.4} lufs {:.1}",
                r.id,
                buf.duration(),
                sg_dsp::gain_to_db(peak),
                buf.rms(),
                buf.lufs_approx()
            );
        }
    }
}
