//! The visual engine: palette tokens, the egui style, and the two layout primitives every tab
//! must use (design §5.6, §1.4 L2/L3/L6).
//!
//! ```
//! use crate::theme::*;
//! ```
//! re-exports the tokens, [`grid`]/[`split`] and the custom-painted widgets, so a tab never has to
//! know which file a widget lives in.

use eframe::egui;

mod grid;
mod tokens;
mod widgets;

pub use grid::{clamp_h, clamp_w, grid, split};
pub use tokens::*;
pub use widgets::*;

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_DARK;
    visuals.faint_bg_color = BG_PANEL;
    visuals.extreme_bg_color = BG_VOID;

    let radius = egui::CornerRadius::same(7);

    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.weak_bg_fill = BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);
    visuals.widgets.noninteractive.corner_radius = radius;

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.weak_bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);
    visuals.widgets.inactive.corner_radius = radius;

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.4, RED_BRIGHT);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.hovered.corner_radius = radius;

    visuals.widgets.active.bg_fill = RED_PRIMARY;
    visuals.widgets.active.weak_bg_fill = RED_PRIMARY;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.6, RED_BRIGHT);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.active.corner_radius = radius;

    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.weak_bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, RED_PRIMARY);
    visuals.widgets.open.corner_radius = radius;

    visuals.selection.bg_fill = RED_PRIMARY;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    ctx.set_visuals(visuals);
}

/// Smooth breathing pulse factor (`0.45..=1.0`) for active status lights and borders
pub fn pulse_factor(ctx: &egui::Context) -> f32 {
    let t = ctx.input(|i| i.time) as f32;
    0.72 + 0.28 * (t * 3.2).sin()
}
