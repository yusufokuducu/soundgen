//! SoundGen's visual language: palette, spacing, typography and shared painters.
//!
//! Everything here is presentation only. The goal is a calm, dark studio
//! surface: deep background, slightly lifted panels, one mint accent for
//! "signal", one pink accent for "stylised / cartoon".

use egui::style::WidgetVisuals;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Ui, Visuals,
};

// ---------------------------------------------------------------- palette ---

/// Application background.
pub const BG: Color32 = Color32::from_rgb(0x12, 0x14, 0x1a);
/// Panels and bars.
pub const PANEL: Color32 = Color32::from_rgb(0x17, 0x1a, 0x21);
/// Cards, rows, raised widgets.
pub const SURFACE: Color32 = Color32::from_rgb(0x1c, 0x20, 0x29);
/// Hovered / selected raised widgets.
pub const SURFACE_HI: Color32 = Color32::from_rgb(0x23, 0x28, 0x33);
/// Hairline borders.
pub const BORDER: Color32 = Color32::from_rgb(0x26, 0x2b, 0x36);
/// Waveform / spectrogram canvas.
pub const CANVAS: Color32 = Color32::from_rgb(0x0e, 0x10, 0x15);
/// Primary text.
pub const TEXT: Color32 = Color32::from_rgb(0xe6, 0xe9, 0xef);
/// Secondary text.
pub const MUTED: Color32 = Color32::from_rgb(0x8b, 0x93, 0xa3);
/// Tertiary text / section labels.
pub const DIM: Color32 = Color32::from_rgb(0x5c, 0x64, 0x74);
/// Mint accent: signal, selection, "real".
pub const ACCENT: Color32 = Color32::from_rgb(0x6e, 0xe7, 0xb7);
/// Pink accent: cartoon, mute, "stylised".
pub const PINK: Color32 = Color32::from_rgb(0xf4, 0x72, 0xb6);
/// Warning amber.
pub const AMBER: Color32 = Color32::from_rgb(0xff, 0xb4, 0x54);
/// Error / hot level.
pub const RED: Color32 = Color32::from_rgb(0xff, 0x5d, 0x5d);
/// Cool secondary (realism side of the morph).
pub const BLUE: Color32 = Color32::from_rgb(0x56, 0xb4, 0xe9);

/// Text colour for content drawn on top of a bright accent fill.
pub const ON_ACCENT: Color32 = Color32::from_rgb(0x0b, 0x0d, 0x12);

// ------------------------------------------------------------------ style ---

fn widget(bg: Color32, fg: Color32, stroke: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: CornerRadius::same(7),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

/// Install the SoundGen style on a context. Call once at startup.
pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    let mut style = (*ctx.global_style()).clone();
    style.visuals = Visuals::dark();

    let v = &mut style.visuals;
    v.panel_fill = PANEL;
    v.window_fill = SURFACE;
    v.extreme_bg_color = CANVAS;
    v.faint_bg_color = SURFACE;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x6e, 0xe7, 0xb7, 55);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.text_cursor.stroke = Stroke::new(1.5, ACCENT);

    v.widgets.noninteractive = widget(PANEL, TEXT, BORDER);
    v.widgets.inactive = widget(SURFACE, TEXT, BORDER);
    v.widgets.hovered = widget(SURFACE_HI, ACCENT, Color32::from_rgb(0x39, 0x42, 0x50));
    v.widgets.active = widget(SURFACE_HI, ACCENT, ACCENT);
    v.widgets.open = widget(SURFACE_HI, TEXT, BORDER);

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size = egui::vec2(28.0, 20.0);
    style.spacing.slider_width = 110.0;
    style.spacing.indent = 14.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = true;
    style.animation_time = 0.08;

    style.text_styles = [
        (TextStyle::Heading, FontId::new(19.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(11.5, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_global_style(style);
}

// ---------------------------------------------------------------- helpers ---

/// Layer identity colours (Okabe-Ito, colour-blind safe). Colour is never the
/// only carrier of information: rows also show an index and a name.
pub fn layer_color(i: usize) -> Color32 {
    const PALETTE: [Color32; 8] = [
        Color32::from_rgb(0x00, 0x72, 0xb2),
        Color32::from_rgb(0xe6, 0x9f, 0x00),
        Color32::from_rgb(0x00, 0x9e, 0x73),
        Color32::from_rgb(0xcc, 0x79, 0xa7),
        Color32::from_rgb(0x56, 0xb4, 0xe9),
        Color32::from_rgb(0xd5, 0x5e, 0x00),
        Color32::from_rgb(0xf0, 0xe4, 0x42),
        Color32::from_rgb(0x8b, 0x94, 0x9e),
    ];
    PALETTE[i % PALETTE.len()]
}

/// Small-caps section label.
pub fn section(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(9.5)
            .strong()
            .color(DIM),
    );
}

/// A 1px horizontal rule that spans the available width.
pub fn rule(ui: &mut Ui) {
    let width = ui.available_width().max(1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 9.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), y),
            egui::pos2(rect.right(), y),
        ],
        Stroke::new(1.0, BORDER),
    );
}

/// Linear blend between two colours.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Same colour at a different opacity.
pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Colour for a peak level in dBFS (green / amber / red, per game-audio habit).
pub fn level_color(db: f32) -> Color32 {
    if db > -6.0 {
        RED
    } else if db > -18.0 {
        AMBER
    } else {
        ACCENT
    }
}
