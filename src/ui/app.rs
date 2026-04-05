use eframe::egui::{self, Key};

use crate::{
    math::grapher::marching_squares,
    ui::{
        grid::{draw_axis_lines, draw_major_lines},
        viewport::Viewport,
    },
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
            let (minus_pressed, plus_pressed) =
                ui.input(|i| (i.key_pressed(Key::Minus), i.key_pressed(Key::Equals)));

            // zoom out
            if minus_pressed {
                let half_width = self.viewport.width() / 2.0;
                let half_height = self.viewport.height() / 2.0;
                self.viewport.x_min -= half_width;
                self.viewport.x_max += half_width;
                self.viewport.y_min -= half_height;
                self.viewport.y_max += half_height;
            }

            // zoom in
            if plus_pressed {
                let quarter_width = self.viewport.width() / 4.0;
                let quarter_height = self.viewport.height() / 4.0;
                self.viewport.x_min += quarter_width;
                self.viewport.x_max -= quarter_width;
                self.viewport.y_min += quarter_height;
                self.viewport.y_max -= quarter_height;
            }

            let graph_rect = ui.available_rect_before_wrap();
            // ensure y axis stays a square
            self.viewport.recalculate_y_axis(graph_rect);

            let painter = ui.painter_at(graph_rect);

            // Draw background
            painter.rect_filled(graph_rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

            // Draw major axis lines
            draw_major_lines(&painter, self.viewport);

            // Draw x and y axis lines
            draw_axis_lines(&painter, self.viewport);

            // draw equation

            let segments =
                marching_squares(|x, y| x.powf(2.0) + y.powf(2.0) - 16.0, self.viewport, 300);
            for ((x0, y0), (x1, y1)) in segments {
                painter.line_segment(
                    [
                        self.viewport.point_to_screen(graph_rect, x0, y0),
                        self.viewport.point_to_screen(graph_rect, x1, y1),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 0)),
                );
            }
        });
    }
}
