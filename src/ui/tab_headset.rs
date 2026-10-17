use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::core::headset::{HeadsetCommand, HeadsetManager, DEFAULT_MAC};
use crate::theme::*;

pub struct TabHeadset {
    pub mac_address: String,
    pub selected_mode: Option<String>,
    pub logs: Vec<String>,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabHeadset {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            mac_address: DEFAULT_MAC.to_string(),
            selected_mode: None,
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

        legend_strip(
            ui,
            "Bluetooth RFCOMM command dispatcher for SoundPEATS Q3 Pro wireless earbuds.",
            &[
                ("RFCOMM Socket", CYAN_INFO),
                ("Unverified Payload", AMBER_WARN),
            ],
        );

        ui.add_space(6.0);

        // 1. UNVERIFIED Protocol Warning Banner
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
                        status_pill(ui, "UNVERIFIED BYTES", RED_BRIGHT, RED_SUBTLE);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "The hex bytes in ~/soundpeats_controller.py are unverified guesses and do NOT change hardware ANC states yet. Enable 'Bluetooth HCI Snoop Log' on Android while using the official app to capture the real vendor packets.",
                    )
                    .size(10.5)
                    .color(TEXT_MAIN),
                );
            });

        ui.add_space(6.0);

        // 2. Device MAC & Audio Mode Cards
        cyber_card(
            ui,
            "🎧 SOUNDPEATS Q3 PRO // AUDIO MODE SELECTOR",
            Some("Select an audio profile or utility command [UNVERIFIED]"),
            RED_PRIMARY,
            Some(("BLUETOOTH SPP PORT 1", CYAN_INFO, CYAN_SUBTLE)),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Target Earbuds Bluetooth MAC:")
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
                });

                ui.add_space(6.0);

                ui.columns(3, |cols| {
                    if mode_selector_card(
                        &mut cols[0],
                        "🔇",
                        "NOISE CANCELING",
                        "• Blocks outside ambient noise",
                        "• Active ANC filter mode",
                        "• Status: [UNVERIFIED]",
                        RED_BRIGHT,
                        self.selected_mode.as_deref() == Some("noise_cancelation"),
                    ) {
                        self.selected_mode = Some("noise_cancelation".into());
                        HeadsetManager::send_command(
                            &self.mac_address,
                            HeadsetCommand::NoiseCancellation,
                            self.tx.clone(),
                        );
                        set_status("Sent Noise Cancellation [UNVERIFIED]".into(), AMBER_WARN);
                    }

                    if mode_selector_card(
                        &mut cols[1],
                        "🔊",
                        "AMBIENT SOUND",
                        "• Lets outside voices through",
                        "• Transparency microphone mode",
                        "• Status: [UNVERIFIED]",
                        CYAN_INFO,
                        self.selected_mode.as_deref() == Some("ambient"),
                    ) {
                        self.selected_mode = Some("ambient".into());
                        HeadsetManager::send_command(
                            &self.mac_address,
                            HeadsetCommand::Ambient,
                            self.tx.clone(),
                        );
                        set_status("Sent Ambient Mode [UNVERIFIED]".into(), AMBER_WARN);
                    }

                    if mode_selector_card(
                        &mut cols[2],
                        "🎧",
                        "NORMAL AUDIO",
                        "• Turns off ANC & passthrough",
                        "• Standard battery-saving sound",
                        "• Status: [UNVERIFIED]",
                        GREEN_OK,
                        self.selected_mode.as_deref() == Some("normal"),
                    ) {
                        self.selected_mode = Some("normal".into());
                        HeadsetManager::send_command(
                            &self.mac_address,
                            HeadsetCommand::Normal,
                            self.tx.clone(),
                        );
                        set_status("Sent Normal Mode [UNVERIFIED]".into(), AMBER_WARN);
                    }
                });

                ui.add_space(6.0);

                ui.columns(2, |cols| {
                    if action_card_button(
                        &mut cols[0],
                        "🚫",
                        "Disable Touch Controls [UNVERIFIED]",
                        "Prevents accidental taps on the earbud touch sensors",
                        Some(("UNVERIFIED", AMBER_WARN)),
                        AMBER_WARN,
                        self.selected_mode.as_deref() == Some("disable_controls"),
                    ) {
                        self.selected_mode = Some("disable_controls".into());
                        HeadsetManager::send_command(
                            &self.mac_address,
                            HeadsetCommand::DisableControls,
                            self.tx.clone(),
                        );
                        set_status("Sent Disable Touch Controls [UNVERIFIED]".into(), AMBER_WARN);
                    }

                    if action_card_button(
                        &mut cols[1],
                        "🎬",
                        "Movie / Low-Latency Mode [UNVERIFIED]",
                        "Reduces wireless audio delay for gaming and video",
                        Some(("UNVERIFIED", AMBER_WARN)),
                        CYAN_INFO,
                        self.selected_mode.as_deref() == Some("movie_mode"),
                    ) {
                        self.selected_mode = Some("movie_mode".into());
                        HeadsetManager::send_command(
                            &self.mac_address,
                            HeadsetCommand::MovieMode,
                            self.tx.clone(),
                        );
                        set_status("Sent Movie Mode [UNVERIFIED]".into(), AMBER_WARN);
                    }
                });
            },
        );

        ui.add_space(6.0);

        // 3. RFCOMM Console Stream
        cyber_console(
            ui,
            "headset_console_scroll",
            "BLUETOOTH RFCOMM // CONNECTION & PACKET LOG",
            &mut self.logs,
            210.0,
        );
    }
}
