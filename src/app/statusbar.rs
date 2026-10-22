//! The status bar: the app's state word and the tab she is on (design §6, AC-37).
//!
//! Design §6 lists four states — `Ready`, `Busy: <label>`, `Paused`, `TEST MODE`. `Busy` and
//! `Paused` need the job registry that lands in 4b, and `TEST MODE` needs 4a-2b's flag parsing;
//! so this bar has one honest state today, [`status_word`] is where the other three will hang,
//! and the unit test covers both branches it can reach.

use eframe::egui;

use crate::app::tabs::AppTab;
use crate::theme::{AMBER_WARN, GREEN_OK, TEXT_DIM, TEXT_MAIN};

/// `"Ready"`, or `"Busy: ping 192.168.1.1 (2 of 4 packets)"` — the label is the owning tab's
/// business, and it is empty only while nothing is running.
pub fn status_word(busy: Option<&str>) -> String {
    match busy {
        Some(label) if !label.is_empty() => format!("Busy: {label}"),
        _ => "Ready".to_string(),
    }
}

/// Paints the bar. `note` is the last thing a tab reported (`"cups stopped"`), and it is the only
/// thing that may be shortened — it is a report, not a number she has to read exactly; the full
/// sentence is its hover text.
pub fn status_bar(
    ui: &mut egui::Ui,
    tab: AppTab,
    busy: Option<&str>,
    note: Option<(&str, egui::Color32)>,
) {
    let busy = busy.filter(|label| !label.is_empty());
    let dot = if busy.is_some() {
        AMBER_WARN
    } else {
        GREEN_OK
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("●").size(11.0).color(dot));
        ui.label(
            egui::RichText::new(status_word(busy))
                .strong()
                .size(11.0)
                .color(TEXT_MAIN),
        );
        ui.label(egui::RichText::new("·").size(11.0).color(TEXT_DIM));
        ui.label(
            egui::RichText::new(tab.title())
                .size(11.0)
                .color(TEXT_MAIN),
        );
        if let Some((note, colour)) = note {
            ui.label(egui::RichText::new("·").size(11.0).color(TEXT_DIM));
            let room = ui.available_width().max(0.0);
            ui.scope(|ui| {
                ui.set_max_width(room);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(note)
                            .size(11.0)
                            .color(colour),
                    )
                    .truncate(),
                )
                .on_hover_text(note);
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two states the bar can be in today, and never an empty word.
    #[test]
    fn the_state_word_is_ready_or_busy() {
        assert_eq!(status_word(None), "Ready");
        assert_eq!(status_word(Some("")), "Ready", "an empty label is not busy");
        assert_eq!(
            status_word(Some("ping 192.168.1.1 (2 of 4 packets)")),
            "Busy: ping 192.168.1.1 (2 of 4 packets)"
        );
    }

    /// It paints at every width the window allows, with and without a note, without panicking
    /// and without a column layout (L1/L3).
    #[test]
    fn the_bar_paints_at_every_allowed_width() {
        for width in [1024.0_f32, 1920.0] {
            for note in [None, Some(("Cooler Boost set to ON", GREEN_OK))] {
                let ctx = egui::Context::default();
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 640.0),
                    )),
                    ..Default::default()
                };
                let output = ctx.run_ui(input, |ui| {
                    ui.set_max_width(width);
                    status_bar(ui, AppTab::Mission, Some("ping"), note);
                });
                output.drop_without_applying_deltas();
            }
        }
    }
}
