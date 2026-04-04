use eframe::egui;

use crate::{
    math::grapher::marching_squares,
    ui::{grid::draw_axis_lines, viewport::Viewport},
};

pub struct GraphingCalculatorApp {
    viewport: Viewport,
    last_viewport: Viewport,
}

impl Default for GraphingCalculatorApp {
    fn default() -> Self {
        Self {
            viewport: Viewport::DEFAULT_VIEWPORT,
            last_viewport: Viewport::DEFAULT_VIEWPORT,
        }
    }
}

impl eframe::App for GraphingCalculatorApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let graph_rect = ui.available_rect_before_wrap();
            // ensure y axis stays a square
            self.viewport.recalculate_y_axis(graph_rect);

            let painter = ui.painter_at(graph_rect);

            // Draw background
            painter.rect_filled(graph_rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

            // Draw x and y axis lines
            draw_axis_lines(&painter, self.viewport);

            // draw equation

            let segments = marching_squares(|x, y| x.powf(2.0) - y, self.viewport, 300);
            let to_screen = |x, y| -> egui::Pos2 {
                let sx = graph_rect.min.x
                    + (((x - self.viewport.x_min) * graph_rect.width() as f64)
                        / self.viewport.width()) as f32;
                let sy = graph_rect.max.y
                    - (((y - self.viewport.y_min) * graph_rect.height() as f64)
                        / self.viewport.height()) as f32;
                egui::pos2(sx, sy)
            };
            for ((x0, y0), (x1, y1)) in segments {
                painter.line_segment(
                    [to_screen(x0, y0), to_screen(x1, y1)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 0)),
                );
            }
        });
    }
}
