use eframe::egui;

// ═══════════════════════════════════════════════════════════════════════════
//  CYBER-DECK V3.0 VISUAL ENGINE & COLOR PALETTE
// ═══════════════════════════════════════════════════════════════════════════

pub const BG_VOID: egui::Color32 = egui::Color32::from_rgb(6, 6, 9);
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(10, 10, 15);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(15, 15, 22);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(22, 22, 32);
pub const BG_CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(30, 24, 36);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(32, 32, 46);
pub const BG_CONSOLE: egui::Color32 = egui::Color32::from_rgb(7, 7, 11);

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

// ═══════════════════════════════════════════════════════════════════════════
//  CUSTOM GPU-PAINTED INTERACTIVE WIDGETS
// ═══════════════════════════════════════════════════════════════════════════

/// Renders a pill-shaped status badge with a subtle background tint and crisp border
pub fn status_pill(ui: &mut egui::Ui, text: &str, fg: egui::Color32, bg: egui::Color32) {
    egui::Frame::new()
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, fg.gamma_multiply(0.65)))
        .corner_radius(12)
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).strong().size(10.0).color(fg));
        });
}

/// Renders a visual color-key guide banner so users immediately understand color meanings
pub fn legend_strip(ui: &mut egui::Ui, summary_hint: &str, items: &[(&str, egui::Color32)]) {
    egui::Frame::new()
        .fill(BG_VOID)
        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("💡 GUIDE:").strong().size(10.0).color(CYAN_INFO));
                ui.label(egui::RichText::new(summary_hint).size(10.0).color(TEXT_MUTED));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (label, color) in items.iter().rev() {
                        ui.label(egui::RichText::new(*label).size(9.5).color(TEXT_MAIN));
                        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.8, *color);
                        ui.add_space(6.0);
                    }
                });
            });
        });
}

/// Custom-painted Left Sidebar Navigation Card Button with icon emblem, title, subtitle, hotkey, and live badge
pub fn nav_sidebar_item(
    ui: &mut egui::Ui,
    hotkey: &str,
    icon: &str,
    title: &str,
    subtitle: &str,
    badge: Option<(&str, egui::Color32)>,
    is_active: bool,
) -> bool {
    let desired = egui::vec2(ui.available_width(), 54.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();
    let pulse = pulse_factor(ui.ctx());

    let bg = if is_active {
        RED_SUBTLE
    } else if hovered {
        BG_CARD_HOVER
    } else {
        BG_CARD
    };

    let border_col = if is_active {
        RED_BRIGHT.gamma_multiply(pulse)
    } else if hovered {
        RED_PRIMARY
    } else {
        BORDER_DARK
    };

    painter.rect_filled(rect, 8.0, bg);
    painter.rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(if is_active { 1.6 } else { 1.0 }, border_col),
        egui::StrokeKind::Inside,
    );

    // Left vertical glowing indicator bar
    if is_active || hovered {
        let bar_h = if is_active { 34.0 } else { 20.0 };
        let bar_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 3.0, rect.center().y),
            egui::vec2(4.0, bar_h),
        );
        painter.rect_filled(bar_rect, 2.0, if is_active { RED_BRIGHT } else { RED_PRIMARY });
    }

    // Icon container box
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 26.0, rect.center().y),
        egui::vec2(32.0, 32.0),
    );
    painter.rect_filled(
        icon_rect,
        6.0,
        if is_active {
            RED_PRIMARY.gamma_multiply(0.35)
        } else {
            BG_VOID
        },
    );
    painter.text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(16.0),
        TEXT_MAIN,
    );

    // Title & Subtitle
    let text_x = rect.left() + 48.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 17.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        if is_active || hovered {
            egui::Color32::WHITE
        } else {
            TEXT_MAIN
        },
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 36.0),
        egui::Align2::LEFT_CENTER,
        subtitle,
        egui::FontId::proportional(9.5),
        if is_active { RED_BRIGHT } else { TEXT_MUTED },
    );

    // Right-side hotkey badge + optional status pill
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 15.0),
        egui::Align2::RIGHT_CENTER,
        hotkey,
        egui::FontId::monospace(9.0),
        TEXT_DIM,
    );

    if let Some((b_txt, b_col)) = badge {
        painter.text(
            egui::pos2(rect.right() - 10.0, rect.top() + 36.0),
            egui::Align2::RIGHT_CENTER,
            b_txt,
            egui::FontId::monospace(9.0),
            b_col,
        );
    }

    resp.clicked()
}

/// Custom-painted Interactive Action Card Button with icon box, bold title, plain-English subtitle, and status badge
pub fn action_card_button(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    subtitle: &str,
    badge: Option<(&str, egui::Color32)>,
    accent: egui::Color32,
    is_active: bool,
) -> bool {
    let desired = egui::vec2(ui.available_width(), 52.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();

    let bg = if is_active {
        accent.gamma_multiply(0.18)
    } else if hovered {
        BG_CARD_HOVER
    } else {
        BG_CARD
    };

    let stroke_col = if is_active || hovered {
        accent
    } else {
        BORDER_DARK
    };

    painter.rect_filled(rect, 8.0, bg);
    painter.rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(if is_active || hovered { 1.5 } else { 1.0 }, stroke_col),
        egui::StrokeKind::Inside,
    );

    // Left accent stripe
    let stripe_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + 8.0),
        egui::vec2(3.5, rect.height() - 16.0),
    );
    painter.rect_filled(stripe_rect, 2.0, accent);

    // Icon badge container
    let icon_box = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 26.0, rect.center().y),
        egui::vec2(32.0, 32.0),
    );
    painter.rect_filled(icon_box, 6.0, accent.gamma_multiply(0.16));
    painter.rect_stroke(
        icon_box,
        6.0,
        egui::Stroke::new(1.0, accent.gamma_multiply(0.45)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        icon_box.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(15.5),
        TEXT_MAIN,
    );

    // Title & plain-English subtitle
    let text_x = rect.left() + 50.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 17.5),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 35.5),
        egui::Align2::LEFT_CENTER,
        subtitle,
        egui::FontId::proportional(10.0),
        TEXT_MUTED,
    );

    // Right status text or action chevron
    if let Some((b_text, b_col)) = badge {
        painter.text(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            b_text,
            egui::FontId::monospace(10.0),
            b_col,
        );
    } else {
        painter.text(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "▶",
            egui::FontId::proportional(11.0),
            if hovered { accent } else { TEXT_DIM },
        );
    }

    resp.clicked()
}

/// Custom-painted Multi-Line Mode Selector Card (for Power Profiles, Battery Caps & Audio Modes)
pub fn mode_selector_card(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    spec_1: &str,
    spec_2: &str,
    spec_3: &str,
    accent: egui::Color32,
    is_active: bool,
) -> bool {
    let desired = egui::vec2(ui.available_width(), 92.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();
    let pulse = pulse_factor(ui.ctx());

    let bg = if is_active {
        accent.gamma_multiply(0.16)
    } else if hovered {
        BG_CARD_HOVER
    } else {
        BG_CARD
    };

    let border_col = if is_active {
        accent.gamma_multiply(pulse)
    } else if hovered {
        accent
    } else {
        BORDER_DARK
    };

    painter.rect_filled(rect, 8.0, bg);
    painter.rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(if is_active { 1.8 } else { 1.0 }, border_col),
        egui::StrokeKind::Inside,
    );

    // Top horizontal glowing accent bar
    let top_bar = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 10.0, rect.top()),
        egui::vec2(rect.width() - 20.0, if is_active { 3.5 } else { 2.0 }),
    );
    painter.rect_filled(
        top_bar,
        2.0,
        if is_active || hovered { accent } else { BORDER_SUBTLE },
    );

    // Header: Icon + Title + Active Badge
    painter.text(
        egui::pos2(rect.left() + 12.0, rect.top() + 19.0),
        egui::Align2::LEFT_CENTER,
        format!("{icon} {title}"),
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    let status_lbl = if is_active { "● ACTIVE" } else { "SELECT" };
    let status_col = if is_active { accent } else if hovered { TEXT_MAIN } else { TEXT_DIM };
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 19.0),
        egui::Align2::RIGHT_CENTER,
        status_lbl,
        egui::FontId::monospace(9.5),
        status_col,
    );

    // Divider line
    painter.line_segment(
        [
            egui::pos2(rect.left() + 10.0, rect.top() + 33.0),
            egui::pos2(rect.right() - 10.0, rect.top() + 33.0),
        ],
        egui::Stroke::new(1.0, BORDER_SUBTLE),
    );

    // 3 Spec bullet points so the user clearly understands what this mode does
    for (idx, spec) in [spec_1, spec_2, spec_3].iter().enumerate() {
        if !spec.is_empty() {
            let y = rect.top() + 46.0 + (idx as f32 * 16.0);
            painter.text(
                egui::pos2(rect.left() + 12.0, y),
                egui::Align2::LEFT_CENTER,
                *spec,
                egui::FontId::proportional(10.0),
                if is_active { TEXT_MAIN } else { TEXT_MUTED },
            );
        }
    }

    resp.clicked()
}

/// Custom-painted Interactive Toggle Card for Cleaner Modules with checkmark badge & plain description
pub fn module_toggle_card(
    ui: &mut egui::Ui,
    checked: bool,
    title: &str,
    description: &str,
    category: &str,
    requires_root: bool,
) -> bool {
    let desired = egui::vec2(ui.available_width(), 46.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();

    let bg = if checked {
        egui::Color32::from_rgb(20, 28, 34)
    } else if hovered {
        BG_CARD_HOVER
    } else {
        BG_CARD
    };

    let border_col = if checked {
        CYAN_INFO.gamma_multiply(0.65)
    } else if hovered {
        RED_PRIMARY
    } else {
        BORDER_SUBTLE
    };

    painter.rect_filled(rect, 6.0, bg);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.0, border_col),
        egui::StrokeKind::Inside,
    );

    // Custom Checkmark Box
    let chk_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        egui::vec2(18.0, 18.0),
    );
    painter.rect_filled(
        chk_rect,
        4.0,
        if checked { GREEN_OK } else { BG_VOID },
    );
    painter.rect_stroke(
        chk_rect,
        4.0,
        egui::Stroke::new(1.0, if checked { GREEN_OK } else { BORDER_DARK }),
        egui::StrokeKind::Inside,
    );
    if checked {
        painter.text(
            chk_rect.center(),
            egui::Align2::CENTER_CENTER,
            "✓",
            egui::FontId::proportional(12.0),
            BG_VOID,
        );
    }

    // Module Name & Plain-English Explanation
    let text_x = rect.left() + 38.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 15.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(11.5),
        if checked { TEXT_MAIN } else { TEXT_MUTED },
    );

    // Truncate long descriptions gracefully so it never overlaps
    let short_desc = if description.len() > 52 {
        format!("{}…", &description[..51])
    } else {
        description.to_string()
    };
    painter.text(
        egui::pos2(text_x, rect.top() + 32.0),
        egui::Align2::LEFT_CENTER,
        short_desc,
        egui::FontId::proportional(9.5),
        TEXT_DIM,
    );

    // Right badge: ADMIN vs SAFE
    let (tag_txt, tag_col) = if requires_root {
        ("🔑 ADMIN", AMBER_WARN)
    } else {
        (category, CYAN_INFO)
    };
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 15.0),
        egui::Align2::RIGHT_CENTER,
        tag_txt,
        egui::FontId::monospace(8.5),
        tag_col,
    );

    resp.clicked()
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
                        egui::RichText::new(format!("— {sub}"))
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

/// Renders a custom vector-painted 250° Radial Cockpit Arc Dial with digital readout & plain status label
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
                ui.label(egui::RichText::new(title).strong().size(10.0).color(TEXT_MAIN));
                ui.add_space(2.0);

                let dial_size = egui::vec2(80.0, 62.0);
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

                // 3. Center Digital Value Text
                painter.text(
                    egui::pos2(center.x, center.y - 2.0),
                    egui::Align2::CENTER_CENTER,
                    primary_value,
                    egui::FontId::proportional(15.0),
                    egui::Color32::WHITE,
                );

                // 4. Bottom Sub-Label
                ui.label(egui::RichText::new(sub_value).size(9.5).color(accent));
            });
        });
}

/// Renders a compact per-thread CPU core load bar (`T0`..`T11`)
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

                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.5), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, BG_VOID);
                if clamped > 0.01 {
                    let fill_w = (rect.width() * clamped).max(2.0);
                    let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                    ui.painter().rect_filled(fill_rect, 2.0, color);
                }
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
                    status_pill(ui, &format!("{pct:.1}% Used"), bar_color, badge_bg);
                });
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
                let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                ui.painter().rect_filled(fill_rect, 5.0, bar_color);
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(used_label).monospace().size(10.5).color(TEXT_MAIN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(detail_label).size(9.5).color(TEXT_MUTED));
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
                    egui::RichText::new(format!("({} entries)", logs.len()))
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
                            egui::Button::new(egui::RichText::new("📋 Copy All").size(10.5).color(TEXT_MUTED))
                                .fill(BG_CARD)
                                .min_size(egui::vec2(76.0, 22.0)),
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
