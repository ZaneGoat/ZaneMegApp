//! Mission Control's three blocks: the alert strip, AC-14's health block, and the four
//! navigate-only quick actions (AC-12, AC-13, AC-14; design §3 rules 3 and 5).
//!
//! Split out of `tab_mission_control.rs` for AC-43's 500-line rule. Nothing here acts: the strip
//! and the actions return an [`AppTab`] for the shell to switch to, and a destination that does
//! not exist yet is disabled and says `Available in the next update`.

use eframe::egui;

use crate::app::tabs::AppTab;
use crate::core::alerts::Alert;
use crate::core::health::SubsystemHealth;
use crate::theme::*;

/// One of the four "take me there" controls. A `None` tab is a tab that does not exist yet, and
/// the control says so instead of pretending (AC-13).
struct QuickAction {
    label: &'static str,
    tab: Option<AppTab>,
}

const QUICK_ACTIONS: [QuickAction; 4] = [
    QuickAction {
        label: "Signal a process",
        tab: None,
    },
    QuickAction {
        label: "Restart a service",
        tab: None,
    },
    QuickAction {
        label: "Check the network",
        tab: None,
    },
    QuickAction {
        label: "Free memory",
        tab: Some(AppTab::Cleaner),
    },
];

/// The strip is alert-coloured only when something is actually wrong; otherwise it says
/// `All systems nominal` and nothing is tinted (AC-12).
pub(super) fn alert_strip(ui: &mut egui::Ui, alerts: &[Alert]) -> Option<AppTab> {
    let mut nav = None;
    let alerts_empty = alerts.is_empty();
    egui::Frame::new()
        .fill(if alerts_empty { BG_VOID } else { RED_SUBTLE })
        .stroke(egui::Stroke::new(
            1.0,
            if alerts_empty { BORDER_SUBTLE } else { RED_PRIMARY },
        ))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(12, 7))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if alerts_empty {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("✓")
                            .strong()
                            .size(12.0)
                            .color(GREEN_OK),
                    );
                    ui.label(
                        egui::RichText::new("All systems nominal")
                            .strong()
                            .size(11.5)
                            .color(GREEN_OK),
                    );
                    ui.label(
                        egui::RichText::new("— no documented rule is firing")
                            .size(10.0)
                            .color(TEXT_MUTED),
                    );
                });
                return;
            }
            for alert in alerts {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("⚠")
                            .strong()
                            .size(12.0)
                            .color(RED_BRIGHT),
                    );
                    ui.label(
                        egui::RichText::new(&alert.headline)
                            .strong()
                            .size(11.5)
                            .color(TEXT_MAIN),
                    );
                    ui.label(
                        egui::RichText::new(format!("— {}", alert.rule.sentence))
                            .size(10.0)
                            .color(TEXT_MUTED),
                    );
                    match alert.rule.destination.tab {
                        Some(tab) => {
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(format!(
                                        "Go to {}",
                                        alert.rule.destination.label
                                    ))
                                    .size(10.5),
                                ))
                                .on_hover_text(format!(
                                    "Switches to {} and changes nothing else.",
                                    alert.rule.destination.label
                                ))
                                .clicked()
                            {
                                nav = Some(tab);
                            }
                        }
                        None => {
                            ui.label(
                                egui::RichText::new("available in the next update")
                                    .size(10.0)
                                    .italics()
                                    .color(TEXT_DIM),
                            );
                        }
                    }
                });
            }
        });
    nav
}

/// AC-14's eight rows: `<subsystem> · <state> · <source>`, the state word painted as a word.
pub(super) fn health_block(ui: &mut egui::Ui, rows: &[SubsystemHealth; 8]) {
    cyber_card(
        ui,
        "🩺 Subsystem health",
        Some("one row per subsystem, and where each word came from"),
        CYAN_INFO,
        None,
        |ui| {
            for row in rows {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} ·", row.subsystem().title()))
                            .size(10.5)
                            .color(TEXT_MAIN),
                    );
                    let (fg, _) = health_colors(row.word());
                    ui.label(
                        egui::RichText::new(row.word().word())
                            .strong()
                            .size(10.5)
                            .color(fg),
                    );
                    ui.label(
                        egui::RichText::new(format!("· {}", row.source()))
                            .size(10.0)
                            .color(TEXT_DIM),
                    );
                    if let Some(note) = row.note() {
                        ui.label(
                            egui::RichText::new(format!("— {note}"))
                                .size(10.0)
                                .color(TEXT_MUTED),
                        );
                    }
                });
            }
        },
    );
}

/// The four actions only take her somewhere. A destination that does not exist yet is disabled
/// and says `Available in the next update` (AC-13, design §3 rule 3).
pub(super) fn quick_actions(ui: &mut egui::Ui) -> Option<AppTab> {
    let mut nav = None;
    cyber_card(
        ui,
        "⚡ Quick actions",
        Some("these take you to the control — they do nothing by themselves"),
        AMBER_WARN,
        None,
        |ui| {
            let items = QUICK_ACTIONS.len();
            grid(ui, items, 4, 200.0, 6.0, |column, index| {
                let action = &QUICK_ACTIONS[index];
                match action.tab {
                    Some(tab) => {
                        let button = egui::Button::new(
                            egui::RichText::new(action.label).size(11.0),
                        )
                        .min_size(egui::vec2(column.available_width(), 30.0));
                        if column
                            .add(button)
                            .on_hover_text(format!(
                                "Opens {}. Changes nothing.",
                                tab.title()
                            ))
                            .clicked()
                        {
                            nav = Some(tab);
                        }
                        column.label(
                            egui::RichText::new(format!("opens {}", tab.title()))
                                .size(9.5)
                                .color(TEXT_DIM),
                        );
                    }
                    None => {
                        let button = egui::Button::new(
                            egui::RichText::new(action.label).size(11.0),
                        )
                        .min_size(egui::vec2(clamp_w(column, column.available_width()), 30.0));
                        column.add_enabled(false, button);
                        column.label(
                            egui::RichText::new("Available in the next update")
                                .size(9.5)
                                .italics()
                                .color(TEXT_DIM),
                        );
                    }
                }
            });
        },
    );
    nav
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quick-action table only offers a door that exists, and `Free memory` is the one that
    /// does today (AC-13).
    #[test]
    fn the_quick_actions_point_only_where_there_is_a_tab() {
        for action in &QUICK_ACTIONS {
            if let Some(tab) = action.tab {
                assert!(AppTab::ALL.contains(&tab), "{} leads nowhere", action.label);
            }
        }
        let open: Vec<&str> = QUICK_ACTIONS
            .iter()
            .filter(|action| action.tab.is_some())
            .map(|action| action.label)
            .collect();
        assert_eq!(open, ["Free memory"]);
    }
}
