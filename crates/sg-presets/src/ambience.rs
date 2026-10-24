//! Ambience beds: long, wide, wet and smooth-edged.
//!
//! These are not loops (the engine has no seamless-wrap renderer), so every
//! recipe starts and ends with a 0.3-0.5 s fade. Reverb sits at 0.25-0.4 wet
//! and the master is normalised to -3 dBFS to leave summing headroom.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        wind_ruined_city(),
        distant_battle(),
        zombie_moans_far(),
        underground_hum(),
        fire_crackle(),
    ]
}

/// Wind through broken concrete: brown bed, moving band, thin whistle, debris.
fn wind_ruined_city() -> Recipe {
    let mut r = base(
        "amb_wind_ruined_city",
        "Ruined City Wind",
        Category::Ambience,
    );
    r.duration = 6.0;
    r.fade_in = 0.40;
    r.fade_out = 0.45;
    r.looped = false;
    r.style = 0.2;
    r.tags = vec![
        "ambience".into(),
        "wind".into(),
        "city".into(),
        "wide".into(),
    ];
    r.description =
        "Six seconds of wind moaning through a ruined city, with a slow filter drift and scattered debris.".into();
    r.master = MasterChain {
        highpass_hz: 30.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 100.0,
        presence_db: 1.0,
        presence_hz: 1500.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.32,
        reverb_size: 0.7,
        reverb_damp: 0.55,
        stereo_width: 1.4,
        target_peak_db: -3.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.50, 5.20),
                filter: FilterEnv::new(420.0, 980.0, 2.4, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-6.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "gust",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.80, 4.70),
                filter: FilterEnv::new(760.0, 520.0, 1.6, 0.8),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-13.0)
        .pan(-0.42)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "whistle",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(1.20, 4.20),
                filter: FilterEnv::new(2400.0, 1850.0, 2.0, 11.0),
                mode: 1,
                tame_hz: 9000.0,
            },
        )
        .gain(-28.0)
        .pan(0.5)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "debris",
            LayerKind::Grain {
                source_hz: 3000.0,
                grain_ms: 30.0,
                density: 2.5,
                spray: 0.95,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.40, 5.00),
                pitch_jitter: 0.5,
            },
        )
        .gain(-30.0)
        .pan(0.3)
        .style(1.0, 1.3),
    );
    r.layer(
        Layer::new(
            "rumble",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.40, 5.20),
                filter: FilterEnv::new(95.0, 70.0, 3.0, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-18.0)
        .style(1.0, 1.0),
    )
}

/// Distant battle: low rumble plus five muffled gunshot thumps at irregular times.
fn distant_battle() -> Recipe {
    let mut r = base("amb_distant_battle", "Distant Battle", Category::Ambience);
    r.duration = 6.5;
    r.fade_in = 0.45;
    r.fade_out = 0.50;
    r.looped = false;
    r.style = 0.2;
    r.tags = vec![
        "ambience".into(),
        "battle".into(),
        "distant".into(),
        "war".into(),
    ];
    r.description =
        "A low war rumble with five muffled gunshot thumps spread irregularly across six seconds.".into();
    r.master = MasterChain {
        highpass_hz: 35.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 90.0,
        presence_db: -1.0,
        presence_hz: 1000.0,
        comp_threshold_db: -16.0,
        comp_ratio: 1.3,
        drive_db: 0.0,
        reverb_wet: 0.35,
        reverb_size: 0.85,
        reverb_damp: 0.65,
        stereo_width: 1.35,
        target_peak_db: -3.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "rumble",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.50, 5.70),
                filter: FilterEnv::new(120.0, 80.0, 2.0, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 0.9),
    );
    let thumps = [
        ("thump_1", 700.0, 0.55, 0.35, -16.0, -0.35),
        ("thump_2", 550.0, 0.45, 1.25, -20.0, 0.40),
        ("thump_3", 820.0, 0.70, 2.15, -15.0, 0.10),
        ("thump_4", 620.0, 0.50, 3.35, -21.0, -0.50),
        ("thump_5", 480.0, 0.65, 4.55, -18.0, 0.35),
    ];
    for (name, hz, decay, delay, gain, pan) in thumps {
        r = r.layer(
            Layer::new(
                name,
                LayerKind::Noise {
                    color: NoiseColor::Brown,
                    env: AdEnv::perc(decay),
                    filter: FilterEnv::new(hz, hz * 0.25, 0.07, 0.9),
                    mode: 0,
                    tame_hz: 3000.0,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(delay)
            .style(1.0, 1.2),
        );
    }
    r
}

/// Three far-off zombie growls behind a filtered night bed.
fn zombie_moans_far() -> Recipe {
    let mut r = base("amb_zombie_moans_far", "Far Zombie Moans", Category::Ambience);
    r.duration = 7.0;
    r.fade_in = 0.45;
    r.fade_out = 0.50;
    r.looped = false;
    r.style = 0.3;
    r.tags = vec![
        "ambience".into(),
        "zombie".into(),
        "distant".into(),
        "night".into(),
    ];
    r.description =
        "Three heavily filtered growls drift in from different directions over a dark low bed.".into();
    r.master = MasterChain {
        highpass_hz: 40.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 110.0,
        presence_db: 1.5,
        presence_hz: 1100.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.5,
        drive_db: 0.0,
        reverb_wet: 0.38,
        reverb_size: 0.9,
        reverb_damp: 0.7,
        stereo_width: 1.3,
        target_peak_db: -3.0,
        ..Default::default()
    };
    let growls = [
        ("growl_a", 78.0_f32, 60.0_f32, 0.80_f32, 0.30_f32, -13.0_f32, -0.40_f32, 2.2_f32),
        ("growl_b", 64.0, 50.0, 1.00, 2.20, -15.0, 0.42, 2.0),
        ("growl_c", 88.0, 68.0, 0.70, 4.00, -17.0, 0.05, 1.8),
    ];
    for (name, start, end, tau, delay, gain, pan, decay) in growls {
        r = r.layer(
            Layer::new(
                name,
                LayerKind::Tone {
                    shape: WaveShape::Saw,
                    start_hz: start,
                    end_hz: end,
                    pitch_tau: tau,
                    env: AdEnv::swelled(0.70, decay),
                    fm_hz: 1.02,
                    fm_depth: 0.18,
                    drive: 0.45,
                    vibrato: 0.6,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(delay)
            .style(1.0, 1.3),
        );
    }
    r = r.layer(
        Layer::new(
            "night_breath",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(1.00, 5.40),
                filter: FilterEnv::new(480.0, 220.0, 1.8, 1.1),
                mode: 1,
                tame_hz: 7000.0,
            },
        )
        .gain(-23.0)
        .pan(0.2)
        .style(1.0, 1.0),
    );
    r = r.layer(
        Layer::new(
            "rasp",
            LayerKind::Grain {
                source_hz: 360.0,
                grain_ms: 22.0,
                density: 55.0,
                spray: 0.75,
                color: NoiseColor::Pink,
                env: AdEnv::swelled(1.20, 4.60),
                pitch_jitter: 0.4,
            },
        )
        .gain(-27.0)
        .pan(-0.25)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "low_bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.50, 6.10),
                filter: FilterEnv::new(85.0, 65.0, 2.5, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-20.0)
        .style(1.0, 0.9),
    )
}

/// Underground hum: two detuned drones, filtered air and three water drips.
fn underground_hum() -> Recipe {
    let mut r = base(
        "amb_underground_hum",
        "Underground Hum",
        Category::Ambience,
    );
    r.duration = 6.0;
    r.fade_in = 0.40;
    r.fade_out = 0.45;
    r.looped = false;
    r.style = 0.25;
    r.tags = vec![
        "ambience".into(),
        "underground".into(),
        "hum".into(),
        "drips".into(),
    ];
    r.description =
        "A subterranean 55 Hz hum with a fifth above it, damp air and three water drips in the dark.".into();
    r.master = MasterChain {
        highpass_hz: 28.0,
        low_shelf_db: 2.5,
        low_shelf_hz: 80.0,
        presence_db: -0.5,
        presence_hz: 800.0,
        comp_threshold_db: -24.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.40,
        reverb_size: 0.9,
        reverb_damp: 0.6,
        stereo_width: 1.25,
        target_peak_db: -3.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "drone",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 55.0,
                end_hz: 54.0,
                pitch_tau: 3.0,
                env: AdEnv::swelled(0.60, 5.10),
                fm_hz: 0.5,
                fm_depth: 0.04,
                drive: 0.15,
                vibrato: 0.1,
            },
        )
        .gain(-10.0)
        .style(1.0, 1.1),
    );
    r = r.layer(
        Layer::new(
            "fifth",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 82.4,
                end_hz: 81.0,
                pitch_tau: 3.0,
                env: AdEnv::swelled(0.70, 5.00),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-17.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "damp_air",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.50, 5.20),
                filter: FilterEnv::new(260.0, 180.0, 2.0, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-22.0)
        .pan(-0.3)
        .style(1.0, 1.0),
    );
    let drips = [
        ("drip_1", 1350.0_f32, 0.35_f32, 1.10_f32, -20.0_f32, -0.45_f32),
        ("drip_2", 1720.0, 0.30, 2.65, -22.0, 0.50),
        ("drip_3", 1120.0, 0.40, 4.35, -23.0, 0.10),
    ];
    for (name, base_hz, decay, delay, gain, pan) in drips {
        r = r.layer(
            Layer::new(
                name,
                LayerKind::Modal {
                    modes: vec![
                        Mode::new(base_hz, 1.0, decay),
                        Mode::new(base_hz * 2.32, 0.45, decay * 0.7),
                        Mode::new(base_hz * 3.5, 0.20, decay * 0.45),
                    ],
                    exciter_ms: 1.5,
                    exciter_color: NoiseColor::White,
                    exciter_hp: 900.0,
                    exciter_lp: 7000.0,
                    detune_cents: 0.0,
                },
            )
            .gain(gain)
            .pan(pan)
            .delay(delay)
            .style(1.0, 1.4),
        );
    }
    r
}

/// Fire: brown bed, band-passed roar, velvet crackle grains and two pops.
fn fire_crackle() -> Recipe {
    let mut r = base("amb_fire_crackle", "Fire Crackle", Category::Ambience);
    r.duration = 5.0;
    r.fade_in = 0.35;
    r.fade_out = 0.45;
    r.looped = false;
    r.style = 0.25;
    r.tags = vec![
        "ambience".into(),
        "fire".into(),
        "crackle".into(),
        "warm".into(),
    ];
    r.description =
        "Five seconds of campfire: a warm brown bed, a breathing roar and sparse velvet crackle pops.".into();
    r.master = MasterChain {
        highpass_hz: 30.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 100.0,
        presence_db: 0.5,
        presence_hz: 2200.0,
        comp_threshold_db: -22.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.28,
        reverb_size: 0.5,
        reverb_damp: 0.5,
        stereo_width: 1.3,
        target_peak_db: -3.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: AdEnv::swelled(0.40, 4.30),
                filter: FilterEnv::new(320.0, 220.0, 1.5, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-10.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "roar",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: AdEnv::swelled(0.50, 4.20),
                filter: FilterEnv::new(720.0, 420.0, 1.2, 0.7),
                mode: 1,
                tame_hz: 8000.0,
            },
        )
        .gain(-16.0)
        .pan(-0.2)
        .style(1.0, 1.0),
    );
    r = r.layer(
        Layer::new(
            "crackle",
            LayerKind::Grain {
                source_hz: 4500.0,
                grain_ms: 6.0,
                density: 45.0,
                spray: 1.0,
                color: NoiseColor::Velvet,
                env: AdEnv::swelled(0.30, 4.40),
                pitch_jitter: 0.6,
            },
        )
        .gain(-18.0)
        .pan(0.25)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "pop_1",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(900.0, 1.0, 0.060),
                    Mode::new(1900.0, 0.45, 0.040),
                    Mode::new(3300.0, 0.18, 0.030),
                ],
                exciter_ms: 1.0,
                exciter_color: NoiseColor::Brown,
                exciter_hp: 300.0,
                exciter_lp: 6000.0,
                detune_cents: 0.0,
            },
        )
        .gain(-22.0)
        .pan(0.30)
        .delay(0.85)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "pop_2",
            LayerKind::Modal {
                modes: vec![
                    Mode::new(1300.0, 1.0, 0.050),
                    Mode::new(2600.0, 0.45, 0.035),
                    Mode::new(4400.0, 0.18, 0.020),
                ],
                exciter_ms: 0.8,
                exciter_color: NoiseColor::Brown,
                exciter_hp: 400.0,
                exciter_lp: 6500.0,
                detune_cents: 0.0,
            },
        )
        .gain(-24.0)
        .pan(-0.40)
        .delay(2.75)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "hiss",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: AdEnv::swelled(0.40, 4.20),
                filter: FilterEnv::new(3500.0, 2800.0, 1.0, 0.8),
                mode: 2,
                tame_hz: 9500.0,
            },
        )
        .gain(-28.0)
        .pan(0.1)
        .style(1.0, 0.8),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambience_presets_render_cleanly() {
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
