//! The 250° cockpit arc dial: one number, its unit and a plain-language state, in a card that is
//! never wider than the column it was given.

use eframe::egui;

use crate::theme::*;

/// Renders a custom vector-painted 250° Radial Cockpit Arc Dial with digital readout & plain
/// status label.
///
/// The readout sits in a fixed box in the middle of the arc, so the card width only has to hold
/// the title and the sub-label; both wrap instead of running past the card edge.
pub fn radial_dial(
    ui: &mut egui::Ui,
    title: &str,
    primary_value: &str,
    sub_value: &str,
    fraction: f32,
    accent: egui::Color32,
) {
    let clamped = fraction.clamp(0.0, 1.0);

    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(title)
                            .strong()
                            .size(10.0)
                            .color(TEXT_MAIN),
                    )
                    .wrap(),
                );
                ui.add_space(2.0);

                let dial_size = egui::vec2(clamp_w(ui, 80.0), 62.0);
                let (rect, _) = ui.allocate_exact_size(dial_size, egui::Sense::hover());
                let painter = ui.painter();

                let center = egui::pos2(rect.center().x, rect.center().y + 4.0);
                let radius = 27.0;

                let start_angle = 145.0_f32.to_radians();
                let sweep_angle = 250.0_f32.to_radians();
                let segments = 28;

                // 1. Background Track Arc
                let bg_pts: Vec<egui::Pos2> = (0..=segments)
                    .map(|i| {
                        let t = i as f32 / segments as f32;
                        let ang = start_angle + t * sweep_angle;
                        egui::pos2(center.x + radius * ang.cos(), center.y + radius * ang.sin())
                    })
                    .collect();
                painter.add(egui::Shape::line(bg_pts, egui::Stroke::new(5.0, BG_VOID)));

                // 2. Active Glowing Value Arc
                let active_segs = ((segments as f32 * clamped).ceil() as usize).max(1);
                if clamped > 0.01 {
                    let fg_pts: Vec<egui::Pos2> = (0..=active_segs)
                        .map(|i| {
                            let t = (i as f32 / active_segs as f32) * clamped;
                            let ang = start_angle + t * sweep_angle;
                            egui::pos2(center.x + radius * ang.cos(), center.y + radius * ang.sin())
                        })
                        .collect();

                    painter.add(egui::Shape::line(
                        fg_pts.clone(),
                        egui::Stroke::new(8.5, accent.gamma_multiply(0.24)),
                    ));
                    painter.add(egui::Shape::line(fg_pts, egui::Stroke::new(5.0, accent)));
                }

                // 3. Center Digital Value Text — the number and its unit, never shortened.
                painter.text(
                    egui::pos2(center.x, center.y - 2.0),
                    egui::Align2::CENTER_CENTER,
                    primary_value,
                    egui::FontId::proportional(15.0),
                    egui::Color32::WHITE,
                );

                // 4. Bottom Sub-Label
                ui.add(
                    egui::Label::new(egui::RichText::new(sub_value).size(9.5).color(accent)).wrap(),
                );
            });
        });
}
