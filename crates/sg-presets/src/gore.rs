//! Gore — wet impacts, bone, splatter and dismemberment.
//!
//! Design notes:
//!  * meat = low Q resonances + lots of wet granular texture (research §3.11)
//!  * bone / skull = plate-series modal banks with short T60 (research §3.12–3.13)
//!  * every recipe has a low "weight" layer (60–140 Hz) so hits read as heavy
//!  * `style` pushes pitch envelopes down, keeps the squish layers and grows tails

use crate::prelude::*;
use sg_dsp::rng::Rng;

// Material mode-ratio tables from the research (§2.3).
const PLATE: [f32; 4] = [1.0, 1.594, 2.136, 2.296];
const BAR: [f32; 4] = [1.0, 2.756, 5.404, 8.933];
const WOOD: [f32; 4] = [1.0, 2.32, 3.71, 5.14];

/// Struck-object mode set: ±1.5 % mistune per mode, upper modes quieter and
/// shorter (material damping, research §2.3).
fn struck(base: f32, ratios: &[f32], decay: f32, seed: u64) -> Vec<Mode> {
    let mut rng = Rng::new(seed);
    ratios
        .iter()
        .enumerate()
        .map(|(i, ratio)| {
            let f = base * ratio * (1.0 + rng.range(-0.015, 0.015));
            let amp = 1.0 / (1.0 + i as f32 * 0.9);
            let d = decay * (1.0 - i as f32 * 0.12).max(0.4);
            Mode::new(f, amp, d)
        })
        .collect()
}

/// Category master chain: weighty, driven, short room.
fn gore_master() -> MasterChain {
    MasterChain {
        highpass_hz: 32.0,
        low_shelf_db: 2.5,
        low_shelf_hz: 130.0,
        presence_db: 2.0,
        presence_hz: 2800.0,
        comp_threshold_db: -16.0,
        comp_ratio: 3.5,
        drive_db: 4.0,
        reverb_wet: 0.15,
        reverb_size: 0.32,
        reverb_damp: 0.55,
        stereo_width: 1.15,
        ..Default::default()
    }
}

pub fn presets() -> Vec<Recipe> {
    vec![
        flesh_hit(),
        flesh_slash(),
        bone_break(),
        head_pop(),
        body_fall(),
        blood_splat(),
        dismember(),
        zombie_crunch(),
    ]
}

/// Dull wet thud: low band-passed noise, a falling body tone and a squelch grain bed.
fn flesh_hit() -> Recipe {
    let mut r = base("gore_flesh_hit", "Flesh Hit", Category::Gore);
    r.duration = 0.34;
    r.fade_in = 0.0;
    r.fade_out = 0.025;
    r.style = 0.3;
    r.tags = vec!["flesh".into(), "impact".into(), "wet".into()];
    r.description = "Dull wet thud: low body noise, a short falling tone and a squelchy grain layer.".into();
    r.master = MasterChain {
        reverb_wet: 0.13,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "slap",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.028),
                filter: FilterEnv::new(1100.0, 520.0, 0.018, 1.2),
                mode: 1,
                tame_hz: 8000.0,
            },
        )
        .gain(9.0)
        .style(1.0, 0.85),
    );
    r = r.layer(
        Layer::new(
            "body thud",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 96.0,
                end_hz: 46.0,
                pitch_tau: 0.018,
                env: AdEnv::perc(0.11),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.45),
    );
    r = r.layer(
        Layer::new(
            "low wet band",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.16),
                filter: FilterEnv::new(400.0, 130.0, 0.05, 1.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-5.0)
        .style(0.9, 1.2),
    );
    r = r.layer(
        Layer::new(
            "squelch",
            LayerKind::Grain {
                source_hz: 760.0,
                grain_ms: 7.0,
                density: 180.0,
                spray: 0.35,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.002, 0.14),
                pitch_jitter: 0.25,
            },
        )
        .gain(2.0)
        .pan(0.1)
        .style(0.8, 1.6),
    );
    r.layer(
        Layer::new(
            "wet tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.2),
                filter: FilterEnv::new(1700.0, 700.0, 0.06, 2.0),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-8.0)
        .pan(-0.2)
        .delay(0.01)
        .style(0.7, 1.7),
    )
}

/// Sharper and airier than the thud: blade air, a cut transient and torn flesh.
fn flesh_slash() -> Recipe {
    let mut r = base("gore_flesh_slash", "Flesh Slash", Category::Gore);
    r.duration = 0.28;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.28;
    r.tags = vec!["flesh".into(), "slash".into(), "blade".into()];
    r.description = "Short airy blade slash with a bright cut transient and a wet tearing tail.".into();
    r.master = MasterChain {
        highpass_hz: 40.0,
        presence_hz: 3600.0,
        presence_db: 2.5,
        reverb_wet: 0.12,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "blade air",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.004, 0.1),
                filter: FilterEnv::new(6000.0, 1900.0, 0.05, 0.7),
                mode: 1,
                tame_hz: 15_000.0,
            },
        )
        .gain(-4.0)
        .pan(-0.15)
        .style(1.0, 0.7),
    );
    r = r.layer(
        Layer::new(
            "cut transient",
            LayerKind::Noise {
                color: NoiseColor::Blue,
                env: AdEnv::perc(0.014),
                filter: FilterEnv::new(4600.0, 2400.0, 0.004, 1.4),
                mode: 1,
                tame_hz: 16_000.0,
            },
        )
        .gain(1.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "torn flesh",
            LayerKind::Grain {
                source_hz: 1250.0,
                grain_ms: 5.0,
                density: 260.0,
                spray: 0.5,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.003, 0.11),
                pitch_jitter: 0.35,
            },
        )
        .gain(0.0)
        .pan(0.2)
        .style(0.9, 1.6),
    );
    r = r.layer(
        Layer::new(
            "wet body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 230.0,
                end_hz: 88.0,
                pitch_tau: 0.012,
                env: AdEnv::perc(0.07),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-11.0)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "air tail",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.17),
                filter: FilterEnv::new(3200.0, 900.0, 0.05, 0.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-7.0)
        .pan(-0.25)
        .style(0.6, 1.7),
    )
}

/// Bright modal crack: plate-series bone modes, high noise crack and debris hits.
fn bone_break() -> Recipe {
    let mut r = base("gore_bone_break", "Bone Break", Category::Gore);
    r.duration = 0.3;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.35;
    r.tags = vec!["bone".into(), "crack".into(), "snap".into()];
    r.description = "Dry bone snap: a bright crack, short plate-mode splinters and falling debris.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.0,
        presence_hz: 4200.0,
        presence_db: 3.0,
        reverb_wet: 0.13,
        stereo_width: 1.2,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.008),
                filter: FilterEnv::new(6500.0, 3000.0, 0.003, 0.9),
                mode: 2,
                tame_hz: 15_000.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "splinters",
            LayerKind::Modal {
                modes: struck(2900.0, &PLATE, 0.045, 0x0B0E_0001),
                exciter_ms: 0.6,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "grind",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.05),
                filter: FilterEnv::new(4800.0, 3200.0, 0.02, 1.2),
                mode: 1,
                tame_hz: 14_000.0,
            },
        )
        .gain(-7.0)
        .pan(0.12)
        .style(0.9, 1.3),
    );
    r = r.layer(
        Layer::new(
            "debris one",
            LayerKind::Modal {
                modes: struck(1750.0, &PLATE, 0.03, 0x0B0E_0002),
                exciter_ms: 0.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 900.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-28.0)
        .pan(-0.35)
        .delay(0.055)
        .style(0.8, 1.5),
    );
    r.layer(
        Layer::new(
            "debris two",
            LayerKind::Modal {
                modes: struck(2350.0, &PLATE, 0.022, 0x0B0E_0003),
                exciter_ms: 0.4,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 10_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-25.0)
        .pan(0.4)
        .delay(0.112)
        .style(0.7, 1.5),
    )
}

/// Deep sub thump, wet burst and skull debris.
fn head_pop() -> Recipe {
    let mut r = base("gore_head_pop", "Head Pop", Category::Gore);
    r.duration = 0.46;
    r.fade_in = 0.0;
    r.fade_out = 0.03;
    r.style = 0.55;
    r.tags = vec!["skull".into(), "explode".into(), "wet".into()];
    r.description = "Skull burst: deep sub thump, wet low burst, splatter and a few skull fragments.".into();
    r.master = MasterChain {
        highpass_hz: 28.0,
        low_shelf_db: 3.5,
        presence_hz: 2200.0,
        comp_threshold_db: -18.0,
        drive_db: 5.0,
        reverb_wet: 0.2,
        reverb_size: 0.38,
        stereo_width: 1.25,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "sub thump",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 70.0,
                end_hz: 31.0,
                pitch_tau: 0.05,
                env: AdEnv::perc(0.28),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.5,
                vibrato: 0.0,
            },
        )
        .gain(0.0)
        .style(1.0, 1.6),
    );
    r = r.layer(
        Layer::new(
            "wet burst",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.1),
                filter: FilterEnv::new(800.0, 280.0, 0.07, 1.1),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(4.0)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "splat",
            LayerKind::Grain {
                source_hz: 900.0,
                grain_ms: 14.0,
                density: 150.0,
                spray: 0.7,
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.3),
                pitch_jitter: 0.4,
            },
        )
        .gain(5.0)
        .pan(0.12)
        .style(0.9, 1.7),
    );
    r = r.layer(
        Layer::new(
            "skull debris",
            LayerKind::Modal {
                modes: struck(2400.0, &PLATE, 0.06, 0x0B0E_0010),
                exciter_ms: 1.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 1100.0,
                exciter_lp: 11_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-25.0)
        .pan(-0.22)
        .delay(0.02)
        .style(0.8, 1.5),
    );
    r.layer(
        Layer::new(
            "crunch tail",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.18),
                filter: FilterEnv::new(2600.0, 800.0, 0.06, 0.7),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-8.0)
        .pan(0.3)
        .style(0.6, 1.8),
    )
}

/// Heavy body hitting the floor: thud, cloth, and two smaller settling impacts.
fn body_fall() -> Recipe {
    let mut r = base("gore_body_fall", "Body Fall", Category::Gore);
    r.duration = 0.6;
    r.fade_in = 0.0;
    r.fade_out = 0.04;
    r.style = 0.32;
    r.tags = vec!["body".into(), "fall".into(), "heavy".into()];
    r.description = "Heavy corpse drop: deep thud, cloth flap and two small secondary impacts.".into();
    r.master = MasterChain {
        highpass_hz: 26.0,
        low_shelf_db: 3.0,
        low_shelf_hz: 110.0,
        presence_hz: 1800.0,
        presence_db: 1.5,
        comp_threshold_db: -18.0,
        drive_db: 3.0,
        reverb_wet: 0.18,
        reverb_size: 0.4,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "heavy thud",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 60.0,
                end_hz: 34.0,
                pitch_tau: 0.03,
                env: AdEnv::perc(0.32),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.4,
                vibrato: 0.0,
            },
        )
        .gain(0.0)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "floor impact",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.14),
                filter: FilterEnv::new(340.0, 140.0, 0.04, 0.9),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(4.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "cloth flap",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.015, 0.32),
                filter: FilterEnv::new(1900.0, 700.0, 0.09, 0.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(5.0)
        .pan(-0.18)
        .style(0.8, 1.5),
    );
    r = r.layer(
        Layer::new(
            "second impact",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 92.0,
                end_hz: 52.0,
                pitch_tau: 0.02,
                env: AdEnv::perc(0.13),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .pan(0.28)
        .delay(0.16)
        .style(0.8, 1.4),
    );
    r.layer(
        Layer::new(
            "settle creak",
            LayerKind::Modal {
                modes: struck(330.0, &WOOD, 0.09, 0x0B0E_0020),
                exciter_ms: 2.0,
                exciter_color: NoiseColor::Pink,
                exciter_hp: 200.0,
                exciter_lp: 3500.0,
                detune_cents: 0.0,
            },
        )
        .gain(-34.0)
        .pan(-0.32)
        .delay(0.3)
        .style(0.7, 1.6),
    )
}

/// Granular wet splatter: dense core, then a delayed sparser drip tail.
fn blood_splat() -> Recipe {
    let mut r = base("gore_blood_splat", "Blood Splat", Category::Gore);
    r.duration = 0.55;
    r.fade_in = 0.0;
    r.fade_out = 0.03;
    r.style = 0.45;
    r.tags = vec!["blood".into(), "splat".into(), "wet".into()];
    r.description = "Wet splatter: a dense granular burst that thins out into dripping droplets.".into();
    r.master = MasterChain {
        highpass_hz: 38.0,
        low_shelf_db: 2.0,
        presence_hz: 2400.0,
        presence_db: 2.0,
        reverb_wet: 0.16,
        stereo_width: 1.25,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "splat burst",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.055),
                filter: FilterEnv::new(1300.0, 500.0, 0.03, 0.9),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(6.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "dense core",
            LayerKind::Grain {
                source_hz: 820.0,
                grain_ms: 8.0,
                density: 320.0,
                spray: 0.6,
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.13),
                pitch_jitter: 0.45,
            },
        )
        .gain(1.0)
        .pan(-0.12)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "drip tail",
            LayerKind::Grain {
                source_hz: 1150.0,
                grain_ms: 18.0,
                density: 85.0,
                spray: 0.85,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.01, 0.44),
                pitch_jitter: 0.55,
            },
        )
        .gain(-12.0)
        .pan(0.28)
        .delay(0.09)
        .style(0.8, 1.8),
    );
    r.layer(
        Layer::new(
            "low wet body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 145.0,
                end_hz: 68.0,
                pitch_tau: 0.03,
                env: AdEnv::perc(0.11),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.0,
            },
        )
        .gain(-15.0)
        .style(1.0, 1.5),
    )
}

/// Tearing limb: granular rip, an optional metal-ish screech and a wet body.
fn dismember() -> Recipe {
    let mut r = base("gore_dismember", "Dismember", Category::Gore);
    r.duration = 0.52;
    r.fade_in = 0.0;
    r.fade_out = 0.035;
    r.style = 0.4;
    r.tags = vec!["tear".into(), "limb".into(), "gore".into()];
    r.description = "Limb torn off: a ripping granular tear, a thin metallic screech and a wet snap.".into();
    r.master = MasterChain {
        highpass_hz: 35.0,
        presence_hz: 3000.0,
        presence_db: 2.5,
        drive_db: 5.0,
        reverb_wet: 0.18,
        stereo_width: 1.3,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "tear",
            LayerKind::Grain {
                source_hz: 1500.0,
                grain_ms: 6.0,
                density: 280.0,
                spray: 0.55,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.005, 0.36),
                pitch_jitter: 0.3,
            },
        )
        .gain(-2.0)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "screech",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 720.0,
                end_hz: 330.0,
                pitch_tau: 0.12,
                env: AdEnv::swelled(0.01, 0.28),
                fm_hz: 1.02,
                fm_depth: 0.12,
                drive: 0.5,
                vibrato: 0.6,
            },
        )
        .gain(-15.0)
        .pan(-0.2)
        .style(0.4, 1.6),
    );
    r = r.layer(
        Layer::new(
            "metal snap",
            LayerKind::Modal {
                modes: struck(1150.0, &BAR, 0.18, 0x0B0E_0030),
                exciter_ms: 0.4,
                exciter_color: NoiseColor::White,
                exciter_hp: 700.0,
                exciter_lp: 13_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-30.0)
        .pan(0.35)
        .delay(0.04)
        .style(0.5, 1.6),
    );
    r = r.layer(
        Layer::new(
            "wet body",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.13),
                filter: FilterEnv::new(950.0, 380.0, 0.05, 1.4),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(3.0)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "snap transient",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.011),
                filter: FilterEnv::new(5000.0, 2600.0, 0.003, 1.1),
                mode: 2,
                tame_hz: 16_000.0,
            },
        )
        .gain(-7.0)
        .pan(0.1)
        .style(1.0, 0.8),
    )
}

/// Repeated crunch: three short modal bites plus a continuous wet chew bed.
fn zombie_crunch() -> Recipe {
    let mut r = base("gore_zombie_crunch", "Zombie Crunch", Category::Gore);
    r.duration = 0.44;
    r.fade_in = 0.0;
    r.fade_out = 0.03;
    r.style = 0.5;
    r.tags = vec!["zombie".into(), "crunch".into(), "bite".into()];
    r.description = "Three bone crunches in quick succession over a wet chewing bed.".into();
    r.master = MasterChain {
        highpass_hz: 36.0,
        low_shelf_db: 2.0,
        presence_hz: 3400.0,
        presence_db: 2.5,
        drive_db: 5.0,
        reverb_wet: 0.14,
        stereo_width: 1.2,
        ..gore_master()
    };
    r = r.layer(
        Layer::new(
            "crunch one",
            LayerKind::Modal {
                modes: struck(1300.0, &PLATE, 0.06, 0x0B0E_0040),
                exciter_ms: 1.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 600.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-18.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "crunch two",
            LayerKind::Modal {
                modes: struck(1520.0, &PLATE, 0.05, 0x0B0E_0041),
                exciter_ms: 1.2,
                exciter_color: NoiseColor::White,
                exciter_hp: 700.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-14.0)
        .pan(0.22)
        .delay(0.085)
        .style(0.9, 1.3),
    );
    r = r.layer(
        Layer::new(
            "crunch three",
            LayerKind::Modal {
                modes: struck(1120.0, &PLATE, 0.045, 0x0B0E_0042),
                exciter_ms: 1.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 600.0,
                exciter_lp: 8000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-18.0)
        .pan(-0.26)
        .delay(0.17)
        .style(0.8, 1.4),
    );
    r = r.layer(
        Layer::new(
            "wet chew",
            LayerKind::Grain {
                source_hz: 720.0,
                grain_ms: 10.0,
                density: 200.0,
                spray: 0.5,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.01, 0.28),
                pitch_jitter: 0.3,
            },
        )
        .gain(3.0)
        .pan(-0.08)
        .style(0.9, 1.6),
    );
    r.layer(
        Layer::new(
            "jaw thud",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 122.0,
                end_hz: 56.0,
                pitch_tau: 0.02,
                env: AdEnv::perc(0.15),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 1.5),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrate_layers() {
        for r in presets().into_iter().chain(crate::foley::presets()) {
            let full = sg_engine::render(&r, 42);
            let full_rms = full.rms().max(1e-9);
            let (pi, _) = full
                .samples
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
                .unwrap();
            println!("== {} peak at {:.4}s ({:.2} dB)", r.id, pi as f32 / 2.0 / 48_000.0, sg_dsp::gain_to_db(full.peak()));
            let tail_start = full.frames().saturating_sub((0.12 * 48_000.0) as usize);
            let tail_peak = full.samples[tail_start * 2..]
                .iter()
                .fold(0.0f32, |m, s| m.max(s.abs()));
            println!("   tail peak {:.2} dB  fade_out {:.3} fade_in {:.4}", sg_dsp::gain_to_db(tail_peak), r.fade_out, r.fade_in);
            for (i, l) in r.layers.iter().enumerate() {
                let mut partial = r.clone();
                partial.layers[i].mute = true;
                let without = sg_engine::render(&partial, 42);
                let diff: f32 = full
                    .samples
                    .iter()
                    .zip(&without.samples)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    / full.samples.len() as f32;
                let mut one = r.clone();
                for (j, other) in one.layers.iter_mut().enumerate() {
                    other.mute = j != i;
                }
                let solo = sg_engine::render(&one, 42);
                let diff_peak = full
                    .samples
                    .iter()
                    .zip(&without.samples)
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0f32, f32::max);
                println!(
                    "   {:<16} g{:6.1} rms {:7.2} dB  pk {:7.2} dB  crest {:5.2}",
                    l.name,
                    l.gain_db,
                    sg_dsp::gain_to_db(diff.sqrt() / full_rms),
                    sg_dsp::gain_to_db(diff_peak / full.peak()),
                    sg_dsp::gain_to_db(solo.peak() / solo.rms().max(1e-9))
                );
            }
        }
    }

    #[test]
    fn gore_presets_render_cleanly() {
        let presets = presets();
        assert!(presets.len() >= 8, "expected at least 8 gore presets");
        for r in presets {
            assert_eq!(r.category, Category::Gore, "{} wrong category", r.id);
            assert!((3..=6).contains(&r.layers.len()), "{} has {} layers", r.id, r.layers.len());
            assert!(!r.description.is_empty(), "{} has no description", r.id);
            assert!(!r.tags.is_empty(), "{} has no tags", r.id);
            for l in &r.layers {
                assert!(!l.name.is_empty(), "{} has an unnamed layer", r.id);
            }

            let buf = sg_engine::render(&r, 42);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.05, "{} too short", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak <= 1.0, "{} clips (peak {peak})", r.id);
            assert!(buf.samples.iter().all(|s| s.is_finite()), "{} has NaN", r.id);
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
