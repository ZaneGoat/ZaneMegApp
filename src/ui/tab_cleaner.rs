use eframe::egui;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::cleaner::{CleanerManager, SystemResources, MODULES};
use crate::theme::*;

pub struct TabCleaner {
    pub selected_ids: HashSet<String>,
    pub resources: SystemResources,
    pub top_procs: Vec<(u32, String, f32)>,
    pub last_reclaimed_banner: Option<String>,
    pub logs: Vec<String>,
    pub is_running: bool,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabCleaner {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let mut selected_ids = HashSet::new();
        for m in MODULES {
            if m.default_checked {
                selected_ids.insert(m.id.to_string());
            }
        }
        Self {
            selected_ids,
            resources: CleanerManager::read_system_resources(),
            top_procs: CleanerManager::read_top_processes(5),
            last_reclaimed_banner: None,
            logs: vec![
                "[*] System Cleaner & Memory Optimizer Ready.".into(),
                "[*] Click any cleanup card on the left to toggle it, then click Execute Cleanup.".into(),
            ],
            is_running: false,
            tx,
            rx,
        }
    }

    pub fn update_resources(&mut self) {
        self.resources = CleanerManager::read_system_resources();
        self.top_procs = CleanerManager::read_top_processes(5);
    }

    pub fn poll_logs(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            if line.contains("RECLAIMED //") {
                self.last_reclaimed_banner = Some(line.replace("[✓] ", ""));
                self.update_resources();
            }
            if line.contains("CLEANUP PROTOCOL COMPLETED") {
                self.is_running = false;
                self.update_resources();
            }
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
            "Reclaim RAM, clear package & browser caches, and stop heavy background apps.",
            &[
                ("Healthy (<65%)", GREEN_OK),
                ("Moderate (65–84%)", AMBER_WARN),
                ("Full (≥85%)", RED_BRIGHT),
            ],
        );

        ui.add_space(6.0);

        // 1. Live System Resource Gauges Deck
        let badge = if let Some(ref banner) = self.last_reclaimed_banner {
            Some((banner.as_str(), GREEN_OK, GREEN_SUBTLE))
        } else {
            Some(("LIVE SYSTEM TELEMETRY", CYAN_INFO, CYAN_SUBTLE))
        };

        cyber_card(
            ui,
            "MEMORY & STORAGE CAPACITY",
            Some("Live usage before and after cleanup"),
            RED_PRIMARY,
            badge,
            |ui| {
                ui.columns(3, |cols| {
                    let ram_frac = self.resources.ram_percent / 100.0;
                    let ram_free_gb = (self.resources.ram_total_gb - self.resources.ram_used_gb).max(0.0);
                    let ram_used_str = format!(
                        "{:.2} GB Used  /  {:.2} GB Total",
                        self.resources.ram_used_gb, self.resources.ram_total_gb
                    );
                    let ram_cache_str = format!("{:.2} GB Free (Cache: {:.0} MB)", ram_free_gb, self.resources.ram_cached_mb);
                    resource_gauge_card(
                        &mut cols[0],
                        "🧠 SYSTEM MEMORY (RAM)",
                        &ram_used_str,
                        &ram_cache_str,
                        ram_frac,
                    );

                    let swap_frac = self.resources.swap_percent / 100.0;
                    let swap_used_str = format!(
                        "{:.2} GB Used  /  {:.2} GB Total",
                        self.resources.swap_used_gb, self.resources.swap_total_gb
                    );
                    let swap_detail = if self.resources.swap_used_gb < 0.05 {
                        "No disk paging active"
                    } else {
                        "Swap paging active"
                    };
                    resource_gauge_card(
                        &mut cols[1],
                        "🔄 VIRTUAL SWAP MEMORY",
                        &swap_used_str,
                        swap_detail,
                        swap_frac,
                    );

                    let disk_frac = self.resources.disk_percent / 100.0;
                    let disk_used_str = format!(
                        "{:.1} GB Used  /  {:.1} GB Total",
                        self.resources.disk_used_gb, self.resources.disk_total_gb
                    );
                    let disk_free_str = format!("{:.1} GB Available Free", self.resources.disk_free_gb);
                    resource_gauge_card(
                        &mut cols[2],
                        "💾 NVMe MAIN DRIVE (/)",
                        &disk_used_str,
                        &disk_free_str,
                        disk_frac,
                    );
                });
            },
        );

        ui.add_space(6.0);

        // 2. Main Split View: Custom Module Cards (Left) + Top Memory Apps & Console (Right)
        ui.horizontal_top(|ui| {
            // Left Sidebar: Custom Interactive Cleanup Module Cards
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(365.0);

                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 15.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, RED_PRIMARY);
                        ui.label(
                            egui::RichText::new("SELECT CLEANUP TASKS")
                                .strong()
                                .size(12.0)
                                .color(TEXT_MAIN),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            status_pill(
                                ui,
                                &format!("{} of {} Active", self.selected_ids.len(), MODULES.len()),
                                GREEN_OK,
                                GREEN_SUBTLE,
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Quick Filter Buttons
                    ui.horizontal(|ui| {
                        if ui.button("✔ Select All").clicked() {
                            for m in MODULES {
                                self.selected_ids.insert(m.id.to_string());
                            }
                        }
                        if ui.button("✖ Clear All").clicked() {
                            self.selected_ids.clear();
                        }
                        if ui.button("🛡 Recommended").clicked() {
                            self.selected_ids.clear();
                            for m in MODULES {
                                if m.default_checked {
                                    self.selected_ids.insert(m.id.to_string());
                                }
                            }
                        }
                        if ui.button("🧠 RAM Only").clicked() {
                            self.selected_ids.clear();
                            self.selected_ids.insert("smart_ram".into());
                            self.selected_ids.insert("kernel_cache".into());
                        }
                    });

                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .id_salt("cleaner_modules_scroll")
                        .max_height(280.0)
                        .show(ui, |ui| {
                            for m in MODULES {
                                let is_checked = self.selected_ids.contains(m.id);
                                if module_toggle_card(
                                    ui,
                                    is_checked,
                                    m.name,
                                    m.description,
                                    m.category,
                                    m.requires_root,
                                ) {
                                    if is_checked {
                                        self.selected_ids.remove(m.id);
                                    } else {
                                        self.selected_ids.insert(m.id.to_string());
                                    }
                                }
                                ui.add_space(3.0);
                            }
                        });

                    ui.add_space(6.0);

                    // Primary Execute Action Card Button
                    let exec_title = if self.is_running {
                        "Cleaning in Progress..."
                    } else {
                        "Run Selected Cleanup Tasks"
                    };
                    if action_card_button(
                        ui,
                        "🚀",
                        exec_title,
                        "Runs all checked modules above (prompts once for Admin tasks)",
                        Some(("START", GREEN_OK)),
                        RED_PRIMARY,
                        !self.is_running,
                    ) && !self.is_running
                        && !self.selected_ids.is_empty()
                    {
                        self.is_running = true;
                        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            CleanerManager::execute_cleanup(&ids, tx);
                        });
                        set_status("Running selected cleanup tasks...".into(), GREEN_OK);
                    }

                    ui.add_space(4.0);

                    if action_card_button(
                        ui,
                        "⚡",
                        "1-Click Instant RAM Boost (No Password)",
                        "Immediately closes background indexers and clears user caches",
                        Some(("FAST", CYAN_INFO)),
                        CYAN_INFO,
                        false,
                    ) && !self.is_running
                    {
                        self.is_running = true;
                        let ids = vec!["smart_ram".to_string(), "thumbnail_cache".to_string()];
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            CleanerManager::execute_cleanup(&ids, tx);
                        });
                        set_status("Running 1-Click Instant RAM Boost...".into(), CYAN_INFO);
                    }
                });

            ui.add_space(6.0);

            // Right Pane: Heaviest Memory Apps + Live Console
            ui.vertical(|ui| {
                cyber_card(
                    ui,
                    "HEAVIEST MEMORY APPS RUNNING NOW",
                    Some("See which apps are using the most RAM and close them if needed"),
                    AMBER_WARN,
                    Some(("TOP 5 APPS", AMBER_WARN, AMBER_SUBTLE)),
                    |ui| {
                        let procs_snapshot = self.top_procs.clone();
                        let mut stop_action: Option<(u32, String)> = None;

                        for (pid, name, rss_mb) in procs_snapshot {
                            egui::Frame::new()
                                .fill(BG_CARD)
                                .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                                .corner_radius(6)
                                .inner_margin(egui::Margin::symmetric(10, 5))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&name)
                                                .strong()
                                                .size(11.5)
                                                .color(TEXT_MAIN),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("(PID {pid})"))
                                                .monospace()
                                                .size(9.5)
                                                .color(TEXT_DIM),
                                        );

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("⏹ Stop App")
                                                            .strong()
                                                            .size(10.0)
                                                            .color(RED_BRIGHT),
                                                    )
                                                    .fill(RED_SUBTLE)
                                                    .min_size(egui::vec2(78.0, 22.0)),
                                                )
                                                .on_hover_text(format!("Gracefully close {name} (PID {pid})"))
                                                .clicked()
                                            {
                                                stop_action = Some((pid, name.clone()));
                                            }

                                            ui.add_space(8.0);
                                            let mem_color = if rss_mb >= 800.0 {
                                                RED_BRIGHT
                                            } else if rss_mb >= 300.0 {
                                                AMBER_WARN
                                            } else {
                                                CYAN_INFO
                                            };
                                            ui.label(
                                                egui::RichText::new(format!("{rss_mb:.0} MB"))
                                                    .monospace()
                                                    .strong()
                                                    .size(11.0)
                                                    .color(mem_color),
                                            );
                                        });
                                    });
                                });
                            ui.add_space(2.0);
                        }

                        if let Some((pid, name)) = stop_action {
                            CleanerManager::snipe_process(pid, &name, &self.tx);
                            self.update_resources();
                            set_status(format!("Requested {name} (PID {pid}) to close"), AMBER_WARN);
                        }
                    },
                );

                ui.add_space(6.0);

                cyber_console(
                    ui,
                    "cleaner_console_scroll",
                    "CLEANUP PROGRESS & ACTIVITY LOG",
                    &mut self.logs,
                    215.0,
                );
            });
        });
    }
}
