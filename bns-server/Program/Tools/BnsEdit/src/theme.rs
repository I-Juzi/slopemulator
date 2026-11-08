//! Look of the app: warm paper ground, one teal accent, big targets.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

pub const GROUND: Color32 = Color32::from_rgb(0xF4, 0xF1, 0xEA);
pub const PAPER: Color32 = Color32::from_rgb(0xFB, 0xF9, 0xF5);
pub const CARD: Color32 = Color32::WHITE;
pub const LINE: Color32 = Color32::from_rgb(0xE2, 0xDC, 0xCF);
pub const INK: Color32 = Color32::from_rgb(0x1D, 0x1B, 0x18);
pub const MUTED: Color32 = Color32::from_rgb(0x5B, 0x56, 0x4C);
pub const ACCENT: Color32 = Color32::from_rgb(0x0E, 0x6B, 0x62);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xE3, 0xF1, 0xEE);
pub const ORANGE: Color32 = Color32::from_rgb(0xB4, 0x50, 0x0F);
pub const ORANGE_SOFT: Color32 = Color32::from_rgb(0xFB, 0xEB, 0xDD);
pub const BLUE: Color32 = Color32::from_rgb(0x33, 0x46, 0xA0);
pub const BLUE_SOFT: Color32 = Color32::from_rgb(0xE7, 0xEA, 0xF6);
#[allow(dead_code)]
pub const DARK: Color32 = Color32::from_rgb(0x1D, 0x1B, 0x18);
#[allow(dead_code)]
pub const ON_DARK: Color32 = Color32::from_rgb(0xF4, 0xF1, 0xEA);
pub const RED: Color32 = Color32::from_rgb(0xB4, 0x23, 0x18);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(30.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(16.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    style.spacing.interact_size.y = 40.0;
    style.spacing.text_edit_width = 320.0;

    let v = &mut style.visuals;
    *v = egui::Visuals::light();
    v.panel_fill = GROUND;
    v.window_fill = PAPER;
    v.extreme_bg_color = CARD;
    v.faint_bg_color = PAPER;
    v.override_text_color = Some(INK);
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.5, ACCENT);
    v.window_corner_radius = CornerRadius::same(16);
    v.window_stroke = Stroke::new(1.0, LINE);
    let r = CornerRadius::same(12);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = r;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK);
    v.widgets.inactive.bg_fill = CARD;
    v.widgets.inactive.weak_bg_fill = CARD;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0xD6, 0xCF, 0xBF));
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
    v.widgets.hovered.bg_fill = PAPER;
    v.widgets.hovered.weak_bg_fill = PAPER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.5, ACCENT);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, INK);
    v.widgets.active.bg_fill = ACCENT_SOFT;
    v.widgets.active.weak_bg_fill = ACCENT_SOFT;
    v.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, INK);
    ctx.set_global_style(style);
}

pub fn big(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).size(size).strong().color(INK)
}

pub fn muted(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).color(MUTED)
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(egui::Margin::same(20))
}

pub fn primary(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_string())
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(ACCENT)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(12))
    .min_size(egui::vec2(0.0, 48.0))
}
