mod physical_specs;
mod channels;
mod density_matrix;
mod qasm;
mod circuit_grid;
mod emitters;
mod postprocessing;
mod app;

use app::TransmonApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([740.0, 480.0])
            .with_title("Qty - Quantum Transmon & Yield Console"),
        ..Default::default()
    };

    eframe::run_native(
        "Qty - Quantum Transmon & Yield Console",
        native_options,
        Box::new(|_cc| Box::new(TransmonApp::default())),
    )
}

