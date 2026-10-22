//! The terminal-style console: a log tail with Copy/Clear, used by four tabs.

use eframe::egui;

use crate::theme::*;

/// Renders a stylised terminal output console with colour-coded log lines.
///
/// A log line is never painted past the console's right edge — it wraps. The header folds onto a
/// second row when the console is too narrow to hold its title beside the buttons.
pub fn cyber_console(
    ui: &mut egui::Ui,
    id_salt: &str,
    title: &str,
    logs: &mut Vec<String>,
    max_height: f32,
) {
    egui::Frame::new()
        .fill(BG_PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(8)
        .inner_margin(12)
        .show(ui, |ui| {
            let title_font = egui::FontId::proportional(11.5);
            let buttons_need = 170.0;
            let title_needs = ui
                .painter()
                .layout_no_wrap(title.to_owned(), title_font, TEXT_MAIN)
                .size()
                .x
                + 90.0; // the dot, the entry count and the gaps

            if ui.available_width() >= title_needs + buttons_need {
                ui.horizontal(|ui| {
                    dot(ui);
                    ui.label(
                        egui::RichText::new(title)
                            .strong()
                            .size(11.5)
                            .color(TEXT_MAIN),
                    );
                    entries(ui, logs.len());
                    buttons(ui, logs);
                });
            } else {
                // Too narrow for the title and the buttons on one row: the title keeps the row
                // and wraps, the buttons move under it. Nothing is shortened.
                ui.horizontal(|ui| {
                    dot(ui);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(title)
                                .strong()
                                .size(11.5)
                                .color(TEXT_MAIN),
                        )
                        .wrap(),
                    );
                });
                ui.horizontal(|ui| {
                    entries(ui, logs.len());
                    buttons(ui, logs);
                });
            }

            ui.add_space(6.0);

            egui::Frame::new()
                .fill(BG_CONSOLE)
                .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                .corner_radius(6)
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::vertical()
                        .id_salt(id_salt)
                        .stick_to_bottom(true)
                        .max_height(max_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (idx, line) in logs.iter().enumerate() {
                                let color = line_color(line);
                                ui.horizontal_top(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{:>3} │", idx + 1))
                                            .monospace()
                                            .size(9.5)
                                            .color(TEXT_DIM),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(line)
                                                .monospace()
                                                .size(10.5)
                                                .color(color),
                                        )
                                        .wrap(),
                                    );
                                });
                            }
                        });
                });
        });
}

fn dot(ui: &mut egui::Ui) {
    let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter()
        .circle_filled(dot_rect.center(), 4.0, RED_PRIMARY);
}

fn entries(ui: &mut egui::Ui, count: usize) {
    ui.label(
        egui::RichText::new(format!("({count} entries)"))
            .monospace()
            .size(10.0)
            .color(TEXT_DIM),
    );
}

fn buttons(ui: &mut egui::Ui, logs: &mut Vec<String>) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui
            .add(
                egui::Button::new(egui::RichText::new("🗑 Clear").size(10.5).color(TEXT_MUTED))
                    .fill(BG_CARD)
                    .min_size(egui::vec2(64.0, 22.0)),
            )
            .clicked()
        {
            logs.clear();
        }
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("📋 Copy All")
                        .size(10.5)
                        .color(TEXT_MUTED),
                )
                .fill(BG_CARD)
                .min_size(egui::vec2(76.0, 22.0)),
            )
            .clicked()
        {
            ui.ctx().copy_text(logs.join("\n"));
        }
    });
}

fn line_color(line: &str) -> egui::Color32 {
    if line.contains("[✓]") || line.contains("COMPLETED") || line.contains("RECLAIMED") {
        GREEN_OK
    } else if line.contains("[!]")
        || line.contains("[ERR]")
        || line.contains("UNVERIFIED")
        || line.contains("WARNING")
    {
        RED_BRIGHT
    } else if line.starts_with("[*]") || line.starts_with("[RAM]") {
        CYAN_INFO
    } else if line.starts_with("===") || line.starts_with("---") {
        TEXT_DIM
    } else {
        TEXT_CONSOLE
    }
}
