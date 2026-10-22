//! The card family: the container every tab paints, plus the three interactive card-shaped
//! controls (an action, a mode, a toggle).
//!
//! All three lay their text out before allocating their height, so a narrow column gives a
//! taller card rather than a title whose last words are missing. The description sentence is the
//! one thing that may end in `…`, and then the full sentence is the card's tooltip.

use eframe::egui;

use super::text;
use crate::theme::*;

/// Header text of a card, at most `rows` rows deep (design §5.6 / AC-42: labels wrap).
const TITLE: f32 = 12.0;
const SUBTITLE: f32 = 10.0;
const SMALL: f32 = 9.5;

/// Renders a sleek Cyber-Deck card container with a top header bar, accent marker, and optional
/// right badge.
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
            // Wrapped, not horizontal: title, plain-English subtitle and the right badge would
            // otherwise overflow a half-width card and clip the badge off the card edge.
            ui.horizontal_wrapped(|ui| {
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
                    status_pill(ui, b_text, b_fg, b_bg);
                }
            });

            ui.add_space(4.0);
            let sep_rect = ui.available_rect_before_wrap();
            let line_y = sep_rect.top();
            ui.painter().line_segment(
                [
                    egui::pos2(sep_rect.left(), line_y),
                    egui::pos2(sep_rect.right(), line_y),
                ],
                egui::Stroke::new(1.0, BORDER_SUBTLE),
            );
            ui.add_space(6.0);

            add_contents(ui)
        })
        .inner
}

/// Custom-painted Interactive Action Card Button with icon box, bold title, plain-English
/// subtitle, and status badge.
///
/// The title always stays whole: it wraps onto a second row and the card grows to hold it. Only
/// the subtitle (a sentence) is shortened, and the tooltip carries the full wording.
pub fn action_card_button(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    subtitle: &str,
    badge: Option<(&str, egui::Color32)>,
    accent: egui::Color32,
    is_active: bool,
) -> bool {
    let card_w = clamp_w(ui, ui.available_width());
    let painter = ui.painter().clone();

    // Reserve the badge (or the chevron) before laying the text out, so the two can never meet.
    let right_reserve = match badge {
        Some((text, color)) => {
            text::measure(&painter, text, egui::FontId::monospace(10.0), color) + 14.0
        }
        None => 26.0,
    };
    let text_x = 50.0;
    let text_width = (card_w - text_x - 14.0 - right_reserve).max(24.0);

    let title_text = text::block(
        &painter,
        title,
        egui::FontId::proportional(TITLE),
        egui::Color32::WHITE,
        text_width,
        2,
    );
    let subtitle_text = text::line(
        &painter,
        subtitle,
        egui::FontId::proportional(SUBTITLE),
        TEXT_MUTED,
        text_width,
    );
    let height = (10.0 + title_text.height() + 3.0 + subtitle_text.height() + 8.0).max(52.0);

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(card_w, height), egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

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
    text::paint(
        &painter,
        egui::pos2(rect.left() + text_x, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        &title_text,
        egui::Color32::WHITE,
    );
    text::paint(
        &painter,
        egui::pos2(
            rect.left() + text_x,
            rect.top() + 13.0 + title_text.height(),
        ),
        egui::Align2::LEFT_TOP,
        &subtitle_text,
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

    resp.on_hover_text(format!("{title}\n{subtitle}")).clicked()
}

/// Custom-painted Multi-Line Mode Selector Card (for Power Profiles, Battery Caps & Audio Modes).
///
/// Heading, `● ACTIVE`/`SELECT` and the spec bullets are laid out to fit; a long heading
/// takes a second row and the card grows, while a spec bullet that cannot fit is shortened with
/// the full sentence in the tooltip. `specs` may hold any number of bullets; empty ones are
/// skipped and the card is only as tall as the rest.
pub fn mode_selector_card(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    specs: &[&str],
    accent: egui::Color32,
    is_active: bool,
) -> bool {
    let card_w = clamp_w(ui, ui.available_width());
    let painter = ui.painter().clone();
    let pulse = pulse_factor(ui.ctx());

    let heading = format!("{icon} {title}");
    let status_lbl = if is_active { "● ACTIVE" } else { "SELECT" };
    let status_font = egui::FontId::monospace(SMALL);
    // Width does not depend on the colour, and hover is only known after the allocate below.
    let status_width = text::measure(&painter, status_lbl, status_font.clone(), TEXT_DIM);

    let heading_text = text::block(
        &painter,
        &heading,
        egui::FontId::proportional(TITLE),
        egui::Color32::WHITE,
        (card_w - 12.0 - status_width - 14.0).max(24.0),
        2,
    );
    let heading_rows = heading_text.galley.rows.len().max(1) as f32;
    let divider_y = 33.0 + (heading_rows - 1.0) * 16.0;

    let spec_texts: Vec<text::Text> = specs
        .iter()
        .filter(|spec| !spec.is_empty())
        .map(|spec| {
            text::line(
                &painter,
                spec,
                egui::FontId::proportional(SUBTITLE),
                if is_active { TEXT_MAIN } else { TEXT_MUTED },
                (card_w - 24.0).max(24.0),
            )
        })
        .collect();
    let spec_rows = spec_texts.len().saturating_sub(1) as f32;
    let height = divider_y + 13.0 + spec_rows * 16.0 + 14.0;

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(card_w, height), egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let status_col = if is_active {
        accent
    } else if hovered {
        TEXT_MAIN
    } else {
        TEXT_DIM
    };

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
        if is_active || hovered {
            accent
        } else {
            BORDER_SUBTLE
        },
    );

    // Header: icon + title, with the active/spec badge held to the right edge.
    text::paint(
        &painter,
        egui::pos2(rect.left() + 12.0, rect.top() + 19.0),
        egui::Align2::LEFT_CENTER,
        &heading_text,
        egui::Color32::WHITE,
    );
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 19.0),
        egui::Align2::RIGHT_CENTER,
        status_lbl,
        status_font,
        status_col,
    );

    // Divider line
    painter.line_segment(
        [
            egui::pos2(rect.left() + 10.0, rect.top() + divider_y),
            egui::pos2(rect.right() - 10.0, rect.top() + divider_y),
        ],
        egui::Stroke::new(1.0, BORDER_SUBTLE),
    );

    // 3 spec bullet points so the user clearly understands what this mode does
    for (row, spec) in spec_texts.iter().enumerate() {
        text::paint(
            &painter,
            egui::pos2(
                rect.left() + 12.0,
                rect.top() + divider_y + 13.0 + row as f32 * 16.0,
            ),
            egui::Align2::LEFT_CENTER,
            spec,
            if is_active { TEXT_MAIN } else { TEXT_MUTED },
        );
    }

    resp.on_hover_text(format!(
        "{heading}\n{}\n{}\n{}",
        specs[0], specs[1], specs[2]
    ))
    .clicked()
}

/// Custom-painted Interactive Toggle Card for Cleaner Modules with checkmark badge & plain
/// description.
///
/// The description is a sentence, so it may end in `…` — the tooltip then carries the whole
/// module name and its sentence. The right-hand tag is measured first, so the sentence is never
/// painted under it.
pub fn module_toggle_card(
    ui: &mut egui::Ui,
    checked: bool,
    title: &str,
    description: &str,
    category: &str,
    requires_root: bool,
) -> bool {
    let desired = egui::vec2(clamp_w(ui, ui.available_width()), 46.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();

    let bg = if checked {
        BG_CARD_CHECKED
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
    painter.rect_filled(chk_rect, 4.0, if checked { GREEN_OK } else { BG_VOID });
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

    // Right badge: ADMIN vs the module's own category. Measured first, reserved from the text.
    let (tag_txt, tag_col) = if requires_root {
        ("🔑 ADMIN", AMBER_WARN)
    } else {
        (category, CYAN_INFO)
    };
    let tag_font = egui::FontId::monospace(8.5);
    let tag_w = text::measure(painter, tag_txt, tag_font.clone(), tag_col);

    // Module name & plain-English explanation
    let text_x = rect.left() + 38.0;
    let text_width = (rect.right() - 10.0 - tag_w - 8.0 - text_x).max(24.0);
    let title_text = text::line(
        painter,
        title,
        egui::FontId::proportional(11.5),
        if checked { TEXT_MAIN } else { TEXT_MUTED },
        text_width,
    );
    text::paint(
        painter,
        egui::pos2(text_x, rect.top() + 15.0),
        egui::Align2::LEFT_CENTER,
        &title_text,
        if checked { TEXT_MAIN } else { TEXT_MUTED },
    );

    let description_text = text::line(
        painter,
        description,
        egui::FontId::proportional(SMALL),
        TEXT_DIM,
        text_width,
    );
    text::paint(
        painter,
        egui::pos2(text_x, rect.top() + 32.0),
        egui::Align2::LEFT_CENTER,
        &description_text,
        TEXT_DIM,
    );

    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 15.0),
        egui::Align2::RIGHT_CENTER,
        tag_txt,
        tag_font,
        tag_col,
    );

    resp.on_hover_text(format!("{title} — {description}"))
        .clicked()
}
