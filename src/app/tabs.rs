//! The shell's tab table — order, names, key hints and the subsystem each one answers to
//! (design §5.10 `app/tabs.rs`, AC-37, AC-10).
//!
//! This slice lists the seven tabs that exist: Mission Control, then the six from V2. AC-37's
//! full ten put Processes (F3), Services (F4) and Diagnostics (F5) between Hardware and Cleaner;
//! those arrive with their tabs in 4b, and are not listed here until they are real (design §8).

use eframe::egui;

use crate::core::health::Subsystem;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppTab {
    Mission,
    Hardware,
    Controller,
    Cleaner,
    Headset,
    Security,
    IDevice,
}

impl AppTab {
    /// AC-37's order, restricted to the tabs that exist. Slice 4b inserts the other three and
    /// renumbers from `Cleaner` on; nothing else has to change.
    pub const ALL: [Self; 7] = [
        Self::Mission,
        Self::Hardware,
        Self::Controller,
        Self::Cleaner,
        Self::Headset,
        Self::Security,
        Self::IDevice,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Mission => "Mission Control",
            Self::Hardware => "Hardware & Cooling",
            Self::Controller => "Controller Cockpit",
            Self::Cleaner => "System Cleaner",
            Self::Headset => "Headset Audio",
            Self::Security => "Network & Guard",
            Self::IDevice => "iOS Device Tools",
        }
    }

    /// One plain line, in her words rather than the app's.
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Mission => "Is my laptop well?",
            Self::Hardware => "Fans, temps & power modes",
            Self::Controller => "Gamepad, LEDs & haptics",
            Self::Cleaner => "Free RAM, cache & disk",
            Self::Headset => "Q3 Pro Bluetooth modes",
            Self::Security => "Firewall, local IPs & bridge",
            Self::IDevice => "USB inspector & tunnel",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Mission => "🎯",
            Self::Hardware => "⚡",
            Self::Controller => "🎮",
            Self::Cleaner => "🧹",
            Self::Headset => "🎧",
            Self::Security => "🛡️",
            Self::IDevice => "📱",
        }
    }

    /// AC-37's key for this tab; `None` for iDevice, which is reachable from the sidebar only.
    pub fn key(self) -> Option<egui::Key> {
        match self {
            Self::Mission => Some(egui::Key::F1),
            Self::Hardware => Some(egui::Key::F2),
            Self::Controller => Some(egui::Key::F3),
            Self::Cleaner => Some(egui::Key::F4),
            Self::Headset => Some(egui::Key::F5),
            Self::Security => Some(egui::Key::F6),
            Self::IDevice => None,
        }
    }

    /// The hint painted in the sidebar: `"F1"`… or `"—"` when the tab has no key.
    pub fn key_hint(self) -> &'static str {
        match self {
            Self::Mission => "F1",
            Self::Hardware => "F2",
            Self::Controller => "F3",
            Self::Cleaner => "F4",
            Self::Headset => "F5",
            Self::Security => "F6",
            Self::IDevice => "—",
        }
    }

    /// The subsystem whose health row is this tab's badge. `Headset` has no AC-14 row of its
    /// own, and `Mission` reports the alert count instead; both return `None` here.
    pub fn subsystem(self) -> Option<Subsystem> {
        match self {
            Self::Mission | Self::Headset => None,
            Self::Hardware => Some(Subsystem::Hardware),
            Self::Controller => Some(Subsystem::Controller),
            Self::Cleaner => Some(Subsystem::Cleaner),
            Self::Security => Some(Subsystem::Security),
            Self::IDevice => Some(Subsystem::IDevice),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-10/AC-37: Mission Control first, one key each for the six tabs that have one, and
    /// iDevice last with no key.
    #[test]
    fn the_sidebar_table_is_ordered_and_keyed() {
        assert_eq!(AppTab::ALL[0], AppTab::Mission);
        assert_eq!(AppTab::ALL[6], AppTab::IDevice);
        let keys: Vec<&str> = AppTab::ALL.iter().map(|tab| tab.key_hint()).collect();
        assert_eq!(keys, ["F1", "F2", "F3", "F4", "F5", "F6", "—"]);
        for tab in AppTab::ALL {
            assert!(!tab.title().is_empty());
            assert!(!tab.subtitle().is_empty());
            assert!(!tab.icon().is_empty());
        }
        // A key selects exactly one tab, and every keyed tab is reachable: the routing in
        // `app.rs` walks this same table, so there is no second key map to drift.
        for key in [
            egui::Key::F1,
            egui::Key::F2,
            egui::Key::F3,
            egui::Key::F4,
            egui::Key::F5,
            egui::Key::F6,
        ] {
            let matches: Vec<AppTab> = AppTab::ALL
                .into_iter()
                .filter(|tab| tab.key() == Some(key))
                .collect();
            assert_eq!(matches.len(), 1, "{key:?} selects {matches:?}");
        }
        assert_eq!(AppTab::ALL.into_iter().filter(|tab| tab.key().is_some()).count(), 6);
    }

    /// Every tab also has a stable identity, so nothing collides in the sidebar or in a test.
    #[test]
    fn no_two_tabs_share_a_name() {
        let mut titles: Vec<&str> = AppTab::ALL.iter().map(|tab| tab.title()).collect();
        let count = titles.len();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), count);
    }
}
