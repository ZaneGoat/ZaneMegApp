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

        legend_strip(
            ui,
            "Manage UFW inbound firewall rules, inspect active local IP addresses, and fix Waydroid container internet.",
            &[
                ("Open / Standard", GREEN_OK),
                ("Isolated (Inbound Blocked)", RED_BRIGHT),
            ],
        );

        ui.add_space(6.0);

        let badge = if self.is_isolated {
            ("🛡️ ISOLATION ACTIVE", RED_BRIGHT, RED_SUBTLE)
        } else {
            ("🔓 OPEN / INBOUND ALLOWED", GREEN_OK, GREEN_SUBTLE)
        };

        // 1. Firewall Shield Status & Action Card
        cyber_card(
            ui,
            "🛡️ UFW FIREWALL & NETWORK ISOLATION",
            Some("Control whether external devices on the network can connect to this laptop"),
            if self.is_isolated { RED_BRIGHT } else { GREEN_OK },
            Some(badge),
            |ui| {
                let (icon, title, sub, tag, accent) = if self.is_isolated {
                    (
                        "🔓",
                        "Disable Network Isolation (Restore Open Inbound Traffic)",
                        "Allows incoming connections for local web servers, listeners, and file transfers",
                        ("CLICK TO OPEN", GREEN_OK),
                        GREEN_OK,
                    )
                } else {
                    (
                        "🔒",
                        "Enable Network Isolation (Block All Incoming Connections)",
                        "Hardens the laptop on untrusted networks by dropping unsolicited inbound packets",
                        ("CLICK TO LOCK", RED_BRIGHT),
                        RED_PRIMARY,
                    )
                };

                if action_card_button(ui, icon, title, sub, Some(tag), accent, true) {
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

        // 2. Dual Row: Network Interfaces & Waydroid Container Bridge
        ui.columns(2, |cols| {
            // Left: Active Network Adapters & Local IPs
            cyber_card(
                &mut cols[0],
                "🌐 ACTIVE NETWORK ADAPTERS & LOCAL IPs",
                Some("Click Copy IP to use an address in your tools"),
                CYAN_INFO,
                Some((
                    &format!("{} ADAPTERS", self.interfaces.len()),
                    CYAN_INFO,
                    CYAN_SUBTLE,
                )),
                |ui| {
                    for iface in &self.interfaces {
                        let is_up = iface.state == "UP";
                        egui::Frame::new()
                            .fill(BG_CARD)
                            .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                            .corner_radius(6)
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let dot_color = if is_up { GREEN_OK } else { TEXT_DIM };
                                    ui.label(egui::RichText::new("●").size(10.0).color(dot_color));
                                    ui.label(
                                        egui::RichText::new(format!("{:<8}", iface.name))
                                            .monospace()
                                            .strong()
                                            .size(11.5)
                                            .color(TEXT_MAIN),
                                    );
                                    ui.label(
                                        egui::RichText::new(&iface.ip_cidr)
                                            .monospace()
                                            .size(11.0)
                                            .color(CYAN_INFO),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("📋 Copy IP")
                                                        .strong()
                                                        .size(10.0)
                                                        .color(TEXT_MAIN),
                                                )
                                                .fill(BG_ELEVATED)
                                                .min_size(egui::vec2(74.0, 22.0)),
                                            )
                                            .on_hover_text(format!("Copy {} to clipboard", iface.clean_ip))
                                            .clicked()
                                        {
                                            ui.ctx().copy_text(iface.clean_ip.clone());
                                            set_status(
                                                format!("Copied IP {} ({})", iface.clean_ip, iface.name),
                                                GREEN_OK,
                                            );
                                        }
                                    });
                                });
                            });
                        ui.add_space(3.0);
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let ports_str = if self.listening_ports.is_empty() {
                        "None".to_string()
                    } else {
                        self.listening_ports
                            .iter()
                            .map(|p| format!("Port {p}"))
                            .collect::<Vec<_>>()
                            .join("  •  ")
                    };

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Open Local TCP Ports:")
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

            // Right: Waydroid Android Container & Network Fix
            cyber_card(
                &mut cols[1],
                "🤖 WAYDROID ANDROID CONTAINER TOOLS",
                Some("Fix Waydroid internet or launch Android session"),
                GREEN_OK,
                Some(("WAYLAND BRIDGE", GREEN_OK, GREEN_SUBTLE)),
                |ui| {
                    if action_card_button(
                        ui,
                        "⚡",
                        "Fix Waydroid Internet Bridge",
                        "Applies Docker FORWARD & NAT rules so Android has internet",
                        Some(("FIX NET", GREEN_OK)),
                        RED_PRIMARY,
                        false,
                    ) {
                        set_status("Applying Waydroid Docker/NAT bridge rules...".into(), CYAN_INFO);
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(SecurityManager::inject_waydroid_bridge());
                        });
                    }

                    ui.add_space(4.0);

                    if action_card_button(
                        ui,
                        "🚀",
                        "Launch Waydroid Full Android UI",
                        "Opens the Android window on your Kali Wayland desktop",
                        None,
                        GREEN_OK,
                        false,
                    ) {
                        let _ = Command::new("waydroid").arg("show-full-ui").spawn();
                        set_status("Launched Waydroid Full UI".into(), GREEN_OK);
                    }

                    ui.add_space(4.0);

                    if action_card_button(
                        ui,
                        "🛑",
                        "Stop Waydroid Background Session",
                        "Shuts down the Android container to free up RAM",
                        None,
                        AMBER_WARN,
                        false,
                    ) {
                        let _ = Command::new("waydroid").args(["session", "stop"]).spawn();
                        set_status("Stopped Waydroid session".into(), AMBER_WARN);
                    }
                },
            );
        });

        ui.add_space(6.0);

        // 3. Operational Impact Advisory Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, AMBER_WARN.gamma_multiply(0.7)))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚠️ WHAT NETWORK ISOLATION AFFECTS (IMPORTANT FOR LAB WORK)")
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
                        "• When Isolation is ENABLED, UFW blocks all incoming connections. This will block inbound listeners (nc/socat), Metasploit handlers, Responder, and local HTTP servers (python3 -m http.server).",
                    )
                    .size(10.0)
                    .color(TEXT_MUTED),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("• Terminal Revert Command:").strong().size(10.0).color(TEXT_MAIN));
                    ui.label(egui::RichText::new("pkexec ufw disable").monospace().size(10.0).color(CYAN_INFO));
                    if ui.small_button("📋 Copy Command").clicked() {
                        ui.ctx().copy_text("pkexec ufw disable".into());
                    }
                });
            });
    }
}
