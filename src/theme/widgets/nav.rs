//! The sidebar entry: shell chrome, painted by hand so the active state is legible without colour
//! (a lit left bar, a brighter border and white text), plus its own live badge.

use eframe::egui;

use super::text;
use crate::theme::*;

/// Custom-painted Left Sidebar Navigation Card Button with icon emblem, title, subtitle, hotkey,
/// and live badge.
///
/// The title and subtitle are laid out to the width that is really free once the hotkey and badge
/// have been reserved, so neither can be painted under them; the full wording is the tooltip.
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
        painter.rect_filled(
            bar_rect,
            2.0,
            if is_active { RED_BRIGHT } else { RED_PRIMARY },
        );
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

    // Title & Subtitle, measured against the space the hotkey badge leaves.
    let hotkey_font = egui::FontId::monospace(9.0);
    let badge_font = egui::FontId::monospace(9.0);
    let mut reserved = text::measure(painter, hotkey, hotkey_font.clone(), TEXT_DIM);
    if let Some((badge_text, badge_col)) = badge {
        reserved = reserved.max(text::measure(
            painter,
            badge_text,
            badge_font.clone(),
            badge_col,
        ));
    }
    let text_x = rect.left() + 48.0;
    let text_width = (rect.right() - 12.0 - reserved - 8.0 - text_x).max(24.0);

    let title_color = if is_active || hovered {
        egui::Color32::WHITE
    } else {
        TEXT_MAIN
    };
    let title_text = text::line(
        painter,
        title,
        egui::FontId::proportional(12.0),
        title_color,
        text_width,
    );
    let subtitle_text = text::line(
        painter,
        subtitle,
        egui::FontId::proportional(9.5),
        if is_active { RED_BRIGHT } else { TEXT_MUTED },
        text_width,
    );
    text::paint(
        painter,
        egui::pos2(text_x, rect.top() + 17.0),
        egui::Align2::LEFT_CENTER,
        &title_text,
        title_color,
    );
    text::paint(
        painter,
        egui::pos2(text_x, rect.top() + 36.0),
        egui::Align2::LEFT_CENTER,
        &subtitle_text,
        if is_active { RED_BRIGHT } else { TEXT_MUTED },
    );

    // Right-side hotkey badge + optional status pill
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 15.0),
        egui::Align2::RIGHT_CENTER,
        hotkey,
        hotkey_font,
        TEXT_DIM,
    );

    if let Some((b_txt, b_col)) = badge {
        painter.text(
            egui::pos2(rect.right() - 10.0, rect.top() + 36.0),
            egui::Align2::RIGHT_CENTER,
            b_txt,
            badge_font,
            b_col,
        );
    }

    resp.on_hover_text(format!("{title} — {subtitle}"))
        .clicked()
}
