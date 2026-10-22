use eframe::egui;
use std::time::{Duration, Instant};

use crate::theme::*;
use crate::ui::AppTab;
use crate::ui::tab_cleaner::TabCleaner;
use crate::ui::tab_controller::TabController;
use crate::ui::tab_hardware::TabHardware;
use crate::ui::tab_headset::TabHeadset;
use crate::ui::tab_idevice::TabIDevice;
use crate::ui::tab_security::TabSecurity;

/// Narrowest width the tab body is allowed to lay out in. `theme::grid` wraps a squeezed body
/// down to one column instead of squeezing every card, but one column of a 120 px body is still
/// unreadable — so the body keeps a floor rather than tracking the window down to nothing.
const MIN_VIEWPORT_WIDTH: f32 = 320.0;

pub struct ZaneMegApp {
    pub current_tab: AppTab,
    pub tab_hardware: TabHardware,
    pub tab_controller: TabController,
    pub tab_cleaner: TabCleaner,
    pub tab_headset: TabHeadset,
    pub tab_security: TabSecurity,
    pub tab_idevice: TabIDevice,
    pub status_message: String,
    pub status_color: egui::Color32,
    pub last_telemetry_tick: Instant,
}

impl ZaneMegApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let mut app = Self {
            current_tab: AppTab::Hardware,
            tab_hardware: TabHardware::new(),
            tab_controller: TabController::new(),
            tab_cleaner: TabCleaner::new(),
            tab_headset: TabHeadset::new(),
            tab_security: TabSecurity::new(),
            tab_idevice: TabIDevice::new(),
            status_message:
                "ZaneMegApp Visual Matrix Ready — Select any module on the left or press F1–F6"
                    .into(),
            status_color: GREEN_OK,
            last_telemetry_tick: Instant::now(),
        };

        app.tab_hardware.update_telemetry();
        app.tab_cleaner.update_resources();
        app
    }

    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.status_message = msg;
        self.status_color = color;
    }
}

impl eframe::App for ZaneMegApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Telemetry update interval (every 1 second)
        if self.last_telemetry_tick.elapsed() >= Duration::from_millis(1000) {
            self.last_telemetry_tick = Instant::now();
            self.tab_hardware.update_telemetry();
            self.tab_cleaner.update_resources();
            self.tab_security.refresh_state();
        }

        // Request repaint every 120ms for smooth hover/pulse animations and graphs
        ui.ctx().request_repaint_after(Duration::from_millis(120));

        // Flight Deck Keyboard Shortcuts (F1 - F6)
        ui.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.current_tab = AppTab::Hardware;
            } else if i.key_pressed(egui::Key::F2) {
                self.current_tab = AppTab::Controller;
            } else if i.key_pressed(egui::Key::F3) {
                self.current_tab = AppTab::Cleaner;
            } else if i.key_pressed(egui::Key::F4) {
                self.current_tab = AppTab::Headset;
            } else if i.key_pressed(egui::Key::F5) {
                self.current_tab = AppTab::Security;
            } else if i.key_pressed(egui::Key::F6) {
                self.current_tab = AppTab::IDevice;
            }
        });

        // ═══════════════════════════════════════════════════════════════════
        //  1. TOP TELEMETRY RIBBON HEADER
        // ═══════════════════════════════════════════════════════════════════
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let pulse = pulse_factor(ui.ctx());
                    let (dot_r, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(
                        dot_r.center(),
                        4.5,
                        RED_BRIGHT.gamma_multiply(pulse),
                    );

                    ui.label(
                        egui::RichText::new("⚡ ZANE MEGAPP")
                            .strong()
                            .size(17.0)
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new("V3.0 VISUAL COMMAND DECK  •  MSI GF65 THIN")
                            .size(10.5)
                            .color(TEXT_MUTED),
                    );

                    // Right-aligned Self-Explanatory Live Telemetry Pills
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (s_text, s_fg, s_bg) = if self.tab_security.is_isolated {
                            ("🛡️ Firewall: Isolated", RED_BRIGHT, RED_SUBTLE)
                        } else {
                            ("🛡️ Firewall: Open", GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(ui, s_text, s_fg, s_bg);

                        let bat_text = format!(
                            "🔋 Battery: {}% ({})",
                            self.tab_hardware.telemetry.battery_pct,
                            self.tab_hardware.telemetry.battery_status
                        );
                        status_pill(ui, &bat_text, TEXT_MAIN, BG_CARD);

                        let ram_pct = self.tab_cleaner.resources.ram_percent;
                        let (ram_fg, ram_bg) = if ram_pct >= 85.0 {
                            (RED_BRIGHT, RED_SUBTLE)
                        } else if ram_pct >= 65.0 {
                            (AMBER_WARN, AMBER_SUBTLE)
                        } else {
                            (GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(
                            ui,
                            &format!("🧠 Memory: {:.0}% Used", ram_pct),
                            ram_fg,
                            ram_bg,
                        );

                        let cpu_t = self.tab_hardware.telemetry.cpu_temp;
                        let cpu_load = self.tab_hardware.telemetry.cpu_load_pct;
                        let (cpu_fg, cpu_bg) = if cpu_t >= 80.0 {
                            (RED_BRIGHT, RED_SUBTLE)
                        } else if cpu_t >= 65.0 {
                            (AMBER_WARN, AMBER_SUBTLE)
                        } else {
                            (GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(
                            ui,
                            &format!("🌡️ CPU: {:.0}°C ({:.0}% Load)", cpu_t, cpu_load),
                            cpu_fg,
                            cpu_bg,
                        );
                    });
                });
            });

        ui.add_space(4.0);

        // ═══════════════════════════════════════════════════════════════════
        //  2. MAIN WIDESCREEN SPLIT: LEFT NAVIGATION SIDEBAR + MAIN VIEWPORT
        // ═══════════════════════════════════════════════════════════════════
        let mut status_change: Option<(String, egui::Color32)> = None;
        let mut status_cb = |msg: String, color: egui::Color32| {
            status_change = Some((msg, color));
        };

        let avail_h = clamp_h(ui, ui.available_height() - 36.0);

        ui.horizontal_top(|ui| {
            // Left Vertical Navigation Sidebar
            // The nav pane body: `Frame::show` inherits the horizontal layout of this split,
            // which would lay the nav out left-to-right, so give it a vertical layout here.
            ui.vertical(|ui| {
                egui::Frame::new()
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                    .corner_radius(8)
                    .inner_margin(10)
                    .show(ui, |ui| {
                        ui.set_width(225.0);
                        ui.set_min_height(avail_h - 20.0);

                        ui.label(
                            egui::RichText::new("CONTROL MODULES")
                                .strong()
                                .size(10.0)
                                .color(TEXT_DIM),
                        );
                        ui.add_space(4.0);

                        let cpu_badge = format!("{:.0}°C", self.tab_hardware.telemetry.cpu_temp);
                        let cpu_col = if self.tab_hardware.telemetry.cpu_temp >= 80.0 {
                            RED_BRIGHT
                        } else {
                            GREEN_OK
                        };
                        if nav_sidebar_item(
                            ui,
                            "F1",
                            "⚡",
                            "Hardware & Cooling",
                            "Fans, Temps & Power Modes",
                            Some((&cpu_badge, cpu_col)),
                            self.current_tab == AppTab::Hardware,
                        ) {
                            self.current_tab = AppTab::Hardware;
                        }

                        ui.add_space(4.0);

                        let (ctrl_txt, ctrl_col) = if self.tab_controller.running_pid.is_some() {
                            ("ONLINE", GREEN_OK)
                        } else {
                            ("READY", TEXT_DIM)
                        };
                        if nav_sidebar_item(
                            ui,
                            "F2",
                            "🎮",
                            "Controller Cockpit",
                            "Gamepad, LEDs & Haptics",
                            Some((ctrl_txt, ctrl_col)),
                            self.current_tab == AppTab::Controller,
                        ) {
                            self.current_tab = AppTab::Controller;
                        }

                        ui.add_space(4.0);

                        let ram_badge = if self.tab_cleaner.is_running {
                            "CLEANING".to_string()
                        } else {
                            format!("{:.0}% RAM", self.tab_cleaner.resources.ram_percent)
                        };
                        if nav_sidebar_item(
                            ui,
                            "F3",
                            "🧹",
                            "System Cleaner",
                            "Free RAM, Cache & Disk",
                            Some((&ram_badge, CYAN_INFO)),
                            self.current_tab == AppTab::Cleaner,
                        ) {
                            self.current_tab = AppTab::Cleaner;
                        }

                        ui.add_space(4.0);

                        if nav_sidebar_item(
                            ui,
                            "F4",
                            "🎧",
                            "Headset Audio",
                            "Q3 Pro Bluetooth Modes",
                            Some(("BT SPP", AMBER_WARN)),
                            self.current_tab == AppTab::Headset,
                        ) {
                            self.current_tab = AppTab::Headset;
                        }

                        ui.add_space(4.0);

                        let (sec_txt, sec_col) = if self.tab_security.is_isolated {
                            ("LOCKED", RED_BRIGHT)
                        } else {
                            ("OPEN", GREEN_OK)
                        };
                        if nav_sidebar_item(
                            ui,
                            "F5",
                            "🛡️",
                            "Network & Guard",
                            "Firewall, Local IPs & Bridge",
                            Some((sec_txt, sec_col)),
                            self.current_tab == AppTab::Security,
                        ) {
                            self.current_tab = AppTab::Security;
                        }

                        ui.add_space(4.0);

                        if nav_sidebar_item(
                            ui,
                            "F6",
                            "📱",
                            "iOS Device Tools",
                            "USB Inspector & Tunnel",
                            Some(("USB", CYAN_INFO)),
                            self.current_tab == AppTab::IDevice,
                        ) {
                            self.current_tab = AppTab::IDevice;
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Sidebar System Quick-Summary Box
                        egui::Frame::new()
                            .fill(BG_VOID)
                            .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                            .corner_radius(6)
                            .inner_margin(10)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new("SYSTEM SUMMARY")
                                        .strong()
                                        .size(9.5)
                                        .color(CYAN_INFO),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "• Uptime: {}\n• CPU Clock: {:.2} GHz\n• Governor: {}\n• NVMe SSD: {:.0}°C ({:.0}% Used)\n• Profile: {}",
                                        self.tab_hardware.telemetry.uptime_formatted,
                                        self.tab_hardware.telemetry.cpu_freq_ghz,
                                        self.tab_hardware.telemetry.cpu_governor,
                                        self.tab_hardware.telemetry.nvme_temp,
                                        self.tab_cleaner.resources.disk_percent,
                                        self.tab_hardware.active_profile.to_uppercase(),
                                    ))
                                    .size(10.0)
                                    .color(TEXT_MUTED),
                                );
                            });
                    });
            });

            ui.add_space(6.0);

            // Right Main Scrollable Tab Viewport.
            // Like the nav pane, the scroll area inherits this horizontal layout, so the tab body
            // is given an explicit vertical layout here. It is also floored to a readable width:
            // when the window is squeezed below the nav pane the body would otherwise be handed a
            // sliver barely wide enough for one card.
            let viewport_width = ui.available_width().max(MIN_VIEWPORT_WIDTH);
            ui.allocate_ui_with_layout(
                egui::vec2(viewport_width, avail_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("main_viewport_scroll")
                        .max_height(avail_h)
                        .auto_shrink([false, false])
                        // A visible bar is how the reader knows the tab continues below the
                        // fold at 1024×640; hiding it until needed made the cut look like the
                        // end of the page.
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| {
                            match self.current_tab {
                                AppTab::Hardware => self.tab_hardware.ui(ui, &mut status_cb),
                                AppTab::Controller => self.tab_controller.ui(ui, &mut status_cb),
                                AppTab::Cleaner => self.tab_cleaner.ui(ui, &mut status_cb),
                                AppTab::Headset => self.tab_headset.ui(ui, &mut status_cb),
                                AppTab::Security => self.tab_security.ui(ui, &mut status_cb),
                                AppTab::IDevice => self.tab_idevice.ui(ui, &mut status_cb),
                            }
                        });
                },
            );
        });

        if let Some((msg, color)) = status_change {
            self.set_status(msg, color);
        }

        // ═══════════════════════════════════════════════════════════════════
        //  3. BOTTOM STATUS & VISUAL COLOR LEGEND BAR
        // ═══════════════════════════════════════════════════════════════════
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(14, 6))
            .show(ui, |ui| {
                let hint = "Shortcuts: [F1–F6] Switch Modules";
                let hint_font = egui::FontId::monospace(10.0);
                let hint_w = ui
                    .painter()
                    .layout_no_wrap(hint.to_owned(), hint_font, TEXT_DIM)
                    .size()
                    .x;
                // Width the message keeps for itself before the shortcut hint may appear. The
                // message is information, the hint is chrome: when both will not fit the hint
                // goes, rather than the message being cut mid-word (the old bar ended at
                // "Switch Modu" at 1024×640).
                const MESSAGE_MIN: f32 = 320.0;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").size(11.0).color(self.status_color));

                    let gap = ui.spacing().item_spacing.x;
                    let room = ui.available_width();
                    let show_hint = room - gap - MESSAGE_MIN >= hint_w;
                    let message_w = if show_hint { room - gap - hint_w } else { room };

                    ui.scope(|ui| {
                        ui.set_max_width(message_w.max(0.0));
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&self.status_message)
                                    .strong()
                                    .size(11.0)
                                    .color(TEXT_MAIN),
                            )
                            .wrap(),
                        );
                    });

                    if show_hint {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(hint)
                                    .monospace()
                                    .size(10.0)
                                    .color(TEXT_DIM),
                            );
                        });
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives every tab body through egui at the widths that matter: the default size, the
    /// enforced minimum, and widths squeezed below the navigation pane. `Ui::columns` partitions
    /// whatever width it is handed without a guard, so this is the regression test that no tab
    /// asks for a layout the body cannot afford.
    #[test]
    fn every_tab_renders_without_panicking_at_any_width() {
        for width in [1140.0_f32, 960.0, 640.0, 320.0, 120.0] {
            let ctx = egui::Context::default();
            let mut hardware = TabHardware::new();
            let mut controller = TabController::new();
            let mut cleaner = TabCleaner::new();
            let mut headset = TabHeadset::new();
            let mut security = TabSecurity::new();
            let mut idevice = TabIDevice::new();

            for _frame in 0..2 {
                for tab in 0..6 {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 640.0),
                        )),
                        ..Default::default()
                    };
                    let output = ctx.run_ui(input, |ui| {
                        ui.set_max_width(width);
                        let mut set_status = |_msg: String, _color: egui::Color32| {};
                        egui::ScrollArea::vertical().show(ui, |ui| match tab {
                            0 => hardware.ui(ui, &mut set_status),
                            1 => controller.ui(ui, &mut set_status),
                            2 => cleaner.ui(ui, &mut set_status),
                            3 => headset.ui(ui, &mut set_status),
                            4 => security.ui(ui, &mut set_status),
                            _ => idevice.ui(ui, &mut set_status),
                        });
                    });
                    output.drop_without_applying_deltas();
                }
            }
        }
    }
}
