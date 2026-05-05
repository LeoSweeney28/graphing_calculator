use std::fmt::Debug;

use anyhow::anyhow;
use eframe::egui::{self, Rect};

#[derive(Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    has_custom_aspect_ratio: bool,
}

impl Debug for Viewport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "X: ({}, {}) Y: ({}, {})",
            self.x_min, self.x_max, self.y_min, self.y_max
        ))
    }
}

impl Viewport {
    pub const DEFAULT_VIEWPORT: Viewport = Viewport {
        x_min: -10.0,
        x_max: 10.0,
        y_min: -10.0,
        y_max: 10.0,
        has_custom_aspect_ratio: false,
    };

    pub fn new(
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        has_custom_aspect_ratio: bool,
    ) -> Result<Self, anyhow::Error> {
        if x_min == x_max || y_min == y_max {
            return Err(anyhow!("Viewport has zero width or height"));
        }
        Ok(Viewport {
            x_min: x_min.min(x_max),
            x_max: x_max.max(x_min),
            y_min: y_min.min(y_max),
            y_max: y_max.max(y_min),
            has_custom_aspect_ratio,
        })
    }

    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn is_valid_viewport(&self) -> bool {
        self.x_min.is_finite()
            && self.x_max.is_finite()
            && self.y_min.is_finite()
            && self.y_max.is_finite()
            && self.x_min < self.x_max
            && self.y_min < self.y_max
    }

    pub fn set_has_custom_aspect_ratio(&mut self, val: bool) {
        self.has_custom_aspect_ratio = val;
    }

    pub fn recalculate_y_axis(&mut self, rect: Rect) {
        if !self.has_custom_aspect_ratio {
            let aspect_ratio = rect.aspect_ratio() as f64;
            let width = self.x_max - self.x_min;
            let y_center = (self.y_min + self.y_max) * 0.5;
            let half_height = (width * 0.5) / aspect_ratio;
            self.y_min = y_center - half_height;
            self.y_max = y_center + half_height;
        }
    }

    pub fn x_to_screen(&self, rect: Rect, x: f64) -> f32 {
        rect.min.x + (((x - self.x_min) * rect.width() as f64) / self.width()) as f32
    }
    pub fn y_to_screen(&self, rect: Rect, y: f64) -> f32 {
        rect.max.y - (((y - self.y_min) * rect.height() as f64) / self.height()) as f32
    }

    pub fn point_to_screen(&self, rect: Rect, x: f64, y: f64) -> egui::Pos2 {
        egui::pos2(self.x_to_screen(rect, x), self.y_to_screen(rect, y))
    }

    /// Zoom out by a factor (values > 1.0 zoom out, < 1.0 zoom in)
    pub fn zoom_out_centered(&mut self, factor: f64) {
        let half_width = self.width() / 2.0;
        let half_height = self.height() / 2.0;
        let margin_x = half_width * (factor - 1.0) / 2.0;
        let margin_y = half_height * (factor - 1.0) / 2.0;
        self.x_min -= margin_x;
        self.x_max += margin_x;
        self.y_min -= margin_y;
        self.y_max += margin_y;
    }

    /// Zoom in/out around a specific world point
    pub fn zoom_around_point(&mut self, world_x: f64, world_y: f64, zoom_factor: f64) {
        self.x_min = world_x + (self.x_min - world_x) * zoom_factor;
        self.x_max = world_x + (self.x_max - world_x) * zoom_factor;
        self.y_min = world_y + (self.y_min - world_y) * zoom_factor;
        self.y_max = world_y + (self.y_max - world_y) * zoom_factor;
    }

    /// Pan the viewport by the given world distance
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.x_min += dx;
        self.x_max += dx;
        self.y_min += dy;
        self.y_max += dy;
    }
}
