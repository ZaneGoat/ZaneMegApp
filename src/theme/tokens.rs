//! The palette — every raw colour the app can paint — AC-43's "colour and sizing come from tokens".
//!
//! `Color32::from_rgb` lives here and nowhere else in `src/`: a widget that wants a colour names
//! a token. The mapping from a *state* to a token is the caller's job (see the tabs), so a green
//! here never means "ok" on its own.

use eframe::egui;

pub const BG_VOID: egui::Color32 = egui::Color32::from_rgb(6, 6, 9);
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(10, 10, 15);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(15, 15, 22);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(22, 22, 32);
pub const BG_CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(30, 24, 36);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(32, 32, 46);
pub const BG_CONSOLE: egui::Color32 = egui::Color32::from_rgb(7, 7, 11);
pub const BG_CARD_CHECKED: egui::Color32 = egui::Color32::from_rgb(20, 28, 34);

pub const BORDER_DARK: egui::Color32 = egui::Color32::from_rgb(42, 42, 58);
pub const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(28, 28, 40);

#[allow(dead_code)]
pub const RED_CRIMSON: egui::Color32 = egui::Color32::from_rgb(95, 0, 20);
pub const RED_PRIMARY: egui::Color32 = egui::Color32::from_rgb(228, 0, 43);
pub const RED_BRIGHT: egui::Color32 = egui::Color32::from_rgb(255, 48, 88);
pub const RED_SUBTLE: egui::Color32 = egui::Color32::from_rgb(48, 8, 18);

pub const GREEN_OK: egui::Color32 = egui::Color32::from_rgb(0, 230, 118);
pub const GREEN_SUBTLE: egui::Color32 = egui::Color32::from_rgb(6, 45, 24);

pub const AMBER_WARN: egui::Color32 = egui::Color32::from_rgb(255, 176, 0);
pub const AMBER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(48, 32, 6);

pub const CYAN_INFO: egui::Color32 = egui::Color32::from_rgb(0, 229, 255);
pub const CYAN_SUBTLE: egui::Color32 = egui::Color32::from_rgb(6, 38, 48);

pub const TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(244, 244, 250);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(148, 148, 170);
pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(94, 94, 116);
/// Console lines that carry no class of their own.
pub const TEXT_CONSOLE: egui::Color32 = egui::Color32::from_rgb(215, 215, 225);
