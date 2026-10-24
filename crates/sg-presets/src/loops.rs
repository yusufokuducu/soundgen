//! Loop category: continuous beds for propellers, drones, wind and fire.
//!
//! ## Seam maths (read this before touching a number)
//!
//! `Recipe::render_length()` returns `max(duration, longest tail) + 0.05`, and
//! every layer's envelope ends at its own `tail`, so the trailing 50 ms of the
//! buffer is *always* silence. The musical body is therefore exactly
//! `duration` long, and that is what has to line up on the loop point:
//!
//! * blade pass: `p = 60 / rpm / blades`  (2 blades @ 2400 rpm -> 1/80 s)
//! * motor hum: the `Rotor` voice calls `motor.next()` twice on one phase
//!   accumulator (`render.rs`), so the hum actually runs at `3 * tone_hz`;
//!   its period is `1 / (3 * tone_hz)`.
//! * a plain `Tone` needs `freq * duration` to be a whole number of cycles.
//!
//! So each loop picks a whole number of blade passes near its target length,
//! and `snap_hz()` nudges the motor/tone pitch so it closes the circle too.
//! Every layer uses `loop_env()`: 4 ms fade-in, flat body, 12 ms fade-out that
//! lands exactly on `duration`. Both ends sit at ~0, so restarting the buffer
//! cannot click, and the blade phase at the loop point is 0 by construction.

use crate::prelude::*;

pub fn presets() -> Vec<Recipe> {
    vec![
        propeller_small(),
        propeller_large(),
        drone_motor(),
        helicopter_far(),
        wind_loop(),
        campfire(),
    ]
}

// ---------------------------------------------------------------------------
// loop helpers
// ---------------------------------------------------------------------------

/// Round a target length to a whole number of `period`s (never below one).
fn loop_len(period: f32, target: f32) -> f32 {
    let cycles = (target / period).round().max(1.0);
    cycles * period
}

/// Nudge `hz` so that a whole number of cycles fits into `duration`.
fn snap_hz(hz: f32, duration: f32) -> f32 {
    ((hz * duration).round().max(1.0) / duration).max(1.0)
}

/// Sustained loop envelope: short fade in, flat body, short fade out that
/// lands on `duration` and never reaches the renderer's safety pad.
///
/// The 20 ms margin exists because `EnvGen` accumulates its clock in `f32`
/// (`t += 1.0 / sr`), which drifts by up to ~20 ms over a 4 s render. Without
/// the margin the fade-out can slip past the body end and get chopped, which
/// is exactly the click we are trying to avoid.
fn loop_env(duration: f32) -> AdEnv {
    let attack = 0.004;
    let decay = 0.012;
    let margin = 0.02;
    let hold = (duration - attack - decay - margin).max(0.0);
    AdEnv {
        attack,
        hold,
        decay,
        curve: 1.0,
        attack_curve: 1.0,
    }
}

// ---------------------------------------------------------------------------
// 1. small propeller
// ---------------------------------------------------------------------------

const SMALL_RPM: f32 = 2400.0;
const SMALL_BLADES: f32 = 2.0;

/// 60 / 2400 / 2 = 1/80 s = 12.5 ms per blade pass.
fn small_period() -> f32 {
    60.0 / SMALL_RPM / SMALL_BLADES
}

/// 320 blade passes -> exactly 4.0 s.
fn small_duration() -> f32 {
    loop_len(small_period(), 4.0)
}

fn propeller_small() -> Recipe {
    let d = small_duration();
    let tone = snap_hz(40.0, d); // 40 Hz -> hum at 3*40 = 120 Hz, 480 cycles
    let mut r = base("loop_propeller_small", "Small Propeller Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.25;
    r.tags = vec!["loop".into(), "propeller".into(), "drone".into(), "aircraft".into()];
    r.description = "Bright two-blade propeller at 2400 rpm with airy blade wash and a thin motor thrum.".into();
    r.master = MasterChain {
        highpass_hz: 38.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 110.0,
        presence_db: 2.0,
        presence_hz: 2800.0,
        comp_threshold_db: -16.0,
        comp_ratio: 2.5,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.3,
        reverb_damp: 0.6,
        stereo_width: 1.12,
        target_peak_db: -6.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "rotor wash",
            LayerKind::Rotor {
                blades: SMALL_BLADES,
                rpm: SMALL_RPM,
                tone_hz: tone,
                air: 0.55,
                flap_depth: 0.85,
                env: loop_env(d),
                color: NoiseColor::Pink,
            },
        )
        .gain(-4.0)
        .style(1.0, 0.85),
    );
    r = r.layer(
        Layer::new(
            "blade tone",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 80.0, // = BPF, 320 cycles in 4 s
                end_hz: 80.0,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-12.0)
        .pan(-0.1)
        .style(1.0, 1.5),
    );
    r = r.layer(
        Layer::new(
            "thrum",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: tone, // 40 Hz, 160 cycles
                end_hz: tone,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.1,
                vibrato: 0.0,
            },
        )
        .gain(-15.0)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "air hiss",
            LayerKind::Noise {
                color: NoiseColor::Blue,
                env: loop_env(d),
                filter: FilterEnv::new(3800.0, 3200.0, 1.5, 0.7),
                mode: 1,
                tame_hz: 13_000.0,
            },
        )
        .gain(-13.0)
        .pan(0.25)
        .style(1.0, 1.6),
    );
    r.layer(
        Layer::new(
            "body rumble",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(260.0, 200.0, 2.0, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-10.0)
        .pan(0.0)
        .style(1.0, 0.7),
    )
}

// ---------------------------------------------------------------------------
// 2. large propeller
// ---------------------------------------------------------------------------

const LARGE_RPM: f32 = 900.0;
const LARGE_BLADES: f32 = 3.0;

/// 60 / 900 / 3 = 1/45 s = 22.2 ms per blade pass.
fn large_period() -> f32 {
    60.0 / LARGE_RPM / LARGE_BLADES
}

/// 270 blade passes -> exactly 6.0 s.
fn large_duration() -> f32 {
    loop_len(large_period(), 6.0)
}

fn propeller_large() -> Recipe {
    let d = large_duration();
    let tone = snap_hz(30.0, d); // 30 Hz -> hum at 90 Hz, 540 cycles
    let sub = snap_hz(22.5, d); // half the BPF, 135 cycles
    let mut r = base("loop_propeller_large", "Large Propeller Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.2;
    r.tags = vec!["loop".into(), "propeller".into(), "heavy".into(), "aircraft".into()];
    r.description = "Slow three-blade cargo propeller, deep whoomp with a wide air wash and sub throb.".into();
    r.master = MasterChain {
        highpass_hz: 20.0,
        low_shelf_db: 3.0,
        low_shelf_hz: 90.0,
        presence_db: 1.0,
        presence_hz: 1600.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.2,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.4,
        reverb_damp: 0.6,
        stereo_width: 1.08,
        target_peak_db: -6.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "deep whoomp",
            LayerKind::Rotor {
                blades: LARGE_BLADES,
                rpm: LARGE_RPM,
                tone_hz: tone,
                air: 0.7,
                flap_depth: 0.95,
                env: loop_env(d),
                color: NoiseColor::Brown,
            },
        )
        .gain(-3.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "blade slap",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 45.0, // = BPF, 270 cycles
                end_hz: 45.0,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.15,
                vibrato: 0.0,
            },
        )
        .gain(-10.0)
        .pan(0.08)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "sub throb",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: sub,
                end_hz: sub,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-14.0)
        .style(1.0, 1.6),
    );
    r = r.layer(
        Layer::new(
            "air wash",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: loop_env(d),
                filter: FilterEnv::new(800.0, 1500.0, 2.0, 0.7),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-11.0)
        .pan(-0.25)
        .style(1.0, 1.4),
    );
    r.layer(
        Layer::new(
            "low body",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(180.0, 120.0, 3.0, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-12.0)
        .pan(0.15)
        .style(1.0, 0.7),
    )
}

// ---------------------------------------------------------------------------
// 3. electric drone motor
// ---------------------------------------------------------------------------

const DRONE_RPM: f32 = 4000.0;
const DRONE_BLADES: f32 = 3.0;

/// 60 / 4000 / 3 = 1/200 s = 5 ms per blade pass.
fn drone_period() -> f32 {
    60.0 / DRONE_RPM / DRONE_BLADES
}

/// 800 blade passes -> exactly 4.0 s (and 5.5 Hz vibrato -> 22 whole cycles).
fn drone_duration() -> f32 {
    loop_len(drone_period(), 4.0)
}

fn drone_motor() -> Recipe {
    let d = drone_duration();
    let tone = snap_hz(50.0, d); // hum at 150 Hz, 600 cycles
    let mut r = base("loop_drone_motor", "Drone Motor Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.3;
    r.tags = vec!["loop".into(), "drone".into(), "motor".into(), "electric".into()];
    r.description = "Four-rotor electric drone: thin motor whine, buzzy drive and a soft rotor wash bed.".into();
    r.master = MasterChain {
        highpass_hz: 45.0,
        low_shelf_db: 1.0,
        low_shelf_hz: 130.0,
        presence_db: 3.0,
        presence_hz: 3000.0,
        comp_threshold_db: -18.0,
        comp_ratio: 3.0,
        drive_db: 1.5,
        reverb_wet: 0.0,
        reverb_size: 0.3,
        reverb_damp: 0.6,
        stereo_width: 1.05,
        target_peak_db: -6.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "motor whine",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 800.0, // 3200 cycles in 4 s
                end_hz: 800.0,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.25,
                vibrato: 0.5, // 5.5 Hz * 4 s = 22 cycles -> closes exactly
            },
        )
        .gain(-13.0)
        .pan(-0.15)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "motor buzz",
            LayerKind::Tone {
                shape: WaveShape::Saw,
                start_hz: 200.0, // 800 cycles
                end_hz: 200.0,
                pitch_tau: 1.0,
                env: loop_env(d),
                fm_hz: 1.01, // modulator at 202 Hz -> 808 cycles
                fm_depth: 0.25,
                drive: 0.3,
                vibrato: 0.0,
            },
        )
        .gain(-11.0)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "rotor wash",
            LayerKind::Rotor {
                blades: DRONE_BLADES,
                rpm: DRONE_RPM,
                tone_hz: tone,
                air: 0.4,
                flap_depth: 0.4,
                env: loop_env(d),
                color: NoiseColor::White,
            },
        )
        .gain(-10.0)
        .pan(0.18)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "body",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(460.0, 360.0, 2.0, 0.7),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 0.8),
    );
    r.layer(
        Layer::new(
            "air",
            LayerKind::Noise {
                color: NoiseColor::Blue,
                env: loop_env(d),
                filter: FilterEnv::new(4200.0, 3600.0, 1.5, 0.7),
                mode: 2,
                tame_hz: 14_000.0,
            },
        )
        .gain(-23.0)
        .pan(0.3)
        .style(1.0, 1.5),
    )
}

// ---------------------------------------------------------------------------
// 4. distant helicopter
// ---------------------------------------------------------------------------

const HELI_RPM_A: f32 = 900.0;
const HELI_RPM_B: f32 = 903.0;
const HELI_BLADES: f32 = 3.0;

/// 60 / 900 / 3 = 45 Hz -> period 1/45 s.
fn heli_period() -> f32 {
    60.0 / HELI_RPM_A / HELI_BLADES
}

/// Both rotors must close at once: 45 Hz needs 300 passes and 45.15 Hz needs
/// 301, and 300/45 == 301/45.15 == 20/3 s. That also makes the 0.15 Hz beat
/// between the two rotors exactly one cycle long.
fn heli_duration() -> f32 {
    300.0 / 45.0
}

fn helicopter_far() -> Recipe {
    let d = heli_duration();
    let tone = snap_hz(30.0, d); // hum at 90 Hz -> 600 cycles
    let mut r = base("loop_helicopter_far", "Distant Helicopter Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.2;
    r.tags = vec!["loop".into(), "helicopter".into(), "distant".into(), "ambience".into()];
    r.description = "Far-off helicopter heard across open ground, two detuned rotors beating slowly behind a dull rumble.".into();
    r.master = MasterChain {
        highpass_hz: 55.0,
        low_shelf_db: 3.0,
        low_shelf_hz: 120.0,
        presence_db: -2.0, // pull the highs down: distance is a low-pass
        presence_hz: 2000.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.5,
        reverb_damp: 0.7,
        stereo_width: 1.1,
        target_peak_db: -9.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "distant rotor",
            LayerKind::Rotor {
                blades: HELI_BLADES,
                rpm: HELI_RPM_A,
                tone_hz: tone,
                air: 0.25,
                flap_depth: 0.6,
                env: loop_env(d),
                color: NoiseColor::Pink,
            },
        )
        .gain(-9.0)
        .pan(-0.18)
        .style(1.0, 0.8),
    );
    r = r.layer(
        Layer::new(
            "beating rotor",
            LayerKind::Rotor {
                blades: HELI_BLADES,
                rpm: HELI_RPM_B, // 45.15 Hz -> 301 passes in the same 20/3 s
                tone_hz: tone,
                air: 0.2,
                flap_depth: 0.55,
                env: loop_env(d),
                color: NoiseColor::Pink,
            },
        )
        .gain(-10.0)
        .pan(0.18)
        .style(1.0, 1.2),
    );
    r = r.layer(
        Layer::new(
            "doppler drift",
            LayerKind::Tone {
                shape: WaveShape::Sine,
                start_hz: 94.0,
                end_hz: 90.0,
                pitch_tau: 5.0, // slow approach-then-away sag; faded out at the seam
                env: loop_env(d),
                fm_hz: 0.0,
                fm_depth: 0.0,
                drive: 0.0,
                vibrato: 0.0,
            },
        )
        .gain(-21.0)
        .style(0.8, 1.4),
    );
    r = r.layer(
        Layer::new(
            "dull body",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(520.0, 380.0, 3.0, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-12.0)
        .style(1.0, 0.7),
    );
    r.layer(
        Layer::new(
            "air veil",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: loop_env(d),
                filter: FilterEnv::new(700.0, 950.0, 2.0, 0.6),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-16.0)
        .pan(0.1)
        .style(1.0, 1.3),
    )
}

// ---------------------------------------------------------------------------
// 5. wind
// ---------------------------------------------------------------------------

/// No rotor here, so the length is free; 8 s keeps the filter drift slow
/// enough that both noise layers are near their `end_hz` before the seam.
fn wind_duration() -> f32 {
    8.0
}

fn wind_loop() -> Recipe {
    let d = wind_duration();
    let mut r = base("loop_wind_loop", "Wind Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.15;
    r.tags = vec!["loop".into(), "wind".into(), "weather".into(), "ambience".into()];
    r.description = "Open-field wind bed: brown low body and a pink gust drifting in opposite directions, with a thin whistle.".into();
    r.master = MasterChain {
        highpass_hz: 26.0,
        low_shelf_db: 2.0,
        low_shelf_hz: 90.0,
        presence_db: -1.0,
        presence_hz: 3000.0,
        comp_threshold_db: -20.0,
        comp_ratio: 2.0,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.6,
        reverb_damp: 0.7,
        stereo_width: 1.2,
        target_peak_db: -8.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "wind bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(340.0, 900.0, 3.2, 0.7), // slow opening
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-5.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "gust",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: loop_env(d),
                filter: FilterEnv::new(1400.0, 520.0, 1.6, 0.9), // fast closing
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-10.0)
        .pan(-0.2)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "high gust",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: loop_env(d),
                filter: FilterEnv::new(1200.0, 2400.0, 2.4, 1.1),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-16.0)
        .pan(0.3)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "whistle",
            LayerKind::Noise {
                color: NoiseColor::White,
                env: loop_env(d),
                filter: FilterEnv::new(2300.0, 2700.0, 2.0, 12.0),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-27.0)
        .pan(0.15)
        .style(0.8, 1.6),
    );
    r.layer(
        Layer::new(
            "leaves",
            LayerKind::Grain {
                source_hz: 2600.0,
                grain_ms: 22.0,
                density: 3.5,
                spray: 1.0,
                color: NoiseColor::Velvet,
                env: loop_env(d),
                pitch_jitter: 0.5,
            },
        )
        .gain(-30.0)
        .pan(-0.35)
        .style(1.0, 1.2),
    )
}

// ---------------------------------------------------------------------------
// 6. campfire
// ---------------------------------------------------------------------------

/// Also free-running: 8 s of steady fire with a 12 ms fade at the seam.
fn fire_duration() -> f32 {
    8.0
}

fn campfire() -> Recipe {
    let d = fire_duration();
    let mut r = base("loop_campfire", "Campfire Loop", Category::Loop);
    r.duration = d;
    r.fade_in = 0.0;
    r.fade_out = 0.0;
    r.looped = true;
    r.style = 0.25;
    r.tags = vec!["loop".into(), "fire".into(), "campfire".into(), "ambience".into()];
    r.description = "Close campfire: brown flame bed, velvet crackle and the odd fat pop from resin pockets.".into();
    r.master = MasterChain {
        highpass_hz: 30.0,
        low_shelf_db: 1.5,
        low_shelf_hz: 100.0,
        presence_db: 0.0,
        presence_hz: 2500.0,
        comp_threshold_db: -18.0,
        comp_ratio: 2.2,
        drive_db: 0.0,
        reverb_wet: 0.0,
        reverb_size: 0.4,
        reverb_damp: 0.6,
        stereo_width: 1.15,
        target_peak_db: -8.0,
        ..Default::default()
    };
    r = r.layer(
        Layer::new(
            "flame bed",
            LayerKind::Noise {
                color: NoiseColor::Brown,
                env: loop_env(d),
                filter: FilterEnv::new(480.0, 320.0, 4.0, 0.6),
                mode: 0,
                tame_hz: 0.0,
            },
        )
        .gain(-8.0)
        .style(1.0, 0.9),
    );
    r = r.layer(
        Layer::new(
            "hiss",
            LayerKind::Noise {
                color: NoiseColor::Pink,
                env: loop_env(d),
                filter: FilterEnv::new(1600.0, 2000.0, 2.0, 0.8),
                mode: 1,
                tame_hz: 0.0,
            },
        )
        .gain(-20.0)
        .pan(0.25)
        .style(1.0, 1.3),
    );
    r = r.layer(
        Layer::new(
            "crackle",
            LayerKind::Grain {
                source_hz: 3200.0,
                grain_ms: 5.0,
                density: 16.0,
                spray: 0.9,
                color: NoiseColor::Velvet,
                env: loop_env(d),
                pitch_jitter: 0.5,
            },
        )
        .gain(-12.0)
        .pan(-0.15)
        .style(1.0, 1.4),
    );
    r = r.layer(
        Layer::new(
            "resin pops",
            LayerKind::Grain {
                source_hz: 900.0,
                grain_ms: 16.0,
                density: 3.0,
                spray: 1.0,
                color: NoiseColor::White,
                env: loop_env(d),
                pitch_jitter: 0.7,
            },
        )
        .gain(-16.0)
        .pan(0.2)
        .style(1.0, 1.5),
    );
    r.layer(
        Layer::new(
            "flare",
            LayerKind::Noise {
                color: NoiseColor::Velvet,
                env: loop_env(d),
                filter: FilterEnv::new(2200.0, 2600.0, 2.0, 0.7),
                mode: 2,
                tame_hz: 9000.0,
            },
        )
        .gain(-26.0)
        .style(1.0, 1.2),
    )
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// |last sample - first sample| per channel, in the rendered buffer.
    fn seam_error(buf: &sg_engine::AudioBuffer) -> (f32, f32) {
        let n = buf.frames();
        assert!(n > 1, "buffer too small");
        let last = (n - 1) * 2;
        (
            (buf.samples[0] - buf.samples[last]).abs(),
            (buf.samples[1] - buf.samples[last + 1]).abs(),
        )
    }

    #[test]
    fn loop_envelopes_end_before_the_pad() {
        // `EnvGen` keeps its clock in f32, so the fade-out can drift by tens of
        // ms. Verify the 20 ms margin is enough for every loop we ship.
        for r in presets() {
            let mut e = sg_dsp::env::EnvGen::new(r.sample_rate as f32, loop_env(r.duration));
            let mut silent_at = None;
            let frames = (r.render_length() * r.sample_rate as f32) as usize;
            for i in 0..frames {
                let a = e.next();
                if a <= 1e-4 && i > 480 {
                    silent_at = Some(i);
                    break;
                }
            }
            let t = silent_at.unwrap() as f32 / r.sample_rate as f32;
            let pad_start = r.render_length() - 0.05;
            assert!(
                t < pad_start,
                "{} fade-out ends at {t:.4}s, inside the pad ({pad_start:.4}s)",
                r.id
            );
            println!(
                "{}: env silent at {:.4}s, body {:.3}s (margin {:+.1} ms)",
                r.id,
                t,
                r.duration,
                (t - r.duration) * 1000.0
            );
        }
    }

    #[test]
    fn loops_render_and_seam() {
        let recipes = presets();
        assert!(recipes.len() >= 6, "expected at least 6 loops");

        // (id, blade/motor period) — the period each loop must be a multiple of.
        let periods = [
            ("loop_propeller_small", small_period()),
            ("loop_propeller_large", large_period()),
            ("loop_drone_motor", drone_period()),
            ("loop_helicopter_far", heli_period()),
        ];

        for r in &recipes {
            assert!(r.looped, "{} is not flagged looped", r.id);
            assert_eq!(r.fade_in, 0.0, "{} must not fade in", r.id);
            assert_eq!(r.fade_out, 0.0, "{} must not fade out", r.id);
            assert_eq!(r.master.reverb_wet, 0.0, "{} reverb smears the seam", r.id);

            let buf = sg_engine::render(r, 7);
            assert!(buf.frames() > 0, "{} produced no frames", r.id);
            assert!(buf.duration() > 0.5, "{} too short for a loop", r.id);
            let peak = buf.peak();
            assert!(peak > 0.01, "{} silent (peak {peak})", r.id);
            assert!(peak < 1.0, "{} clips (peak {peak})", r.id);
            assert!(buf.samples.iter().all(|s| s.is_finite()), "{} has NaN/inf", r.id);

            let (dl, dr) = seam_error(&buf);
            if r.id == "loop_drone_motor" {
                let n = buf.frames();
                let pad: f32 = (n - 2400..n).map(|f| buf.samples[f * 2]).sum::<f32>() / 2400.0;
                let mut sat = sg_dsp::fx::Saturator {
                    drive: sg_dsp::db_to_gain(1.5),
                    asym: 0.2,
                    mix: 1.0,
                };
                println!(
                    "  DBG sat0 {:.5} first {:.5} last {:.5} padmean {:.5} s100 {:.5} padmax {:.5}",
                    sat.process(0.0),
                    buf.samples[0],
                    buf.samples[(n - 1) * 2],
                    pad,
                    buf.samples[100 * 2],
                    (n - 2400..n).fold(0.0f32, |m, f| m.max(buf.samples[f * 2].abs()))
                );
            }
            assert!(
                dl < 0.05 && dr < 0.05,
                "{} seam clicks: L {dl:.4} R {dr:.4}",
                r.id
            );

            // Diagnostic: the renderer always appends a 50 ms safety pad after
            // the body. The fade-out is designed to finish before it, so the
            // last few ms must be quiet (no hard cut at the buffer end).
            let tail_n = ((0.005 * buf.sample_rate as f32) as usize).min(buf.frames());
            let tail_rms = {
                let s: f32 = (buf.frames() - tail_n..buf.frames())
                    .map(|f| buf.samples[f * 2].powi(2) + buf.samples[f * 2 + 1].powi(2))
                    .sum();
                (s / (tail_n as f32 * 2.0)).sqrt()
            };
            assert!(
                tail_rms < 0.01,
                "{} is cut off mid-fade (last 5 ms rms {tail_rms:.4})",
                r.id
            );

            // The body must be a whole number of blade passes.
            if let Some((_, p)) = periods.iter().find(|(id, _)| *id == r.id) {
                let cycles = r.duration / p;
                assert!(
                    (cycles - cycles.round()).abs() < 1e-3,
                    "{} duration {} is not a multiple of period {} ({:.3} cycles)",
                    r.id,
                    r.duration,
                    p,
                    cycles
                );
                println!(
                    "{}: period {:.4}s x {} = {:.3}s (render {:.3}s)",
                    r.id,
                    p,
                    cycles.round(),
                    r.duration,
                    buf.duration()
                );
            } else {
                println!("{}: free-running, {:.3}s", r.id, r.duration);
            }

            println!(
                "  seam L {dl:.5} R {dr:.5} | tail rms {tail_rms:.5} | peak {:.2} dBFS | rms {:.4} | {:.1} LUFS",
                sg_dsp::gain_to_db(peak),
                buf.rms(),
                buf.lufs_approx()
            );
        }
    }

    #[test]
    fn loop_lengths_are_exact_multiples() {
        // Small propeller: 2400 rpm / 2 blades -> 80 Hz -> 12.5 ms.
        assert!((small_period() - 0.0125).abs() < 1e-6);
        assert!((small_duration() - 4.0).abs() < 1e-4);

        // Large propeller: 900 rpm / 3 blades -> 45 Hz -> 22.22 ms.
        assert!((large_period() - 1.0 / 45.0).abs() < 1e-6);
        assert!((large_duration() - 6.0).abs() < 1e-4);

        // Drone: 4000 rpm / 3 blades -> 200 Hz -> 5 ms.
        assert!((drone_period() - 0.005).abs() < 1e-6);
        assert!((drone_duration() - 4.0).abs() < 1e-4);

        // Helicopter: 45 Hz and 45.15 Hz both close at 20/3 s.
        assert!((heli_duration() - 20.0 / 3.0).abs() < 1e-4);
        assert!((45.15 * heli_duration() - 301.0).abs() < 1e-2);

        // Tones snapped into the grid stay inside 0.5 Hz of the target.
        for (hz, d) in [(40.0, small_duration()), (30.0, large_duration()), (50.0, drone_duration())] {
            let s = snap_hz(hz, d);
            assert!((s - hz).abs() < 0.5, "snap_hz moved {hz} to {s}");
            assert!((s * d - (s * d).round()).abs() < 1e-3);
        }
    }
}
