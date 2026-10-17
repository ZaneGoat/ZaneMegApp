use eframe::egui;

// ═══════════════════════════════════════════════════════════════════════════
//  CYBER-DECK V2.5 COLOR PALETTE
// ═══════════════════════════════════════════════════════════════════════════

pub const BG_VOID: egui::Color32 = egui::Color32::from_rgb(6, 6, 8);
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(10, 10, 14);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(16, 16, 22);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(23, 23, 31);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(32, 32, 44);
pub const BG_CONSOLE: egui::Color32 = egui::Color32::from_rgb(7, 7, 10);

pub const BORDER_DARK: egui::Color32 = egui::Color32::from_rgb(40, 40, 54);
pub const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(28, 28, 38);

pub const RED_CRIMSON: egui::Color32 = egui::Color32::from_rgb(95, 0, 18);
pub const RED_PRIMARY: egui::Color32 = egui::Color32::from_rgb(225, 0, 42);
pub const RED_BRIGHT: egui::Color32 = egui::Color32::from_rgb(255, 45, 85);
pub const RED_SUBTLE: egui::Color32 = egui::Color32::from_rgb(48, 8, 16);

pub const GREEN_OK: egui::Color32 = egui::Color32::from_rgb(0, 230, 118);
pub const GREEN_SUBTLE: egui::Color32 = egui::Color32::from_rgb(6, 45, 24);

pub const AMBER_WARN: egui::Color32 = egui::Color32::from_rgb(255, 171, 0);
pub const AMBER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(48, 32, 6);

pub const CYAN_INFO: egui::Color32 = egui::Color32::from_rgb(0, 229, 255);
pub const CYAN_SUBTLE: egui::Color32 = egui::Color32::from_rgb(6, 38, 46);

pub const TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(242, 242, 247);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(138, 138, 158);
pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(90, 90, 110);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_DARK;
    visuals.faint_bg_color = BG_PANEL;
    visuals.extreme_bg_color = BG_VOID;

    let radius = egui::CornerRadius::same(6);

    // Non-interactive widgets
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.weak_bg_fill = BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);
    visuals.widgets.noninteractive.corner_radius = radius;

    // Inactive buttons / widgets
    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.weak_bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_MAIN);
    visuals.widgets.inactive.corner_radius = radius;

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = RED_SUBTLE;
    visuals.widgets.hovered.weak_bg_fill = RED_SUBTLE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.2, RED_BRIGHT);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.hovered.corner_radius = radius;

    // Active / pressed widgets
    visuals.widgets.active.bg_fill = RED_PRIMARY;
    visuals.widgets.active.weak_bg_fill = RED_PRIMARY;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, RED_BRIGHT);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.active.corner_radius = radius;

    // Open menus / comboboxes
    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.weak_bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, RED_PRIMARY);
    visuals.widgets.open.corner_radius = radius;

    visuals.selection.bg_fill = RED_PRIMARY;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    ctx.set_visuals(visuals);
}

// ═══════════════════════════════════════════════════════════════════════════
//  REUSABLE CYBER-HUD COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

/// Renders a pill-shaped status badge with a subtle background tint and crisp border
pub fn status_pill(ui: &mut egui::Ui, text: &str, fg: egui::Color32, bg: egui::Color32) {
    egui::Frame::new()
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, fg.gamma_multiply(0.65)))
        .corner_radius(12)
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).strong().size(10.5).color(fg));
        });
}

/// Renders a sleek Cyber-Deck card container with a top header bar, accent marker, and optional right badge
pub fn cyber_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: Option<&str>,
    accent: egui::Color32,
    badge: Option<(&str, egui::Color32, egui::Color32)>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(BG_PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(8)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left vertical neon bar
                let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, accent);

                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(12.5)
                        .color(TEXT_MAIN),
                );

                if let Some(sub) = subtitle {
                    ui.label(
                        egui::RichText::new(format!("// {sub}"))
                            .size(10.5)
                            .color(TEXT_MUTED),
                    );
                }

                if let Some((b_text, b_fg, b_bg)) = badge {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_pill(ui, b_text, b_fg, b_bg);
                    });
                }
            });

            ui.add_space(4.0);
            let sep_rect = ui.available_rect_before_wrap();
            let line_y = sep_rect.top();
            ui.painter().line_segment(
                [egui::pos2(sep_rect.left(), line_y), egui::pos2(sep_rect.right(), line_y)],
                egui::Stroke::new(1.0, BORDER_SUBTLE),
            );
            ui.add_space(6.0);

            add_contents(ui)
        })
        .inner
}

/// Renders a compact telemetry tile with a left neon stripe, title, big primary value, and sub-label
pub fn metric_tile(
    ui: &mut egui::Ui,
    title: &str,
    primary_value: &str,
    sub_value: &str,
    accent: egui::Color32,
) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(7)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(3.5, 38.0), egui::Sense::hover());
                ui.painter().rect_filled(bar_rect, 2.0, accent);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).strong().size(9.5).color(TEXT_MUTED));
                    ui.label(egui::RichText::new(primary_value).strong().size(16.5).color(TEXT_MAIN));
                    ui.label(egui::RichText::new(sub_value).monospace().size(9.5).color(accent));
                });
            });
        });
}

/// Renders a custom Cyber-HUD resource gauge bar with dynamic threshold coloring
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
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).strong().size(11.0).color(TEXT_MAIN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    status_pill(ui, &format!("{pct:.1}%"), bar_color, badge_bg);
                });
            });

            ui.add_space(4.0);

            // Custom painted progress track + glowing bar
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
                let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                ui.painter().rect_filled(fill_rect, 5.0, bar_color);
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(used_label).monospace().size(10.5).color(TEXT_MAIN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(detail_label).monospace().size(9.5).color(TEXT_MUTED));
                });
            });
        });
}

/// Renders a stylized terminal output console with color-coded log lines
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
            ui.horizontal(|ui| {
                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, RED_PRIMARY);

                ui.label(egui::RichText::new(title).strong().size(11.5).color(TEXT_MAIN));
                ui.label(
                    egui::RichText::new(format!("({} lines)", logs.len()))
                        .monospace()
                        .size(10.0)
                        .color(TEXT_DIM),
                );

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
                            egui::Button::new(egui::RichText::new("📋 Copy").size(10.5).color(TEXT_MUTED))
                                .fill(BG_CARD)
                                .min_size(egui::vec2(64.0, 22.0)),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(logs.join("\n"));
                    }
                });
            });

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
                                let color = if line.contains("[✓]") || line.contains("COMPLETED") || line.contains("RECLAIMED") {
                                    GREEN_OK
                                } else if line.contains("[!]") || line.contains("[ERR]") || line.contains("UNVERIFIED") || line.contains("WARNING") {
                                    RED_BRIGHT
                                } else if line.starts_with("[*]") || line.starts_with("[RAM]") {
                                    CYAN_INFO
                                } else if line.starts_with("===") || line.starts_with("---") {
                                    TEXT_DIM
                                } else {
                                    egui::Color32::from_rgb(215, 215, 225)
                                };

                                ui.horizontal_top(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{:>3} │", idx + 1))
                                            .monospace()
                                            .size(9.5)
                                            .color(TEXT_DIM),
                                    );
                                    ui.label(egui::RichText::new(line).monospace().size(10.5).color(color));
                                });
                            }
                        });
                });
        });
}
