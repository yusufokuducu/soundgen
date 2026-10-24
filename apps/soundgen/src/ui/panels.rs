//! The five surfaces of the studio.
//!
//! * `transport`  — permanent top bar: brand, breadcrumb, actions, playback, meters
//! * `browser`    — searchable preset library grouped by category
//! * `editor`     — waveform, spectrum, style morph and the layer stack
//! * `inspector`  — every parameter of the selected layer + master chain
//! * `variations` — a generated family of seeds, audition / keep / export

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Ui, Vec2,
};

use sg_dsp::env::AdEnv;
use sg_dsp::noise::NoiseColor;
use sg_dsp::osc::WaveShape;
use sg_engine::{Category, FilterEnv, Layer, LayerKind, Recipe};

use super::theme::*;
use super::waveform;
use super::widgets::{self, Param};
use crate::{App, StatusKind, Variation};

// ================================================================ transport ===

pub fn transport(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (bar, _) = ui.allocate_exact_size(Vec2::new(3.0, 20.0), Sense::hover());
        ui.painter().rect_filled(bar, 1.5, ACCENT);
        ui.label(RichText::new("SoundGen").size(15.0).strong().color(TEXT));
        ui.label(RichText::new("STUDIO").size(8.0).strong().color(DIM));
        ui.add_space(6.0);

        match app.current() {
            Some(r) => {
                let cat = r.category.label().to_uppercase();
                let name = r.name.clone();
                ui.label(RichText::new(cat).size(8.5).strong().color(DIM));
                ui.label(RichText::new("/").size(9.0).color(DIM));
                ui.label(RichText::new(name).size(11.0).color(MUTED));
            }
            None => {
                ui.label(RichText::new("NO PRESETS").size(9.5).strong().color(AMBER));
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::accent_button(ui, "EXPORT WAV").clicked() {
                app.export_current();
            }
            if widgets::ghost_button(ui, "RANDOMIZE").clicked() {
                app.randomize();
            }
            ui.label(
                RichText::new(format!("{:.2}", app.random_amount))
                    .monospace()
                    .size(9.5)
                    .color(ACCENT),
            );
            let mut amount = app.random_amount;
            if widgets::mini_slider(
                ui,
                "random_amount",
                &mut amount,
                0.1,
                1.0,
                64.0,
                ACCENT,
            )
            .changed()
            {
                app.random_amount = amount;
            }
            ui.label(RichText::new("AMOUNT").size(8.5).strong().color(DIM));
        });
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let playing = app.is_playing();
        if widgets::transport_button(ui, playing).clicked() {
            app.toggle_play();
        }
        if widgets::ghost_button(ui, "STOP").clicked() {
            app.stop();
        }
        let looped = app.loop_on;
        if ui
            .add(widgets::chip_button("LOOP", looped, ACCENT))
            .on_hover_text("Loop the current sound")
            .clicked()
        {
            app.loop_on = !looped;
        }
        if ui
            .add(widgets::chip_button("AUTO", app.auto_audition, ACCENT))
            .on_hover_text("Auto-audition 250 ms after the last edit")
            .clicked()
        {
            app.auto_audition = !app.auto_audition;
        }

        divider(ui);

        ui.label(RichText::new("VOL").size(8.5).strong().color(DIM));
        let mut vol = app.volume;
        if widgets::mini_slider(ui, "volume", &mut vol, 0.0, 1.2, 78.0, ACCENT).changed() {
            app.volume = vol;
            if let Some(p) = &app.player {
                p.set_volume(vol);
            }
        }
        ui.label(
            RichText::new(format!("{:>3.0}%", vol * 100.0))
                .monospace()
                .size(9.0)
                .color(MUTED),
        );

        divider(ui);

        widgets::meter(ui, app.meter_db, 84.0);
        ui.label(
            RichText::new(format!("{:>6.1} dBFS", app.meter_db))
                .monospace()
                .size(9.0)
                .color(level_color(app.meter_db)),
        );
        ui.label(
            RichText::new(format!("LUFS {:>6.1}", app.lufs))
                .monospace()
                .size(9.0)
                .color(MUTED),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.dirty {
                ui.label(RichText::new("RENDERING").size(8.5).strong().color(AMBER));
            }
            ui.label(RichText::new(app.status.as_str()).size(9.5).color(app.status_color()));
            divider(ui);
            match (&app.player, &app.audio_err) {
                (Some(p), _) => {
                    let name = p.device_name().to_owned();
                    let rate = p.output_rate();
                    ui.label(
                        RichText::new(format!("{} · {} Hz", short_device(&name), rate))
                            .size(8.5)
                            .color(DIM),
                    )
                    .on_hover_text(name);
                }
                (None, err) => {
                    ui.label(RichText::new("NO AUDIO DEVICE").size(8.5).strong().color(AMBER))
                        .on_hover_text(err.clone().unwrap_or_default());
                }
            }
        });
    });
}

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, 18.0), Sense::hover());
    let x = rect.center().x;
    ui.painter().line_segment(
        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
        Stroke::new(1.0, BORDER),
    );
}

fn short_device(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.chars().count() > 26 {
        let short: String = trimmed.chars().take(25).collect();
        format!("{short}…")
    } else {
        trimmed.to_owned()
    }
}

// ================================================================== browser ===

pub fn browser(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("LIBRARY").size(9.5).strong().color(DIM));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}", app.presets.len()))
                    .monospace()
                    .size(9.0)
                    .color(DIM),
            );
        });
    });
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let w = (ui.available_width() - 26.0).max(60.0);
        ui.add(
            TextEdit::singleline(&mut app.search)
                .hint_text("Search presets")
                .desired_width(w),
        );
        if ui.add(widgets::chip_button("X", false, DIM)).clicked() {
            app.search.clear();
        }
    });

    let query = app.search.trim().to_lowercase();
    let mut rows: Vec<(usize, Category)> = Vec::new();
    for (i, r) in app.presets.iter().enumerate() {
        if query.is_empty() || matches_query(r, &query) {
            rows.push((i, r.category));
        }
    }
    rows.sort_by_key(|(_, c)| {
        Category::all()
            .iter()
            .position(|x| x == c)
            .unwrap_or(usize::MAX)
    });

    ui.add_space(4.0);
    if rows.is_empty() {
        ui.add_space(18.0);
        ui.label(
            RichText::new(if query.is_empty() {
                "No presets loaded."
            } else {
                "Nothing matches that search."
            })
            .size(11.0)
            .color(MUTED),
        );
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut idx = 0usize;
            while idx < rows.len() {
                let cat = rows[idx].1;
                let mut end = idx;
                while end < rows.len() && rows[end].1 == cat {
                    end += 1;
                }
                category_header(ui, cat, end - idx);
                for &(i, _) in &rows[idx..end] {
                    preset_row(app, ui, i);
                }
                idx = end;
            }
            ui.add_space(8.0);
        });
}

fn category_header(ui: &mut Ui, cat: Category, count: usize) {
    ui.add_space(9.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(cat.label().to_uppercase())
                .size(8.5)
                .strong()
                .color(DIM),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{count}"))
                    .monospace()
                    .size(8.5)
                    .color(DIM),
            );
        });
    });
    ui.add_space(1.0);
}

fn preset_row(app: &mut App, ui: &mut Ui, i: usize) {
    let Some(r) = app.presets.get(i) else {
        return;
    };
    let name = r.name.clone();
    let dur = r.render_length();
    let looped = r.looped;
    let selected = app.selected == i;

    let width = ui.available_width().max(80.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 25.0), Sense::click());
    let p = ui.painter_at(rect);
    if selected {
        p.rect_filled(rect, 6.0, SURFACE_HI);
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left() + 3.0, rect.top() + 5.0),
                Vec2::new(3.0, rect.height() - 10.0),
            ),
            1.5,
            ACCENT,
        );
    } else if resp.hovered() {
        p.rect_filled(rect, 6.0, SURFACE);
    }
    let text_color = if selected {
        TEXT
    } else if resp.hovered() {
        TEXT
    } else {
        MUTED
    };
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(12.0),
        text_color,
    );
    p.text(
        Pos2::new(rect.right() - 8.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{dur:.2}s{}", if looped { " · loop" } else { "" }),
        FontId::monospace(8.5),
        DIM,
    );

    if resp.clicked() {
        if selected {
            app.play_current();
        } else {
            app.select_preset(i);
        }
    }
    if selected && app.scroll_to_selected {
        resp.scroll_to_me(Some(Align::Center));
        app.scroll_to_selected = false;
    }
}

fn matches_query(r: &Recipe, q: &str) -> bool {
    r.name.to_lowercase().contains(q)
        || r.id.to_lowercase().contains(q)
        || r.category.label().to_lowercase().contains(q)
        || r.tags.iter().any(|t| t.to_lowercase().contains(q))
}

// =================================================================== editor ===

pub fn editor(app: &mut App, ui: &mut Ui) {
    let Some(recipe) = app.current() else {
        empty_editor(ui);
        return;
    };
    let name = recipe.name.clone();
    let desc = recipe.description.clone();
    let cat = recipe.category;
    let tags = recipe.tags.clone();
    let duration = recipe.render_length();
    let looped = recipe.looped;
    let sample_rate = recipe.sample_rate;
    let layer_count = recipe.layers.len();
    let seed = app.seed;

    // ---- header -----------------------------------------------------------
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).size(21.0).strong().color(TEXT));
        ui.add_space(4.0);
        pill(ui, cat.label(), ACCENT);
        for tag in tags.iter().take(3) {
            pill(ui, tag, MUTED);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, "NEW SEED").clicked() {
                app.reseed();
            }
            ui.label(
                RichText::new(format!("{:06}", seed % 1_000_000))
                    .monospace()
                    .size(13.0)
                    .color(ACCENT),
            );
            ui.label(RichText::new("SEED").size(7.5).strong().color(DIM));
        });
    });
    if !desc.is_empty() {
        ui.add_space(2.0);
        let short: String = desc.chars().take(118).collect();
        ui.horizontal(|ui| {
            ui.label(RichText::new(short).size(11.0).color(MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{duration:.2} s · {sample_rate} Hz{}",
                        if looped { " · LOOP" } else { "" }
                    ))
                    .monospace()
                    .size(8.5)
                    .color(DIM),
                );
            });
        });
    }

    // ---- waveform ---------------------------------------------------------
    ui.add_space(10.0);
    let full_w = ui.available_width();
    let avail_h = ui.available_height();
    let wave_h = (avail_h * 0.27).clamp(110.0, 200.0);
    let (wrect, wresp) = ui.allocate_exact_size(Vec2::new(full_w, wave_h), Sense::hover());
    let playhead = app.playhead_seconds();
    waveform::paint_wave(ui, wrect, &app.wave, playhead, wresp.hovered());
    {
        let p = ui.painter_at(wrect);
        if app.wave.is_empty() {
            p.text(
                wrect.center(),
                Align2::CENTER_CENTER,
                "Nothing rendered",
                FontId::proportional(11.0),
                DIM,
            );
        } else {
            p.text(
                Pos2::new(wrect.left() + 12.0, wrect.top() + 9.0),
                Align2::LEFT_TOP,
                format!("{:.3} s", app.wave.duration),
                FontId::monospace(9.5),
                MUTED,
            );
            p.text(
                Pos2::new(wrect.right() - 12.0, wrect.top() + 9.0),
                Align2::RIGHT_TOP,
                format!(
                    "PEAK {:+.1} dBFS   LUFS {:+.1}   {} Hz",
                    app.peak_db, app.lufs, sample_rate
                ),
                FontId::monospace(9.0),
                DIM,
            );
        }
    }

    // ---- spectrogram ------------------------------------------------------
    ui.add_space(6.0);
    let (srect, _) = ui.allocate_exact_size(Vec2::new(full_w, 48.0), Sense::hover());
    waveform::paint_spectrogram(ui, srect, &app.wave);
    {
        let p = ui.painter_at(srect);
        let label = |pos: Pos2, align: Align2, text: &str, font: FontId| {
            let galley = p.layout_no_wrap(text.to_owned(), font, alpha(TEXT, 220));
            let placed = align.anchor_size(pos, galley.size());
            p.rect_filled(placed.expand2(Vec2::new(4.0, 2.0)), 3.0, alpha(CANVAS, 170));
            p.galley(placed.min, galley, alpha(TEXT, 220));
        };
        label(
            Pos2::new(srect.left() + 10.0, srect.top() + 6.0),
            Align2::LEFT_TOP,
            "SPECTRUM",
            FontId::proportional(8.0),
        );
        label(
            Pos2::new(srect.right() - 10.0, srect.top() + 6.0),
            Align2::RIGHT_TOP,
            "70 Hz - 16 kHz",
            FontId::monospace(8.0),
        );
    }

    // ---- style morph ------------------------------------------------------
    ui.add_space(10.0);
    let mut style = app.current_style();
    ui.horizontal(|ui| {
        ui.label(RichText::new("REALISM").size(8.5).strong().color(BLUE));
        let w = (ui.available_width() - 150.0).clamp(110.0, 620.0);
        let resp = widgets::morph_slider(ui, &mut style, w);
        if resp.changed() {
            app.set_style(style);
        }
        ui.label(RichText::new("CARTOON").size(8.5).strong().color(PINK));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{:>3.0}% TOON", style * 100.0))
                    .monospace()
                    .size(9.5)
                    .color(mix(BLUE, PINK, style)),
            );
        });
    });

    // ---- layers -----------------------------------------------------------
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("LAYERS").size(9.5).strong().color(DIM));
        ui.label(
            RichText::new(format!("{layer_count}"))
                .monospace()
                .size(9.0)
                .color(DIM),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new("SELECT A LAYER TO EDIT IT ON THE RIGHT")
                    .size(8.0)
                    .color(DIM),
            );
        });
    });
    ui.add_space(3.0);
    if layer_count == 0 {
        ui.label(RichText::new("This preset has no layers.").size(11.0).color(MUTED));
        return;
    }
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for i in 0..layer_count {
                layer_row(app, ui, i);
                ui.add_space(4.0);
            }
        });
}

fn empty_editor(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(140.0);
        ui.label(RichText::new("NO PRESETS").size(17.0).strong().color(MUTED));
        ui.add_space(6.0);
        ui.label(
            RichText::new("The factory library is empty - add a category module in crates/sg-presets.")
                .size(11.0)
                .color(DIM),
        );
    });
}

fn pill(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(alpha(color, 24))
        .stroke(Stroke::new(1.0, alpha(color, 70)))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text.to_uppercase())
                    .size(8.0)
                    .strong()
                    .color(color),
            );
        });
}

fn layer_row(app: &mut App, ui: &mut Ui, i: usize) {
    let (name, kind, gain, mute, solo) = {
        let Some(r) = app.presets.get(app.selected) else {
            return;
        };
        let Some(l) = r.layers.get(i) else {
            return;
        };
        (l.name.clone(), kind_label(&l.kind), l.gain_db, l.mute, l.solo)
    };
    let selected = app.selected_layer == i;
    let locked = app.is_locked(app.selected, i);

    let width = ui.available_width().max(120.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 44.0), Sense::click());
    let p = ui.painter_at(rect);
    p.rect_filled(
        rect,
        8.0,
        if selected { SURFACE_HI } else { SURFACE },
    );
    if selected {
        p.rect_stroke(rect, 8.0, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
    } else if resp.hovered() {
        p.rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0, Color32::from_rgb(0x39, 0x42, 0x50)),
            StrokeKind::Inside,
        );
    }
    let color = layer_color(i);
    p.rect_filled(
        Rect::from_min_size(
            Pos2::new(rect.left() + 6.0, rect.top() + 10.0),
            Vec2::new(3.0, rect.height() - 20.0),
        ),
        1.5,
        color,
    );
    p.text(
        Pos2::new(rect.left() + 17.0, rect.center().y),
        Align2::LEFT_CENTER,
        format!("{:02}", i + 1),
        FontId::monospace(9.0),
        DIM,
    );
    p.text(
        Pos2::new(rect.left() + 38.0, rect.top() + 7.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(12.5),
        TEXT,
    );
    p.text(
        Pos2::new(rect.left() + 38.0, rect.top() + 25.0),
        Align2::LEFT_TOP,
        format!("{kind} · {gain:+.1} dB"),
        FontId::proportional(9.0),
        DIM,
    );

    // Mini waveform: first 200 ms of the layer alone.
    let thumb = Rect::from_min_size(
        Pos2::new(rect.right() - 104.0, rect.top() + 9.0),
        Vec2::new(96.0, 28.0),
    );
    p.rect_filled(thumb, 4.0, CANVAS);
    if let Some(peaks) = app.layer_thumbs.get(i) {
        waveform::paint_mini(&p, thumb.shrink(3.0), peaks, alpha(color, 235));
    }

    // M / S / L
    let chip_y = rect.center().y - 9.0;
    let mut x = thumb.left() - 82.0;
    if ui
        .put(
            Rect::from_min_size(Pos2::new(x, chip_y), Vec2::new(24.0, 18.0)),
            widgets::chip_button("M", mute, PINK),
        )
        .on_hover_text("Mute (M)")
        .clicked()
    {
        app.toggle_mute(i);
    }
    x += 26.0;
    if ui
        .put(
            Rect::from_min_size(Pos2::new(x, chip_y), Vec2::new(24.0, 18.0)),
            widgets::chip_button("S", solo, AMBER),
        )
        .on_hover_text("Solo (S)")
        .clicked()
    {
        app.toggle_solo(i);
    }
    x += 26.0;
    if ui
        .put(
            Rect::from_min_size(Pos2::new(x, chip_y), Vec2::new(24.0, 18.0)),
            widgets::chip_button("L", locked, ACCENT),
        )
        .on_hover_text("Lock against randomize (L)")
        .clicked()
    {
        app.selected_layer = i;
        app.toggle_lock();
    }

    if resp.clicked() {
        app.selected_layer = i;
    }
}

fn kind_label(kind: &LayerKind) -> &'static str {
    match kind {
        LayerKind::Noise { .. } => "NOISE",
        LayerKind::Tone { .. } => "TONE",
        LayerKind::Modal { .. } => "MODAL",
        LayerKind::Grain { .. } => "GRAIN",
        LayerKind::Rotor { .. } => "ROTOR",
        LayerKind::Sweep { .. } => "SWEEP",
    }
}

// ================================================================ inspector ===

pub fn inspector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("INSPECTOR").size(9.5).strong().color(DIM));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!(
                    "LAYER {}/{}",
                    app.selected_layer + 1,
                    app.layer_count().max(1)
                ))
                .monospace()
                .size(8.5)
                .color(DIM),
            );
        });
    });

    if app.presets.is_empty() {
        ui.add_space(16.0);
        ui.label(RichText::new("Nothing to inspect.").size(11.0).color(MUTED));
        return;
    }
    let layer_count = app.layer_count();
    if layer_count == 0 {
        ui.add_space(16.0);
        ui.label(
            RichText::new("This preset has no layers - nothing to edit.")
                .size(11.0)
                .color(MUTED),
        );
        return;
    }
    let idx = app.selected_layer.min(layer_count - 1);
    app.selected_layer = idx;

    let (name, kind, mute, solo, locked) = {
        let Some(r) = app.current() else { return };
        let Some(l) = r.layers.get(idx) else { return };
        (
            l.name.clone(),
            kind_label(&l.kind),
            l.mute,
            l.solo,
            app.is_locked(app.selected, idx),
        )
    };

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let (bar, _) = ui.allocate_exact_size(Vec2::new(3.0, 26.0), Sense::hover());
        ui.painter().rect_filled(bar, 1.5, layer_color(idx));
        ui.vertical(|ui| {
            ui.label(RichText::new(name).size(15.0).strong().color(TEXT));
            ui.label(RichText::new(kind).size(8.0).strong().color(DIM));
        });
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui
            .add(widgets::chip_button("MUTE", mute, PINK))
            .on_hover_text("Mute this layer (M)")
            .clicked()
        {
            app.toggle_mute(idx);
        }
        if ui
            .add(widgets::chip_button("SOLO", solo, AMBER))
            .on_hover_text("Solo this layer (S)")
            .clicked()
        {
            app.toggle_solo(idx);
        }
        if ui
            .add(widgets::chip_button("LOCK", locked, ACCENT))
            .on_hover_text("Exclude from randomize (L)")
            .clicked()
        {
            app.toggle_lock();
        }
    });
    ui.add_space(4.0);

    let mut changed = false;
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(recipe) = app.presets.get_mut(app.selected) {
                if let Some(layer) = recipe.layers.get_mut(idx) {
                    changed |= level_ui(ui, layer);
                    rule(ui);
                    changed |= kind_ui(ui, &mut layer.kind);
                }
                rule(ui);
                section(ui, "Master chain");
                changed |= master_ui(ui, &mut recipe.master);
                ui.add_space(10.0);
            }
        });
    if changed {
        app.mark_dirty();
    }
}

fn level_ui(ui: &mut Ui, layer: &mut Layer) -> bool {
    let mut changed = false;
    section(ui, "Level & placement");
    changed |= widgets::param(
        ui,
        Param::new("GAIN", "lvl_gain")
            .range(-60.0, 6.0)
            .unit("dB")
            .decimals(1),
        &mut layer.gain_db,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("PAN", "lvl_pan").range(-1.0, 1.0).decimals(2),
        &mut layer.pan,
    )
    .changed();
    let mut delay_ms = layer.delay * 1000.0;
    if widgets::param(
        ui,
        Param::new("DELAY", "lvl_delay")
            .range(0.0, 2000.0)
            .unit("ms")
            .decimals(0),
        &mut delay_ms,
    )
    .changed()
    {
        layer.delay = (delay_ms / 1000.0).max(0.0);
        changed = true;
    }
    changed |= widgets::param(
        ui,
        Param::new("STYLE REAL", "lvl_sr")
            .range(0.0, 2.0)
            .unit("x")
            .decimals(2)
            .color(BLUE),
        &mut layer.style_gain[0],
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("STYLE TOON", "lvl_st")
            .range(0.0, 2.0)
            .unit("x")
            .decimals(2)
            .color(PINK),
        &mut layer.style_gain[1],
    )
    .changed();
    changed
}

fn env_ui(ui: &mut Ui, env: &mut AdEnv) -> bool {
    let mut changed = false;
    section(ui, "Envelope");
    let mut attack_ms = env.attack * 1000.0;
    if widgets::param(
        ui,
        Param::new("ATTACK", "env_a")
            .range(0.2, 2000.0)
            .log()
            .unit("ms")
            .decimals(1),
        &mut attack_ms,
    )
    .changed()
    {
        env.attack = (attack_ms / 1000.0).max(0.0);
        changed = true;
    }
    let mut hold_ms = env.hold * 1000.0;
    if widgets::param(
        ui,
        Param::new("HOLD", "env_h")
            .range(0.0, 1000.0)
            .unit("ms")
            .decimals(0),
        &mut hold_ms,
    )
    .changed()
    {
        env.hold = (hold_ms / 1000.0).max(0.0);
        changed = true;
    }
    let mut decay_ms = env.decay * 1000.0;
    if widgets::param(
        ui,
        Param::new("DECAY", "env_d")
            .range(1.0, 8000.0)
            .log()
            .unit("ms")
            .decimals(0),
        &mut decay_ms,
    )
    .changed()
    {
        env.decay = (decay_ms / 1000.0).max(0.001);
        changed = true;
    }
    changed |= widgets::param(
        ui,
        Param::new("CURVE", "env_c").range(0.5, 6.0).decimals(2),
        &mut env.curve,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("ATK CURVE", "env_ac").range(0.2, 4.0).decimals(2),
        &mut env.attack_curve,
    )
    .changed();
    changed
}

fn filter_ui(ui: &mut Ui, filter: &mut FilterEnv) -> bool {
    let mut changed = false;
    section(ui, "Filter sweep");
    changed |= widgets::param(
        ui,
        Param::new("START", "flt_s")
            .range(20.0, 20000.0)
            .log()
            .unit("Hz")
            .decimals(0),
        &mut filter.start_hz,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("END", "flt_e")
            .range(20.0, 20000.0)
            .log()
            .unit("Hz")
            .decimals(0),
        &mut filter.end_hz,
    )
    .changed();
    let mut tau_ms = filter.tau * 1000.0;
    if widgets::param(
        ui,
        Param::new("TAU", "flt_t")
            .range(0.5, 2000.0)
            .log()
            .unit("ms")
            .decimals(1),
        &mut tau_ms,
    )
    .changed()
    {
        filter.tau = (tau_ms / 1000.0).max(1e-4);
        changed = true;
    }
    changed |= widgets::param(
        ui,
        Param::new("Q", "flt_q").range(0.3, 12.0).decimals(2),
        &mut filter.q,
    )
    .changed();
    changed
}

fn color_row(ui: &mut Ui, color: &mut NoiseColor) -> bool {
    let mut changed = false;
    section(ui, "Noise colour");
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (c, label) in [
            (NoiseColor::White, "WHT"),
            (NoiseColor::Pink, "PNK"),
            (NoiseColor::Brown, "BRN"),
            (NoiseColor::Blue, "BLU"),
            (NoiseColor::Velvet, "VLV"),
        ] {
            if ui
                .add(widgets::chip_button(label, *color == c, ACCENT))
                .clicked()
            {
                *color = c;
                changed = true;
            }
        }
    });
    changed
}

fn shape_row(ui: &mut Ui, shape: &mut WaveShape) -> bool {
    let mut changed = false;
    section(ui, "Waveform");
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (s, label) in [
            (WaveShape::Sine, "SIN"),
            (WaveShape::Triangle, "TRI"),
            (WaveShape::Saw, "SAW"),
            (WaveShape::Square, "SQR"),
        ] {
            if ui
                .add(widgets::chip_button(label, *shape == s, ACCENT))
                .clicked()
            {
                *shape = s;
                changed = true;
            }
        }
    });
    changed
}

fn hz(ui: &mut Ui, label: &str, id: &str, value: &mut f32, lo: f32, hi: f32) -> bool {
    widgets::param(
        ui,
        Param::new(label, id).range(lo, hi).log().unit("Hz").decimals(0),
        value,
    )
    .changed()
}

fn kind_ui(ui: &mut Ui, kind: &mut LayerKind) -> bool {
    let mut changed = false;
    match kind {
        LayerKind::Noise {
            color,
            env,
            filter,
            mode,
            tame_hz,
        } => {
            section(ui, "Noise layer");
            changed |= color_row(ui, color);
            changed |= env_ui(ui, env);
            changed |= filter_ui(ui, filter);
            section(ui, "Filter mode");
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (m, label) in [(0u8, "LOW"), (1, "BAND"), (2, "HIGH")] {
                    if ui
                        .add(widgets::chip_button(label, *mode == m, ACCENT))
                        .clicked()
                    {
                        *mode = m;
                        changed = true;
                    }
                }
            });
            changed |= hz(ui, "TAME", "nz_tame", tame_hz, 200.0, 20000.0);
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
            section(ui, "Tone layer");
            changed |= shape_row(ui, shape);
            changed |= hz(ui, "START", "tn_s", start_hz, 15.0, 12000.0);
            changed |= hz(ui, "END", "tn_e", end_hz, 15.0, 12000.0);
            let mut tau_ms = *pitch_tau * 1000.0;
            if widgets::param(
                ui,
                Param::new("PITCH TAU", "tn_t")
                    .range(1.0, 1000.0)
                    .log()
                    .unit("ms")
                    .decimals(1),
                &mut tau_ms,
            )
            .changed()
            {
                *pitch_tau = (tau_ms / 1000.0).max(1e-4);
                changed = true;
            }
            changed |= env_ui(ui, env);
            section(ui, "Character");
            changed |= widgets::param(
                ui,
                Param::new("FM RATIO", "tn_fr").range(0.0, 8.0).decimals(2),
                fm_hz,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("FM DEPTH", "tn_fd").range(0.0, 1.0).decimals(2),
                fm_depth,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("DRIVE", "tn_dr").range(0.0, 1.0).decimals(2),
                drive,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("VIBRATO", "tn_vb").range(0.0, 1.0).decimals(2),
                vibrato,
            )
            .changed();
        }
        LayerKind::Modal {
            modes,
            exciter_ms,
            exciter_color,
            exciter_hp,
            exciter_lp,
            detune_cents,
        } => {
            section(ui, "Modal bank");
            changed |= widgets::param(
                ui,
                Param::new("DETUNE", "md_dt")
                    .range(-1200.0, 1200.0)
                    .unit("ct")
                    .decimals(0),
                detune_cents,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("EXCITER", "md_ex")
                    .range(0.0, 200.0)
                    .unit("ms")
                    .decimals(0),
                exciter_ms,
            )
            .changed();
            changed |= hz(ui, "EXC HP", "md_hp", exciter_hp, 20.0, 12000.0);
            changed |= hz(ui, "EXC LP", "md_lp", exciter_lp, 200.0, 20000.0);
            changed |= color_row(ui, exciter_color);
            section(ui, &format!("Modes ({})", modes.len()));
            let shown = modes.len().min(6);
            for i in 0..shown {
                if let Some(m) = modes.get_mut(i) {
                    let label = format!("M{} FREQ", i + 1);
                    changed |= hz(ui, &label, "md_f", &mut m.freq, 40.0, 14000.0);
                    let label = format!("M{} DECAY", i + 1);
                    let mut ms = m.decay * 1000.0;
                    if widgets::param(
                        ui,
                        Param::new(&label, "md_d")
                            .range(5.0, 6000.0)
                            .log()
                            .unit("ms")
                            .decimals(0),
                        &mut ms,
                    )
                    .changed()
                    {
                        m.decay = (ms / 1000.0).max(0.005);
                        changed = true;
                    }
                }
            }
            if modes.len() > shown {
                ui.label(
                    RichText::new(format!("+ {} more modes", modes.len() - shown))
                        .size(9.0)
                        .color(DIM),
                );
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
            section(ui, "Granular layer");
            changed |= widgets::param(
                ui,
                Param::new("DENSITY", "gr_d")
                    .range(1.0, 300.0)
                    .unit("/s")
                    .decimals(0),
                density,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("GRAIN", "gr_g")
                    .range(5.0, 300.0)
                    .unit("ms")
                    .decimals(0),
                grain_ms,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("SPRAY", "gr_s").range(0.0, 1.0).decimals(2),
                spray,
            )
            .changed();
            changed |= hz(ui, "SOURCE", "gr_src", source_hz, 30.0, 6000.0);
            changed |= widgets::param(
                ui,
                Param::new("PITCH JIT", "gr_pj").range(0.0, 0.4).decimals(3),
                pitch_jitter,
            )
            .changed();
            changed |= color_row(ui, color);
            changed |= env_ui(ui, env);
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
            section(ui, "Rotor layer");
            changed |= widgets::param(
                ui,
                Param::new("BLADES", "rt_b")
                    .range(2.0, 8.0)
                    .step(1.0)
                    .decimals(0),
                blades,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("RPM", "rt_r")
                    .range(100.0, 6000.0)
                    .unit("rpm")
                    .decimals(0),
                rpm,
            )
            .changed();
            changed |= hz(ui, "MOTOR", "rt_t", tone_hz, 20.0, 2000.0);
            changed |= widgets::param(
                ui,
                Param::new("AIR", "rt_a").range(0.0, 1.0).decimals(2),
                air,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("FLAP", "rt_f").range(0.0, 1.0).decimals(2),
                flap_depth,
            )
            .changed();
            changed |= color_row(ui, color);
            changed |= env_ui(ui, env);
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
            section(ui, "Sweep layer");
            changed |= hz(ui, "START", "sw_s", start_hz, 20.0, 18000.0);
            changed |= hz(ui, "END", "sw_e", end_hz, 20.0, 18000.0);
            let mut tau_ms = *tau * 1000.0;
            if widgets::param(
                ui,
                Param::new("TAU", "sw_t")
                    .range(1.0, 3000.0)
                    .log()
                    .unit("ms")
                    .decimals(0),
                &mut tau_ms,
            )
            .changed()
            {
                *tau = (tau_ms / 1000.0).max(1e-4);
                changed = true;
            }
            changed |= widgets::param(
                ui,
                Param::new("Q", "sw_q").range(0.3, 12.0).decimals(2),
                q,
            )
            .changed();
            changed |= widgets::param(
                ui,
                Param::new("TONE MIX", "sw_tm").range(0.0, 1.0).decimals(2),
                tone_mix,
            )
            .changed();
            changed |= color_row(ui, color);
            changed |= env_ui(ui, env);
        }
    }
    changed
}

fn master_ui(ui: &mut Ui, master: &mut sg_engine::MasterChain) -> bool {
    let mut changed = false;
    changed |= widgets::param(
        ui,
        Param::new("DRIVE", "ms_dr")
            .range(-12.0, 24.0)
            .unit("dB")
            .decimals(1)
            .color(PINK),
        &mut master.drive_db,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("REVERB", "ms_rv").range(0.0, 1.0).decimals(2),
        &mut master.reverb_wet,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("SIZE", "ms_sz").range(0.0, 1.0).decimals(2),
        &mut master.reverb_size,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("PRESENCE", "ms_pr")
            .range(-12.0, 12.0)
            .unit("dB")
            .decimals(1),
        &mut master.presence_db,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("WIDTH", "ms_wd").range(0.0, 2.0).decimals(2),
        &mut master.stereo_width,
    )
    .changed();
    changed |= widgets::param(
        ui,
        Param::new("TARGET", "ms_tp")
            .range(-12.0, 0.0)
            .unit("dB")
            .decimals(1),
        &mut master.target_peak_db,
    )
    .changed();
    changed
}

// =============================================================== variations ===

pub fn variations(app: &mut App, ui: &mut Ui) {
    let kept = app.variations.iter().filter(|v| v.kept).count();
    ui.horizontal(|ui| {
        ui.label(RichText::new("VARIATIONS").size(9.5).strong().color(DIM));
        ui.label(
            RichText::new(format!("{} seeds · {kept} kept", app.variations.len()))
                .monospace()
                .size(8.5)
                .color(DIM),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, "EXPORT KEPT").clicked() {
                app.export_kept();
            }
            if widgets::accent_button(ui, "GENERATE 12").clicked() {
                app.generate_variations(12);
            }
        });
    });
    ui.add_space(6.0);

    if app.variations.is_empty() {
        let hint_h = (ui.available_height() - 20.0).clamp(40.0, 96.0);
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), hint_h), Sense::hover());
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 8.0, alpha(SURFACE, 110));
        p.rect_stroke(rect, 8.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No variations yet - press GENERATE 12 to build a seed family",
            FontId::proportional(11.0),
            DIM,
        );
    } else {
        let mut play: Option<usize> = None;
        let mut keep: Option<usize> = None;
        let now_playing = app.now_playing_var;
        ScrollArea::horizontal()
            .max_height(96.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for (i, v) in app.variations.iter().enumerate() {
                        let (p, k) = variation_card(ui, v, i, now_playing == Some(i));
                        if p {
                            play = Some(i);
                        }
                        if k {
                            keep = Some(i);
                        }
                    }
                });
            });
        if let Some(i) = play {
            app.play_variation(i);
        }
        if let Some(i) = keep {
            app.toggle_keep(i);
        }
    }

    ui.add_space(6.0);
    ui.label(
        RichText::new(
            "SPACE play · R randomize · L lock · S solo · M mute · E export · CTRL+Z undo · UP/DOWN browse",
        )
        .size(8.0)
        .color(DIM),
    );
}

fn variation_card(ui: &mut Ui, v: &Variation, idx: usize, playing: bool) -> (bool, bool) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(152.0, 92.0), Sense::click());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 8.0, SURFACE);
    if playing {
        p.rect_stroke(rect, 8.0, Stroke::new(1.5, ACCENT), StrokeKind::Inside);
    } else if resp.hovered() {
        p.rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0, Color32::from_rgb(0x39, 0x42, 0x50)),
            StrokeKind::Inside,
        );
    }
    if v.kept {
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left() + 3.0, rect.top() + 10.0),
                Vec2::new(3.0, rect.height() - 20.0),
            ),
            1.5,
            ACCENT,
        );
    }
    p.text(
        Pos2::new(rect.left() + 11.0, rect.top() + 6.0),
        Align2::LEFT_TOP,
        format!("s{}", v.seed % 100_000),
        FontId::monospace(8.5),
        DIM,
    );
    let star = Pos2::new(rect.right() - 15.0, rect.top() + 13.0);
    widgets::paint_star(&p, star, 6.0, if v.kept { ACCENT } else { DIM }, v.kept);
    let thumb = Rect::from_min_size(
        Pos2::new(rect.left() + 10.0, rect.top() + 25.0),
        Vec2::new(rect.width() - 20.0, 32.0),
    );
    p.rect_filled(thumb, 4.0, CANVAS);
    waveform::paint_mini(
        &p,
        thumb.shrink(3.0),
        &v.thumb,
        alpha(ACCENT, if v.kept { 255 } else { 200 }),
    );
    p.text(
        Pos2::new(rect.left() + 11.0, rect.bottom() - 8.0),
        Align2::LEFT_BOTTOM,
        format!("{:.2}s  {:.1} LUFS", v.buffer.duration(), v.lufs),
        FontId::monospace(8.5),
        MUTED,
    );
    p.text(
        Pos2::new(rect.right() - 11.0, rect.bottom() - 8.0),
        Align2::RIGHT_BOTTOM,
        format!("{:02}", idx + 1),
        FontId::monospace(8.5),
        DIM,
    );

    let star_hit = Rect::from_center_size(star, Vec2::splat(24.0));
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if star_hit.contains(pos) {
                return (false, true);
            }
        }
        return (true, false);
    }
    (false, false)
}

/// Exported for completeness: status helper used by the transport badge.
pub fn _status_hint(_kind: StatusKind) {}
