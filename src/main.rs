use crate::ui::GraphingCalculatorApp;

mod math;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Graphing Calculator",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<GraphingCalculatorApp>::default())
        }),
    )
}
