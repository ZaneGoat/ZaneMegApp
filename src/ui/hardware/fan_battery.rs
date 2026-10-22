//! The fan cooling card: Cooler Boost, the 80 °C thermostat, the MSI curve and BIOS auto.

use eframe::egui;
use std::thread;

use crate::core::hardware::HardwareManager;
use crate::theme::*;
use crate::ui::tab_hardware::TabHardware;

/// Fan cooling controls. Each button is a request to `isw`; the tab reports the worker's reply in
/// the status bar rather than claiming the fans moved.
pub fn fan_controls(
    tab: &mut TabHardware,
    ui: &mut egui::Ui,
    set_status: &mut dyn FnMut(String, egui::Color32),
) {
    let boost_badge = if tab.cooler_boost {
        ("MAX RPM ACTIVE", RED_BRIGHT, RED_SUBTLE)
    } else {
        ("AUTO COOLING", GREEN_OK, GREEN_SUBTLE)
    };

    cyber_card(
        ui,
        "FAN COOLING CONTROLS",
        Some("Override or automate laptop fans"),
        RED_PRIMARY,
        Some(boost_badge),
        |ui| {
            let boost_title = if tab.cooler_boost {
                "Cooler Boost is ON (100% Max Fan Speed)"
            } else {
                "Toggle Cooler Boost (100% Max Fan Speed)"
            };
            let boost_tag = if tab.cooler_boost {
                ("ON", RED_BRIGHT)
            } else {
                ("OFF", TEXT_MUTED)
            };

            if action_card_button(
                ui,
                "❄️",
                boost_title,
                "Forces both CPU and GPU fans to spin at maximum RPM immediately",
                Some(boost_tag),
                RED_BRIGHT,
                tab.cooler_boost,
            ) {
                tab.cooler_boost = !tab.cooler_boost;
                let target = tab.cooler_boost;
                set_status(
                    format!(
                        "Setting Cooler Boost {}...",
                        if target { "ON" } else { "OFF" }
                    ),
                    CYAN_INFO,
                );
                let tx = tab.tx.clone();
                thread::spawn(move || {
                    let _ = tx.send(HardwareManager::set_cooler_boost(target));
                });
            }

            ui.add_space(4.0);

            let smart_tag = if tab.smart_boost {
                ("ENABLED", GREEN_OK)
            } else {
                ("DISABLED", TEXT_DIM)
            };
            if action_card_button(
                ui,
                "🤖",
                "Auto Smart Thermostat (80°C Trigger)",
                "Automatically turns ON Cooler Boost if CPU or GPU reaches 80°C",
                Some(smart_tag),
                CYAN_INFO,
                tab.smart_boost,
            ) {
                tab.smart_boost = !tab.smart_boost;
                set_status(
                    format!(
                        "Smart Thermostat {}",
                        if tab.smart_boost {
                            "ENABLED"
                        } else {
                            "DISABLED"
                        }
                    ),
                    GREEN_OK,
                );
            }

            ui.add_space(4.0);

            // Two half-width buttons need ~190 px each for the full label; below that they go
            // one above the other instead of shortening both.
            grid(ui, 2, 2, 190.0, 4.0, |column, index| {
                if index == 0 {
                    if action_card_button(
                        column,
                        "📈",
                        "Custom Fan Curve",
                        "Load tuned MSI 16G41 table",
                        None,
                        RED_PRIMARY,
                        false,
                    ) {
                        set_status("Applying ISW custom curve (16G41)...".into(), CYAN_INFO);
                        let tx = tab.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(HardwareManager::apply_custom_fan_curve());
                        });
                    }
                } else if action_card_button(
                    column,
                    "🔄",
                    "Reset BIOS Auto",
                    "Return fans to default curve",
                    None,
                    GREEN_OK,
                    false,
                ) {
                    tab.cooler_boost = false;
                    set_status("Resetting fans to BIOS auto...".into(), CYAN_INFO);
                    let tx = tab.tx.clone();
                    thread::spawn(move || {
                        let _ = tx.send(HardwareManager::reset_fans_auto());
                    });
                }
            });
        },
    );
}
