use eframe::egui;

// Theme color constants matching the dark cyber aesthetic
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(10, 10, 10);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(18, 18, 18);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(26, 26, 26);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(36, 36, 36);

pub const BORDER_DARK: egui::Color32 = egui::Color32::from_rgb(42, 42, 42);

#[allow(dead_code)]
pub const RED_CRIMSON: egui::Color32 = egui::Color32::from_rgb(107, 0, 0);
pub const RED_PRIMARY: egui::Color32 = egui::Color32::from_rgb(204, 0, 0);
pub const RED_BRIGHT: egui::Color32 = egui::Color32::from_rgb(255, 51, 51);
pub const RED_SUBTLE: egui::Color32 = egui::Color32::from_rgb(58, 10, 10);

pub const GREEN_OK: egui::Color32 = egui::Color32::from_rgb(0, 204, 68);
pub const TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(240, 240, 240);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_DARK;
    visuals.faint_bg_color = BG_PANEL;
    visuals.extreme_bg_color = egui::Color32::from_rgb(8, 8, 8);
    
    // Non-interactive widgets
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);

    // Inactive buttons / widgets
    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = RED_SUBTLE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, RED_PRIMARY);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    // Active / pressed widgets
    visuals.widgets.active.bg_fill = RED_PRIMARY;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, RED_BRIGHT);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    // Open menus / comboboxes
    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, RED_PRIMARY);

    visuals.selection.bg_fill = RED_PRIMARY;
    visuals.selection.stroke = egui::Stroke::new(1.0, RED_BRIGHT);

    ctx.set_visuals(visuals);
}
