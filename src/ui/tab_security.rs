use eframe::egui;

use crate::core::security::SecurityManager;
use crate::theme::*;

pub struct TabSecurity {
    pub is_isolated: bool,
}

impl TabSecurity {
    pub fn new() -> Self {
        Self {
            is_isolated: SecurityManager::is_isolation_active(),
        }
    }

    pub fn refresh_state(&mut self) {
        self.is_isolated = SecurityManager::is_isolation_active();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        ui.add_space(6.0);

        // 1. Hero Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🛡️ NETWORK ISOLATION MATRIX (UFW FIREWALL)").strong().size(13.0).color(RED_PRIMARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (badge_text, badge_color) = if self.is_isolated {
                            ("● ISOLATION ACTIVE", RED_BRIGHT)
                        } else {
                            ("● ISOLATION INACTIVE (OPEN)", GREEN_OK)
                        };
                        ui.label(egui::RichText::new(badge_text).strong().size(12.0).color(badge_color));
                    });
                });
            });

        ui.add_space(8.0);

        // 2. Action Toggle Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("FIREWALL HARDENING CONTROLS").strong().size(12.0).color(TEXT_MAIN));
                ui.add_space(8.0);

                let btn_text = if self.is_isolated {
                    "🔓 DISABLE ISOLATION (Allow Inbound Traffic / Pentest Mode)"
                } else {
                    "🔒 ENABLE ISOLATION (Block Inbound Traffic / Strict Defense)"
                };

                let btn_color = if self.is_isolated { GREEN_OK } else { RED_PRIMARY };

                let toggle_btn = egui::Button::new(egui::RichText::new(btn_text).strong().size(12.5))
                    .fill(btn_color)
                    .min_size(egui::vec2(ui.available_width(), 44.0));

                if ui.add(toggle_btn).clicked() {
                    let target = !self.is_isolated;
                    match SecurityManager::toggle_isolation(target) {
                        Ok(new_state) => {
                            self.is_isolated = new_state;
                            set_status(format!("Network isolation {}", if new_state { "ENABLED" } else { "DISABLED" }), GREEN_OK);
                        }
                        Err(e) => {
                            set_status(e, RED_BRIGHT);
                        }
                    }
                }
            });

        ui.add_space(8.0);

        // 3. Pentest Security Advisory Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("⚠️ PENTESTING OPERATIONAL IMPACT NOTES").strong().size(11.5).color(RED_BRIGHT));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("• When Isolation is ENABLED: Default-deny incoming blocks inbound connections.\n  This WILL break: Reverse shell catchers (nc, netcat, pwncat), Metasploit multi/handler,\n  Responder LLMNR/NBT-NS poisoners, and local HTTP staging servers (python -m http.server).\n• Revert Command: pkexec ufw disable (or in shell: sudo ufw disable).").size(10.5).color(TEXT_MUTED));
            });
    }
}
