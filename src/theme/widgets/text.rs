//! Text laid out for the custom-painted widgets.
//!
//! `Painter::text` neither wraps nor clips: a long string is painted straight over whatever sits
//! next to it. That is how a card title ran under its status badge and how a metric row's two
//! numbers ended up on top of each other at 1024×640. Every string painted by a widget goes
//! through one of the two functions here instead:
//!
//! * [`line`] — one row, cut with `…` at the width the caller can spare. Only for prose (a
//!   description sentence) and chrome labels; the full text is the card's hover tooltip.
//! * [`block`] — wraps onto as many rows as the caller allows, so a title that does not fit gets
//!   taller instead of losing its last words.
//!
//! Neither ever paints outside `max_width`, and neither takes a width that has not been clamped.

use eframe::egui;
use std::sync::Arc;

/// A laid-out string, plus the height it occupies once painted.
pub(crate) struct Text {
    pub galley: Arc<egui::Galley>,
}

impl Text {
    /// The painted width. Only the tests ask for it: production always lays text out to a width
    /// it has already been given, and reads back the height it has to advance by.
    #[cfg(test)]
    pub fn width(&self) -> f32 {
        self.galley.size().x
    }

    pub fn height(&self) -> f32 {
        self.galley.size().y
    }
}

/// One row of `text`, shortened with `…` when it does not fit `max_width`.
pub(crate) fn line(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    max_width: f32,
) -> Text {
    Text {
        galley: layout(painter, text, font, color, max_width, 1, true),
    }
}

/// `text` wrapped over at most `max_rows` rows of `max_width`. Words are kept whole; only a row
/// that cannot hold the next word breaks inside it.
pub(crate) fn block(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    max_width: f32,
    max_rows: usize,
) -> Text {
    Text {
        galley: layout(painter, text, font, color, max_width, max_rows, false),
    }
}

/// Paints a laid-out [`Text`] so that `pos` sits at `align` within it.
pub(crate) fn paint(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &Text,
    color: egui::Color32,
) {
    let size = text.galley.size();
    let [ax, ay] = align.0;
    let factor = |align: egui::Align| match align {
        egui::Align::Min => 0.0,
        egui::Align::Center => 0.5,
        egui::Align::Max => 1.0,
    };
    painter.galley(
        egui::pos2(pos.x - factor(ax) * size.x, pos.y - factor(ay) * size.y),
        text.galley.clone(),
        color,
    );
}

/// The width `text` needs on one row, for callers that have to reserve space for it.
pub(crate) fn measure(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) -> f32 {
    painter
        .layout_no_wrap(text.to_owned(), font, color)
        .size()
        .x
}

fn layout(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    max_width: f32,
    max_rows: usize,
    cut_with_ellipsis: bool,
) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width: max_width.max(0.0),
        max_rows,
        break_anywhere: cut_with_ellipsis,
        overflow_character: Some('…'),
    };
    painter.layout_job(job)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painter() -> egui::Painter {
        let ctx = egui::Context::default();
        let mut painter = None;
        let input = egui::RawInput::default();
        let output = ctx.run_ui(input, |ui| {
            painter = Some(ui.painter().clone());
        });
        output.drop_without_applying_deltas();
        painter.expect("a painter from a ui")
    }

    /// A short string is painted whole; a long one is cut inside the width it was given and ends
    /// in `…` — it is never painted over the edge.
    ///
    /// `line` breaks anywhere, because that is the only way to honour `max_width` at all. Keeping
    /// numbers and state words whole is the caller's contract — `line` is only handed prose and
    /// chrome, and cards use [`block`] for anything that must not lose a token.
    #[test]
    fn a_line_never_exceeds_its_width() {
        let painter = painter();
        let font = egui::FontId::proportional(12.0);
        let short = line(&painter, "RESET", font.clone(), egui::Color32::WHITE, 200.0);
        assert!(short.width() <= 200.0);
        assert_eq!(short.galley.text(), "RESET");

        let long = line(
            &painter,
            "Toggle Cooler Boost (100% Max Fan Speed)",
            font,
            egui::Color32::WHITE,
            80.0,
        );
        assert!(long.width() <= 80.0, "painted {} px wide", long.width());
        let painted: String = long.galley.rows[0].glyphs.iter().map(|g| g.chr).collect();
        assert!(
            painted.ends_with('…'),
            "cut without a mark: {painted:?} (source {:?})",
            long.galley.text()
        );
    }

    /// A block grows downwards instead: the last word survives, the width does not.
    #[test]
    fn a_block_wraps_instead_of_cutting() {
        let painter = painter();
        let font = egui::FontId::proportional(12.0);
        let one_row = block(
            &painter,
            "RESET",
            font.clone(),
            egui::Color32::WHITE,
            200.0,
            2,
        );
        let two_rows = block(
            &painter,
            "Toggle Cooler Boost (100% Max Fan Speed)",
            font,
            egui::Color32::WHITE,
            120.0,
            2,
        );
        assert!(one_row.width() <= 200.0);
        assert!(
            two_rows.height() > one_row.height(),
            "wrapped text must take more height: {} vs {}",
            two_rows.height(),
            one_row.height()
        );
        assert!(two_rows.width() <= 120.0);
        assert!(
            !two_rows.galley.text().contains('…'),
            "a block with room left must not cut"
        );
    }
}
