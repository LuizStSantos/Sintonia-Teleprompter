//! Sintonia Teleprompter — teleprompter multiplataforma (Windows, macOS e Linux)
//! integrado ao Sintonia (login e Espelho do programa).

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod config;
mod visual;

fn main() -> eframe::Result<()> {
    let opcoes = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sintonia Teleprompter — Operador")
            .with_inner_size([1360.0, 820.0])
            .with_min_inner_size([1180.0, 640.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(visual::tema::ICONE_PNG).unwrap_or_default(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Sintonia Teleprompter",
        opcoes,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
