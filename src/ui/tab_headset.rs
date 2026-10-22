use eframe::egui;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::core::headset::{DEFAULT_MAC, HeadsetCommand, HeadsetManager};
use crate::theme::*;

/// One Bluetooth audio mode as the tab hands it to the grid: icon, title, its spec bullets, the
/// accent colour, the `mode` key sent to the headset, and the status line to report on click.
type HeadsetMode = (
    &'static str,
    &'static str,
    [&'static str; 3],
    egui::Color32,
    &'static str,
    &'static str,
);

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
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("⚠️ PROTOCOL REQUIREMENT: UNVERIFIED PLACEHOLDER PAYLOADS")
                            .strong()
                            .size(12.0)
                            .color(RED_BRIGHT),
                    );
                    status_pill(ui, "UNVERIFIED BYTES", RED_BRIGHT, RED_SUBTLE);
                });
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(
                            "The hex bytes in ~/soundpeats_controller.py are unverified guesses and do NOT change hardware ANC states yet. Enable 'Bluetooth HCI Snoop Log' on Android while using the official app to capture the real vendor packets.",
                        )
                        .size(10.5)
                        .color(TEXT_MAIN),
                    )
                    .wrap(),
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
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Target Earbuds Bluetooth MAC:")
                            .strong()
                            .size(11.0)
                            .color(TEXT_MAIN),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.mac_address)
                            .desired_width(180.0)
                            .font(egui::TextStyle::Monospace),
                    );
                });

                ui.add_space(6.0);

                let modes: [HeadsetMode; 3] = [
                    (
                        "🔇",
                        "NOISE CANCELING",
                        [
                            "• Blocks outside ambient noise",
                            "• Active ANC filter mode",
                            "• Status: [UNVERIFIED]",
                        ],
                        RED_BRIGHT,
                        "noise_cancelation",
                        "Sent Noise Cancellation [UNVERIFIED]",
                    ),
                    (
                        "🔊",
                        "AMBIENT SOUND",
                        [
                            "• Lets outside voices through",
                            "• Transparency microphone mode",
                            "• Status: [UNVERIFIED]",
                        ],
                        CYAN_INFO,
                        "ambient",
                        "Sent Ambient Mode [UNVERIFIED]",
                    ),
                    (
                        "🎧",
                        "NORMAL AUDIO",
                        [
                            "• Turns off ANC & passthrough",
                            "• Standard battery-saving sound",
                            "• Status: [UNVERIFIED]",
                        ],
                        GREEN_OK,
                        "normal",
                        "Sent Normal Mode [UNVERIFIED]",
                    ),
                ];
                let mut chosen: Option<(HeadsetCommand, &'static str, &str)> = None;
                grid(ui, modes.len(), 3, 185.0, 6.0, |column, index| {
                    let (icon, title, specs, accent, mode, status) = modes[index];
                    if mode_selector_card(
                        column,
                        icon,
                        title,
                        &specs,
                        accent,
                        self.selected_mode.as_deref() == Some(mode),
                    ) {
                        let command = match mode {
                            "noise_cancelation" => HeadsetCommand::NoiseCancellation,
                            "ambient" => HeadsetCommand::Ambient,
                            _ => HeadsetCommand::Normal,
                        };
                        chosen = Some((command, mode, status));
                    }
                });
                if let Some((command, mode, status)) = chosen {
                    self.selected_mode = Some(mode.to_string());
                    HeadsetManager::send_command(&self.mac_address, command, self.tx.clone());
                    set_status(status.to_string(), AMBER_WARN);
                }

                ui.add_space(6.0);

                let helpers: [(&str, &str, &str, &str, &str, egui::Color32, &str); 2] = [
                    (
                        "🚫",
                        "Disable Touch Controls [UNVERIFIED]",
                        "Prevents accidental taps on the earbud touch sensors",
                        "UNVERIFIED",
                        "disable_controls",
                        AMBER_WARN,
                        "Sent Disable Touch Controls [UNVERIFIED]",
                    ),
                    (
                        "🎬",
                        "Movie / Low-Latency Mode [UNVERIFIED]",
                        "Reduces wireless audio delay for gaming and video",
                        "UNVERIFIED",
                        "movie_mode",
                        CYAN_INFO,
                        "Sent Movie Mode [UNVERIFIED]",
                    ),
                ];
                let mut helper_clicked: Option<(HeadsetCommand, &str, &str)> = None;
                grid(ui, helpers.len(), 2, 190.0, 6.0, |column, index| {
                    let (icon, title, subtitle, tag, mode, accent, status) = helpers[index];
                    if action_card_button(
                        column,
                        icon,
                        title,
                        subtitle,
                        Some((tag, AMBER_WARN)),
                        accent,
                        self.selected_mode.as_deref() == Some(mode),
                    ) {
                        let command = if mode == "disable_controls" {
                            HeadsetCommand::DisableControls
                        } else {
                            HeadsetCommand::MovieMode
                        };
                        helper_clicked = Some((command, mode, status));
                    }
                });
                if let Some((command, mode, status)) = helper_clicked {
                    self.selected_mode = Some(mode.to_string());
                    HeadsetManager::send_command(&self.mac_address, command, self.tx.clone());
                    set_status(status.to_string(), AMBER_WARN);
                }
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
