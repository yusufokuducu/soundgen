//! Zombie vocal presets: growls, screams, pain, feeding and a horde bed.
//!
//! The "human but wrong" recipe from the research (§3.7-3.10): a low, jittery
//! glottal saw, a sub-octave, granular rasp for the wet throat, and breath.
//! Formants are implied by narrow grain bands and drive rather than a real
//! formant bank. Cartoon mode widens the pitch contour, adds ring-mod-ish
//! grain and pushes the tail.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        growl_low(),
        growl_high(),
        scream(),
        pain_hit(),
        attack_lunge(),
        bite(),
        death_rattle(),
        idle_breath(),
        horde_bed(),
    ]
}

/// Short helper for the bone/plate material used by the bite.
fn bone_modes(base: f32, decay: f32) -> Vec<Mode> {
    const RATIO: [f32; 4] = [1.0, 1.594, 2.136, 2.296];
    const MISTUNE: [f32; 4] = [1.012, 0.989, 1.017, 0.994];
    const AMP: [f32; 4] = [1.0, 0.55, 0.38, 0.26];
    RATIO
        .iter()
        .zip(MISTUNE)
        .zip(AMP)
        .map(|((r, m), a)| Mode::new(base * r * m, a, decay * (0.8 + a * 0.3)))
        .collect()
}

// ---------------------------------------------------------------------------
// growls
// ---------------------------------------------------------------------------

fn growl_low() -> Recipe {
    let mut r = base("zombie_growl_low", "Zombie Growl (Low)", Category::Zombie);
    r.duration = 1.50;
    r.fade_out = 0.05;
    r.style = 0.45;
    r.tags = vec![
        "zombie".into(),
        "growl".into(),
        "low".into(),
        "throat".into(),
        "loopable".into(),
    ];
    r.description =
        "Wet, low, tired growl: saw glottal 92->61 Hz, sub-octave, granular throat rasp and a dry breath bed.".into();
    r.master = MasterChain {
        highpass_hz: 35.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 110.0,
        presence_db: 2.5,
        presence_hz: 1700.0,
        comp_threshold_db: -16.0,
        comp_ratio: 3.5,
        drive_db: 4.0,
        reverb_wet: 0.22,
        reverb_size: 0.55,
        reverb_damp: 0.50,
        stereo_width: 1.20,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "glottal",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 92.0,
                end_hz: 61.0,
                pitch_tau: 0.35,
                env: AdEnv::swelled(0.070, 1.15),
                fm_hz: 1.02,
                fm_depth: 0.20,
                drive: 0.45,
                vibrato: 0.70,
            },
        )
        .gain(-7.0)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 58.0,
                end_hz: 44.0,
                pitch_tau: 0.40,
                env: AdEnv::swelled(0.060, 1.05),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.10,
                vibrato: 0.0,
            },
        )
        .gain(-9.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "throat rasp",
            LayerKind::Grain {
                source_hz: 420.0,
                grain_ms: 16.0,
                density: 110.0,
                spray: 0.55,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.100, 0.95),
                pitch_jitter: 0.25,
            },
        )
        .gain(2.0)
        .pan(-0.15)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "creak",
            LayerKind::Grain {
                source_hz: 180.0,
                grain_ms: 8.0,
                density: 34.0,
                spray: 0.80,
                color: NoiseColor::Velvet,
                env: AdEnv::swelled(0.200, 0.90),
                pitch_jitter: 0.60,
            },
        )
        .gain(-10.0)
        .pan(0.25)
        .style(1.0, 1.50),
    );
    r.layer(
        Layer::new(
            "breath",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.140, 0.85),
                filter: FilterEnv::new(1100.0, 380.0, 0.50, 0.8),
                mode: 1,
                tame_hz: 5000.0,
            },
        )
        .gain(-6.0)
        .pan(0.12)
        .style(1.0, 1.30),
    )
}

fn growl_high() -> Recipe {
    let mut r = base("zombie_growl_high", "Zombie Growl (High)", Category::Zombie);
    r.duration = 1.00;
    r.fade_out = 0.04;
    r.style = 0.50;
    r.tags = vec![
        "zombie".into(),
        "growl".into(),
        "high".into(),
        "snarl".into(),
        "aggressive".into(),
    ];
    r.description =
        "Higher, snarlier growl: a 155->118 Hz throat, fast granular rasp, a noisy snarl and small spit clicks.".into();
    r.master = MasterChain {
        highpass_hz: 60.0,
        low_shelf_db: 0.5,
        low_shelf_hz: 140.0,
        presence_db: 3.0,
        presence_hz: 2200.0,
        comp_threshold_db: -15.0,
        comp_ratio: 3.0,
        drive_db: 3.5,
        reverb_wet: 0.18,
        reverb_size: 0.45,
        reverb_damp: 0.50,
        stereo_width: 1.25,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "throat",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 155.0,
                end_hz: 118.0,
                pitch_tau: 0.28,
                env: AdEnv::swelled(0.050, 0.85),
                fm_hz: 1.03,
                fm_depth: 0.15,
                drive: 0.40,
                vibrato: 0.90,
            },
        )
        .gain(-7.0)
        .pan(0.05)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "rasp",
            LayerKind::Grain {
                source_hz: 780.0,
                grain_ms: 12.0,
                density: 150.0,
                spray: 0.50,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.060, 0.70),
                pitch_jitter: 0.30,
            },
        )
        .gain(0.0)
        .pan(-0.20)
        .style(1.0, 1.45),
    );
    r = r.layer(
        Layer::new(
            "snarl",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.040, 0.50),
                filter: FilterEnv::new(2400.0, 1200.0, 0.20, 1.4),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-10.0)
        .pan(0.20)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 82.0,
                end_hz: 68.0,
                pitch_tau: 0.35,
                env: AdEnv::swelled(0.050, 0.80),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-13.0)
        .style(1.0, 1.20),
    );
    r.layer(
        Layer::new(
            "spit",
            LayerKind::Grain {
                source_hz: 1400.0,
                grain_ms: 5.0,
                density: 22.0,
                spray: 0.90,
                color: NoiseColor::Velvet,
                env: AdEnv::swelled(0.020, 0.40),
                pitch_jitter: 0.60,
            },
        )
        .gain(-14.0)
        .pan(0.30)
        .style(1.0, 1.50),
    )
}

// ---------------------------------------------------------------------------
// scream / pain
// ---------------------------------------------------------------------------

fn scream() -> Recipe {
    let mut r = base("zombie_scream", "Zombie Scream", Category::Zombie);
    r.duration = 1.70;
    r.fade_out = 0.05;
    r.style = 0.55;
    r.tags = vec![
        "zombie".into(),
        "scream".into(),
        "shrill".into(),
        "distorted".into(),
        "long".into(),
    ];
    r.description =
        "Shrill, distorted zombie scream: a 700->520 Hz saw voice, formant-like grain, torn throat and a long room tail.".into();
    r.master = MasterChain {
        highpass_hz: 70.0,
        presence_db: 3.0,
        presence_hz: 2600.0,
        comp_threshold_db: -12.0,
        comp_ratio: 4.0,
        drive_db: 6.0,
        reverb_wet: 0.25,
        reverb_size: 0.65,
        reverb_damp: 0.45,
        stereo_width: 1.25,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "voice",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 700.0,
                end_hz: 520.0,
                pitch_tau: 0.45,
                env: AdEnv::swelled(0.050, 1.35),
                fm_hz: 1.01,
                fm_depth: 0.20,
                drive: 0.60,
                vibrato: 1.00,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "formant",
            LayerKind::Grain {
                source_hz: 1700.0,
                grain_ms: 20.0,
                density: 90.0,
                spray: 0.40,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.100, 1.10),
                pitch_jitter: 0.20,
            },
        )
        .gain(-4.0)
        .pan(-0.20)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "distorted edge",
            LayerKind::Tone {
                shape: WaveShape::Square,
                start_hz: 350.0,
                end_hz: 260.0,
                pitch_tau: 0.50,
                env: AdEnv::swelled(0.040, 1.10),
                fm_hz: 1.50,
                fm_depth: 0.30,
                drive: 0.85,
                vibrato: 0.50,
            },
        )
        .gain(-10.0)
        .pan(0.20)
        .style(1.0, 1.50),
    );
    r = r.layer(
        Layer::new(
            "tear",
            LayerKind::Grain {
                source_hz: 2600.0,
                grain_ms: 10.0,
                density: 220.0,
                spray: 0.60,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.150, 0.90),
                pitch_jitter: 0.50,
            },
        )
        .gain(-11.0)
        .pan(0.30)
        .style(0.8, 1.50),
    );
    r = r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.050, 0.90),
                filter: FilterEnv::new(3200.0, 1800.0, 0.40, 1.5),
                mode: 1,
                tame_hz: 12_000.0,
            },
        )
        .gain(-13.0)
        .pan(-0.30)
        .style(1.0, 1.40),
    );
    r.layer(
        Layer::new(
            "room",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.90),
                filter: FilterEnv::new(900.0, 400.0, 0.50, 0.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-14.0)
        .delay(0.050)
        .style(0.6, 1.70),
    )
}

fn pain_hit() -> Recipe {
    let mut r = base("zombie_pain_hit", "Zombie Pain Hit", Category::Zombie);
    r.duration = 0.45;
    r.fade_out = 0.03;
    r.style = 0.40;
    r.tags = vec![
        "zombie".into(),
        "pain".into(),
        "grunt".into(),
        "hit".into(),
        "short".into(),
    ];
    r.description =
        "Short pained grunt: a 180->105 Hz strained saw, throat rasp, chest thud and a wet exhale.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 130.0,
        presence_db: 2.0,
        presence_hz: 2000.0,
        comp_threshold_db: -14.0,
        comp_ratio: 3.0,
        drive_db: 4.5,
        reverb_wet: 0.16,
        reverb_size: 0.40,
        reverb_damp: 0.50,
        stereo_width: 1.15,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "grunt",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 180.0,
                end_hz: 105.0,
                pitch_tau: 0.09,
                env: AdEnv::swelled(0.020, 0.28),
                fm_hz: 1.04,
                fm_depth: 0.22,
                drive: 0.55,
                vibrato: 0.40,
            },
        )
        .gain(-6.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "throat rasp",
            LayerKind::Grain {
                source_hz: 560.0,
                grain_ms: 11.0,
                density: 130.0,
                spray: 0.50,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.030, 0.24),
                pitch_jitter: 0.35,
            },
        )
        .gain(-3.0)
        .pan(-0.15)
        .style(1.0, 1.40),
    );
    r = r.layer(
        Layer::new(
            "chest thud",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 110.0,
                end_hz: 70.0,
                pitch_tau: 0.03,
                env: AdEnv::perc(0.120),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.10,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "exhale",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.020, 0.25),
                filter: FilterEnv::new(1300.0, 500.0, 0.10, 0.9),
                mode: 1,
                tame_hz: 6000.0,
            },
        )
        .gain(-8.0)
        .pan(0.15)
        .style(1.0, 1.30),
    );
    r.layer(
        Layer::new(
            "wet click",
            LayerKind::Grain {
                source_hz: 1600.0,
                grain_ms: 6.0,
                density: 45.0,
                spray: 0.80,
                color: NoiseColor::Velvet,
                env: AdEnv::perc(0.10),
                pitch_jitter: 0.70,
            },
        )
        .gain(-14.0)
        .pan(0.25)
        .style(1.0, 1.40),
    )
}

// ---------------------------------------------------------------------------
// attack / feed
// ---------------------------------------------------------------------------

fn attack_lunge() -> Recipe {
    let mut r = base("zombie_attack_lunge", "Zombie Attack Lunge", Category::Zombie);
    r.duration = 1.10;
    r.fade_out = 0.04;
    r.style = 0.50;
    r.tags = vec![
        "zombie".into(),
        "attack".into(),
        "lunge".into(),
        "growl".into(),
        "cloth".into(),
    ];
    r.description =
        "Lunging attack: a rising growl, a forced breath, cloth movement and a fast air swipe as the body closes in.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 120.0,
        presence_db: 2.5,
        presence_hz: 1900.0,
        comp_threshold_db: -15.0,
        comp_ratio: 3.5,
        drive_db: 4.0,
        reverb_wet: 0.20,
        reverb_size: 0.50,
        reverb_damp: 0.50,
        stereo_width: 1.25,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "growl",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 120.0,
                end_hz: 75.0,
                pitch_tau: 0.25,
                env: AdEnv::swelled(0.060, 0.90),
                fm_hz: 1.02,
                fm_depth: 0.25,
                drive: 0.50,
                vibrato: 0.80,
            },
        )
        .gain(-10.0)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "breath",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.180, 0.55),
                filter: FilterEnv::new(1500.0, 700.0, 0.25, 1.0),
                mode: 1,
                tame_hz: 7000.0,
            },
        )
        .gain(-4.0)
        .pan(-0.20)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "cloth",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.120, 0.40),
                filter: FilterEnv::new(2400.0, 900.0, 0.20, 0.6),
                mode: 1,
                tame_hz: 10_000.0,
            },
        )
        .gain(-12.0)
        .pan(0.30)
        .style(0.8, 1.50),
    );
    r = r.layer(
        Layer::new(
            "rasp",
            LayerKind::Grain {
                source_hz: 400.0,
                grain_ms: 14.0,
                density: 120.0,
                spray: 0.60,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.080, 0.80),
                pitch_jitter: 0.30,
            },
        )
        .gain(-5.0)
        .pan(0.10)
        .style(1.0, 1.40),
    );
    r.layer(
        Layer::new(
            "swipe",
            LayerKind::Sweep {
                start_hz: 400.0,
                end_hz: 2200.0,
                tau: 0.22,
                q: 1.2,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.100, 0.35),
                tone_mix: 0.15,
            },
        )
        .gain(-9.0)
        .pan(-0.30)
        .delay(0.120)
        .style(0.7, 1.50),
    )
}

fn bite() -> Recipe {
    let mut r = base("zombie_bite", "Zombie Bite", Category::Zombie);
    r.duration = 0.50;
    r.fade_out = 0.03;
    r.style = 0.50;
    r.tags = vec![
        "zombie".into(),
        "bite".into(),
        "chomp".into(),
        "wet".into(),
        "bone".into(),
    ];
    r.description =
        "Wet chomp: a tooth snap, upward squelch sweep, short bone modal, sine squish and a granular tear.".into();
    r.master = MasterChain {
        highpass_hz: 50.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 130.0,
        presence_db: 2.5,
        presence_hz: 2200.0,
        comp_threshold_db: -13.0,
        comp_ratio: 3.0,
        drive_db: 4.0,
        reverb_wet: 0.15,
        reverb_size: 0.35,
        reverb_damp: 0.50,
        stereo_width: 1.20,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "tooth snap",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.030),
                filter: FilterEnv::new(1100.0, 600.0, 0.010, 1.4),
                mode: 1,
                tame_hz: 8000.0,
            },
        )
        .gain(0.0)
        .style(1.0, 1.10),
    );
    r = r.layer(
        Layer::new(
            "squelch",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.090),
                filter: FilterEnv::new(600.0, 1800.0, 0.050, 3.0),
                mode: 1,
                tame_hz: 6000.0,
            },
        )
        .gain(-2.0)
        .pan(-0.10)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "bone",
            LayerKind::Modal {
                modes: bone_modes(900.0, 0.050),
                exciter_ms: 1.2,
                exciter_color: NoiseColor::White,
                exciter_hp: 600.0,
                exciter_lp: 6000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-34.0)
        .pan(0.15)
        .style(1.0, 0.65),
    );
    r = r.layer(
        Layer::new(
            "squish",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 180.0,
                end_hz: 110.0,
                pitch_tau: 0.040,
                env: AdEnv::perc(0.090),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.20,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "tear",
            LayerKind::Grain {
                source_hz: 2400.0,
                grain_ms: 8.0,
                density: 260.0,
                spray: 0.50,
                color: NoiseColor::White,
                env: AdEnv::perc(0.25),
                pitch_jitter: 0.45,
            },
        )
        .gain(-12.0)
        .pan(0.25)
        .style(0.8, 1.50),
    );
    r.layer(
        Layer::new(
            "wet tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.30),
                filter: FilterEnv::new(900.0, 300.0, 0.12, 1.0),
                mode: 1,
                tame_hz: 5000.0,
            },
        )
        .gain(-10.0)
        .pan(-0.20)
        .style(0.7, 1.50),
    )
}

// ---------------------------------------------------------------------------
// death / ambience
// ---------------------------------------------------------------------------

fn death_rattle() -> Recipe {
    let mut r = base("zombie_death_rattle", "Zombie Death Rattle", Category::Zombie);
    r.duration = 2.40;
    r.fade_out = 0.05;
    r.style = 0.50;
    r.tags = vec![
        "zombie".into(),
        "death".into(),
        "rattle".into(),
        "gurgle".into(),
        "fading".into(),
    ];
    r.description =
        "Dying zombie: a 78->38 Hz groan falling into granular gurgle, a bone rattle and one last shallow breath.".into();
    r.master = MasterChain {
        highpass_hz: 40.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 110.0,
        presence_db: 1.5,
        presence_hz: 1500.0,
        comp_threshold_db: -18.0,
        comp_ratio: 3.0,
        drive_db: 5.0,
        reverb_wet: 0.28,
        reverb_size: 0.70,
        reverb_damp: 0.55,
        stereo_width: 1.30,
        target_peak_db: -2.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "groan",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 78.0,
                end_hz: 38.0,
                pitch_tau: 0.70,
                env: AdEnv::swelled(0.080, 2.00),
                fm_hz: 1.01,
                fm_depth: 0.25,
                drive: 0.50,
                vibrato: 0.60,
            },
        )
        .gain(-7.0)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "fall",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.020, 0.45),
                filter: FilterEnv::new(180.0, 80.0, 0.40, 0.8),
                mode: 0,
                tame_hz: 600.0,
            },
        )
        .gain(-3.0)
        .delay(0.150)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "gurgle",
            LayerKind::Grain {
                source_hz: 320.0,
                grain_ms: 30.0,
                density: 45.0,
                spray: 0.80,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.200, 1.50),
                pitch_jitter: 0.50,
            },
        )
        .gain(-4.0)
        .pan(-0.20)
        .style(1.0, 1.50),
    );
    r = r.layer(
        Layer::new(
            "rattle",
            LayerKind::Grain {
                source_hz: 1100.0,
                grain_ms: 5.0,
                density: 70.0,
                spray: 0.90,
                color: NoiseColor::Velvet,
                env: AdEnv::swelled(0.300, 1.40),
                pitch_jitter: 0.80,
            },
        )
        .gain(-10.0)
        .pan(0.25)
        .style(1.0, 1.60),
    );
    r = r.layer(
        Layer::new(
            "last breath",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.300, 0.70),
                filter: FilterEnv::new(700.0, 250.0, 0.50, 1.0),
                mode: 1,
                tame_hz: 4000.0,
            },
        )
        .gain(-10.0)
        .pan(0.10)
        .delay(0.900)
        .style(0.8, 1.50),
    );
    r.layer(
        Layer::new(
            "sub",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 50.0,
                end_hz: 32.0,
                pitch_tau: 0.80,
                env: AdEnv::swelled(0.100, 1.80),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.30),
    )
}

fn idle_breath() -> Recipe {
    let mut r = base("zombie_idle_breath", "Zombie Idle Breath", Category::Zombie);
    r.duration = 2.30;
    r.fade_out = 0.05;
    r.style = 0.35;
    r.looped = false;
    r.tags = vec![
        "zombie".into(),
        "breath".into(),
        "idle".into(),
        "bed".into(),
        "smooth".into(),
    ];
    r.description =
        "Smooth idle breathing bed: a soft inhale, two dry exhales, a wheeze and a faint chest rumble, no hard edges.".into();
    r.master = MasterChain {
        highpass_hz: 50.0,
        low_shelf_db: 0.5,
        low_shelf_hz: 120.0,
        presence_db: 1.0,
        presence_hz: 1200.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.0,
        drive_db: 2.0,
        reverb_wet: 0.12,
        reverb_size: 0.35,
        reverb_damp: 0.55,
        stereo_width: 1.10,
        target_peak_db: -5.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "inhale",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.350, 0.50),
                filter: FilterEnv::new(900.0, 1400.0, 0.50, 1.1),
                mode: 1,
                tame_hz: 6000.0,
            },
        )
        .gain(-14.0)
        .pan(-0.05)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "exhale",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.250, 0.90),
                filter: FilterEnv::new(700.0, 300.0, 0.60, 0.9),
                mode: 1,
                tame_hz: 5000.0,
            },
        )
        .gain(-12.0)
        .pan(0.08)
        .delay(0.550)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "wheeze",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.400, 0.45),
                filter: FilterEnv::new(2200.0, 1600.0, 0.40, 2.5),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-31.0)
        .pan(0.15)
        .delay(0.600)
        .style(0.8, 1.40),
    );
    r = r.layer(
        Layer::new(
            "chest rumble",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 62.0,
                end_hz: 54.0,
                pitch_tau: 0.80,
                env: AdEnv::swelled(0.500, 1.20),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.05,
                vibrato: 0.20,
            },
        )
        .gain(-28.0)
        .style(1.0, 1.20),
    );
    r = r.layer(
        Layer::new(
            "second exhale",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.300, 0.80),
                filter: FilterEnv::new(600.0, 250.0, 0.50, 0.8),
                mode: 1,
                tame_hz: 4500.0,
            },
        )
        .gain(-16.0)
        .pan(-0.12)
        .delay(1.600)
        .style(1.0, 1.30),
    );
    r.layer(
        Layer::new(
            "chest creak",
            LayerKind::Grain {
                source_hz: 300.0,
                grain_ms: 20.0,
                density: 18.0,
                spray: 0.90,
                color: NoiseColor::Velvet,
                env: AdEnv::swelled(0.500, 1.00),
                pitch_jitter: 0.70,
            },
        )
        .gain(-20.0)
        .pan(0.20)
        .delay(1.200)
        .style(1.0, 1.50),
    )
}

fn horde_bed() -> Recipe {
    let mut r = base("zombie_horde_bed", "Zombie Horde Bed", Category::Zombie);
    r.duration = 2.80;
    r.fade_out = 0.05;
    r.style = 0.45;
    r.tags = vec![
        "zombie".into(),
        "horde".into(),
        "bed".into(),
        "ambience".into(),
        "layered".into(),
    ];
    r.description =
        "Three offset growls panned across the field over a granular rasp bed and a low sub drone.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 110.0,
        presence_db: 1.0,
        presence_hz: 1600.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.5,
        drive_db: 3.0,
        reverb_wet: 0.30,
        reverb_size: 0.75,
        reverb_damp: 0.55,
        stereo_width: 1.35,
        target_peak_db: -4.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "growl left",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 88.0,
                end_hz: 62.0,
                pitch_tau: 0.50,
                env: AdEnv::swelled(0.200, 2.20),
                fm_hz: 1.02,
                fm_depth: 0.20,
                drive: 0.45,
                vibrato: 0.60,
            },
        )
        .gain(-10.0)
        .pan(-0.50)
        .style(1.0, 1.25),
    );
    r = r.layer(
        Layer::new(
            "growl right",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 110.0,
                end_hz: 78.0,
                pitch_tau: 0.45,
                env: AdEnv::swelled(0.250, 2.00),
                fm_hz: 1.03,
                fm_depth: 0.18,
                drive: 0.40,
                vibrato: 0.80,
            },
        )
        .gain(-11.0)
        .pan(0.45)
        .delay(0.350)
        .style(1.0, 1.30),
    );
    r = r.layer(
        Layer::new(
            "growl center",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 72.0,
                end_hz: 52.0,
                pitch_tau: 0.60,
                env: AdEnv::swelled(0.300, 2.40),
                fm_hz: 1.01,
                fm_depth: 0.22,
                drive: 0.50,
                vibrato: 0.50,
            },
        )
        .gain(-12.0)
        .pan(0.05)
        .delay(0.900)
        .style(1.0, 1.35),
    );
    r = r.layer(
        Layer::new(
            "rasp bed",
            LayerKind::Grain {
                source_hz: 380.0,
                grain_ms: 22.0,
                density: 90.0,
                spray: 0.70,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.400, 2.00),
                pitch_jitter: 0.40,
            },
        )
        .gain(-5.0)
        .style(1.0, 1.45),
    );
    r = r.layer(
        Layer::new(
            "distant breath",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.500, 1.80),
                filter: FilterEnv::new(800.0, 350.0, 0.80, 0.7),
                mode: 1,
                tame_hz: 5000.0,
            },
        )
        .gain(-10.0)
        .pan(-0.30)
        .delay(0.400)
        .style(1.0, 1.40),
    );
    r.layer(
        Layer::new(
            "sub drone",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 44.0,
                end_hz: 36.0,
                pitch_tau: 1.00,
                env: AdEnv::swelled(0.400, 2.20),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.10,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.30),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zombie_presets_render_cleanly() {
        let presets = presets();
        assert!(presets.len() >= 8, "need at least 8 zombie presets");
        for r in presets {
            let buf = sg_engine::render(&r, 42);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.05, "{} too short", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak <= 1.0, "{} clips (peak {peak})", r.id);
            assert!(buf.samples.iter().all(|s| s.is_finite()), "{} has NaN", r.id);
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
