use eframe::egui;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::security::{NetInterface, SecurityManager};
use crate::theme::*;

pub struct TabSecurity {
    pub is_isolated: bool,
    pub interfaces: Vec<NetInterface>,
    pub listening_ports: Vec<u16>,
    pub tx: Sender<Result<String, String>>,
    pub rx: Receiver<Result<String, String>>,
}

impl TabSecurity {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            is_isolated: SecurityManager::is_isolation_active(),
            interfaces: SecurityManager::read_interfaces(),
            listening_ports: SecurityManager::read_listening_ports(),
            tx,
            rx,
        }
    }

    pub fn refresh_state(&mut self) {
        self.is_isolated = SecurityManager::is_isolation_active();
        self.interfaces = SecurityManager::read_interfaces();
        self.listening_ports = SecurityManager::read_listening_ports();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        while let Ok(res) = self.rx.try_recv() {
            match res {
                Ok(msg) => set_status(msg, GREEN_OK),
                Err(err) => set_status(err, RED_BRIGHT),
            }
        }

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

                ui.add_space(8.0);

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
                .min_size(egui::vec2(ui.available_width(), 42.0));

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

        ui.add_space(6.0);

        // 2. Dual Tactical Row: LHOST Network Matrix + Waydroid Bridge Ops
        ui.columns(2, |cols| {
            // Left: Pentest LHOST & Active Listeners Matrix
            cyber_card(
                &mut cols[0],
                "🎯 PENTEST LHOST & PORT MATRIX",
                Some("LIVE INTERFACES"),
                CYAN_INFO,
                Some((
                    &format!("{} IFACES", self.interfaces.len()),
                    CYAN_INFO,
                    CYAN_SUBTLE,
                )),
                |ui| {
                    for iface in &self.interfaces {
                        let is_up = iface.state == "UP";
                        egui::Frame::new()
                            .fill(BG_CARD)
                            .corner_radius(5)
                            .inner_margin(egui::Margin::symmetric(8, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let dot_color = if is_up { GREEN_OK } else { TEXT_DIM };
                                    ui.label(egui::RichText::new("●").size(10.0).color(dot_color));
                                    ui.label(
                                        egui::RichText::new(format!("{:<8}", iface.name))
                                            .monospace()
                                            .strong()
                                            .size(11.0)
                                            .color(TEXT_MAIN),
                                    );
                                    ui.label(
                                        egui::RichText::new(&iface.ip_cidr)
                                            .monospace()
                                            .size(10.5)
                                            .color(CYAN_INFO),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("📋 LHOST")
                                                        .strong()
                                                        .size(9.5)
                                                        .color(TEXT_MAIN),
                                                )
                                                .fill(BG_ELEVATED)
                                                .min_size(egui::vec2(64.0, 20.0)),
                                            )
                                            .on_hover_text(format!("Copy {} to clipboard", iface.clean_ip))
                                            .clicked()
                                        {
                                            ui.ctx().copy_text(iface.clean_ip.clone());
                                            set_status(
                                                format!("Copied LHOST IP {} ({})", iface.clean_ip, iface.name),
                                                GREEN_OK,
                                            );
                                        }
                                    });
                                });
                            });
                        ui.add_space(2.0);
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Active Listening TCP Ports
                    let ports_str = if self.listening_ports.is_empty() {
                        "None detected".to_string()
                    } else {
                        self.listening_ports
                            .iter()
                            .map(|p| format!(":{p}"))
                            .collect::<Vec<_>>()
                            .join("  ")
                    };

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Active TCP Listeners:")
                                .strong()
                                .size(10.0)
                                .color(TEXT_MUTED),
                        );
                        ui.label(
                            egui::RichText::new(ports_str)
                                .monospace()
                                .size(10.0)
                                .color(AMBER_WARN),
                        );
                    });
                },
            );

            // Right: Waydroid & Container Bridge Tactical Ops
            cyber_card(
                &mut cols[1],
                "🤖 WAYDROID & BRIDGE OPS",
                Some("DOCKER NAT FIX"),
                GREEN_OK,
                Some(("KALI WAYLAND", GREEN_OK, GREEN_SUBTLE)),
                |ui| {
                    ui.label(
                        egui::RichText::new(
                            "Resolves Docker FORWARD-DROP conflicts so Waydroid Android containers retain full internet & DNS access:",
                        )
                        .size(10.0)
                        .color(TEXT_MUTED),
                    );

                    ui.add_space(6.0);

                    let fix_btn = egui::Button::new(
                        egui::RichText::new("⚡ INJECT WAYDROID DOCKER/NAT BRIDGE (pkexec)")
                            .strong()
                            .size(11.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(RED_PRIMARY)
                    .min_size(egui::vec2(ui.available_width(), 34.0));

                    if ui.add(fix_btn).clicked() {
                        set_status("Injecting Waydroid DOCKER-USER & MASQUERADE rules...".into(), CYAN_INFO);
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(SecurityManager::inject_waydroid_bridge());
                        });
                    }

                    ui.add_space(6.0);

                    ui.columns(2, |w_cols| {
                        if w_cols[0]
                            .add_sized(
                                [w_cols[0].available_width(), 30.0],
                                egui::Button::new(egui::RichText::new("🚀 Start Waydroid UI").size(10.5)),
                            )
                            .clicked()
                        {
                            let _ = Command::new("waydroid").arg("show-full-ui").spawn();
                            set_status("Launched Waydroid Full UI".into(), GREEN_OK);
                        }

                        if w_cols[1]
                            .add_sized(
                                [w_cols[1].available_width(), 30.0],
                                egui::Button::new(egui::RichText::new("🛑 Stop Session").size(10.5)),
                            )
                            .clicked()
                        {
                            let _ = Command::new("waydroid").args(["session", "stop"]).spawn();
                            set_status("Stopped Waydroid session".into(), AMBER_WARN);
                        }
                    });
                },
            );
        });

        ui.add_space(6.0);

        // 3. Pentesting Operational Impact Advisory Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, AMBER_WARN.gamma_multiply(0.7)))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚠️ PENTESTING OPERATIONAL IMPACT ADVISORY")
                            .strong()
                            .size(11.5)
                            .color(AMBER_WARN),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_pill(ui, "REVERSIBLE TOGGLE", AMBER_WARN, AMBER_SUBTLE);
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "• Enabling Isolation enforces default-deny incoming, which blocks: Reverse shells (nc/pwncat), Metasploit multi/handler, Responder LLMNR/NBT-NS poisoners, and python3 -m http.server.",
                    )
                    .size(10.0)
                    .color(TEXT_MUTED),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("• Emergency CLI Revert:").strong().size(10.0).color(TEXT_MAIN));
                    ui.label(egui::RichText::new("pkexec ufw disable").monospace().size(10.0).color(CYAN_INFO));
                    if ui.small_button("📋 Copy").clicked() {
                        ui.ctx().copy_text("pkexec ufw disable".into());
                    }
                });
            });
    }
}
