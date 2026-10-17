use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::core::headset::{HeadsetCommand, HeadsetManager, DEFAULT_MAC};
use crate::theme::*;

pub struct TabHeadset {
    pub mac_address: String,
    pub logs: Vec<String>,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabHeadset {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            mac_address: DEFAULT_MAC.to_string(),
            logs: vec![
                "[*] Headset RFCOMM module initialized.".into(),
                "[!] PROTOCOL NOTICE: SoundPEATS Q3 Pro vendor payloads are UNVERIFIED placeholders.".into(),
                "[*] Capture Android HCI Snoop log from vendor app to extract verified packets.".into(),
            ],
            tx,
            rx,
        }
    }

    pub fn poll_logs(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            self.logs.push(line);
            if self.logs.len() > 1000 {
                self.logs.remove(0);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        self.poll_logs();
        ui.add_space(4.0);

        // 1. Device Configuration Card
        cyber_card(
            ui,
            "🎧 SOUNDPEATS Q3 PRO // RFCOMM CONTROLLER",
            Some("BLUETOOTH SPP DISPATCH"),
            RED_PRIMARY,
            Some(("RFCOMM PORT 1", CYAN_INFO, CYAN_SUBTLE)),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Target Bluetooth MAC:")
                            .strong()
                            .size(11.0)
                            .color(TEXT_MAIN),
                    );
                    ui.add_space(8.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.mac_address)
                            .desired_width(180.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    ui.label(
                        egui::RichText::new("(Default: SoundPEATS Q3 Pro)")
                            .size(10.0)
                            .color(TEXT_MUTED),
                    );
                });
            },
        );

        ui.add_space(8.0);

        // 2. UNVERIFIED Protocol Warning Banner
        egui::Frame::new()
            .fill(RED_SUBTLE)
            .stroke(egui::Stroke::new(1.2, RED_PRIMARY))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚠️ PROTOCOL REQUIREMENT: UNVERIFIED PLACEHOLDER PAYLOADS")
                            .strong()
                            .size(12.0)
                            .color(RED_BRIGHT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_pill(ui, "REVERSE-ENG REQUIRED", RED_BRIGHT, RED_SUBTLE);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "The RFCOMM command bytes dispatched to ~/soundpeats_controller.py are unverified guesses and do NOT trigger true hardware states on the Q3 Pro.\nReal protocol frames must be extracted by enabling 'Bluetooth HCI Snoop Log' in Android Developer Options while operating the official vendor app, then inspecting with Wireshark.",
                    )
                    .size(10.5)
                    .color(TEXT_MAIN),
                );
            });

        ui.add_space(8.0);

        // 3. Hardware Mode Dispatch Card
        cyber_card(
            ui,
            "HARDWARE MODE DISPATCH MATRIX",
            Some("UNVERIFIED RFCOMM FRAMES"),
            RED_PRIMARY,
            None,
            |ui| {
                let commands = [
                    (
                        HeadsetCommand::NoiseCancellation,
                        "🔇 ACTIVE NOISE CANCELLATION (ANC)",
                        "Places earbuds into ANC mode [UNVERIFIED]",
                    ),
                    (
                        HeadsetCommand::Ambient,
                        "🔊 TRANSPARENCY / AMBIENT SOUND",
                        "Enables external microphone passthrough [UNVERIFIED]",
                    ),
                    (
                        HeadsetCommand::Normal,
                        "🎧 NORMAL MODE (ANC OFF)",
                        "Disables ANC and ambient processing [UNVERIFIED]",
                    ),
                    (
                        HeadsetCommand::DisableControls,
                        "🚫 DISABLE TOUCH & SENSOR CONTROLS",
                        "Locks capacitive touch surfaces [UNVERIFIED]",
                    ),
                    (
                        HeadsetCommand::MovieMode,
                        "🎬 MOVIE / LOW LATENCY MODE",
                        "Reduces audio buffer latency [UNVERIFIED]",
                    ),
                ];

                for (cmd, title, desc) in commands {
                    ui.horizontal(|ui| {
                        let btn = egui::Button::new(
                            egui::RichText::new(title)
                                .strong()
                                .size(11.0)
                                .color(TEXT_MAIN),
                        )
                        .fill(BG_CARD)
                        .min_size(egui::vec2(280.0, 32.0));

                        if ui.add(btn).clicked() {
                            HeadsetManager::send_command(&self.mac_address, cmd, self.tx.clone());
                            set_status(format!("Sent {} [UNVERIFIED]", cmd.as_str()), AMBER_WARN);
                        }

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(desc).size(10.0).color(TEXT_MUTED));
                    });
                    ui.add_space(3.0);
                }
            },
        );

        ui.add_space(8.0);

        // 4. RFCOMM Console Stream
        cyber_console(
            ui,
            "headset_console_scroll",
            "HEADSET RFCOMM SOCKET // LOG & STATUS STREAM",
            &mut self.logs,
            240.0,
        );
    }
}
