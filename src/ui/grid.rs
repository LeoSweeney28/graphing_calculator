use eframe::egui::{self, Align2, FontId, Painter, Stroke, pos2};

use crate::ui::viewport::Viewport;

const MIN_PIXEL_SIZE: f64 = 80.0;

// returns (fraction, exponent)
fn normalize(x: f64) -> (f64, f64) {
    let exponent = x.log10().floor();
    (x / 10f64.powf(exponent), exponent)
}

fn nice_number(x: f64) -> f64 {
    let (fraction, exponent) = normalize(x);

    let nice_fraction = if fraction < 1.5 {
        1.0
    } else if fraction < 3.0 {
        2.0
    } else if fraction < 7.0 {
        5.0
    } else {
        10.0
    };

    nice_fraction * 10f64.powf(exponent)
}

fn format_number(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let exponent = x.abs().log10().floor();
    let fraction = x / 10f64.powf(exponent);
    let fractional_str = format!("{fraction:.1}").trim_end_matches(".0").to_string();
    if exponent >= 6.0 || exponent <= -5.0 {
        format!("{}e{exponent}", fractional_str)
    } else {
        format!("{:.5}", x)
            .trim_end_matches("0")
            .trim_end_matches(".")
            .to_string()
    }
}

const AXIS_LINE_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 80, 80);
const MAJOR_GRID_LINE_COLOR: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);
const MINOR_GRID_LINE_COLOR: egui::Color32 = egui::Color32::from_rgb(40, 40, 40);
const LABEL_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

fn draw_minor_lines(
    painter: &Painter,
    viewport: Viewport,
    major_spacing_x: f64,
    major_spacing_y: f64,
) {
    let rect = painter.clip_rect();

    // if normalize(major_spacing) == 2.0, then we should have 4 minor lines, else 5

    let minor_spacing_x = if (normalize(major_spacing_x).0 - 2.0).abs() < 1e-10 {
        major_spacing_x / 4.0
    } else {
        major_spacing_x / 5.0
    };
    let minor_spacing_y = if (normalize(major_spacing_y).0 - 2.0).abs() < 1e-10 {
        major_spacing_y / 4.0
    } else {
        major_spacing_y / 5.0
    };

    // round to nearest minor spacing
    let start_x = (viewport.x_min / minor_spacing_x).floor() * minor_spacing_x;
    let start_y = (viewport.y_min / minor_spacing_y).floor() * minor_spacing_y;
    // Vertical lines (x-axis)
    let mut x = start_x;

    while x <= viewport.x_max {
        let screen_x = viewport.x_to_screen(rect, x);
        painter.line_segment(
            [
                egui::pos2(screen_x, rect.min.y),
                egui::pos2(screen_x, rect.max.y),
            ],
            Stroke::new(1.0, MINOR_GRID_LINE_COLOR),
        );
        x += minor_spacing_x;
    }

    // Horizontal lines (y-axis)
    let mut y = start_y;

    while y <= viewport.y_max {
        let screen_y = viewport.y_to_screen(rect, y);
        painter.line_segment(
            [
                egui::pos2(rect.min.x, screen_y),
                egui::pos2(rect.max.x, screen_y),
            ],
            Stroke::new(1.0, MINOR_GRID_LINE_COLOR),
        );
        y += minor_spacing_y;
    }
}

pub fn draw_grid_lines(painter: &Painter, viewport: Viewport) {
    let rect = painter.clip_rect();
    let pixels_per_unit_x = rect.width() as f64 / viewport.width();
    let pixels_per_unit_y = rect.height() as f64 / viewport.height();

    let spacing_x = MIN_PIXEL_SIZE / pixels_per_unit_x;
    let spacing_y = MIN_PIXEL_SIZE / pixels_per_unit_y;

    let major_spacing_x = nice_number(spacing_x);
    let major_spacing_y = nice_number(spacing_y);

    // round to nearest major spacing
    let start_x = (viewport.x_min / major_spacing_x).floor() * major_spacing_x;
    let start_y = (viewport.y_min / major_spacing_y).floor() * major_spacing_y;

    let x_axis_visible = viewport.y_min <= 0.0 && viewport.y_max >= 0.0;
    let x_axis_screen_y = viewport.y_to_screen(rect, 0.0);
    let y_axis_visible = viewport.x_min <= 0.0 && viewport.x_max >= 0.0;
    let y_axis_screen_x = viewport.x_to_screen(rect, 0.0);

    draw_minor_lines(painter, viewport, major_spacing_x, major_spacing_y);

    // Vertical lines (x-axis)
    let mut x = start_x;

    while x <= viewport.x_max {
        let screen_x = viewport.x_to_screen(rect, x);
        painter.line_segment(
            [
                egui::pos2(screen_x, rect.min.y),
                egui::pos2(screen_x, rect.max.y),
            ],
            Stroke::new(1.0, MAJOR_GRID_LINE_COLOR),
        );
        // sometimes x is very close to zero but not at it
        if x_axis_visible && x.abs() >= major_spacing_x / 100.0 {
            painter.text(
                egui::pos2(screen_x, x_axis_screen_y),
                Align2::CENTER_TOP,
                format_number(x),
                FontId::default(),
                LABEL_COLOR,
            );
        }
        x += major_spacing_x;
    }

    // Horizontal lines (y-axis)
    let mut y = start_y;

    while y <= viewport.y_max {
        let screen_y = viewport.y_to_screen(rect, y);
        painter.line_segment(
            [
                egui::pos2(rect.min.x, screen_y),
                egui::pos2(rect.max.x, screen_y),
            ],
            Stroke::new(1.0, MAJOR_GRID_LINE_COLOR),
        );
        // sometimes y is very close to zero but not at it
        if y_axis_visible && y.abs() >= major_spacing_y / 100.0 {
            painter.text(
                egui::pos2(y_axis_screen_x - 5.0, screen_y),
                Align2::RIGHT_CENTER,
                format_number(y),
                FontId::default(),
                LABEL_COLOR,
            );
        }
        y += major_spacing_y;
    }

    if x_axis_visible && y_axis_visible {
        painter.text(
            egui::pos2(y_axis_screen_x - 5.0, x_axis_screen_y),
            Align2::RIGHT_TOP,
            "0",
            FontId::default(),
            LABEL_COLOR,
        );
    }
}

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
            Stroke::new(2.0, AXIS_LINE_COLOR),
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
            Stroke::new(2.0, AXIS_LINE_COLOR),
        );
    }
}
