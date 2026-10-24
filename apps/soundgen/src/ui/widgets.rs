//! Hand-painted widgets that give SoundGen its studio feel: drag-param rows,
//! the gradient style-morph slider, level meters, chips and small icons.

use eframe::egui::{
    self, Align2, Color32, FontId, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    Ui, Vec2,
};

use super::theme::*;

// ------------------------------------------------------------ small chips ---

/// A compact toggle chip used for M / S / L and mode selectors.
pub fn chip_button(text: &str, active: bool, color: Color32) -> egui::Button<'static> {
    let fg = if active { ON_ACCENT } else { MUTED };
    let txt = egui::RichText::new(text.to_owned())
        .size(10.0)
        .strong()
        .color(fg);
    let mut b = egui::Button::new(txt)
        .corner_radius(5)
        .min_size(Vec2::new(24.0, 18.0));
    b = if active {
        b.fill(color).stroke(Stroke::NONE)
    } else {
        b.fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, BORDER))
    };
    b
}

/// A quiet outline chip for secondary actions ("EXPORT", "NEW SEED").
pub fn ghost_button(ui: &mut Ui, text: &str) -> Response {
    let txt = egui::RichText::new(text.to_owned())
        .size(10.0)
        .strong()
        .color(MUTED);
    ui.add(
        egui::Button::new(txt)
            .corner_radius(6)
            .min_size(Vec2::new(24.0, 24.0))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, BORDER)),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A filled accent button for the one primary action on a surface.
pub fn accent_button(ui: &mut Ui, text: &str) -> Response {
    let txt = egui::RichText::new(text.to_owned())
        .size(10.5)
        .strong()
        .color(ON_ACCENT);
    ui.add(
        egui::Button::new(txt)
            .corner_radius(6)
            .min_size(Vec2::new(24.0, 24.0))
            .fill(ACCENT)
            .stroke(Stroke::NONE),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ------------------------------------------------------------------ icons ---

pub fn paint_play(p: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let h = size * 0.5;
    p.add(Shape::convex_polygon(
        vec![
            Pos2::new(center.x - h * 0.55, center.y - h),
            Pos2::new(center.x - h * 0.55, center.y + h),
            Pos2::new(center.x + h * 0.85, center.y),
        ],
        color,
        Stroke::NONE,
    ));
}

pub fn paint_stop(p: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let h = size * 0.42;
    p.rect_filled(
        Rect::from_center_size(center, Vec2::splat(h * 2.0)),
        2.0,
        color,
    );
}

/// Five-pointed star, drawn as ten convex triangles (no font dependency).
pub fn paint_star(p: &egui::Painter, center: Pos2, r: f32, color: Color32, filled: bool) {
    use std::f32::consts::{FRAC_PI_2, PI};
    let outer = |i: usize| -> Pos2 {
        let a = -FRAC_PI_2 + i as f32 * 2.0 * PI / 5.0;
        center + Vec2::new(a.cos(), a.sin()) * r
    };
    let inner = |i: usize| -> Pos2 {
        let a = -FRAC_PI_2 + (i as f32 + 0.5) * 2.0 * PI / 5.0;
        center + Vec2::new(a.cos(), a.sin()) * r * 0.46
    };
    if filled {
        for i in 0..5 {
            let o0 = outer(i);
            let in0 = inner(i);
            let o1 = outer((i + 1) % 5);
            p.add(Shape::convex_polygon(vec![center, o0, in0], color, Stroke::NONE));
            p.add(Shape::convex_polygon(vec![center, in0, o1], color, Stroke::NONE));
        }
    } else {
        let mut prev = outer(0);
        for i in 0..5 {
            p.line_segment([prev, inner(i)], Stroke::new(1.2, color));
            let o = outer((i + 1) % 5);
            p.line_segment([inner(i), o], Stroke::new(1.2, color));
            prev = o;
        }
    }
}

// -------------------------------------------------------------- transport ---

/// The big play/stop button in the transport bar.
pub fn transport_button(ui: &mut Ui, playing: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(88.0, 30.0), Sense::click());
    let fill = if playing { PINK } else { ACCENT };
    let hovered = resp.hovered();
    let p = ui.painter();
    p.rect_filled(rect, 7.0, if hovered { mix(fill, TEXT, 0.15) } else { fill });
    let icon_c = Pos2::new(rect.left() + 18.0, rect.center().y);
    if playing {
        paint_stop(p, icon_c, 11.0, ON_ACCENT);
    } else {
        paint_play(p, icon_c, 12.0, ON_ACCENT);
    }
    p.text(
        Pos2::new(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        if playing { "STOP" } else { "PLAY" },
        FontId::proportional(11.0),
        ON_ACCENT,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Level meter with a -60..0 dBFS scale, coloured green/amber/red.
pub fn meter(ui: &mut Ui, db: f32, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 9.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, CANVAS);
    let t = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
    if t > 0.0 {
        let steps = 48;
        let seg = rect.width() / steps as f32;
        for i in 0..steps {
            let f = i as f32 / steps as f32;
            if f > t {
                break;
            }
            let level_db = -60.0 + f * 60.0;
            let x = rect.left() + i as f32 * seg;
            p.rect_filled(
                Rect::from_min_size(
                    Pos2::new(x, rect.top() + 1.0),
                    Vec2::new(seg + 0.5, rect.height() - 2.0),
                ),
                1.0,
                level_color(level_db),
            );
        }
    }
    for mark in [-18.0f32, -9.0, -6.0] {
        let x = rect.left() + rect.width() * ((mark + 60.0) / 60.0);
        p.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, alpha(CANVAS, 200)),
        );
    }
    resp
}

/// A slim 16 px value slider for the transport bar (no egui chrome).
pub fn mini_slider(
    ui: &mut Ui,
    id: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    width: f32,
    color: Color32,
) -> Response {
    let width = width.max(40.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 16.0), Sense::hover());
    let mut resp = ui.interact(
        rect,
        ui.scope_id().with(("mini_slider", id)),
        Sense::click_and_drag(),
    );

    let span = (max - min).max(1e-6);
    let mut t = ((*value - min) / span).clamp(0.0, 1.0);
    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            t = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
            let next = min + span * t;
            if (next - *value).abs() > 1e-6 {
                *value = next;
                resp.mark_changed();
            }
        }
    }

    let active = resp.hovered() || resp.dragged();
    let track = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), 4.0));
    let p = ui.painter();
    p.rect_filled(track, 2.0, CANVAS);
    if t > 0.001 {
        p.rect_filled(
            Rect::from_min_size(track.min, Vec2::new(track.width() * t, track.height())),
            2.0,
            if active { color } else { alpha(color, 190) },
        );
    }
    let knob = Pos2::new(track.left() + track.width() * t, track.center().y);
    p.circle_filled(knob, if active { 5.0 } else { 4.2 }, Color32::WHITE);
    p.circle_stroke(
        knob,
        if active { 5.0 } else { 4.2 },
        Stroke::new(1.5, CANVAS),
    );

    resp.on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
}

// ------------------------------------------------------------------ morph ---

/// The Realism <-> Cartoon slider: gradient track, detent, ticks, glowing thumb.
pub fn morph_slider(ui: &mut Ui, value: &mut f32, width: f32) -> Response {
    let width = width.max(70.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::hover());
    let mut resp = ui.interact(rect, ui.scope_id().with("morph_slider"), Sense::click_and_drag());
    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
            if (t - *value).abs() > 1e-4 {
                *value = t;
                resp.mark_changed();
            }
        }
    }
    let v = value.clamp(0.0, 1.0);
    let p = ui.painter();
    let track = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.center().y - 2.0),
        Vec2::new(rect.width(), 10.0),
    );

    // Gradient: cool blue (real) -> mint -> pink (cartoon).
    let mut mesh = Mesh::default();
    let base = mesh.vertices.len() as u32;
    mesh.colored_vertex(track.left_top(), BLUE);
    mesh.colored_vertex(track.right_top(), PINK);
    mesh.colored_vertex(track.right_bottom(), PINK);
    mesh.colored_vertex(track.left_bottom(), BLUE);
    mesh.add_triangle(base, base + 1, base + 2);
    mesh.add_triangle(base, base + 2, base + 3);
    p.add(Shape::mesh(mesh));

    // Round the two end caps so the gradient reads as a pill.
    let cap = 6.0;
    p.rect_filled(
        Rect::from_min_max(track.left_top(), Pos2::new(track.left() + cap, track.bottom())),
        5.0,
        BLUE,
    );
    p.rect_filled(
        Rect::from_min_max(Pos2::new(track.right() - cap, track.top()), track.right_bottom()),
        5.0,
        PINK,
    );

    // Dim everything past the thumb: "amount of cartoon".
    let x = track.left() + track.width() * v;
    if x < track.right() - 0.5 {
        p.rect_filled(
            Rect::from_min_max(Pos2::new(x, track.top()), track.right_bottom()),
            0.0,
            alpha(CANVAS, 165),
        );
    }
    p.rect_stroke(track, 5.0, Stroke::new(1.0, alpha(TEXT, 26)), StrokeKind::Inside);

    // Detent at the centre + quarter ticks.
    let cx = track.left() + track.width() * 0.5;
    p.line_segment(
        [
            Pos2::new(cx, track.bottom() + 2.0),
            Pos2::new(cx, track.bottom() + 6.0),
        ],
        Stroke::new(1.0, DIM),
    );
    for f in [0.0f32, 0.25, 0.75, 1.0] {
        let tx = track.left() + track.width() * f;
        p.line_segment(
            [
                Pos2::new(tx, track.bottom() + 3.0),
                Pos2::new(tx, track.bottom() + 5.0),
            ],
            Stroke::new(1.0, alpha(DIM, 140)),
        );
    }

    // Thumb.
    let thumb = Pos2::new(x, track.center().y);
    if resp.hovered() || resp.dragged() {
        p.circle_filled(thumb, 10.0, alpha(ACCENT, 40));
    }
    p.circle_filled(thumb, 6.5, Color32::WHITE);
    p.circle_stroke(thumb, 6.5, Stroke::new(2.0, CANVAS));

    resp.on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
}

// ------------------------------------------------------------- param rows ---
#[derive(Clone, Copy, PartialEq)]
pub enum Scale {
    Linear,
    Log,
}

/// Builder for a compact label + value + track parameter row.
pub struct Param<'a> {
    label: &'a str,
    id: &'a str,
    min: f32,
    max: f32,
    scale: Scale,
    unit: &'a str,
    decimals: usize,
    color: Color32,
    step: Option<f32>,
}

impl<'a> Param<'a> {
    pub fn new(label: &'a str, id: &'a str) -> Self {
        Self {
            label,
            id,
            min: 0.0,
            max: 1.0,
            scale: Scale::Linear,
            unit: "",
            decimals: 2,
            color: ACCENT,
            step: None,
        }
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max.max(min + 1e-6);
        self
    }
    pub fn log(mut self) -> Self {
        self.scale = Scale::Log;
        self
    }
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.color = c;
        self
    }
    pub fn step(mut self, s: f32) -> Self {
        self.step = Some(s);
        self
    }
}

/// A horizontal drag strip: label top-left, mono value top-right, track below.
/// Drag = coarse, Shift+drag = fine, wheel = single steps.
pub fn param(ui: &mut Ui, spec: Param<'_>, value: &mut f32) -> Response {
    let width = ui.available_width().max(80.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 27.0), Sense::hover());
    let mut resp = ui.interact(
        rect,
        ui.scope_id().with(("param", spec.id)),
        Sense::click_and_drag(),
    );

    let span = (spec.max - spec.min).max(1e-6);
    let to_norm = |v: f32| -> f32 {
        match spec.scale {
            Scale::Linear => ((v - spec.min) / span).clamp(0.0, 1.0),
            Scale::Log => {
                let v = v.clamp(spec.min, spec.max);
                (v / spec.min).ln() / (spec.max / spec.min).ln()
            }
        }
    };
    let from_norm = |t: f32| -> f32 {
        let t = t.clamp(0.0, 1.0);
        match spec.scale {
            Scale::Linear => spec.min + span * t,
            Scale::Log => spec.min * (spec.max / spec.min).powf(t),
        }
    };

    let mut changed = false;
    let mut t = to_norm(*value);
    if resp.dragged() {
        let fine = ui.input(|i| i.modifiers.shift);
        let speed = if fine { 0.0016 } else { 0.0062 };
        t = (t + resp.drag_delta().x * speed).clamp(0.0, 1.0);
        *value = from_norm(t);
        changed = true;
    } else if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.5 {
            let dir = if scroll > 0.0 { 1.0 } else { -1.0 };
            *value = from_norm((t + dir * 0.02).clamp(0.0, 1.0));
            changed = true;
        }
    }
    if let Some(step) = spec.step {
        if changed {
            *value = (*value / step).round() * step;
        }
    }
    *value = value.clamp(spec.min, spec.max);
    if changed {
        resp.mark_changed();
    }

    let active = resp.hovered() || resp.dragged();
    let t = to_norm(*value);
    let p = ui.painter();
    p.text(
        Pos2::new(rect.left(), rect.top() + 1.0),
        Align2::LEFT_TOP,
        spec.label.to_uppercase(),
        FontId::proportional(9.5),
        if active { TEXT } else { MUTED },
    );
    let value_text = format!(
        "{:.*}{}",
        spec.decimals,
        *value,
        if spec.unit.is_empty() {
            String::new()
        } else {
            format!(" {}", spec.unit)
        }
    );
    p.text(
        Pos2::new(rect.right(), rect.top() + 1.0),
        Align2::RIGHT_TOP,
        value_text,
        FontId::monospace(10.0),
        if active { spec.color } else { TEXT },
    );

    let track = Rect::from_min_size(
        Pos2::new(rect.left(), rect.bottom() - 6.0),
        Vec2::new(rect.width(), 4.0),
    );
    p.rect_filled(track, 2.0, CANVAS);
    if t > 0.001 {
        p.rect_filled(
            Rect::from_min_size(track.min, Vec2::new(track.width() * t, track.height())),
            2.0,
            if active {
                spec.color
            } else {
                alpha(spec.color, 190)
            },
        );
        let knob = Pos2::new(track.left() + track.width() * t, track.center().y);
        p.circle_filled(knob, if active { 3.6 } else { 3.0 }, Color32::WHITE);
    }
    if active {
        resp = resp.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
    }
    resp
}
