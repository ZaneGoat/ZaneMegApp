//! The only two functions that paint a value (design §5.1, §5.6, SEC-V4-25).
//!
//! [`metric`] paints a [`Measured`]; [`belief`] paints a [`Belief`]. There is deliberately no
//! third function and no `&f32` parameter anywhere: a number reaches the screen only through
//! [`Measured::render`], which returns `None` for a stale or unavailable reading — so a failed
//! source cannot print `0`, `NaN` or a blank cell, and a *requested* value cannot print a unit of
//! measurement and be mistaken for one.
//!
//! The words a cell can show are decided in [`metric_text`], which is pure and unit-tested:
//! `75 °C`, `stale — last read 09:41:07`, or `unavailable — permission denied on /proc/meminfo`.

use eframe::egui;

use crate::core::measured::{Belief, Measured, Unit};
use crate::theme::{AMBER_WARN, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};

/// What a value cell says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetricText {
    /// `"75 °C"` — live, with the unit.
    Number(String),
    /// `"stale — last read 09:41:07"` — the number is not shown at all.
    Stale(String),
    /// `"unavailable — <reason>"` — never a zero, never blank.
    Unavailable(String),
}

impl MetricText {
    pub fn words(&self) -> &str {
        match self {
            Self::Number(text) | Self::Stale(text) | Self::Unavailable(text) => text,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
}

/// The pure half of [`metric`]: the exact words a cell will paint, for a given reading and moment.
pub fn metric_text<T: Copy + Into<f64>>(
    measured: &Measured<T>,
    now: std::time::Instant,
    unit: Unit,
    decimals: u8,
) -> MetricText {
    if let Some(observed) = measured.observed(now) {
        let number: f64 = (*observed.value).into();
        if !number.is_finite() {
            return MetricText::Unavailable(
                "unavailable — the source returned a value that is not a number".to_string(),
            );
        }
        if let Some(text) = measured.render(now, unit, decimals) {
            return MetricText::Number(text);
        }
    }
    if let Some(note) = measured.stale_note(now) {
        return MetricText::Stale(note);
    }
    let reason = measured
        .reason()
        .map_or_else(|| "not read yet".to_string(), |reason| reason.human());
    MetricText::Unavailable(format!("unavailable — {reason}"))
}

/// Paints a label and its value. The source and the age are the value's hover text, so the tile
/// stays one number high while the provenance is always one hover away (§3 rule 1).
pub fn metric<T: Copy + Into<f64>>(
    ui: &mut egui::Ui,
    label: &str,
    measured: &Measured<T>,
    now: std::time::Instant,
    unit: Unit,
    decimals: u8,
) {
    let text = metric_text(measured, now, unit, decimals);
    paint(ui, label, &text, hover_text(measured, now));
}

/// A reading that is several numbers at once — memory used/total, disk used/free — described by a
/// formatter that is only ever called with a *live* value from [`Measured::observed`]. The three
/// states and their words are [`metric_text`]'s, so a failed read still says `unavailable — …`.
pub fn composite<T>(
    ui: &mut egui::Ui,
    label: &str,
    measured: &Measured<T>,
    now: std::time::Instant,
    describe: impl FnOnce(&T) -> String,
) {
    let text = composite_text(measured, now, describe);
    paint(ui, label, &text, hover_text(measured, now));
}

/// The pure half of [`composite`]: the words a value that is several numbers at once will paint.
///
/// It exists so chrome that is not a [`metric`] cell — the ribbon's pills — still prints the same
/// words as the tile for the same reading, instead of growing a second formatter that could
/// disagree with it.
pub fn composite_text<T>(
    measured: &Measured<T>,
    now: std::time::Instant,
    describe: impl FnOnce(&T) -> String,
) -> MetricText {
    if let Some(observed) = measured.observed(now) {
        MetricText::Number(describe(observed.value))
    } else if let Some(note) = measured.stale_note(now) {
        MetricText::Stale(note)
    } else {
        MetricText::Unavailable(format!(
            "unavailable — {}",
            measured
                .reason()
                .map_or_else(|| "not read yet".to_string(), |reason| reason.human())
        ))
    }
}

fn paint(ui: &mut egui::Ui, label: &str, text: &MetricText, hover: String) {
    ui.label(egui::RichText::new(label).size(10.0).color(TEXT_DIM));
    let (colour, size) = match text {
        MetricText::Number(_) => (TEXT_MAIN, 20.0),
        MetricText::Stale(_) => (AMBER_WARN, 11.5),
        MetricText::Unavailable(_) => (TEXT_MUTED, 11.5),
    };
    ui.add(egui::Label::new(egui::RichText::new(text.words()).size(size).color(colour)).wrap())
        .on_hover_text(hover);
}

/// Paints a *requested* value — always in the belief's own words, never with a unit, and by a
/// different function from [`metric`] so the two can never be confused (SEC-V4-26).
pub fn belief<T: Copy + Into<f64>>(
    ui: &mut egui::Ui,
    label: &str,
    value: &Belief<T>,
    now: std::time::Instant,
) {
    ui.label(egui::RichText::new(label).size(10.0).color(TEXT_DIM));
    let colour = if value.is_requested() {
        AMBER_WARN
    } else {
        TEXT_MUTED
    };
    ui.add(
        egui::Label::new(
            egui::RichText::new(value.render(now))
                .size(11.0)
                .color(colour),
        )
        .wrap(),
    )
    .on_hover_text(if value.is_requested() {
        "A request is not a measurement: the app has not read this back, so it cannot tell you what the machine is doing."
    } else {
        "This value was read back from the platform."
    });
}

/// The provenance and age of a reading, for a cell's hover text: `"/proc/meminfo — … · 0.4 s old"`.
pub fn hover_text<T>(measured: &Measured<T>, now: std::time::Instant) -> String {
    let provenance = measured
        .provenance()
        .unwrap_or_else(|| "no source named".to_string());
    match measured.status(now) {
        crate::core::measured::ValueStatus::Live => {
            let age = measured.age(now).map_or(0.0, |age| age.as_secs_f32());
            format!("{provenance} · {age:.1} s old")
        }
        crate::core::measured::ValueStatus::Stale => format!(
            "{provenance} · {}",
            measured.stale_note(now).unwrap_or_default()
        ),
        crate::core::measured::ValueStatus::Unavailable => measured.reason().map_or_else(
            || provenance.clone(),
            |reason| format!("{provenance} · {}", reason.human()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::measured::{Cadence, Source, Unavailable};
    use std::time::{Duration, Instant, SystemTime};

    const CADENCE: Duration = Duration::from_secs(1);

    fn reading(value: f32, read_at: Instant) -> Measured<f32> {
        Measured::read(
            value,
            Source::new("/sys/class/thermal", "thermal_zone0"),
            read_at,
            SystemTime::now(),
            Cadence::every(CADENCE),
        )
    }

    /// A live reading is the number and its unit — nothing else.
    #[test]
    fn a_live_cell_is_the_number_and_its_unit() {
        let now = Instant::now();
        let text = metric_text(&reading(75.0, now), now, Unit::Celsius, 0);
        assert_eq!(text, MetricText::Number("75 °C".to_string()));
        assert!(text.is_number());
    }

    /// The tile that matters most: an unreadable source says `unavailable` and why, and there is
    /// no `0`, no `NaN` and no empty string anywhere in it.
    #[test]
    fn an_unreadable_source_says_unavailable_and_why() {
        let now = Instant::now();
        for reason in [
            Unavailable::Denied {
                source: Source::new("/proc/meminfo", "MemTotal"),
            },
            Unavailable::Missing {
                source: Source::new("/sys/class/thermal", "thermal_zone0"),
            },
            Unavailable::NotReported {
                source: Source::new("/sys/class/hwmon", "no fan*_input"),
                why: "this firmware does not report fan speed",
            },
            Unavailable::TestMode,
        ] {
            let measured: Measured<f32> = Measured::unavailable(reason.clone());
            let text = metric_text(&measured, now, Unit::Celsius, 0);
            let words = text.words();
            assert!(words.starts_with("unavailable — "), "{words}");
            assert!(words.contains(&reason.human()), "{words}");
            assert!(!words.contains('0'), "{words}");
            assert!(!words.contains("NaN"), "{words}");
            assert!(!words.trim().is_empty(), "{words}");
            assert!(!text.is_number());
        }
    }

    /// A stale reading is not a number either: the cell says when it was last read.
    #[test]
    fn a_stale_cell_says_when_it_was_read() {
        let read_at = Instant::now();
        let measured = reading(75.0, read_at);
        let later = read_at + Duration::from_secs(4);
        let text = metric_text(&measured, later, Unit::Celsius, 0);
        match &text {
            MetricText::Stale(words) => {
                assert!(words.starts_with("stale — last read "), "{words}");
                assert!(!words.contains("75"), "a stale value must not be shown: {words}");
            }
            other => panic!("expected a stale word, got {other:?}"),
        }
    }

    /// A source that answers with a non-number does not paint `NaN`.
    #[test]
    fn a_non_finite_reading_is_not_a_number_on_screen() {
        let now = Instant::now();
        let text = metric_text(&reading(f32::NAN, now), now, Unit::Celsius, 0);
        assert!(!text.words().contains("NaN"), "{}", text.words());
        assert!(text.words().starts_with("unavailable — "), "{}", text.words());
    }

    /// A requested belief is painted by its own function and never reads as a measurement.
    #[test]
    fn a_belief_cell_is_worded_as_a_request() {
        let now = Instant::now();
        let requested: Belief<u32> = Belief::Requested {
            value: 80,
            at: SystemTime::now(),
            note: "the firmware does not confirm it",
        };
        let words = requested.render(now);
        assert!(words.contains("last requested"), "{words}");
        assert!(words.contains("not read back"), "{words}");
        assert!(!words.contains('%'), "{words}");
        assert!(!words.contains('°'), "{words}");
    }

    /// Both painters survive every width the window allows, with either kind of value.
    #[test]
    fn the_painters_survive_every_allowed_width() {
        for width in [220.0_f32, 763.0, 1920.0] {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 400.0),
                )),
                ..Default::default()
            };
            let now = Instant::now();
            let output = ctx.run_ui(input, |ui| {
                ui.set_max_width(width);
                let live = reading(75.0, now);
                metric(ui, "CPU", &live, now, Unit::Celsius, 0);
                let dead: Measured<f32> = Measured::unavailable(Unavailable::TestMode);
                metric(ui, "CPU", &dead, now, Unit::Celsius, 0);
                belief(
                    ui,
                    "Charge limit",
                    &Belief::Requested {
                        value: 80,
                        at: SystemTime::now(),
                        note: "not confirmed",
                    },
                    now,
                );
            });
            output.drop_without_applying_deltas();
        }
    }
}
