//! The four cockpit dials above the power profiles.

use eframe::egui;

use crate::theme::*;
use crate::ui::tab_hardware::TabHardware;

/// CPU activity, CPU temperature, GPU/SSD temperature and battery level, four across and
/// stacking down to two and one as the viewport narrows.
pub fn dials(tab: &TabHardware, ui: &mut egui::Ui) {
    let load = tab.telemetry.cpu_load_pct;
    let load_color = if load >= 80.0 {
        RED_BRIGHT
    } else if load >= 45.0 {
        AMBER_WARN
    } else {
        CYAN_INFO
    };
    let cpu_temp = tab.telemetry.cpu_temp;
    let (cpu_color, cpu_desc) = if cpu_temp >= 80.0 {
        (RED_BRIGHT, "High Thermal Load")
    } else if cpu_temp >= 65.0 {
        (AMBER_WARN, "Warm / Active")
    } else {
        (GREEN_OK, "Cooling Normally")
    };

    let dials: [(&str, String, String, f32, egui::Color32); 4] = [
        (
            "CPU ACTIVITY",
            format!("{load:.0}%"),
            format!(
                "{:.2} GHz • {}",
                tab.telemetry.cpu_freq_ghz, tab.telemetry.cpu_governor
            ),
            load / 100.0,
            load_color,
        ),
        (
            "CPU TEMPERATURE",
            format!("{cpu_temp:.0}°C"),
            cpu_desc.to_string(),
            ((cpu_temp - 25.0) / 75.0).clamp(0.0, 1.0),
            cpu_color,
        ),
        (
            "GPU & SSD TEMP",
            format!("{:.0}°C", tab.telemetry.gpu_temp),
            format!("NVMe SSD: {:.0}°C", tab.telemetry.nvme_temp),
            ((tab.telemetry.gpu_temp - 25.0) / 75.0).clamp(0.0, 1.0),
            RED_PRIMARY,
        ),
        (
            "BATTERY LEVEL",
            format!("{}%", tab.telemetry.battery_pct),
            format!(
                "{} (Limit {}%)",
                tab.telemetry.battery_status, tab.battery_limit
            ),
            tab.telemetry.battery_pct as f32 / 100.0,
            if tab.telemetry.battery_pct <= 20 {
                RED_BRIGHT
            } else {
                GREEN_OK
            },
        ),
    ];

    // Four dials need 4×150 px of card to stay readable, two need 308 px; the count comes from
    // the width that is really there, so a narrow window stacks them instead of squeezing them.
    grid(ui, dials.len(), 4, 150.0, 6.0, |column, index| {
        let (title, value, sub, fraction, accent) = &dials[index];
        radial_dial(column, title, value, sub, *fraction, *accent);
    });
}
