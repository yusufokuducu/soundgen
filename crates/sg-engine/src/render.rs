//! Offline renderer: recipe -> stereo buffer. Fast enough (~ms) to re-render
//! on every knob move and still feel instant.

use crate::buffer::AudioBuffer;
use crate::layer::{Layer, LayerKind};
use crate::recipe::Recipe;
use sg_dsp::env::EnvGen;
use sg_dsp::excite::{ImpactExcitation, ImpactParams};
use sg_dsp::filter::{Biquad, Svf, SvfMode};
use sg_dsp::fx::{Compressor, DcBlocker, Limiter, Reverb, Saturator};
use sg_dsp::modal::ModalBank;
use sg_dsp::noise::{NoiseColor, NoiseGen};
use sg_dsp::osc::{Osc, PitchEnv, WaveShape};
use sg_dsp::rng::Rng;
use sg_dsp::{db_to_gain, pan_gains, soft_clip};

const TAU: f32 = std::f32::consts::TAU;

pub fn render(recipe: &Recipe, seed: u64) -> AudioBuffer {
    render_with_style(recipe, seed, recipe.style)
}

pub fn render_with_style(recipe: &Recipe, seed: u64, style: f32) -> AudioBuffer {
    let sr = recipe.sample_rate as f32;
    let loop_frames_base = (recipe.render_length() * sr).ceil() as usize;
    let xfade_secs = if recipe.looped {
        recipe.loop_crossfade.clamp(0.0, 0.5)
    } else {
        0.0
    };
    let xfade_frames = (xfade_secs * sr) as usize;
    let frames = loop_frames_base + xfade_frames;
    let mut left = vec![0.0f32; frames];
    let mut right = vec![0.0f32; frames];
    let any_solo = recipe.layers.iter().any(|l| l.solo);

    for (idx, layer) in recipe.layers.iter().enumerate() {
        if layer.mute || (any_solo && !layer.solo) {
            continue;
        }
        let mut rng = Rng::new(seed.wrapping_mul(0x9E37_79B9) ^ (idx as u64 + 1).wrapping_mul(0xD1B5_4A32));
        let mut mono = vec![0.0f32; frames];
        render_layer(layer, &mut rng, sr, style, recipe.velocity, &mut mono);

        let gain = db_to_gain(layer.gain_db) * layer.style_multiplier(style);
        let (lg, rg) = pan_gains(layer.pan);
        let offset = (layer.delay * sr) as usize;
        for i in 0..frames.saturating_sub(offset) {
            let s = mono[i] * gain;
            left[i + offset] += s * lg;
            right[i + offset] += s * rg;
        }
    }

    // Fade-in must happen BEFORE normalisation, otherwise a 1 ms ramp eats the
    // first milliseconds of a transient (a gunshot crack lives there).
    if recipe.fade_in > 0.0 {
        let n = ((recipe.fade_in * sr) as usize).min(left.len());
        for i in 0..n {
            let g = i as f32 / n as f32;
            left[i] *= g;
            right[i] *= g;
        }
    }

    master_chain(recipe, sr, &mut left, &mut right);

    // Circular crossfade AFTER the stateful chain: the limiter gain at the head
    // differs from the tail, so folding earlier would put a step at the seam.
    let mut out_frames = frames;
    if xfade_frames > 0 && loop_frames_base > xfade_frames {
        for i in 0..xfade_frames {
            let w = i as f32 / xfade_frames as f32;
            let tail = loop_frames_base + i;
            left[i] = left[i] * w + left[tail] * (1.0 - w);
            right[i] = right[i] * w + right[tail] * (1.0 - w);
        }
        out_frames = loop_frames_base;
        left.truncate(out_frames);
        right.truncate(out_frames);
    }

    let mut out = AudioBuffer::new(recipe.sample_rate, out_frames);
    for i in 0..out_frames {
        out.samples[i * 2] = left[i];
        out.samples[i * 2 + 1] = right[i];
    }
    if recipe.fade_out > 0.0 {
        out.fade_out(recipe.fade_out * 1000.0);
    }
    out.normalize_peak(recipe.master.target_peak_db);
    out
}

pub fn render_variations(recipe: &Recipe, count: usize, base_seed: u64) -> Vec<AudioBuffer> {
    (0..count)
        .map(|i| render(recipe, base_seed + i as u64 * 7919))
        .collect()
}

fn render_layer(layer: &Layer, rng: &mut Rng, sr: f32, style: f32, velocity: f32, out: &mut [f32]) {
    match &layer.kind {
        LayerKind::Noise {
            color,
            env,
            filter,
            mode,
            tame_hz,
        } => {
            let mut nz = NoiseGen::new(*color);
            let mut e = EnvGen::new(sr, *env);
            let mut svf = Svf::new(sr);
            let mut tame = Svf::new(sr);
            tame.set(*tame_hz, 0.707);
            let mode = match mode {
                0 => SvfMode::LowPass,
                1 => SvfMode::BandPass,
                _ => SvfMode::HighPass,
            };
            let mut i = 0usize;
            while i < out.len() {
                let t = e.time();
                if i % 16 == 0 {
                    svf.set(filter.at(t), filter.q);
                }
                let amp = e.next();
                if e.finished() && amp <= 0.0 {
                    break;
                }
                let n = nz.next(rng);
                let mut s = svf.process(n, mode);
                if *tame_hz > 0.0 && *tame_hz < sr * 0.45 {
                    s = tame.process(s, SvfMode::LowPass);
                }
                out[i] += s * amp;
                i += 1;
            }
        }
        LayerKind::Tone {
            shape,
            start_hz,
            end_hz,
            pitch_tau,
            env,
            fm_hz,
            fm_depth,
            drive,
            vibrato,
        } => {
            let mut osc = Osc::new(sr);
            let mut fm = Osc::new(sr);
            // Cartoon mode digs a deeper pitch hole (research: x2.2-3.5 on the
            // pitch envelope depth) and pushes the drive harder.
            let end = end_hz + (start_hz - end_hz) * style * 0.45;
            let drive_eff = (drive + style * 0.4).clamp(0.0, 1.0);
            let mut penv = PitchEnv::new(sr, *start_hz, end, *pitch_tau);
            let mut e = EnvGen::new(sr, *env);
            let mut sat = Saturator {
                drive: 1.0 + drive_eff * 2.0,
                asym: 0.15 * drive_eff,
                mix: drive_eff,
            };
            let mut i = 0usize;
            while i < out.len() {
                let t = e.time();
                let amp = e.next();
                if e.finished() && amp <= 0.0 {
                    break;
                }
                let mut f = penv.next();
                if *vibrato > 0.0 {
                    f *= 1.0 + vibrato * 0.06 * (TAU * 5.5 * t).sin();
                }
                if *fm_depth > 0.0 {
                    let m = fm.next(f * fm_hz.max(0.1), WaveShape::Sine);
                    f *= 1.0 + m * fm_depth * 0.6;
                }
                let raw = osc.next(f.max(10.0), *shape);
                let s = if drive_eff > 0.0 { sat.process(raw) } else { raw };
                out[i] += s * amp;
                i += 1;
            }
        }
        LayerKind::Modal {
            modes,
            exciter_ms,
            exciter_color,
            exciter_hp,
            exciter_lp,
            detune_cents,
        } => {
            let ratio = 2f32.powf((detune_cents + style * 12.0) / 1200.0);
            // Cartoon: resonators ring longer, but stay in tune (detuning is
            // what makes them read as broken, not as stylised).
            let decay_scale = 1.0 + style * 0.45;
            let shifted: Vec<_> = modes
                .iter()
                .map(|m| sg_dsp::modal::Mode {
                    freq: m.freq * ratio,
                    amp: m.amp,
                    decay: m.decay * decay_scale,
                })
                .collect();
            let mut bank = ModalBank::new(sr, &shifted);

            // `exciter_ms` is the macro contact time: how long the two bodies
            // stay touching. That single number decides where the pulse rolls
            // off (f_-3dB = 0.634 / Tc), so it is clamped to what a real
            // surface can physically do. Zero means "unspecified", which for
            // this library is always a metal-on-metal mechanism: 0.15 ms.
            let contact = if *exciter_ms <= 0.0 {
                0.000_15
            } else {
                (*exciter_ms * 0.001).clamp(0.000_05, 0.006)
            };
            let mut exc =
                ImpactExcitation::new(sr, ImpactParams::new(contact, 0.35 + 0.45 * style));
            exc.prepare(velocity, rng);

            let mut nz = NoiseGen::new(*exciter_color);
            let mut hp = Svf::new(sr);
            hp.set(*exciter_hp, 0.707);
            let mut lp = Svf::new(sr);
            lp.set(*exciter_lp, 0.707);

            for sample in out.iter_mut() {
                let (force, asperity) = exc.next(rng);
                // A little filtered air riding on the pulse keeps the body
                // from sounding like a pure sine stack, but it is a garnish:
                // the Hertz force is what excites the modes.
                let air = lp.process(hp.process(nz.next(rng), SvfMode::HighPass), SvfMode::LowPass)
                    * force
                    * 0.35;
                *sample += bank.process(force + asperity + air);
            }
        }
        LayerKind::Grain {
            source_hz,
            grain_ms,
            density,
            spray,
            color,
            env,
            pitch_jitter,
        } => {
            let mut e = EnvGen::new(sr, *env);
            // Cartoon grains are longer and softer.
            let gm = grain_ms * (1.0 + style * 0.5);
            let grain_n = ((gm * 0.001 * sr) as usize).max(2);
            // Research: normalise by 1/sqrt(overlap) so density changes keep level.
            let norm = 1.0 / (density * gm * 0.001).max(1.0).sqrt();
            let mut next_grain = 0.0f32;
            let mut grains: Vec<Grain> = Vec::new();
            let mut nz = NoiseGen::new(*color);
            let mut i = 0usize;
            while i < out.len() {
                let amp = e.next();
                if e.finished() && amp <= 0.0 && grains.is_empty() {
                    break;
                }
                if i as f32 >= next_grain {
                    let jitter = 1.0 + rng.normal() * spray * 0.6;
                    let len = (grain_n as f32 * jitter).max(2.0) as usize;
                    let pitch = source_hz * 2f32.powf(rng.normal() * pitch_jitter);
                    let mut f = Svf::new(sr);
                    f.set(pitch.max(30.0), 6.0);
                    grains.push(Grain {
                        pos: 0,
                        len,
                        filter: f,
                        nz: NoiseGen::new(*color),
                        amp: rng.range(0.2, 1.0).powf(1.5),
                    });
                    // Poisson process: exponential inter-arrival keeps the cloud
                    // irregular instead of metronomic (research: constant density
                    // reads as a "shaker", not as paper/rasp).
                    let mean_interval = sr / density.max(0.5);
                    let u = rng.next_f32().max(1e-6);
                    next_grain += -u.ln() * mean_interval;
                }
                let mut s = 0.0f32;
                for g in grains.iter_mut() {
                    if g.pos < g.len {
                        let w = 0.5 - 0.5 * (TAU * g.pos as f32 / g.len as f32).cos();
                        s += g.filter.process(g.nz.next(rng), SvfMode::BandPass) * w * g.amp;
                        g.pos += 1;
                    }
                }
                grains.retain(|g| g.pos < g.len);
                s += nz.next(rng) * 0.01;
                out[i] += s * amp * norm;
                i += 1;
            }
        }
        LayerKind::Rotor {
            blades,
            rpm,
            tone_hz,
            air,
            flap_depth,
            env,
            color,
        } => {
            let mut e = EnvGen::new(sr, *env);
            let blade_hz = (rpm / 60.0 * blades).clamp(5.0, sr * 0.4);
            let mut nz = NoiseGen::new(*color);
            let mut air_nz = NoiseGen::new(NoiseColor::Blue);
            let mut air_lp = Svf::new(sr);
            air_lp.set(2500.0, 0.707);
            let mut motor = Osc::new(sr);
            let mut motor2 = Osc::new(sr);
            let mut i = 0usize;
            while i < out.len() {
                let t = i as f32 / sr;
                let amp = e.next();
                if e.finished() && amp <= 0.0 {
                    break;
                }
                let ph = (t * blade_hz).fract();
                let pulse = (0.5 + 0.5 * (TAU * ph).cos()).powi(4);
                let am = 1.0 - flap_depth + flap_depth * pulse;
                let body = nz.next(rng) * am * 0.5;
                let hiss = air_lp.process(air_nz.next(rng), SvfMode::LowPass) * air;
                let hum = motor.next(tone_hz.max(20.0), WaveShape::Saw) * 0.18
                    + motor2.next(tone_hz.max(20.0) * 2.0, WaveShape::Sine) * 0.08;
                out[i] += (body + hiss + hum) * amp;
                i += 1;
            }
        }
        LayerKind::Sweep {
            start_hz,
            end_hz,
            tau,
            q,
            color,
            env,
            tone_mix,
        } => {
            let mut nz = NoiseGen::new(*color);
            let mut e = EnvGen::new(sr, *env);
            let mut svf = Svf::new(sr);
            let mut osc = Osc::new(sr);
            let ratio = (end_hz / start_hz).max(1e-4);
            let mut i = 0usize;
            while i < out.len() {
                let t = e.time();
                let amp = e.next();
                if e.finished() && amp <= 0.0 {
                    break;
                }
                // Exponential glide over the layer's lifetime.
                let x = (t / tau.max(1e-4)).min(1.0);
                let fc = start_hz * ratio.powf(x);
                if i % 16 == 0 {
                    svf.set(fc, *q);
                }
                let n = svf.process(nz.next(rng), SvfMode::BandPass);
                let tone = osc.next(fc, WaveShape::Sine) * 0.35;
                out[i] += (n + tone * tone_mix) * amp;
                i += 1;
            }
        }
    }
    // Style-dependent grit: more cartoon => a touch of bitcrush-y edge.
    if style > 0.6 {
        let steps = 512.0 - (style - 0.6) / 0.4 * 384.0;
        for s in out.iter_mut() {
            *s = (*s * steps).round() / steps;
        }
    }
    let _ = soft_clip;
}

struct Grain {
    pos: usize,
    len: usize,
    filter: Svf,
    nz: NoiseGen,
    amp: f32,
}

fn master_chain(recipe: &Recipe, sr: f32, left: &mut [f32], right: &mut [f32]) {
    let m = recipe.master;
    let mut dc_l = DcBlocker::default();
    let mut dc_r = DcBlocker::default();
    let mut hp_l = Svf::new(sr);
    let mut hp_r = Svf::new(sr);
    hp_l.set(m.highpass_hz, 0.707);
    hp_r.set(m.highpass_hz, 0.707);
    let mut shelf_l = Biquad::new(sr);
    let mut shelf_r = Biquad::new(sr);
    shelf_l.set_low_shelf(m.low_shelf_hz, m.low_shelf_db, 0.8);
    shelf_r.set_low_shelf(m.low_shelf_hz, m.low_shelf_db, 0.8);
    let mut pres_l = Biquad::new(sr);
    let mut pres_r = Biquad::new(sr);
    pres_l.set_peak(m.presence_hz, 1.1, m.presence_db);
    pres_r.set_peak(m.presence_hz, 1.1, m.presence_db);

    for i in 0..left.len() {
        let l = dc_l.process(left[i]);
        let r = dc_r.process(right[i]);
        let l = hp_l.process(l, SvfMode::HighPass);
        let r = hp_r.process(r, SvfMode::HighPass);
        let l = shelf_l.process(l);
        let r = shelf_r.process(r);
        left[i] = pres_l.process(l);
        right[i] = pres_r.process(r);
    }

    if m.drive_db.abs() > 0.01 {
        let mut sat = Saturator {
            drive: db_to_gain(m.drive_db),
            asym: 0.2,
            mix: 1.0,
        };
        for i in 0..left.len() {
            left[i] = sat.process(left[i]);
            right[i] = sat.process(right[i]);
        }
    }

    if m.comp_ratio > 1.01 {
        let mut comp = Compressor::new(sr);
        comp.configure(m.comp_threshold_db, m.comp_ratio, 8.0, 140.0);
        comp.set_makeup_db(2.0);
        for i in 0..left.len() {
            let (l, r) = comp.process_stereo(left[i], right[i]);
            left[i] = l;
            right[i] = r;
        }
    }

    if m.reverb_wet > 0.001 {
        let mut rv = Reverb::new(sr);
        rv.configure(m.reverb_size, m.reverb_damp);
        let wet = m.reverb_wet;
        for i in 0..left.len() {
            let (wl, wr) = rv.process((left[i] + right[i]) * 0.5);
            left[i] += wl * wet;
            right[i] += wr * wet;
        }
    }

    if (m.stereo_width - 1.0).abs() > 0.01 {
        for i in 0..left.len() {
            let (l, r) = sg_dsp::fx::widen(left[i], right[i], m.stereo_width);
            left[i] = l;
            right[i] = r;
        }
    }

    let mut lim = Limiter::new(sr, m.ceiling_db);
    // The reverb and the wide-band layers can leave a DC offset behind the
    // first DC blocker; strip it again before limiting.
    let mut dc2_l = DcBlocker::default();
    let mut dc2_r = DcBlocker::default();
    for i in 0..left.len() {
        let l = dc2_l.process(left[i]);
        let r = dc2_r.process(right[i]);
        let (l, r) = lim.process_stereo(l, r);
        left[i] = l;
        right[i] = r;
    }
    // Peak normalisation is applied by the caller, after the loop crossfade,
    // so that stateful processing cannot put a step at the loop seam.
}


