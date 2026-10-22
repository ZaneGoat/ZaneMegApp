//! The two gauges: a per-thread load bar and the RAM/swap/disk capacity card.

use eframe::egui;

use super::text;
use crate::theme::*;

/// Renders a compact per-thread CPU core load bar (`Core #1`..`Core #12`).
pub fn mini_core_bar(ui: &mut egui::Ui, idx: usize, load_pct: f32) {
    let clamped = (load_pct / 100.0).clamp(0.0, 1.0);
    let color = if load_pct >= 80.0 {
        RED_BRIGHT
    } else if load_pct >= 45.0 {
        AMBER_WARN
    } else {
        CYAN_INFO
    };

    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Core #{}", idx + 1))
                            .size(8.5)
                            .color(TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{load_pct:>3.0}%"))
                                .monospace()
                                .size(8.5)
                                .color(color),
                        );
                    });
                });

                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 4.5),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, BG_VOID);
                if clamped > 0.01 {
                    let fill_w = (rect.width() * clamped).max(2.0);
                    let fill_rect =
                        egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                    ui.painter().rect_filled(fill_rect, 2.0, color);
                }
            });
        });
}

/// Renders a custom Cyber-HUD resource gauge bar with dynamic threshold colouring.
///
/// The two labels under the bar — `8.32 GB Used / 15.44 GB Total` and `7.41 GB Free (Cache: 968
/// MB)` — are numbers, so they are never cut and never painted over each other: they share a row
/// only when the row is wide enough to hold both, otherwise each gets its own line.
pub fn resource_gauge_card(
    ui: &mut egui::Ui,
    title: &str,
    used_label: &str,
    detail_label: &str,
    fraction: f32,
) {
    let clamped = fraction.clamp(0.0, 1.0);
    let pct = clamped * 100.0;
    let (bar_color, badge_bg) = if pct >= 85.0 {
        (RED_BRIGHT, RED_SUBTLE)
    } else if pct >= 65.0 {
        (AMBER_WARN, AMBER_SUBTLE)
    } else {
        (GREEN_OK, GREEN_SUBTLE)
    };

    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(7)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(11.0)
                        .color(TEXT_MAIN),
                );
                status_pill(ui, &format!("{pct:.1}% Used"), bar_color, badge_bg);
            });

            ui.add_space(4.0);

            let desired_size = egui::vec2(ui.available_width(), 10.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
            ui.painter().rect_filled(rect, 5.0, BG_VOID);
            ui.painter().rect_stroke(
                rect,
                5.0,
                egui::Stroke::new(1.0, BORDER_DARK),
                egui::StrokeKind::Inside,
            );

            if clamped > 0.01 {
                let fill_w = (rect.width() * clamped).max(6.0);
                let fill_rect =
                    egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                ui.painter().rect_filled(fill_rect, 5.0, bar_color);
            }

            ui.add_space(4.0);

            let used_font = egui::FontId::monospace(10.5);
            let detail_font = egui::FontId::proportional(9.5);
            let painter = ui.painter().clone();
            let used_w = text::measure(&painter, used_label, used_font, TEXT_MAIN);
            let detail_w = text::measure(&painter, detail_label, detail_font, TEXT_MUTED);
            let both_fit = used_w + 12.0 + detail_w <= ui.available_width();

            if both_fit {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(used_label)
                            .monospace()
                            .size(10.5)
                            .color(TEXT_MAIN),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(detail_label)
                                .size(9.5)
                                .color(TEXT_MUTED),
                        );
                    });
                });
            } else {
                ui.label(
                    egui::RichText::new(used_label)
                        .monospace()
                        .size(10.5)
                        .color(TEXT_MAIN),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(detail_label)
                            .size(9.5)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
            }
        });
}
