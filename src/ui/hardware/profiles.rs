//! The power-profile chooser and the battery charge ceiling: what the machine is asked to do,
//! and how full it is allowed to charge.

use eframe::egui;
use std::thread;

use crate::core::hardware::HardwareManager;
use crate::theme::*;
use crate::ui::tab_hardware::TabHardware;

/// One row of the profile table: the wording, the accent and the request it makes.
struct ProfileCard {
    icon: &'static str,
    title: &'static str,
    specs: [&'static str; 3],
    accent: egui::Color32,
    id: &'static str,
    status: &'static str,
    cooler_boost: bool,
}

const PROFILES: [ProfileCard; 3] = [
    ProfileCard {
        icon: "🔥",
        title: "EXTREME GAMING",
        specs: [
            "🖥️ Screen: 144Hz Ultra Smooth",
            "🌀 Fans: 100% Cooler Boost",
            "⚡ Power: Maximum AC Speed",
        ],
        accent: RED_BRIGHT,
        id: "extreme",
        status: "Applying EXTREME Gaming Profile...",
        cooler_boost: true,
    },
    ProfileCard {
        icon: "⚖️",
        title: "BALANCED DAILY",
        specs: [
            "🖥️ Screen: 48Hz Power Saving",
            "🌀 Fans: Automatic Smart Curve",
            "⚡ Power: Balanced AC/Battery",
        ],
        accent: CYAN_INFO,
        id: "balance",
        status: "Applying BALANCED Daily Profile...",
        cooler_boost: false,
    },
    ProfileCard {
        icon: "🔋",
        title: "SUPER BATTERY",
        specs: [
            "🖥️ Screen: 48Hz Low Power",
            "🌀 Fans: Quiet / Silent Mode",
            "📡 Radio: Turns Bluetooth OFF",
        ],
        accent: GREEN_OK,
        id: "battery",
        status: "Applying SUPER BATTERY Profile...",
        cooler_boost: false,
    },
];

/// One row of the charge-ceiling table.
struct ChargeCap {
    icon: &'static str,
    title: &'static str,
    specs: [&'static str; 3],
    accent: egui::Color32,
    limit: u32,
}

const CHARGE_CAPS: [ChargeCap; 3] = [
    ChargeCap {
        icon: "🛡️",
        title: "60% MAX LIFE",
        specs: [
            "• Best if always plugged in",
            "• Minimizes cell voltage stress",
            "• Doubles battery lifespan",
        ],
        accent: GREEN_OK,
        limit: 60,
    },
    ChargeCap {
        icon: "⚖️",
        title: "80% BALANCED",
        specs: [
            "• Ideal for mixed daily use",
            "• Prevents 100% heat wear",
            "• Good portable runtime",
        ],
        accent: CYAN_INFO,
        limit: 80,
    },
    ChargeCap {
        icon: "⚡",
        title: "100% FULL TRIP",
        specs: [
            "• Charges battery to 100%",
            "• Maximum unplugged hours",
            "• Use before travel",
        ],
        accent: AMBER_WARN,
        limit: 100,
    },
];

/// The 1-click power profiles. A profile is a request to three subsystems, so the status line
/// says "Applying" until the worker reports back.
pub fn power_profiles(
    tab: &mut TabHardware,
    ui: &mut egui::Ui,
    set_status: &mut dyn FnMut(String, egui::Color32),
) {
    let active_badge = format!("CURRENT MODE: {}", tab.active_profile.to_uppercase());
    cyber_card(
        ui,
        "1-CLICK SYSTEM POWER PROFILES",
        Some("Choose how your display, fans, and battery behave"),
        RED_PRIMARY,
        Some((&active_badge, RED_BRIGHT, RED_SUBTLE)),
        |ui| {
            grid(ui, PROFILES.len(), 3, 190.0, 6.0, |column, index| {
                let profile = &PROFILES[index];
                let is_active = tab.active_profile == profile.id;
                if mode_selector_card(
                    column,
                    profile.icon,
                    profile.title,
                    &profile.specs,
                    profile.accent,
                    is_active,
                ) {
                    tab.active_profile = profile.id.to_string();
                    tab.cooler_boost = profile.cooler_boost;
                    set_status(profile.status.to_string(), CYAN_INFO);
                    let tx = tab.tx.clone();
                    let id = profile.id;
                    thread::spawn(move || {
                        let _ = tx.send(HardwareManager::apply_power_profile(id));
                    });
                }
            });
        },
    );
}

/// The battery lifespan card: three charge ceilings, each reversible by clicking another.
pub fn battery_ceiling(
    tab: &mut TabHardware,
    ui: &mut egui::Ui,
    set_status: &mut dyn FnMut(String, egui::Color32),
) {
    let cap_badge = format!("CHARGE CEILING: {}%", tab.battery_limit);
    cyber_card(
        ui,
        "BATTERY LIFESPAN PROTECTION",
        Some("Stop charging early to prevent battery wear"),
        GREEN_OK,
        Some((&cap_badge, GREEN_OK, GREEN_SUBTLE)),
        |ui| {
            // Three caps side by side need ~500 px; inside a half-width card the row drops to two
            // and then to one rather than squeezing the bullets off the cards.
            let mut clicked_limit = None;
            grid(ui, CHARGE_CAPS.len(), 3, 165.0, 6.0, |column, index| {
                let cap = &CHARGE_CAPS[index];
                if mode_selector_card(
                    column,
                    cap.icon,
                    cap.title,
                    &cap.specs,
                    cap.accent,
                    tab.battery_limit == cap.limit,
                ) {
                    clicked_limit = Some(cap.limit);
                }
            });

            if let Some(limit) = clicked_limit {
                tab.battery_limit = limit;
                set_status(
                    format!("Setting battery charge limit to {limit}%..."),
                    CYAN_INFO,
                );
                let tx = tab.tx.clone();
                thread::spawn(move || {
                    let _ = tx.send(HardwareManager::set_battery_limit(limit));
                });
            }

            ui.add_space(6.0);
            egui::Frame::new()
                .fill(BG_VOID)
                .corner_radius(6)
                .inner_margin(8)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(
                                "ℹ️ Tip: Keeping a lithium-ion battery at 60%–80% while on AC power prevents chemical degradation and swelling.",
                            )
                            .size(9.5)
                            .color(TEXT_MUTED),
                        )
                        .wrap(),
                    );
                });
        },
    );
}
