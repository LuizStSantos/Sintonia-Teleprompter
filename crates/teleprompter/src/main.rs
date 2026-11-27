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
    let renderer = match config::Settings::load().renderizador {
        config::Renderizador::OpenGl => eframe::Renderer::Glow,
        config::Renderizador::Moderno => eframe::Renderer::Wgpu,
    };
    let opcoes = eframe::NativeOptions {
        renderer,
        viewport: egui::ViewportBuilder::default()
            .with_title("Sintonia Teleprompter — Operador")
            .with_inner_size([1360.0, 760.0])
            .with_min_inner_size([1024.0, 600.0])
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
