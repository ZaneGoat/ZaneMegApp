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
        ui.add_space(4.0);

        let badge = if self.is_isolated {
            ("🛡️ ISOLATION ACTIVE", RED_BRIGHT, RED_SUBTLE)
        } else {
            ("🔓 OPEN / PENTEST READY", GREEN_OK, GREEN_SUBTLE)
        };

        // 1. Hero Status Card
        cyber_card(
            ui,
            "🛡️ NETWORK ISOLATION MATRIX",
            Some("UFW HOST DEFENSE FIREWALL"),
            if self.is_isolated { RED_BRIGHT } else { GREEN_OK },
            Some(badge),
            |ui| {
                ui.label(
                    egui::RichText::new(
                        "Provides single-click network isolation by enforcing default-deny inbound packet filtering via UFW.",
                    )
                    .size(11.0)
                    .color(TEXT_MUTED),
                );

                ui.add_space(10.0);

                let btn_text = if self.is_isolated {
                    "🔓 DISABLE ISOLATION (Restore Open Inbound / Pentest Mode)"
                } else {
                    "🔒 ENGAGE ISOLATION (Block Inbound Traffic / Hardened Defense)"
                };
                let btn_fill = if self.is_isolated { GREEN_OK } else { RED_PRIMARY };

                let toggle_btn = egui::Button::new(
                    egui::RichText::new(btn_text)
                        .strong()
                        .size(12.5)
                        .color(egui::Color32::WHITE),
                )
                .fill(btn_fill)
                .min_size(egui::vec2(ui.available_width(), 44.0));

                if ui.add(toggle_btn).clicked() {
                    let target = !self.is_isolated;
                    match SecurityManager::toggle_isolation(target) {
                        Ok(new_state) => {
                            self.is_isolated = new_state;
                            set_status(
                                format!(
                                    "Network isolation {}",
                                    if new_state { "ENABLED" } else { "DISABLED" }
                                ),
                                if new_state { RED_BRIGHT } else { GREEN_OK },
                            );
                        }
                        Err(e) => {
                            set_status(e, RED_BRIGHT);
                        }
                    }
                }
            },
        );

        ui.add_space(8.0);

        // 2. Pentesting Operational Impact Advisory Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, AMBER_WARN.gamma_multiply(0.7)))
            .corner_radius(8)
            .inner_margin(14)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚠️ PENTESTING OPERATIONAL IMPACT ADVISORY")
                            .strong()
                            .size(12.0)
                            .color(AMBER_WARN),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_pill(ui, "CRITICAL ADVISORY", AMBER_WARN, AMBER_SUBTLE);
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new("Before enabling Network Isolation on this penetration testing rig, be aware that default-deny inbound traffic WILL disrupt the following activities:")
                        .size(11.0)
                        .color(TEXT_MAIN),
                );

                ui.add_space(4.0);

                let points = [
                    "Reverse shell catchers (nc, netcat, pwncat, socat listeners)",
                    "Metasploit Framework multi/handler inbound payloads (meterpreter)",
                    "Responder LLMNR / NBT-NS / MDNS poisoners and SMB credential traps",
                    "Local HTTP/FTP staging servers (python3 -m http.server, impacket)",
                    "Inbound SSH / ADB / Waydroid bridge connectivity",
                ];

                for p in points {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(" • ").color(RED_BRIGHT));
                        ui.label(egui::RichText::new(p).size(10.5).color(TEXT_MUTED));
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Manual Emergency Revert Command:").strong().size(10.5).color(TEXT_MAIN));
                    ui.label(egui::RichText::new("pkexec ufw disable").monospace().size(10.5).color(CYAN_INFO));
                    if ui.small_button("📋 Copy").clicked() {
                        ui.ctx().copy_text("pkexec ufw disable".into());
                    }
                });
            });
    }
}
