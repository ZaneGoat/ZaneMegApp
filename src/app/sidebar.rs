//! The shell's sidebar (design §6, AC-37, AC-10): one entry per tab that exists, in order, each
//! with its key hint, its own live badge, and an active marker that is not colour alone.
//!
//! The badges are the AC-14 health words, read straight out of the derived rows: `Hardware · ok`,
//! `Security · low` — never a hand-written "ONLINE"/"READY" over a value nobody read. Mission
//! Control's badge is the alert count, because that is what it is about (AC-12).
//!
//! The 4a slice lists seven entries. Processes, Services and Diagnostics are added in 4b
//! together with their tabs, so no entry here leads anywhere that does not exist (design §8).

use eframe::egui;

use crate::app::tabs::AppTab;
use crate::core::health::{HealthWord, SubsystemHealth};
use crate::theme::{BG_PANEL, BORDER_DARK, TEXT_DIM, health_colors, nav_sidebar_item};

/// Paints the sidebar and returns the tab she clicked, if any. The caller owns `current`, so
/// there is exactly one place a tab switch happens (design §6's shortcut routing).
pub fn sidebar(
    ui: &mut egui::Ui,
    current: AppTab,
    rows: &[SubsystemHealth; 8],
    alerts: usize,
) -> Option<AppTab> {
    let mut clicked = None;
    egui::Frame::new()
        .fill(BG_PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(8)
        .inner_margin(10)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("MODULES")
                        .strong()
                        .size(9.5)
                        .color(TEXT_DIM),
                );
                ui.add_space(4.0);
                for (index, tab) in AppTab::ALL.into_iter().enumerate() {
                    if index > 0 {
                        ui.add_space(4.0);
                    }
                    let badge = badge_for(tab, rows, alerts);
                    let badge_fg = badge
                        .as_ref()
                        .map(|(_, word)| health_colors(*word).0);
                    let badge_arg = badge
                        .as_ref()
                        .zip(badge_fg)
                        .map(|((text, _), fg)| (text.as_str(), fg));
                    if nav_sidebar_item(
                        ui,
                        tab.key_hint(),
                        tab.icon(),
                        tab.title(),
                        tab.subtitle(),
                        badge_arg,
                        tab == current,
                    ) {
                        clicked = Some(tab);
                    }
                }
            });
        });
    clicked
}

/// The live badge for one entry: its subsystem's state word, or the alert count for Mission
/// Control. `Headset` has no AC-14 row, so it carries no badge rather than an invented one.
pub fn badge_for(
    tab: AppTab,
    rows: &[SubsystemHealth; 8],
    alerts: usize,
) -> Option<(String, HealthWord)> {
    if tab == AppTab::Mission {
        return Some(if alerts == 0 {
            ("nominal".to_string(), HealthWord::Ok)
        } else if alerts == 1 {
            ("1 alert".to_string(), HealthWord::Attention)
        } else {
            (format!("{alerts} alerts"), HealthWord::Attention)
        });
    }
    let subsystem = tab.subsystem()?;
    let row = rows.iter().find(|row| row.subsystem() == subsystem)?;
    Some((row.word().word().to_string(), row.word()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::health::{HealthSnapshot, Roots, Sampler};
    use std::path::Path;
    use std::time::Instant;

    /// A sampler over a fixture tree, so no test reads this machine's temperature or battery.
    fn snapshot_from(root: &Path, now: Instant) -> HealthSnapshot {
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(root).expect("fixture root");
        let roots = Roots {
            proc_root: root.join("proc"),
            sys_root: root.join("sys"),
            etc_root: root.join("etc"),
            root_fs: root.to_path_buf(),
        };
        Sampler::new(roots, now).snapshot().clone()
    }

    /// Every sidebar entry gets the word its own subsystem's row carries — the badge cannot say
    /// `ok` while the row says `unavailable` (AC-14, SEC-V4-25).
    #[test]
    fn a_badge_is_the_rows_word() {
        let root = std::env::temp_dir().join(format!("zma-sidebar-{}", std::process::id()));
        let now = Instant::now();
        let snapshot = snapshot_from(&root, now);
        let rows = snapshot.health_rows(now);

        for tab in AppTab::ALL {
            let badge = badge_for(tab, &rows, 0);
            match tab.subsystem() {
                Some(subsystem) => {
                    let (_, word) = badge.expect("a subsystem tab has a badge");
                    let row = rows
                        .iter()
                        .find(|row| row.subsystem() == subsystem)
                        .expect("a row per subsystem");
                    assert_eq!(word, row.word(), "{} disagrees with its row", tab.title());
                }
                None => {
                    // Mission reports alerts; Headset has no row and no badge.
                    if tab == AppTab::Mission {
                        assert_eq!(badge.expect("mission badge").0, "nominal");
                    } else {
                        assert!(badge.is_none(), "{} invented a badge", tab.title());
                    }
                }
            }
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The alert count is on Mission Control's badge, and one alert is not "1 alerts".
    #[test]
    fn mission_counts_alerts_in_words() {
        let now = Instant::now();
        let root = std::env::temp_dir().join(format!("zma-sidebar-alerts-{}", std::process::id()));
        let snapshot = snapshot_from(&root, now);
        let rows = snapshot.health_rows(now);
        assert_eq!(badge_for(AppTab::Mission, &rows, 0).unwrap().0, "nominal");
        assert_eq!(badge_for(AppTab::Mission, &rows, 1).unwrap().0, "1 alert");
        assert_eq!(badge_for(AppTab::Mission, &rows, 3).unwrap().0, "3 alerts");
        let _ = std::fs::remove_dir_all(&root);
    }
}
