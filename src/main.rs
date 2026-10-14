mod theme;
mod core;
mod ui;
mod app;

use app::ZaneMegApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 780.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ZANE MEGAPP // CORE MATRIX (V2 RUST)"),
        ..Default::default()
    };

    eframe::run_native(
        "ZaneMegApp",
        native_options,
        Box::new(|cc| Ok(Box::new(ZaneMegApp::new(cc)))),
    )
}
