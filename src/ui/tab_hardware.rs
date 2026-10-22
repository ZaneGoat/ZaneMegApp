//! Hardware & Cooling: the tab's state and its section order. Each card group lives in
//! `crate::ui::hardware`.

use eframe::egui;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::core::hardware::{CpuTimes, HardwareManager, HardwareTelemetry};
use crate::theme::*;
use crate::ui::hardware;

pub struct TabHardware {
    pub telemetry: HardwareTelemetry,
    pub prev_cpu_overall: CpuTimes,
    pub prev_cpu_cores: Vec<CpuTimes>,
    pub cpu_history: VecDeque<[f64; 2]>,
    pub gpu_history: VecDeque<[f64; 2]>,
    pub nvme_history: VecDeque<[f64; 2]>,
    pub active_profile: String,
    pub cooler_boost: bool,
    pub smart_boost: bool,
    pub battery_limit: u32,
    pub step_counter: usize,
    pub max_history: usize,
    pub tx: Sender<Result<String, String>>,
    pub rx: Receiver<Result<String, String>>,
}

impl TabHardware {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let (prev_cpu_overall, prev_cpu_cores) = HardwareManager::read_cpu_times();
        Self {
            telemetry: HardwareTelemetry::default(),
            prev_cpu_overall,
            prev_cpu_cores,
            cpu_history: VecDeque::with_capacity(60),
            gpu_history: VecDeque::with_capacity(60),
            nvme_history: VecDeque::with_capacity(60),
            active_profile: "balance".into(),
            cooler_boost: false,
            smart_boost: false,
            battery_limit: 80,
            step_counter: 0,
            max_history: 60,
            tx,
            rx,
        }
    }

    pub fn update_telemetry(&mut self) {
        self.telemetry =
            HardwareManager::read_telemetry(&mut self.prev_cpu_overall, &mut self.prev_cpu_cores);
        self.step_counter += 1;

        let x = self.step_counter as f64;
        let cpu_y = self.telemetry.cpu_temp as f64;
        let gpu_y = self.telemetry.gpu_temp as f64;
        let nvme_y = self.telemetry.nvme_temp as f64;

        if self.cpu_history.len() >= self.max_history {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back([x, cpu_y]);

        if self.gpu_history.len() >= self.max_history {
            self.gpu_history.pop_front();
        }
        self.gpu_history.push_back([x, gpu_y]);

        if self.nvme_history.len() >= self.max_history {
            self.nvme_history.pop_front();
        }
        self.nvme_history.push_back([x, nvme_y]);

        if self.smart_boost
            && (self.telemetry.cpu_temp >= 80.0 || self.telemetry.gpu_temp >= 80.0)
            && !self.cooler_boost
        {
            self.cooler_boost = true;
            let tx = self.tx.clone();
            thread::spawn(move || {
                let _ = tx.send(HardwareManager::set_cooler_boost(true));
            });
        }
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
            "Real-time temperatures, CPU core activity, display refresh rate, and fan cooling controls.",
            &[
                ("Optimal (<65°C)", GREEN_OK),
                ("Warm (65–79°C)", AMBER_WARN),
                ("High Heat (≥80°C)", RED_BRIGHT),
            ],
        );

        ui.add_space(6.0);
        hardware::dials::dials(self, ui);
        ui.add_space(6.0);

        hardware::profiles::power_profiles(self, ui, set_status);
        ui.add_space(6.0);

        // Fan controls beside the battery ceiling; one above the other when both cannot fit.
        grid(ui, 2, 2, 300.0, 6.0, |column, index| {
            if index == 0 {
                hardware::fan_battery::fan_controls(self, column, set_status);
            } else {
                hardware::profiles::battery_ceiling(self, column, set_status);
            }
        });

        ui.add_space(6.0);

        grid(ui, 2, 2, 300.0, 6.0, |column, index| {
            if index == 0 {
                hardware::plots::thermal_history(self, column);
            } else {
                hardware::plots::cores(self, column);
            }
        });
    }
}
