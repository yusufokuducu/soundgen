//! Foley — footsteps, body landings, cloth, creaking wood and breaking glass.
//!
//! Design notes:
//!  * footsteps are short (0.2–0.4 s) with only a small room tail; body landings
//!    are allowed to breathe
//!  * surface identity lives in the modal bank (research §3.14): concrete 180 Hz,
//!    metal grate ~900 Hz plate modes, wood 320 Hz box modes
//!  * `style` keeps the tonal "boing"/squish layers quiet in realism and pushes
//!    them forward (plus longer tails) in cartoon mode

use crate::prelude::*;
use sg_dsp::rng::Rng;

/// Struck-object mode set from a measured material table, plus a small
/// deterministic mistune so two presets never share an identical body.
fn struck(mat: &Material, base: f32, decay: f32, seed: u64) -> Vec<Mode> {
    let mut rng = Rng::new(seed);
    mat.modes_with_t60(base, decay, 0.5)
        .into_iter()
        .map(|m| Mode::new(m.freq * (1.0 + rng.range(-0.012, 0.012)), m.amp, m.decay))
        .collect()
}

/// Category master chain: clean, small room, minimal drive.
fn foley_master() -> MasterChain {
    MasterChain {
        highpass_hz: 50.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 140.0,
        presence_db: 1.0,
        presence_hz: 3000.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.2,
        drive_db: 0.8,
        reverb_wet: 0.09,
        reverb_size: 0.24,
        reverb_damp: 0.6,
        stereo_width: 1.08,
        ..Default::default()
    }
}

pub fn presets() -> Vec<Recipe> {
    vec![
        footstep_concrete(),
        footstep_grass(),
        footstep_mud(),
        footstep_metal(),
        body_land(),
        cloth_rustle(),
        door_creak(),
        debris_glass(),
    ]
}

/// Boot on concrete: hard heel transient, short scuff and a low body mode.
fn footstep_concrete() -> Recipe {
    let mut r = base("foley_footstep_concrete", "Footstep Concrete", Category::Foley);
    r.duration = 0.26;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.2;
    r.tags = vec!["footstep".into(), "concrete".into(), "walk".into()];
    r.description = "Boot heel on concrete: a hard tick, a short scuff and a tight low body.".into();
    r.master = MasterChain {
        highpass_hz: 60.0,
        presence_hz: 3800.0,
        presence_db: 1.5,
        reverb_wet: 0.08,
        reverb_size: 0.2,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "heel tick",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.009),
                filter: FilterEnv::new(4400.0, 2800.0, 0.006, 1.2),
                mode: 1,
                tame_hz: 15_000.0,
            },
        )
        .gain(5.0)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "scuff",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.055),
                filter: FilterEnv::new(1800.0, 900.0, 0.03, 0.7),
                mode: 2,
                tame_hz: 0.0,
            },
        )
        .gain(-8.0)
        .pan(-0.1)
        .style(0.9, 1.2),
    );
    r = r.layer(
        Layer::new(
            "sole body",
            LayerKind::Modal {
                modes: struck(&CONCRETE, 185.0, 0.055, 0x0F01_0001),
                exciter_ms: 2.0,
                exciter_color: NoiseColor::White,
                exciter_hp: 60.0,
                exciter_lp: 4000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-33.0)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "grit",
            LayerKind::Grain {
                source_hz: 2600.0,
                grain_ms: 5.0,
                density: 170.0,
                spray: 0.6,
                color: NoiseColor::White,
                env: AdEnv::perc(0.06),
                pitch_jitter: 0.3,
            },
        )
        .gain(-6.0)
        .pan(0.15)
        .style(0.7, 1.5),
    )
}

/// Grass: soft filtered swish plus a dense granular rustle.
fn footstep_grass() -> Recipe {
    let mut r = base("foley_footstep_grass", "Footstep Grass", Category::Foley);
    r.duration = 0.24;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.2;
    r.tags = vec!["footstep".into(), "grass".into(), "soft".into()];
    r.description = "Soft step through grass: a filtered swish with a crunchy granular rustle.".into();
    r.master = MasterChain {
        highpass_hz: 70.0,
        low_shelf_db: 0.0,
        presence_hz: 2600.0,
        presence_db: 1.0,
        reverb_wet: 0.07,
        reverb_size: 0.18,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "swish",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.008, 0.14),
                filter: FilterEnv::new(2400.0, 1100.0, 0.05, 0.7),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(0.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "rustle",
            LayerKind::Grain {
                source_hz: 2500.0,
                grain_ms: 20.0,
                density: 95.0,
                spray: 0.8,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.01, 0.16),
                pitch_jitter: 0.45,
            },
        )
        .gain(-4.0)
        .pan(0.18)
        .style(0.9, 1.6),
    );
    r = r.layer(
        Layer::new(
            "soft body",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 118.0,
                end_hz: 68.0,
                pitch_tau: 0.02,
                env: AdEnv::perc(0.065),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.2,
                vibrato: 0.0,
            },
        )
        .gain(-20.0)
        .style(0.7, 1.5),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.08),
                filter: FilterEnv::new(6500.0, 4200.0, 0.03, 0.7),
                mode: 2,
                tame_hz: 16_000.0,
            },
        )
        .gain(-22.0)
        .pan(-0.2)
        .style(0.6, 1.4),
    )
}

/// Mud: a wet squelch with a slipping low band and a few escaping bubbles.
fn footstep_mud() -> Recipe {
    let mut r = base("foley_footstep_mud", "Footstep Mud", Category::Foley);
    r.duration = 0.26;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.3;
    r.tags = vec!["footstep".into(), "mud".into(), "wet".into()];
    r.description = "Step in mud: a squelchy pitch-drop, a wet low band and bubbling suction.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.5,
        presence_hz: 1600.0,
        presence_db: 1.0,
        reverb_wet: 0.07,
        reverb_size: 0.18,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "squelch",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 195.0,
                end_hz: 82.0,
                pitch_tau: 0.035,
                env: AdEnv::perc(0.12),
                fm_hz: 2.4,
                fm_depth: 0.45,
                drive: 0.2,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .style(0.5, 1.9),
    );
    r = r.layer(
        Layer::new(
            "wet noise",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::perc(0.13),
                filter: FilterEnv::new(650.0, 260.0, 0.05, 2.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-2.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "suction",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.15),
                filter: FilterEnv::new(720.0, 300.0, 0.06, 0.9),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-9.0)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "bubbles",
            LayerKind::Grain {
                source_hz: 430.0,
                grain_ms: 12.0,
                density: 60.0,
                spray: 0.9,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.02, 0.15),
                pitch_jitter: 0.6,
            },
        )
        .gain(-10.0)
        .pan(-0.22)
        .style(0.8, 1.7),
    )
}

/// Boot on a metal grate: bright strike plus resonant plate modes.
fn footstep_metal() -> Recipe {
    let mut r = base("foley_footstep_metal", "Footstep Metal", Category::Foley);
    r.duration = 0.32;
    r.fade_in = 0.0;
    r.fade_out = 0.02;
    r.style = 0.22;
    r.tags = vec!["footstep".into(), "metal".into(), "grate".into()];
    r.description = "Boot on a metal grate: a bright strike with ringing plate modes and a low boom.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        presence_hz: 4200.0,
        presence_db: 1.5,
        reverb_wet: 0.09,
        reverb_size: 0.22,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "heel strike",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.007),
                filter: FilterEnv::new(5200.0, 3400.0, 0.004, 1.0),
                mode: 1,
                tame_hz: 15_000.0,
            },
        )
        .gain(3.0)
        .style(1.0, 0.85),
    );
    r = r.layer(
        Layer::new(
            "plate ring",
            LayerKind::Modal {
                modes: struck(&STEEL, 900.0, 0.260, 0x0F02_0001),
                exciter_ms: 0.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 300.0,
                exciter_lp: 11_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-19.0)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "grate rattle",
            LayerKind::Modal {
                modes: struck(&STEEL, 1450.0, 0.120, 0x0F02_0002),
                exciter_ms: 0.4,
                exciter_color: NoiseColor::White,
                exciter_hp: 600.0,
                exciter_lp: 13_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-26.0)
        .pan(0.22)
        .delay(0.008)
        .style(0.8, 1.6),
    );
    r.layer(
        Layer::new(
            "low boom",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 165.0,
                end_hz: 82.0,
                pitch_tau: 0.02,
                env: AdEnv::perc(0.1),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.25,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .pan(-0.18)
        .style(0.8, 1.5),
    )
}

/// Body landing: heavy thud, gear rattle and cloth.
fn body_land() -> Recipe {
    let mut r = base("foley_body_land", "Body Land", Category::Foley);
    r.duration = 0.78;
    r.fade_in = 0.0;
    r.fade_out = 0.04;
    r.style = 0.3;
    r.tags = vec!["body".into(), "land".into(), "gear".into()];
    r.description = "Body hitting the ground: a heavy thud with a clattering gear rattle and cloth.".into();
    r.master = MasterChain {
        highpass_hz: 35.0,
        low_shelf_db: 2.5,
        low_shelf_hz: 120.0,
        presence_hz: 2200.0,
        presence_db: 1.5,
        comp_threshold_db: -18.0,
        drive_db: 1.5,
        reverb_wet: 0.14,
        reverb_size: 0.34,
        stereo_width: 1.15,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "thud",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 72.0,
                end_hz: 38.0,
                pitch_tau: 0.04,
                env: AdEnv::perc(0.34),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.35,
                vibrato: 0.0,
            },
        )
        .gain(0.0)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "impact",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::perc(0.13),
                filter: FilterEnv::new(420.0, 160.0, 0.04, 0.9),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-1.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "gear rattle",
            LayerKind::Modal {
                modes: struck(&STEEL, 2100.0, 0.100, 0x0F03_0001),
                exciter_ms: 1.2,
                exciter_color: NoiseColor::White,
                exciter_hp: 900.0,
                exciter_lp: 12_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-30.0)
        .pan(0.3)
        .delay(0.03)
        .style(0.8, 1.6),
    );
    r = r.layer(
        Layer::new(
            "buckle clink",
            LayerKind::Modal {
                modes: struck(&STEEL, 3200.0, 0.070, 0x0F03_0002),
                exciter_ms: 0.5,
                exciter_color: NoiseColor::White,
                exciter_hp: 1400.0,
                exciter_lp: 14_000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-20.0)
        .pan(-0.35)
        .delay(0.115)
        .style(0.7, 1.6),
    );
    r.layer(
        Layer::new(
            "cloth",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.02, 0.38),
                filter: FilterEnv::new(1700.0, 650.0, 0.1, 0.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(4.0)
        .pan(-0.2)
        .style(0.8, 1.5),
    )
}

/// Cloth rustle: a short grain bed, a fabric swish and a tiny seam tick.
fn cloth_rustle() -> Recipe {
    let mut r = base("foley_cloth_rustle", "Cloth Rustle", Category::Foley);
    r.duration = 0.42;
    r.fade_in = 0.0;
    r.fade_out = 0.025;
    r.style = 0.3;
    r.tags = vec!["cloth".into(), "rustle".into(), "movement".into()];
    r.description = "Short fabric rustle: a granular grain bed over a soft high swish.".into();
    r.master = MasterChain {
        highpass_hz: 80.0,
        low_shelf_db: 0.0,
        presence_hz: 3200.0,
        presence_db: 1.5,
        reverb_wet: 0.1,
        reverb_size: 0.25,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "rustle bed",
            LayerKind::Grain {
                source_hz: 2800.0,
                grain_ms: 14.0,
                density: 150.0,
                spray: 0.75,
                color: NoiseColor::White,
                env: AdEnv::swelled(0.02, 0.3),
                pitch_jitter: 0.4,
            },
        )
        .gain(-7.0)
        .pan(-0.12)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "fabric swish",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.04, 0.26),
                filter: FilterEnv::new(2500.0, 1200.0, 0.08, 0.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(1.0)
        .pan(0.22)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "seam tick",
            LayerKind::Modal {
                modes: struck(&CARDBOARD, 1900.0, 0.020, 0x0F04_0001),
                exciter_ms: 0.3,
                exciter_color: NoiseColor::White,
                exciter_hp: 1200.0,
                exciter_lp: 9000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-32.0)
        .pan(0.32)
        .delay(0.09)
        .style(0.6, 1.5),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.03, 0.22),
                filter: FilterEnv::new(7000.0, 4500.0, 0.05, 0.7),
                mode: 2,
                tame_hz: 17_000.0,
            },
        )
        .gain(-21.0)
        .pan(-0.28)
        .style(0.6, 1.4),
    )
}

/// Creaking door: a slow stick-slip saw drift against wooden body modes.
fn door_creak() -> Recipe {
    let mut r = base("foley_door_creak", "Door Creak", Category::Foley);
    r.duration = 2.1;
    r.fade_in = 0.0;
    r.fade_out = 0.08;
    r.style = 0.35;
    r.tags = vec!["door".into(), "creak".into(), "wood".into(), "long".into()];
    r.description = "Long wooden door creak: a slowly drifting saw hinge, box resonances and dry grit.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        low_shelf_db: 1.0,
        presence_hz: 1800.0,
        presence_db: 1.5,
        comp_threshold_db: -20.0,
        comp_ratio: 2.5,
        drive_db: 1.2,
        reverb_wet: 0.13,
        reverb_size: 0.3,
        reverb_damp: 0.65,
        stereo_width: 1.05,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "hinge slip",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 122.0,
                end_hz: 94.0,
                pitch_tau: 1.4,
                env: AdEnv::swelled(0.18, 1.5),
                fm_hz: 1.9,
                fm_depth: 0.22,
                drive: 0.3,
                vibrato: 0.9,
            },
        )
        .gain(2.0)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "wood groan",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 63.0,
                end_hz: 52.0,
                pitch_tau: 1.8,
                env: AdEnv::swelled(0.26, 1.4),
                fm_hz: 1.01,
                fm_depth: 0.1,
                drive: 0.15,
                vibrato: 0.5,
            },
        )
        .gain(-4.0)
        .style(0.9, 1.4),
    );
    r = r.layer(
        Layer::new(
            "frame modes",
            LayerKind::Modal {
                modes: struck(&OAK, 215.0, 0.300, 0x0F05_0001),
                exciter_ms: 3.0,
                exciter_color: NoiseColor::Pink,
                exciter_hp: 80.0,
                exciter_lp: 2800.0,
                detune_cents: 0.0,
            },
        )
        .gain(-56.0)
        .delay(0.02)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "hinge grit",
            LayerKind::Grain {
                source_hz: 950.0,
                grain_ms: 18.0,
                density: 120.0,
                spray: 0.8,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.15, 1.5),
                pitch_jitter: 0.5,
            },
        )
        .gain(3.0)
        .pan(0.2)
        .style(0.5, 1.8),
    );
    r.layer(
        Layer::new(
            "dust air",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.3, 1.2),
                filter: FilterEnv::new(1500.0, 900.0, 0.6, 0.9),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-4.0)
        .pan(-0.25)
        .style(0.7, 1.7),
    )
}

/// Breaking glass: six detuned shard banks scattered in time and stereo.
fn debris_glass() -> Recipe {
    let mut r = base("foley_debris_glass", "Debris Glass", Category::Foley);
    r.duration = 0.62;
    r.fade_in = 0.0;
    r.fade_out = 0.03;
    r.style = 0.35;
    r.tags = vec!["glass".into(), "debris".into(), "break".into(), "shards".into()];
    r.description = "Window break: an initial crack, six scattering glass shards and a fine dust of splinters.".into();
    r.master = MasterChain {
        highpass_hz: 65.0,
        low_shelf_db: -0.5,
        presence_hz: 5200.0,
        presence_db: 2.0,
        reverb_wet: 0.13,
        reverb_size: 0.3,
        stereo_width: 1.3,
        ..foley_master()
    };
    r = r.layer(
        Layer::new(
            "crack",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::perc(0.012),
                filter: FilterEnv::new(7000.0, 3500.0, 0.004, 1.0),
                mode: 2,
                tame_hz: 17_000.0,
            },
        )
        .gain(3.0)
        .style(1.0, 0.8),
    );

    // Six shards: different pitches, decays, delays and pans.
    let shards: [(f32, f32, f32, f32, f32, u64); 6] = [
        (1250.0, 0.16, 0.018, -0.45, -12.0, 0x0F06_0001),
        (1580.0, 0.11, 0.085, 0.5, -13.0, 0x0F06_0002),
        (1950.0, 0.09, 0.19, -0.2, -13.5, 0x0F06_0003),
        (2320.0, 0.075, 0.29, 0.35, -15.0, 0x0F06_0004),
        (2680.0, 0.06, 0.41, -0.55, -20.0, 0x0F06_0005),
        (3050.0, 0.05, 0.52, 0.15, -19.0, 0x0F06_0006),
    ];
    for (base_hz, decay, delay, pan, gain, seed) in shards {
        r = r.layer(
            Layer::new(
                "shard",
                LayerKind::Modal {
                    modes: struck(&GLASS, base_hz, decay, seed),
                    exciter_ms: 0.3,
                    exciter_color: NoiseColor::White,
                    exciter_hp: 1500.0,
                    exciter_lp: 16_000.0,
                    detune_cents: 0.0,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(delay)
            .style(0.85, 1.5),
        );
    }

    r.layer(
        Layer::new(
            "glass dust",
            LayerKind::Grain {
                source_hz: 5200.0,
                grain_ms: 4.0,
                density: 210.0,
                spray: 0.85,
                color: NoiseColor::White,
                env: AdEnv::perc(0.32),
                pitch_jitter: 0.5,
            },
        )
        .gain(-22.0)
        .pan(0.1)
        .style(0.7, 1.6),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foley_presets_render_cleanly() {
        let presets = presets();
        assert!(presets.len() >= 8, "expected at least 8 foley presets");
        let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), presets.len(), "duplicate preset ids");
        for r in presets {
            assert_eq!(r.category, Category::Foley, "{} wrong category", r.id);
            assert!((3..=8).contains(&r.layers.len()), "{} has {} layers", r.id, r.layers.len());
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
            // Both ends of the realism <-> cartoon axis must stay sane.
            for s in [0.0f32, 1.0] {
                let styled = sg_engine::render_with_style(&r, 42, s);
                assert!(styled.samples.iter().all(|x| x.is_finite()), "{} style {s} has NaN", r.id);
                assert!(styled.peak() > 0.01, "{} style {s} is silent", r.id);
                assert!(styled.peak() <= 1.0, "{} style {s} clips", r.id);
            }

            if r.id.starts_with("foley_footstep") {
                assert!(
                    (0.15..=0.45).contains(&buf.duration()),
                    "{} footstep should be short, got {:.3}s",
                    r.id,
                    buf.duration()
                );
                assert!(
                    r.master.reverb_size <= 0.3 && r.master.reverb_wet <= 0.12,
                    "{} has too much room",
                    r.id
                );
            }
            if r.id == "foley_body_land" {
                assert!(buf.duration() > 0.5, "body land should breathe");
            }
            if r.id == "foley_door_creak" {
                assert!(
                    (1.5..=2.5).contains(&buf.duration()),
                    "door creak should be long, got {:.3}s",
                    buf.duration()
                );
            }
            if r.id == "foley_debris_glass" {
                let shards = r.layers.iter().filter(|l| matches!(l.kind, LayerKind::Modal { .. })).count();
                assert!((6..=8).contains(&shards), "expected 6-8 glass shards, got {shards}");
            }

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
