//! The two live graphs: 60 seconds of thermal history, and per-thread load.

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::theme::*;
use crate::ui::tab_hardware::TabHardware;

/// Temperature history for CPU, GPU/zone and NVMe, beside the 12-thread load matrix.
pub fn thermal_history(tab: &TabHardware, ui: &mut egui::Ui) {
    cyber_card(
        ui,
        "TEMPERATURE HISTORY (LAST 60 SECONDS)",
        Some("Live sensor graph"),
        RED_PRIMARY,
        None,
        |ui| {
            let cpu_pts: Vec<[f64; 2]> = tab.cpu_history.iter().copied().collect();
            let gpu_pts: Vec<[f64; 2]> = tab.gpu_history.iter().copied().collect();
            let nvme_pts: Vec<[f64; 2]> = tab.nvme_history.iter().copied().collect();

            let plot = Plot::new("thermal_plot")
                .legend(Legend::default())
                .height(135.0)
                .include_y(25.0)
                .include_y(95.0)
                .show_axes([false, true])
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false);

            plot.show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("CPU (°C)", PlotPoints::new(cpu_pts))
                        .color(RED_BRIGHT)
                        .width(2.2),
                );
                plot_ui.line(
                    Line::new("GPU/Zone (°C)", PlotPoints::new(gpu_pts))
                        .color(AMBER_WARN)
                        .width(1.8),
                );
                plot_ui.line(
                    Line::new("NVMe SSD (°C)", PlotPoints::new(nvme_pts))
                        .color(CYAN_INFO)
                        .width(1.5),
                );
            });
        },
    );
}

/// The per-thread load bars: three across while the card is wide enough, otherwise two, then one.
pub fn cores(tab: &TabHardware, ui: &mut egui::Ui) {
    cyber_card(
        ui,
        "12-CORE PROCESSOR ACTIVITY",
        Some("Per-thread load on Intel Core i5-10500H"),
        CYAN_INFO,
        None,
        |ui| {
            let n = tab.telemetry.core_loads.len().min(12);
            if n >= 6 {
                grid(ui, n, 3, 100.0, 3.0, |column, index| {
                    mini_core_bar(
                        column,
                        index,
                        tab.telemetry.core_loads.get(index).copied().unwrap_or(0.0),
                    );
                });
            }
        },
    );
}
