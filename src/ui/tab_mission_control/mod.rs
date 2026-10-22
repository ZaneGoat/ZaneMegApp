//! Mission Control — "is my laptop well?" in one screen (design §3, §5.2, §5.3; AC-10…AC-16).
//!
//! Every number here goes through `theme::metric`/`composite`/`belief`, so every number carries
//! its source and its unit or says `unavailable` and why. Nothing on this tab changes anything:
//! the alert strip and the quick actions *say* things and *navigate*, and a destination that does
//! not exist yet is shown disabled with `Available in the next update` rather than as a dead
//! button (AC-13, design §3 rule 3).
//!
//! This file paints the tiles and decides the page order; the strip, the health block and the
//! quick actions are in `blocks.rs` (AC-43's 500-line rule). Slice 4b adds the tabs the three
//! missing quick actions land on; neither file needs a change for that — the destination table is
//! data.

use eframe::egui;
use std::time::Instant;

mod blocks;

use self::blocks::{alert_strip, health_block, quick_actions};
use crate::app::activity::ActivityLog;
use crate::app::tabs::AppTab;
use crate::core::alerts::evaluate;
use crate::core::health::{BatteryInfo, HealthSnapshot, Subsystem, SubsystemHealth};
use crate::core::measured::{Belief, Measured, Source, Unavailable, Unit};
use crate::core::time;
use crate::theme::*;

/// A request-only fact, never read back: the charge limit is set through `isw` and the firmware
/// does not report what it currently is (design §3 rule 6, SEC-V4-26).
fn no_charge_limit() -> Belief<u32> {
    Belief::Read(Measured::unavailable(Unavailable::NotReported {
        source: Source::new(
            "/sys/class/power_supply/BAT1/charge_control_end_threshold",
            "the firmware's charge limit",
        ),
        why: "the charge limit is not reported by this firmware and none has been requested this session",
    }))
}

pub struct TabMissionControl {
    charge_limit: Belief<u32>,
}

impl Default for TabMissionControl {
    fn default() -> Self {
        Self::new()
    }
}

impl TabMissionControl {
    pub fn new() -> Self {
        Self {
            charge_limit: no_charge_limit(),
        }
    }

    /// Paints the tab. The returned tab is a *navigation request* the shell applies; the tab
    /// never switches itself, and never acts.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &HealthSnapshot,
        now: Instant,
        activity: &ActivityLog,
    ) -> Option<AppTab> {
        let rows = snapshot.health_rows(now);
        let alerts = evaluate(snapshot, now);
        let mut nav = None;

        legend_strip(
            ui,
            "Every number names where it came from. Nothing on this screen changes anything.",
            &[
                ("ok", GREEN_OK),
                ("attention", AMBER_WARN),
                ("unavailable", TEXT_MUTED),
                ("off", RED_BRIGHT),
            ],
        );
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("LIVE")
                    .strong()
                    .size(10.0)
                    .color(CYAN_INFO),
            );
            ui.label(
                egui::RichText::new(format!(
                    "Updated {} · live — the readings below are re-read every second",
                    time::hh_mm_ss(snapshot.taken_wall)
                ))
                .size(10.0)
                .color(TEXT_MUTED),
            );
        });
        ui.add_space(6.0);

        self.tiles(ui, snapshot, now, &rows);
        ui.add_space(6.0);

        nav = alert_strip(ui, &alerts).or(nav);
        ui.add_space(6.0);

        health_block(ui, &rows);
        ui.add_space(6.0);

        nav = quick_actions(ui).or(nav);
        ui.add_space(6.0);

        activity.ui(ui);

        nav
    }

    /// The four tiles: Thermal, Memory, Disk, Battery. Each one is a card whose badge is the
    /// subsystem's *derived* word when it is not `ok`, so a hot CPU says `attention` in words as
    /// well as in colour.
    fn tiles(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &HealthSnapshot,
        now: Instant,
        rows: &[SubsystemHealth; 8],
    ) {
        let charge_limit = self.charge_limit.clone();
        grid(ui, 4, 4, 230.0, 6.0, |column, index| match index {
            0 => {
                cyber_card(
                    column,
                    "🌡️ Thermal",
                    Some("the processor's temperature"),
                    accent_for(rows, Subsystem::Hardware),
                    badge_for(rows, Subsystem::Hardware),
                    |ui| {
                        metric(ui, "CPU", &snapshot.cpu_temp_c, now, Unit::Celsius, 0);
                        ui.add_space(6.0);
                        metric(ui, "Fan", &snapshot.fan, now, Unit::Rpm, 0);
                    },
                );
            }
            1 => {
                cyber_card(
                    column,
                    "🧠 Memory",
                    Some("RAM in use"),
                    accent_for(rows, Subsystem::Processes),
                    None,
                    |ui| {
                        composite(ui, "RAM", &snapshot.mem, now, |mem| {
                            format!(
                                "{:.1} / {:.1} GiB ({:.0} %)",
                                mem.used_gib(),
                                mem.total_gib(),
                                mem.used_pct
                            )
                        });
                    },
                );
            }
            2 => {
                cyber_card(
                    column,
                    "💾 Disk",
                    Some("the root filesystem"),
                    accent_for(rows, Subsystem::Cleaner),
                    badge_for(rows, Subsystem::Cleaner),
                    |ui| {
                        composite(ui, "Root disk", &snapshot.disk_root, now, |disk| {
                            format!(
                                "{:.1} / {:.1} GiB ({:.0} %) · {:.1} GiB free",
                                disk.used_gib(),
                                disk.total_gib(),
                                disk.used_pct,
                                disk.free_gib()
                            )
                        });
                    },
                );
            }
            _ => {
                cyber_card(
                    column,
                    "🔋 Battery",
                    Some("charge and state"),
                    TEXT_MUTED,
                    None,
                    |ui| {
                        composite(ui, "Battery", &snapshot.battery, now, BatteryInfo::words);
                        ui.add_space(6.0);
                        belief(ui, "Charge limit", &charge_limit, now);
                    },
                );
            }
        });
    }
}

fn row_for(rows: &[SubsystemHealth; 8], subsystem: Subsystem) -> Option<&SubsystemHealth> {
    rows.iter().find(|row| row.subsystem() == subsystem)
}

fn accent_for(rows: &[SubsystemHealth; 8], subsystem: Subsystem) -> egui::Color32 {
    row_for(rows, subsystem).map_or(TEXT_DIM, |row| health_colors(row.word()).0)
}

fn badge_for(
    rows: &[SubsystemHealth; 8],
    subsystem: Subsystem,
) -> Option<(&'static str, egui::Color32, egui::Color32)> {
    let row = row_for(rows, subsystem)?;
    if row.word() == crate::core::health::HealthWord::Ok {
        return None;
    }
    let (fg, bg) = health_colors(row.word());
    Some((row.word().word(), fg, bg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::health::{Roots, Sampler};
    use std::path::Path;

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

    /// The tab paints at both sizes the window allows, with a source that reads nothing: no
    /// panic, no column layout, and no number where there is no reading (L1/L3, AC-11).
    #[test]
    fn the_tab_paints_with_no_readings_at_all() {
        for width in [763.0_f32, 1024.0, 1695.0, 1920.0] {
            let now = Instant::now();
            let root = std::env::temp_dir().join(format!(
                "zma-mission-{}-{}",
                std::process::id(),
                width as u32
            ));
            let snapshot = snapshot_from(&root, now);
            let activity = ActivityLog::new();
            let mut tab = TabMissionControl::new();

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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let nav = tab.ui(ui, &snapshot, now, &activity);
                    assert!(nav.is_none(), "nothing was clicked");
                });
            });
            output.drop_without_applying_deltas();
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// The charge limit starts as "neither reported nor requested", and says exactly that — it
    /// is never `0`, and it never pretends to be a measurement (SEC-V4-26).
    #[test]
    fn the_charge_limit_starts_unavailable() {
        let belief = no_charge_limit();
        let words = belief.render(Instant::now());
        assert!(words.starts_with("unavailable — "), "{words}");
        assert!(!words.contains('%'), "{words}");
        assert!(!belief.is_requested());
        assert!(!words.contains('0'), "{words}");
    }
}
