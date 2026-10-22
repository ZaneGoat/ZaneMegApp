//! "What this app has done — this session only" (AC-15, design §5.11).
//!
//! Rules the type keeps: only *completed* actions are recorded (a cancelled job never appears —
//! there is no `Outcome::Cancelled` to record), the list is in memory only and is cleared by
//! restart, it keeps the newest twenty, and a line's outcome is one of four words, never "done"
//! for something that was only requested (design §3 rule 6).
//!
//! Slice 4a defines the log and paints it; 4b feeds it from `CoreEvent::ActionFinished`, so the
//! only writer today is the unit test below (design §13.8: "4a defines, 4b uses").

use std::time::SystemTime;

use eframe::egui;

use crate::core::time;
use crate::theme::{BORDER_DARK, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};

/// What an action produced, in the words the list prints.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))] // Constructed by 4b's ActionFinished handler.
pub enum Outcome {
    /// Read back and it matched.
    Confirmed,
    /// Issued; the state was not read back.
    Requested,
    /// A forward action was reversed.
    Undone,
    /// It failed, with the reason.
    Failed(String),
}

impl Outcome {
    pub fn word(&self) -> String {
        match self {
            Self::Confirmed => "confirmed".to_string(),
            Self::Requested => "requested — not read back".to_string(),
            Self::Undone => "undone".to_string(),
            Self::Failed(reason) => format!("failed — {reason}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityEntry {
    pub at: SystemTime,
    /// What it did, in plain words: `"Sent SIGTERM to firefox (PID 4821)"`.
    pub text: String,
    pub outcome: Outcome,
}

/// Newest first, capped at twenty. In memory only: a restart is a clean sheet (AC-15).
#[derive(Clone, Debug, Default)]
pub struct ActivityLog {
    entries: Vec<ActivityEntry>,
}

impl ActivityLog {
    /// AC-15's twenty.
    pub const CAP: usize = 20;
    /// The words the empty list shows.
    pub const EMPTY: &'static str = "No activity this session.";

    pub fn new() -> Self {
        Self::default()
    }

    /// One completed action. Cancelled work is simply never pushed.
    #[cfg_attr(not(test), allow(dead_code))] // 4b: CoreEvent::ActionFinished.
    pub fn push(&mut self, entry: ActivityEntry) {
        self.entries.insert(0, entry);
        self.entries.truncate(Self::CAP);
    }

    pub fn entries(&self) -> &[ActivityEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `"09:41 · Sent SIGTERM to firefox (PID 4821) · confirmed"` for one line.
    pub fn line(entry: &ActivityEntry) -> String {
        format!(
            "{} · {} · {}",
            time::hh_mm(entry.at),
            entry.text,
            entry.outcome.word()
        )
    }

    /// The block, always inside a card so it reads as one of the tab's sections.
    pub fn ui(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(crate::theme::BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(14)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    egui::RichText::new("What this app has done — this session only")
                        .strong()
                        .size(12.0)
                        .color(TEXT_MAIN),
                );
                ui.add_space(4.0);
                if self.entries.is_empty() {
                    ui.label(
                        egui::RichText::new(Self::EMPTY)
                            .size(10.5)
                            .color(TEXT_DIM)
                            .italics(),
                    );
                    return;
                }
                for entry in &self.entries {
                    ui.label(
                        egui::RichText::new(Self::line(entry))
                            .size(10.5)
                            .color(TEXT_MUTED),
                    );
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn entry(seconds: u64, text: &str, outcome: Outcome) -> ActivityEntry {
        ActivityEntry {
            at: UNIX_EPOCH + Duration::from_secs(seconds),
            text: text.to_string(),
            outcome,
        }
    }

    /// Newest first, and never more than twenty (AC-15).
    #[test]
    fn the_list_keeps_twenty_newest_first() {
        let mut log = ActivityLog::new();
        assert!(log.is_empty());
        for index in 0..25_u64 {
            log.push(entry(
                1_700_000_000 + index,
                &format!("action {index}"),
                Outcome::Confirmed,
            ));
        }
        assert_eq!(log.entries().len(), ActivityLog::CAP);
        assert_eq!(log.entries()[0].text, "action 24", "newest first");
        assert_eq!(
            log.entries()[ActivityLog::CAP - 1].text,
            "action 5",
            "the five oldest fell off"
        );
    }

    /// A line is `HH:MM · action · outcome`, and the outcome words never claim a request was done.
    #[test]
    fn a_line_says_when_what_and_how_it_ended() {
        let confirmed = ActivityLog::line(&entry(1_700_000_000, "Stopped cups (system)", Outcome::Confirmed));
        assert_eq!(confirmed.split(" · ").count(), 3, "{confirmed}");
        assert!(confirmed.ends_with(" · confirmed"), "{confirmed}");
        assert_eq!(confirmed.as_bytes()[2], b':', "{confirmed}");

        let requested = Outcome::Requested.word();
        assert!(requested.contains("not read back"), "{requested}");
        assert_ne!(requested, "done");
        assert_eq!(Outcome::Undone.word(), "undone");
        assert!(Outcome::Failed("access denied".into()).word().starts_with("failed — "));
    }

    /// An empty list says so, in words rather than a blank box (AC-15).
    #[test]
    fn an_empty_list_has_its_own_sentence() {
        let log = ActivityLog::new();
        assert!(log.is_empty());
        assert_eq!(ActivityLog::EMPTY, "No activity this session.");
        assert!(!ActivityLog::EMPTY.contains('0'));
    }
}
