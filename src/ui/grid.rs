use eframe::egui::{self, Painter, Stroke, pos2};

use crate::ui::viewport::Viewport;

pub fn draw_axis_lines(painter: &Painter, viewport: Viewport) {
    let graph_rect = painter.clip_rect();

    let x_axis_line_norm = -viewport.y_min / viewport.height();
    // check if the x-axis line is visible
    if (0.0..=1.0).contains(&x_axis_line_norm) {
        let screen_y = graph_rect.max.y - x_axis_line_norm as f32 * graph_rect.height();
        painter.line_segment(
            [
                pos2(graph_rect.min.x, screen_y),
                pos2(graph_rect.max.x, screen_y),
            ],
            Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 80)),
        );
    }

    let y_axis_line_norm = -viewport.x_min / viewport.width();
    // check if the x-axis line is visible
    if (0.0..=1.0).contains(&y_axis_line_norm) {
        let screen_x = graph_rect.min.x + y_axis_line_norm as f32 * graph_rect.width();
        painter.line_segment(
            [
                pos2(screen_x, graph_rect.min.y),
                pos2(screen_x, graph_rect.max.y),
            ],
            Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 80)),
        );
    }
}
