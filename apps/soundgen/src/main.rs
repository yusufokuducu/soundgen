//! SoundGen — procedural sound-effect generator for a zombie game.
//!
//! A single-window studio: preset browser on the left, the sound itself in the
//! middle, a layer inspector on the right, a variation family at the bottom and
//! one permanent transport on top. The DSP lives in `sg-engine`; this file owns
//! the app state, the actions and the keyboard map.

mod ui;

use eframe::egui;
use egui::Color32;
use sg_engine::{render, AudioBuffer, Recipe};
use sg_io::{write_wav, ExportOptions, Player};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ui::waveform::{peaks_mini, WaveCache};

fn main() -> eframe::Result<()> {
    // `--demo` opens the app on a curated preset with a populated variation
    // family: useful for screenshots and for showing the tool off.
    let demo = std::env::args().any(|a| a == "--demo");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1480.0, 940.0])
            .with_min_inner_size([1120.0, 720.0])
            .with_title("SoundGen - Procedural SFX"),
        ..Default::default()
    };
    eframe::run_native(
        "SoundGen",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, demo)))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Good,
    Warn,
    Error,
}

pub struct Variation {
    pub seed: u64,
    pub buffer: Arc<AudioBuffer>,
    pub thumb: Vec<(f32, f32)>,
    pub kept: bool,
    pub lufs: f32,
}

pub struct App {
    pub ctx: egui::Context,
    pub presets: Vec<Recipe>,
    pub selected: usize,
    pub search: String,
    pub buffer: Option<Arc<AudioBuffer>>,
    pub wave: WaveCache,
    pub variations: Vec<Variation>,
    pub seed: u64,
    pub player: Option<Player>,
    pub audio_err: Option<String>,
    pub status: String,
    pub status_kind: StatusKind,
    pub export_dir: PathBuf,
    pub volume: f32,
    pub loop_on: bool,
    pub selected_layer: usize,
    /// (preset index, layer index) pairs the randomizer must leave alone.
    pub locked: HashSet<(usize, usize)>,
    pub random_amount: f32,
    pub auto_audition: bool,
    pub undo: Vec<(usize, Recipe)>,
    last_undo: f64,
    pub dirty: bool,
    dt_acc: f32,
    pub now: f64,
    pub lufs: f32,
    pub peak_db: f32,
    pub meter_db: f32,
    pub render_ms: f32,
    pub layer_thumbs: Vec<Vec<(f32, f32)>>,
    pub now_playing_var: Option<usize>,
    pub scroll_to_selected: bool,
    audition_at: Option<f64>,
    thumbs_at: Option<f64>,
    pub last_render_at: f64,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, demo: bool) -> Self {
        ui::theme::install(&cc.egui_ctx);

        let presets = sg_presets::all();
        let (player, audio_err) = match Player::new() {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e.to_string())),
        };

        let mut app = Self {
            ctx: cc.egui_ctx.clone(),
            presets,
            selected: 0,
            search: String::new(),
            buffer: None,
            wave: WaveCache::default(),
            variations: Vec::new(),
            seed: 1,
            player,
            audio_err,
            status: "Ready".to_owned(),
            status_kind: StatusKind::Info,
            export_dir: PathBuf::from("out"),
            volume: 0.85,
            loop_on: false,
            selected_layer: 0,
            locked: HashSet::new(),
            random_amount: 0.35,
            auto_audition: true,
            undo: Vec::new(),
            last_undo: -10.0,
            dirty: false,
            dt_acc: 0.0,
            now: 0.0,
            lufs: -70.0,
            peak_db: -70.0,
            meter_db: -70.0,
            render_ms: 0.0,
            layer_thumbs: Vec::new(),
            now_playing_var: None,
            scroll_to_selected: false,
            audition_at: None,
            thumbs_at: None,
            last_render_at: -1.0,
        };
        if let Some(p) = &app.player {
            p.set_volume(app.volume);
        }
        app.loop_on = app.presets.first().map(|r| r.looped).unwrap_or(false);
        app.rerender();
        app.schedule_thumbs();
        if let Some(err) = app.audio_err.clone() {
            app.set_status(format!("Audio unavailable: {err}"), StatusKind::Warn);
        } else {
            app.set_status("Ready - press SPACE to audition", StatusKind::Info);
        }

        if demo {
            if let Some(i) = app.presets.iter().position(|r| r.id == "weapon_shotgun_fire") {
                app.selected = i;
                app.loop_on = false;
                app.auto_audition = false;
                app.selected_layer = app.presets[i].layers.len().min(2) - 1;
                app.rerender();
                app.schedule_thumbs();
                app.generate_variations(12);
                for v in app.variations.iter_mut().take(3) {
                    v.kept = true;
                }
                app.set_status("Demo - shotgun preset, 12 variations", StatusKind::Good);
            }
        }
        app
    }

    // ------------------------------------------------------------- queries ---

    pub fn current(&self) -> Option<&Recipe> {
        self.presets.get(self.selected)
    }

    pub fn current_style(&self) -> f32 {
        self.current().map(|r| r.style).unwrap_or(0.5)
    }

    pub fn is_playing(&self) -> bool {
        self.player.as_ref().map(|p| p.is_playing()).unwrap_or(false)
    }

    pub fn audio_ok(&self) -> bool {
        self.player.is_some()
    }

    pub fn is_locked(&self, preset: usize, layer: usize) -> bool {
        self.locked.contains(&(preset, layer))
    }

    pub fn layer_count(&self) -> usize {
        self.current().map(|r| r.layers.len()).unwrap_or(0)
    }

    pub fn playhead_seconds(&self) -> Option<f32> {
        if !self.is_playing() || self.now_playing_var.is_some() {
            return None;
        }
        self.player.as_ref().map(|p| p.position_seconds())
    }

    pub fn status_color(&self) -> Color32 {
        match self.status_kind {
            StatusKind::Info => ui::theme::MUTED,
            StatusKind::Good => ui::theme::ACCENT,
            StatusKind::Warn => ui::theme::AMBER,
            StatusKind::Error => ui::theme::RED,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, kind: StatusKind) {
        self.status = msg.into();
        self.status_kind = kind;
    }

    // ------------------------------------------------------------- actions ---

    /// Mark the sound dirty. Rendering is throttled to 25 Hz in `tick`.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.dt_acc = 0.0;
        self.audition_at = Some(self.now + 0.25);
        self.thumbs_at = Some(self.now + 0.40);
    }

    pub fn rerender(&mut self) {
        let Some(recipe) = self.presets.get(self.selected) else {
            self.buffer = None;
            self.wave.clear();
            self.layer_thumbs.clear();
            return;
        };
        let started = std::time::Instant::now();
        let buf = render(recipe, self.seed);
        self.lufs = buf.lufs_approx();
        self.peak_db = sg_dsp::gain_to_db(buf.peak().max(1e-9));
        self.wave.rebuild(&buf, &self.ctx);
        self.buffer = Some(Arc::new(buf));
        self.render_ms = started.elapsed().as_secs_f32() * 1000.0;
        self.last_render_at = self.now;
    }

    pub fn schedule_thumbs(&mut self) {
        self.thumbs_at = Some(self.now + 0.35);
    }

    /// Per-layer preview: render just that layer, keep the first 200 ms.
    fn rebuild_layer_thumbs(&mut self) {
        let Some(recipe) = self.presets.get(self.selected) else {
            self.layer_thumbs.clear();
            return;
        };
        let count = recipe.layers.len().min(8);
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let Some(layer) = recipe.layers.get(i) else {
                break;
            };
            let mut sub = recipe.clone();
            sub.layers = vec![layer.clone()];
            if let Some(l) = sub.layers.first_mut() {
                l.mute = false;
                l.solo = false;
            }
            sub.duration = 0.2;
            sub.fade_in = 0.0;
            sub.fade_out = 0.0;
            sub.looped = false;
            sub.master.reverb_wet = 0.0;
            let buf = render(&sub, self.seed);
            out.push(peaks_mini(&buf, 64, Some(0.2)));
        }
        self.layer_thumbs = out;
    }

    pub fn play_current(&mut self) {
        self.now_playing_var = None;
        let Some(player) = &self.player else {
            self.set_status("No audio device - cannot audition", StatusKind::Warn);
            return;
        };
        let Some(buf) = &self.buffer else {
            return;
        };
        player.play(buf.clone(), self.loop_on);
    }

    pub fn stop(&mut self) {
        if let Some(p) = &self.player {
            p.stop();
        }
        self.now_playing_var = None;
        self.meter_db = -70.0;
    }

    pub fn toggle_play(&mut self) {
        if self.is_playing() {
            self.stop();
        } else {
            self.play_current();
        }
    }

    pub fn play_variation(&mut self, idx: usize) {
        let Some(v) = self.variations.get(idx) else {
            return;
        };
        let buf = v.buffer.clone();
        let Some(player) = &self.player else {
            self.set_status("No audio device - cannot audition", StatusKind::Warn);
            return;
        };
        player.play(buf, false);
        self.now_playing_var = Some(idx);
    }

    pub fn toggle_keep(&mut self, idx: usize) {
        if let Some(v) = self.variations.get_mut(idx) {
            v.kept = !v.kept;
        }
        let Some(v) = self.variations.get(idx) else {
            return;
        };
        let kept = self.variations.iter().filter(|v| v.kept).count();
        let msg = if v.kept {
            format!("Kept variation {:02} - {kept} in the family", idx + 1)
        } else {
            format!("Released variation {:02} - {kept} kept", idx + 1)
        };
        let kind = if kept > 0 {
            StatusKind::Good
        } else {
            StatusKind::Info
        };
        self.set_status(msg, kind);
    }

    pub fn select_preset(&mut self, idx: usize) {
        if idx >= self.presets.len() || idx == self.selected {
            return;
        }
        self.selected = idx;
        self.selected_layer = 0;
        self.variations.clear();
        self.now_playing_var = None;
        self.loop_on = self.presets.get(idx).map(|r| r.looped).unwrap_or(false);
        self.rerender();
        self.schedule_thumbs();
        if self.auto_audition {
            self.play_current();
        }
        if let Some(r) = self.presets.get(idx) {
            self.set_status(format!("{} - {}", r.category.label(), r.name), StatusKind::Info);
        }
    }

    pub fn step_preset(&mut self, delta: i32) {
        let len = self.presets.len();
        if len == 0 {
            return;
        }
        let next = if delta < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(len - 1)
        };
        if next != self.selected {
            self.select_preset(next);
            self.scroll_to_selected = true;
        }
    }

    pub fn set_style(&mut self, style: f32) {
        let style = style.clamp(0.0, 1.0);
        let differs = self
            .presets
            .get(self.selected)
            .map(|r| (r.style - style).abs() > 1e-5)
            .unwrap_or(false);
        if !differs {
            return;
        }
        self.snapshot(false);
        if let Some(r) = self.presets.get_mut(self.selected) {
            r.style = style;
        }
        self.mark_dirty();
    }

    pub fn reseed(&mut self) {
        self.snapshot(true);
        self.seed = self.seed.wrapping_add(1);
        self.rerender();
        self.schedule_thumbs();
        self.play_current();
        self.set_status(format!("New seed {}", self.seed), StatusKind::Info);
    }

    pub fn randomize(&mut self) {
        if self.presets.is_empty() {
            return;
        }
        self.snapshot(true);
        self.seed = self.seed.wrapping_add(1);
        let seed = self.seed;
        let amount = self.random_amount.clamp(0.05, 1.0);
        let preset = self.selected;
        let mut kept_layers = 0usize;
        if let Some(r) = self.presets.get_mut(preset) {
            let backup = r.layers.clone();
            r.randomize(amount, seed);
            for (i, layer) in r.layers.iter_mut().enumerate() {
                if self.locked.contains(&(preset, i)) {
                    if let Some(b) = backup.get(i) {
                        *layer = b.clone();
                        kept_layers += 1;
                    }
                }
            }
        }
        self.rerender();
        self.schedule_thumbs();
        if self.auto_audition {
            self.play_current();
        }
        let msg = if kept_layers > 0 {
            format!("Randomized (amount {amount:.2}) - {kept_layers} locked layers untouched")
        } else {
            format!("Randomized (amount {amount:.2}) - seed {seed}")
        };
        self.set_status(msg, StatusKind::Good);
    }

    pub fn toggle_lock(&mut self) {
        let key = (self.selected, self.selected_layer);
        if self.locked.contains(&key) {
            self.locked.remove(&key);
        } else {
            self.locked.insert(key);
        }
        let locked = self.locked.contains(&key);
        self.set_status(
            format!(
                "Layer {} {}",
                self.selected_layer + 1,
                if locked { "locked" } else { "unlocked" }
            ),
            StatusKind::Info,
        );
    }

    pub fn toggle_mute(&mut self, layer: usize) {
        self.snapshot(false);
        if let Some(r) = self.presets.get_mut(self.selected) {
            if let Some(l) = r.layers.get_mut(layer) {
                l.mute = !l.mute;
                let muted = l.mute;
                self.set_status(
                    format!("Layer {} {}", layer + 1, if muted { "muted" } else { "unmuted" }),
                    StatusKind::Info,
                );
            }
        }
        self.mark_dirty();
    }

    pub fn toggle_solo(&mut self, layer: usize) {
        self.snapshot(false);
        if let Some(r) = self.presets.get_mut(self.selected) {
            if let Some(l) = r.layers.get_mut(layer) {
                l.solo = !l.solo;
                let solo = l.solo;
                self.set_status(
                    format!("Layer {} {}", layer + 1, if solo { "soloed" } else { "un-soloed" }),
                    StatusKind::Info,
                );
            }
        }
        self.mark_dirty();
    }

    pub fn generate_variations(&mut self, count: usize) {
        let Some(recipe) = self.presets.get(self.selected).cloned() else {
            self.set_status("No preset to vary", StatusKind::Warn);
            return;
        };
        let started = std::time::Instant::now();
        let base = self.seed.wrapping_add(1000);
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let seed = base.wrapping_add(i as u64 * 7919);
            let buf = render(&recipe, seed);
            out.push(Variation {
                seed,
                thumb: peaks_mini(&buf, 48, None),
                lufs: buf.lufs_approx(),
                buffer: Arc::new(buf),
                kept: false,
            });
        }
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        self.variations = out;
        self.now_playing_var = None;
        self.set_status(
            format!("Rendered {count} variations in {ms:.0} ms"),
            StatusKind::Good,
        );
    }

    // -------------------------------------------------------------- export ---

    pub fn export_current(&mut self) {
        let Some(recipe) = self.presets.get(self.selected) else {
            self.set_status("Nothing to export", StatusKind::Warn);
            return;
        };
        let path = self
            .export_dir
            .join(recipe.category.folder())
            .join(format!("{}.wav", recipe.id));
        let Some(buf) = self.buffer.clone() else {
            self.set_status("Nothing rendered yet", StatusKind::Warn);
            return;
        };
        match write_wav(&path, &buf, &ExportOptions::default()) {
            Ok(()) => self.set_status(format!("Exported {}", path.display()), StatusKind::Good),
            Err(e) => self.set_status(format!("Export failed: {e}"), StatusKind::Error),
        }
    }

    pub fn export_kept(&mut self) {
        let Some(recipe) = self.presets.get(self.selected) else {
            return;
        };
        let folder = recipe.category.folder().to_owned();
        let id = recipe.id.clone();
        let kept: Vec<Arc<AudioBuffer>> = self
            .variations
            .iter()
            .filter(|v| v.kept)
            .map(|v| v.buffer.clone())
            .collect();
        if kept.is_empty() {
            self.set_status(
                "No variations kept - click the star on the ones you want",
                StatusKind::Warn,
            );
            return;
        }
        let dir = self.export_dir.join(&folder);
        let mut written = 0usize;
        let mut last_err = None;
        for (i, buf) in kept.iter().enumerate() {
            let path = dir.join(format!("{id}_v{:02}.wav", i + 1));
            match write_wav(&path, buf, &ExportOptions::default()) {
                Ok(()) => written += 1,
                Err(e) => last_err = Some(e.to_string()),
            }
        }
        match last_err {
            None => self.set_status(
                format!("Exported {written} WAVs to {}", dir.display()),
                StatusKind::Good,
            ),
            Some(e) => self.set_status(format!("Export failed: {e}"), StatusKind::Error),
        }
    }

    // ---------------------------------------------------------------- undo ---

    pub fn snapshot(&mut self, force: bool) {
        if !force && self.now - self.last_undo < 0.45 {
            return;
        }
        let Some(recipe) = self.presets.get(self.selected) else {
            return;
        };
        let snap = recipe.clone();
        self.undo.push((self.selected, snap));
        if self.undo.len() > 50 {
            self.undo.remove(0);
        }
        self.last_undo = self.now;
    }

    pub fn undo(&mut self) {
        let Some((idx, snap)) = self.undo.pop() else {
            self.set_status("Nothing to undo", StatusKind::Info);
            return;
        };
        if idx < self.presets.len() {
            self.presets[idx] = snap;
            self.selected = idx;
            let layers = self.presets[idx].layers.len();
            self.selected_layer = self.selected_layer.min(layers.saturating_sub(1));
            self.rerender();
            self.schedule_thumbs();
            self.set_status("Undo", StatusKind::Info);
        }
    }

    // ------------------------------------------------------------- per-frame ---

    fn tick(&mut self) {
        self.now = self.ctx.input(|i| i.time);
        let dt = self.ctx.input(|i| i.stable_dt).clamp(0.0, 0.1);

        // Re-render at most every 40 ms so a knob drag stays smooth.
        if self.dirty {
            self.dt_acc += dt;
            if self.dt_acc >= 0.04 {
                self.dt_acc = 0.0;
                self.dirty = false;
                self.rerender();
            } else {
                self.ctx.request_repaint();
            }
        }

        if let Some(at) = self.thumbs_at {
            if self.now >= at {
                self.thumbs_at = None;
                self.rebuild_layer_thumbs();
            } else {
                self.ctx.request_repaint_after(Duration::from_millis(80));
            }
        }

        if let Some(at) = self.audition_at {
            if self.now >= at {
                self.audition_at = None;
                let looped = self.current().map(|r| r.looped).unwrap_or(false);
                if self.auto_audition && !looped && !self.is_playing() {
                    self.play_current();
                }
            } else {
                self.ctx.request_repaint_after(Duration::from_millis(60));
            }
        }

        if self.is_playing() {
            self.ctx.request_repaint();
            self.update_meter(dt);
        } else {
            self.meter_db = (self.meter_db - 260.0 * dt).max(-70.0);
            if self.now_playing_var.is_some() {
                self.now_playing_var = None;
            }
        }
    }

    fn update_meter(&mut self, dt: f32) {
        let Some(player) = &self.player else {
            return;
        };
        let pos = player.position_seconds();
        let source = match self.now_playing_var {
            Some(i) => self.variations.get(i).map(|v| &v.buffer),
            None => self.buffer.as_ref(),
        };
        let mut target = -70.0f32;
        if let Some(buf) = source {
            let sr = buf.sample_rate.max(1) as f32;
            let frames = buf.frames();
            let start = (((pos - 0.012).max(0.0)) * sr) as usize;
            let end = (((pos + 0.012).max(0.0) * sr) as usize).min(frames);
            let mut peak = 0.0f32;
            let mut f = start;
            while f < end {
                if let (Some(l), Some(r)) = (
                    buf.samples.get(f * 2).copied(),
                    buf.samples.get(f * 2 + 1).copied(),
                ) {
                    peak = peak.max(l.abs()).max(r.abs());
                }
                f += 1;
            }
            target = sg_dsp::gain_to_db(peak.max(1e-7));
        }
        if target > self.meter_db {
            self.meter_db = target;
        } else {
            self.meter_db = (self.meter_db - 90.0 * dt).max(target);
        }
    }

    fn handle_keys(&mut self) {
        if self.ctx.egui_wants_keyboard_input() {
            return;
        }
        let undo = self.ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z));
        if undo {
            self.undo();
            return;
        }
        let (space, r, l, s, m, e, up, down) = self.ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space),
                i.consume_key(egui::Modifiers::NONE, egui::Key::R),
                i.consume_key(egui::Modifiers::NONE, egui::Key::L),
                i.consume_key(egui::Modifiers::NONE, egui::Key::S),
                i.consume_key(egui::Modifiers::NONE, egui::Key::M),
                i.consume_key(egui::Modifiers::NONE, egui::Key::E),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
            )
        });
        if space {
            self.toggle_play();
        }
        if r {
            self.randomize();
        }
        if l {
            self.toggle_lock();
        }
        if s {
            let layer = self.selected_layer;
            self.toggle_solo(layer);
        }
        if m {
            let layer = self.selected_layer;
            self.toggle_mute(layer);
        }
        if e {
            self.export_current();
        }
        if up {
            self.step_preset(-1);
        }
        if down {
            self.step_preset(1);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tick();
        self.handle_keys();

        let panel_frame = |fill: Color32, margin: i8| {
            egui::Frame::new()
                .fill(fill)
                .inner_margin(egui::Margin::same(margin))
        };

        egui::Panel::top("transport")
            .frame(panel_frame(ui::theme::PANEL, 12))
            .show(root, |ui| ui::panels::transport(self, ui));

        let var_h = if self.variations.is_empty() { 140.0 } else { 176.0 };
        egui::Panel::bottom("variations")
            .exact_size(var_h)
            .frame(panel_frame(ui::theme::PANEL, 12))
            .show(root, |ui| ui::panels::variations(self, ui));

        egui::Panel::left("browser")
            .resizable(true)
            .default_size(272.0)
            .min_size(200.0)
            .max_size(420.0)
            .frame(panel_frame(ui::theme::PANEL, 12))
            .show(root, |ui| ui::panels::browser(self, ui));

        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(316.0)
            .min_size(240.0)
            .max_size(460.0)
            .frame(panel_frame(ui::theme::PANEL, 12))
            .show(root, |ui| ui::panels::inspector(self, ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui::theme::BG)
                    .inner_margin(egui::Margin::symmetric(18, 14)),
            )
            .show(root, |ui| ui::panels::editor(self, ui));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bottom strip is generated headlessly: 12 seeds must render, survive
    /// the thumbnail pass and write `out/<category>/<id>_vNN.wav` like the UI does.
    #[test]
    fn variation_family_renders_and_exports() {
        let presets = sg_presets::all();
        let recipe = presets.first().expect("factory library is empty");
        let target = recipe.render_length();

        let mut family = Vec::new();
        for i in 0..12u64 {
            let seed = 1000u64.wrapping_add(i.wrapping_mul(7919));
            let buf = render(recipe, seed);
            assert!(buf.frames() > 0, "seed {seed} rendered no frames");
            assert!(buf.peak() > 1e-3, "seed {seed} rendered silence");
            let thumb = peaks_mini(&buf, 48, None);
            assert!(!thumb.is_empty());
            assert!(buf.lufs_approx().is_finite());
            family.push(buf);
        }

        let dir = std::env::temp_dir()
            .join("soundgen_variation_test")
            .join(recipe.category.folder());
        std::fs::create_dir_all(&dir).expect("temp dir");
        for (i, buf) in family.iter().enumerate() {
            let path = dir.join(format!("{}_v{:02}.wav", recipe.id, i + 1));
            write_wav(&path, buf, &ExportOptions::default()).expect("write wav");
            assert!(path.exists());
        }

        let mean = family.iter().map(|b| b.duration()).sum::<f32>() / family.len() as f32;
        assert!(
            (mean - target).abs() < target * 0.25 + 0.05,
            "variations drifted from the preset length: {mean:.3}s vs {target:.3}s"
        );
    }
}
