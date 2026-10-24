//! soundgen-cli â€” batch render and QA for the SoundGen library.
//!
//! Usage:
//!   soundgen-cli list
//!   soundgen-cli render  --out out --variations 4 --style 0.35 --bits 16
//!   soundgen-cli render  --only weapon --out out
//!   soundgen-cli stats

use anyhow::Result;
use sg_engine::{render, render_variations, AudioBuffer};
use sg_io::{write_wav, ExportOptions};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("list");

    match cmd {
        "list" => {
            for r in sg_presets::all() {
                println!("{:28} {:10} {:>5.2}s  {}", r.id, r.category.folder(), r.duration, r.name);
            }
        }
        "stats" => stats(),
        "bands" => bands(),
        "render" => {
            let out = arg_value(&args, "--out").unwrap_or_else(|| "out".into());
            let variations: usize = arg_value(&args, "--variations")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let style: f32 = arg_value(&args, "--style")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.35);
            let bits: u16 = arg_value(&args, "--bits")
                .and_then(|v| v.parse().ok())
                .unwrap_or(16);
            let only = arg_value(&args, "--only");
            let seed: u64 = arg_value(&args, "--seed")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            render_all(&PathBuf::from(out), variations, style, bits, only.as_deref(), seed)?;
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: soundgen-cli [list|stats|render] [--out DIR] [--variations N] [--style 0..1] [--bits 16|24|32] [--only PREFIX] [--seed N]");
        }
    }
    Ok(())
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn render_all(
    out: &PathBuf,
    variations: usize,
    style: f32,
    bits: u16,
    only: Option<&str>,
    base_seed: u64,
) -> Result<()> {
    let presets = sg_presets::all();
    let mut written = 0usize;
    let mut worst_peak = 0.0f32;
    let start = std::time::Instant::now();

    for recipe in &presets {
        if let Some(prefix) = only {
            if !recipe.id.starts_with(prefix) && recipe.category.folder() != prefix {
                continue;
            }
        }
        let mut r = recipe.clone();
        r.style = style;
        let bufs: Vec<AudioBuffer> = if variations <= 1 {
            vec![render(&r, base_seed)]
        } else {
            render_variations(&r, variations, base_seed)
        };
        for (i, buf) in bufs.iter().enumerate() {
            worst_peak = worst_peak.max(buf.peak());
            let name = if variations <= 1 {
                format!("{}.wav", r.id)
            } else {
                format!("{}_v{:02}.wav", r.id, i + 1)
            };
            let path = out.join(r.category.folder()).join(name);
            let mut opts = ExportOptions::default();
            opts.bits = bits;
            write_wav(&path, buf, &opts)?;
            written += 1;
        }
    }
    let elapsed = start.elapsed();
    println!(
        "wrote {written} files to {} in {:.1} ms ({:.1} ms/file), worst peak {:.2} dBFS",
        out.display(),
        elapsed.as_secs_f64() * 1000.0,
        elapsed.as_secs_f64() * 1000.0 / written.max(1) as f64,
        sg_dsp::gain_to_db(worst_peak)
    );
    Ok(())
}

fn stats() {
    let presets = sg_presets::all();
    println!(
        "{:30} {:>6} {:>7} {:>7} {:>6} {:>7}",
        "id", "dur", "peak", "lufs", "frames", "seam"
    );
    let mut total_ms = 0.0f64;
    let mut seen = std::collections::HashSet::new();
    let mut duplicates = Vec::new();
    for r in &presets {
        if !seen.insert(r.id.clone()) {
            duplicates.push(r.id.clone());
        }
        let t0 = std::time::Instant::now();
        let buf = render(r, 7);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        total_ms += ms;
        let finite = buf.samples.iter().all(|s| s.is_finite());
        let seam = if r.looped && buf.frames() > 1 {
            let n = buf.frames() - 1;
            let dl = (buf.samples[0] - buf.samples[n * 2]).abs();
            let dr = (buf.samples[1] - buf.samples[n * 2 + 1]).abs();
            // Compare the boundary step to the largest step inside the file:
            // a ratio <= 1 means the join is indistinguishable from the signal.
            let mut max_step = 1e-9f32;
            for i in 1..buf.frames() {
                let l = (buf.samples[i * 2] - buf.samples[(i - 1) * 2]).abs();
                let rr = (buf.samples[i * 2 + 1] - buf.samples[(i - 1) * 2 + 1]).abs();
                max_step = max_step.max(l).max(rr);
            }
            let ratio = dl.max(dr) / max_step;
            if ratio > 1.5 {
                format!("{ratio:.2}!")
            } else {
                format!("{ratio:.2}")
            }
        } else {
            "-".to_string()
        };
        println!(
            "{:30} {:6.2} {:7.2} {:7.1} {:6} {:>7}{}",
            r.id,
            buf.duration(),
            sg_dsp::gain_to_db(buf.peak()),
            buf.lufs_approx(),
            buf.frames(),
            seam,
            if finite { "" } else { "  <-- NaN!" }
        );
    }
    if !duplicates.is_empty() {
        println!("\nDUPLICATE IDS: {duplicates:?}");
    }
    println!(
        "\n{} presets, total render {:.0} ms, avg {:.1} ms",
        presets.len(),
        total_ms,
        total_ms / presets.len().max(1) as f64
    );
}


/// Objective QA: attack time, decay time, crest factor and 4-band energy split.
/// Useful to catch "all fizz" or "all boom" presets without listening.
fn bands() {
    let presets = sg_presets::all();
    println!(
        "{:30} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>5}",
        "id", "atk_ms", "dec_ms", "crest", "low%", "mid%", "hi%", "air%"
    );
    for r in &presets {
        let buf = render(r, 7);
        let sr = buf.sample_rate as f32;
        let frames = buf.frames();
        if frames == 0 {
            continue;
        }
        // Envelope (peak follower, 5 ms release).
        let rel = (-1.0 / (0.005 * sr)).exp();
        let mut env = 0.0f32;
        let mut peak = 0.0f32;
        let mut peak_at = 0usize;
        let mut envs = Vec::with_capacity(frames);
        for i in 0..frames {
            let x = buf.samples[i * 2].abs().max(buf.samples[i * 2 + 1].abs());
            env = if x > env { x } else { x + (env - x) * rel };
            if env > peak {
                peak = env;
                peak_at = i;
            }
            envs.push(env);
        }
        let atk_ms = peak_at as f32 / sr * 1000.0;
        let thresh = peak * 0.1;
        let mut last = peak_at;
        for i in peak_at..frames {
            if envs[i] > thresh {
                last = i;
            }
        }
        let dec_ms = (last - peak_at) as f32 / sr * 1000.0;
        let rms = buf.rms().max(1e-9);
        let crest = sg_dsp::gain_to_db(peak / rms);

        // 4-band split with simple one-poles (LP 200, LP 2k, LP 8k).
        let c1 = (-std::f32::consts::TAU * 200.0 / sr).exp();
        let c2 = (-std::f32::consts::TAU * 2000.0 / sr).exp();
        let c3 = (-std::f32::consts::TAU * 8000.0 / sr).exp();
        let (mut l1, mut l2, mut l3) = (0.0f32, 0.0f32, 0.0f32);
        let (mut e_low, mut e_mid, mut e_hi, mut e_air) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for i in 0..frames {
            let x = (buf.samples[i * 2] + buf.samples[i * 2 + 1]) * 0.5;
            l1 = x + (l1 - x) * c1;
            l2 = x + (l2 - x) * c2;
            l3 = x + (l3 - x) * c3;
            let low = l1;
            let mid = l2 - l1;
            let hi = l3 - l2;
            let air = x - l3;
            e_low += (low * low) as f64;
            e_mid += (mid * mid) as f64;
            e_hi += (hi * hi) as f64;
            e_air += (air * air) as f64;
        }
        let total = (e_low + e_mid + e_hi + e_air).max(1e-12);
        println!(
            "{:30} {:6.1} {:6.1} {:6.1} {:5.0} {:5.0} {:5.0} {:5.0}",
            r.id,
            atk_ms,
            dec_ms,
            crest,
            e_low / total * 100.0,
            e_mid / total * 100.0,
            e_hi / total * 100.0,
            e_air / total * 100.0
        );
    }
}
