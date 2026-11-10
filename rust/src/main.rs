use eframe::egui::{self, ViewportBuilder};
use spacescanner::app::SpaceScannerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([980.0, 640.0])
            .with_min_inner_size([740.0, 440.0])
            .with_title("SpaceScanner - Analizador de Espacio"),
        ..Default::default()
    };

    eframe::run_native(
        "SpaceScanner",
        native_options,
        Box::new(|cc| {
            // Configurar tema y fuentes nítidas
            let mut visuals = egui::Visuals::dark();
            visuals.window_rounding = egui::Rounding::same(6.0);
            visuals.panel_fill = egui::Color32::from_rgb(20, 22, 27);
            visuals.faint_bg_color = egui::Color32::from_rgb(26, 29, 36);
            visuals.extreme_bg_color = egui::Color32::from_rgb(14, 15, 19);
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(SpaceScannerApp::new(cc)))
        }),
    )
}
