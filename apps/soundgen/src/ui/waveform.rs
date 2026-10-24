//! Waveform, spectrogram and thumbnail rendering.
//!
//! Everything expensive (min/max decimation, the FFT spectrogram) is computed
//! once per render and cached; drawing is pure painting.

use eframe::egui::{
    Color32, ColorImage, Painter, Pos2, Rect, Stroke, StrokeKind, TextureHandle, TextureOptions, Ui,
};
use sg_engine::AudioBuffer;

use super::theme::*;

// --------------------------------------------------------------- analysis ---

/// Peak-normalised min/max pairs plus an RMS envelope.
pub fn decimate(
    buf: &AudioBuffer,
    buckets: usize,
    max_seconds: Option<f32>,
) -> (Vec<(f32, f32)>, Vec<f32>) {
    let buckets = buckets.max(1);
    let frames = buf.frames();
    if frames == 0 {
        return (vec![(0.0, 0.0); buckets], vec![0.0; buckets]);
    }
    let limit = match max_seconds {
        Some(s) => ((s * buf.sample_rate as f32) as usize).clamp(1, frames),
        None => frames,
    };
    let peak = buf.peak().max(1e-6);
    let step = (limit as f32 / buckets as f32).ceil().max(1.0) as usize;
    let mut out = Vec::with_capacity(buckets);
    let mut rms = Vec::with_capacity(buckets);
    for b in 0..buckets {
        let start = b * step;
        if start >= limit {
            out.push((0.0, 0.0));
            rms.push(0.0);
            continue;
        }
        let end = (start + step).min(limit);
        let mut lo = 0.0f32;
        let mut hi = 0.0f32;
        let mut sum = 0.0f64;
        let mut n = 0usize;
        for f in start..end {
            let l = buf.samples.get(f * 2).copied().unwrap_or(0.0);
            let r = buf.samples.get(f * 2 + 1).copied().unwrap_or(0.0);
            let m = (l + r) * 0.5;
            lo = lo.min(m);
            hi = hi.max(m);
            sum += (m as f64) * (m as f64);
            n += 1;
        }
        out.push(((lo / peak).clamp(-1.0, 1.0), (hi / peak).clamp(-1.0, 1.0)));
        rms.push(if n > 0 {
            ((sum / n as f64).sqrt() as f32 / peak).clamp(0.0, 1.0)
        } else {
            0.0
        });
    }
    (out, rms)
}

/// Min/max pairs only, for thumbnails and variation cards.
pub fn peaks_mini(buf: &AudioBuffer, buckets: usize, max_seconds: Option<f32>) -> Vec<(f32, f32)> {
    decimate(buf, buckets, max_seconds).0
}

// ------------------------------------------------------------- wave cache ---

/// Cached drawing data for one rendered buffer.
#[derive(Default)]
pub struct WaveCache {
    pub peaks: Vec<(f32, f32)>,
    pub rms: Vec<f32>,
    pub duration: f32,
    pub frames: usize,
    pub peak: f32,
    tex: Option<TextureHandle>,
}

impl WaveCache {
    pub fn rebuild(&mut self, buf: &AudioBuffer, ctx: &egui::Context) {
        self.frames = buf.frames();
        self.duration = buf.duration();
        self.peak = buf.peak();
        let (peaks, rms) = decimate(buf, 1400, None);
        self.peaks = peaks;
        self.rms = rms;
        let img = spectrogram_image(buf, 180, 64);
        self.tex = Some(ctx.load_texture("sg_spectrogram", img, TextureOptions::LINEAR));
    }

    pub fn clear(&mut self) {
        self.peaks.clear();
        self.rms.clear();
        self.duration = 0.0;
        self.frames = 0;
        self.peak = 0.0;
        self.tex = None;
    }

    pub fn is_empty(&self) -> bool {
        self.frames == 0
    }
}

// ---------------------------------------------------------------- painting ---

/// Full-size waveform: RMS body, peak bars, centre line, playhead and hover.
pub fn paint_wave(ui: &Ui, rect: Rect, cache: &WaveCache, playhead: Option<f32>, hovered: bool) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 8.0, CANVAS);

    let mid = rect.center().y;
    if cache.peaks.is_empty() {
        p.line_segment(
            [Pos2::new(rect.left(), mid), Pos2::new(rect.right(), mid)],
            Stroke::new(1.0, BORDER),
        );
        return;
    }

    let half = rect.height() * 0.42;
    let n = cache.peaks.len();
    let bars = ((rect.width() / 2.0) as usize).max(1);
    let bar_w = rect.width() / bars as f32;
    for b in 0..bars {
        let t0 = b as f32 / bars as f32;
        let t1 = (b + 1) as f32 / bars as f32;
        let i0 = ((t0 * n as f32) as usize).min(n - 1);
        let i1 = (((t1 * n as f32) as usize).max(i0 + 1)).min(n);
        let mut lo = 0.0f32;
        let mut hi = 0.0f32;
        let mut rms = 0.0f32;
        for i in i0..i1 {
            let (l, h) = cache.peaks[i];
            lo = lo.min(l);
            hi = hi.max(h);
            rms = rms.max(cache.rms.get(i).copied().unwrap_or(0.0));
        }
        let x = rect.left() + (b as f32 + 0.5) * bar_w;
        if rms > 0.001 {
            p.line_segment(
                [
                    Pos2::new(x, mid - rms * half),
                    Pos2::new(x, mid + rms * half),
                ],
                Stroke::new(bar_w.max(1.0) + 0.5, alpha(TEXT, 26)),
            );
        }
        p.line_segment(
            [Pos2::new(x, mid - hi * half), Pos2::new(x, mid - lo * half)],
            Stroke::new(bar_w.max(1.0), alpha(ACCENT, 225)),
        );
    }

    p.line_segment(
        [Pos2::new(rect.left(), mid), Pos2::new(rect.right(), mid)],
        Stroke::new(1.0, alpha(BORDER, 180)),
    );

    if let Some(sec) = playhead {
        if cache.duration > 0.0 {
            let t = (sec / cache.duration).clamp(0.0, 1.0);
            let x = rect.left() + rect.width() * t;
            p.line_segment(
                [
                    Pos2::new(x, rect.top() + 4.0),
                    Pos2::new(x, rect.bottom() - 4.0),
                ],
                Stroke::new(5.0, alpha(Color32::WHITE, 22)),
            );
            p.line_segment(
                [
                    Pos2::new(x, rect.top() + 4.0),
                    Pos2::new(x, rect.bottom() - 4.0),
                ],
                Stroke::new(1.5, Color32::WHITE),
            );
        }
    }

    if hovered {
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(pos) && cache.duration > 0.0 {
                let t = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
                p.line_segment(
                    [Pos2::new(pos.x, rect.top()), Pos2::new(pos.x, rect.bottom())],
                    Stroke::new(1.0, alpha(TEXT, 45)),
                );
                let label = format!("{:.3} s", t * cache.duration);
                let galley = p.layout_no_wrap(label, egui::FontId::monospace(9.5), TEXT);
                let size = galley.size() + egui::vec2(10.0, 4.0);
                let mut chip = Rect::from_min_size(
                    Pos2::new(pos.x + 8.0, rect.bottom() - 8.0 - size.y),
                    size,
                );
                if chip.right() > rect.right() - 4.0 {
                    chip = chip.translate(egui::vec2(
                        -(chip.width() + 16.0),
                        0.0,
                    ));
                }
                p.rect_filled(chip, 4.0, alpha(SURFACE_HI, 235));
                p.galley(chip.min + egui::vec2(5.0, 2.0), galley, TEXT);
            }
        }
    }

    p.rect_stroke(rect, 8.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
}

/// Spectrogram strip (log-frequency, 64 bands, 8-bit heat map).
pub fn paint_spectrogram(ui: &Ui, rect: Rect, cache: &WaveCache) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 6.0, CANVAS);
    if let Some(tex) = &cache.tex {
        p.image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    p.rect_stroke(rect, 6.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
}

/// Miniature waveform for layer rows, variation cards and preset previews.
pub fn paint_mini(p: &Painter, rect: Rect, peaks: &[(f32, f32)], color: Color32) {
    let mid = rect.center().y;
    if peaks.is_empty() {
        p.line_segment(
            [Pos2::new(rect.left(), mid), Pos2::new(rect.right(), mid)],
            Stroke::new(1.0, alpha(color, 60)),
        );
        return;
    }
    let half = rect.height() * 0.45;
    let n = peaks.len();
    let bars = ((rect.width() / 2.0) as usize).max(1);
    let bar_w = rect.width() / bars as f32;
    for b in 0..bars {
        let t0 = b as f32 / bars as f32;
        let t1 = (b + 1) as f32 / bars as f32;
        let i0 = ((t0 * n as f32) as usize).min(n - 1);
        let i1 = (((t1 * n as f32) as usize).max(i0 + 1)).min(n);
        let mut lo = 0.0f32;
        let mut hi = 0.0f32;
        for i in i0..i1 {
            lo = lo.min(peaks[i].0);
            hi = hi.max(peaks[i].1);
        }
        let x = rect.left() + (b as f32 + 0.5) * bar_w;
        p.line_segment(
            [Pos2::new(x, mid - hi * half), Pos2::new(x, mid - lo * half)],
            Stroke::new(bar_w.max(1.0), color),
        );
    }
}

// ------------------------------------------------------------ spectrogram ---

/// Cooley-Tukey radix-2 FFT, in place. `re.len()` must be a power of two.
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    if n <= 1 || !n.is_power_of_two() || im.len() != n {
        return;
    }
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2usize;
    while len <= n {
        let ang = -std::f32::consts::TAU / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        let half = len / 2;
        let mut start = 0usize;
        while start < n {
            let mut cr = 1.0f32;
            let mut ci = 0.0f32;
            for k in 0..half {
                let i0 = start + k;
                let i1 = i0 + half;
                let vr = re[i1] * cr - im[i1] * ci;
                let vi = re[i1] * ci + im[i1] * cr;
                let ur = re[i0];
                let ui = im[i0];
                re[i0] = ur + vr;
                im[i0] = ui + vi;
                re[i1] = ur - vr;
                im[i1] = ui - vi;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
            start += len;
        }
        len <<= 1;
    }
}

/// 8-bit heat ramp: near-black -> indigo -> magenta -> orange -> cream.
fn heat(t: f32) -> Color32 {
    const STOPS: [(f32, (u8, u8, u8)); 6] = [
        (0.00, (0x0e, 0x10, 0x18)),
        (0.18, (0x24, 0x18, 0x45)),
        (0.38, (0x5c, 0x1e, 0x6e)),
        (0.58, (0xb8, 0x33, 0x5c)),
        (0.78, (0xf0, 0x8c, 0x3c)),
        (1.00, (0xff, 0xf0, 0xc8)),
    ];
    let t = t.clamp(0.0, 1.0);
    let mut a = STOPS[0];
    let mut b = STOPS[STOPS.len() - 1];
    for w in STOPS.windows(2) {
        if t >= w[0].0 && t <= w[1].0 {
            a = w[0];
            b = w[1];
            break;
        }
    }
    let span = (b.0 - a.0).max(1e-6);
    let f = ((t - a.0) / span).clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * f).round() as u8;
    let (ar, ag, ab) = a.1;
    let (br, bg, bb) = b.1;
    Color32::from_rgb(lerp(ar, br), lerp(ag, bg), lerp(ab, bb))
}

fn spectrogram_image(buf: &AudioBuffer, cols: usize, bands: usize) -> ColorImage {
    let cols = cols.clamp(8, 512);
    let bands = bands.clamp(8, 128);
    let frames = buf.frames();
    if frames < 16 {
        return ColorImage::filled([cols, bands], CANVAS);
    }
    let n = if frames >= 512 {
        512
    } else if frames >= 256 {
        256
    } else {
        64
    };
    let sr = buf.sample_rate.max(1) as f32;
    let span = frames.saturating_sub(n).max(1);
    let hop = (span / cols).max(1);

    let mut re = vec![0.0f32; n];
    let mut im = vec![0.0f32; n];
    let mut mags = vec![0.0f32; cols * bands];
    let mut global_max = 1e-9f32;

    let lo_hz = 70.0f32;
    let hi_hz = 16_000.0f32;
    let ratio = hi_hz / lo_hz;

    for c in 0..cols {
        let start = (c * hop).min(frames.saturating_sub(1));
        for i in 0..n {
            let idx = start + i;
            let s = if idx < frames {
                let l = buf.samples.get(idx * 2).copied().unwrap_or(0.0);
                let r = buf.samples.get(idx * 2 + 1).copied().unwrap_or(0.0);
                (l + r) * 0.5
            } else {
                0.0
            };
            let w = 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (n as f32 - 1.0)).cos();
            re[i] = s * w;
            im[i] = 0.0;
        }
        fft(&mut re, &mut im);
        for b in 0..bands {
            let f0 = lo_hz * ratio.powf(b as f32 / bands as f32);
            let f1 = lo_hz * ratio.powf((b + 1) as f32 / bands as f32);
            let k0 = ((f0 * n as f32 / sr) as usize).clamp(1, n / 2 - 1);
            let k1 = ((f1 * n as f32 / sr).ceil() as usize).clamp(k0 + 1, n / 2);
            let mut m = 0.0f32;
            for k in k0..k1 {
                let v = (re[k] * re[k] + im[k] * im[k]).sqrt();
                m = m.max(v);
            }
            mags[c * bands + b] = m;
            global_max = global_max.max(m);
        }
    }

    let mut px = vec![CANVAS; cols * bands];
    for c in 0..cols {
        for b in 0..bands {
            let m = mags[c * bands + b];
            let db = 20.0 * (m / global_max).max(1e-6).log10();
            let t = ((db + 68.0) / 68.0).clamp(0.0, 1.0);
            let row = bands - 1 - b;
            px[row * cols + c] = heat(t);
        }
    }
    ColorImage::new([cols, bands], px)
}
