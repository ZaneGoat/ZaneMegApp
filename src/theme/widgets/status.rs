//! Small chrome: the pill that carries a state word, and the legend strip above every tab body.

use eframe::egui;

use super::text;
use crate::theme::{BG_VOID, BORDER_SUBTLE, CYAN_INFO, TEXT_MAIN, TEXT_MUTED};

const PILL_TEXT: f32 = 10.0;
const PILL_HEIGHT: f32 = 18.0;
const PILL_PADDING: f32 = 20.0;

/// Renders a pill-shaped status badge with a subtle background tint and crisp border.
///
/// The text is a state word (`OPEN`, `BLOCKED`, `79 °C`), so it is never cut and never wrapped:
/// the pill measures its word and allocates exactly that width, which lets the wrapping row it
/// sits in (a card header) see the true size and move it to the next line. Handing the same text
/// to a `Frame` left the row unable to measure it, so a header that was a word short of room
/// shrank the pill to a sliver and stacked `AUTO COOLING` one letter per line.
///
/// The response comes back so a caller can hang its own hover text on the pill (the ribbon names
/// the source of each number that way).
pub fn status_pill(ui: &mut egui::Ui, word: &str, fg: egui::Color32, bg: egui::Color32) -> egui::Response {
    let font = egui::FontId::proportional(PILL_TEXT);
    let painter = ui.painter().clone();

    let size = egui::vec2(status_pill_width(ui, word), PILL_HEIGHT);
    let radius = PILL_HEIGHT / 2.0;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

    painter.rect_filled(rect, radius, bg);
    painter.rect_stroke(
        rect,
        radius,
        egui::Stroke::new(1.0, fg.gamma_multiply(0.65)),
        egui::StrokeKind::Inside,
    );

    let label = text::line(&painter, word, font, fg, size.x);
    text::paint(
        &painter,
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &label,
        fg,
    );
    response
}

/// The width [`status_pill`] takes for `word`, so a caller can decide whether the pill fits
/// *before* painting it — the ribbon drops a pill it cannot fit instead of painting it off the
/// edge of the window.
pub fn status_pill_width(ui: &egui::Ui, word: &str) -> f32 {
    let font = egui::FontId::proportional(PILL_TEXT);
    text::measure(ui.painter(), word, font, TEXT_MAIN) + PILL_PADDING
}

/// Renders a visual colour-key guide banner so the user immediately understands the tab.
///
/// The hint is a full sentence and the keys follow it, so the row wraps: a right-to-left legend
/// block used to be painted on top of the sentence as soon as the window narrowed.
pub fn legend_strip(ui: &mut egui::Ui, summary_hint: &str, items: &[(&str, egui::Color32)]) {
    egui::Frame::new()
        .fill(BG_VOID)
        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("💡 GUIDE:")
                        .strong()
                        .size(10.0)
                        .color(CYAN_INFO),
                );
                ui.label(
                    egui::RichText::new(summary_hint)
                        .size(10.0)
                        .color(TEXT_MUTED),
                );

                for (label, color) in items {
                    ui.add_space(6.0);
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.8, *color);
                    ui.label(egui::RichText::new(*label).size(9.5).color(TEXT_MAIN));
                }
            });
        });
}
