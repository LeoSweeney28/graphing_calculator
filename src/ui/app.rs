use eframe::egui::{self, Key};
use mexpr::{Store, TypedFunc, compile_function};
use std::cell::RefCell;

use crate::{
    math::grapher::marching_squares,
    ui::{
        grid::{draw_axis_lines, draw_major_lines},
        viewport::Viewport,
    },
};

pub struct Equation {
    raw: String,
    store: RefCell<Store<()>>,
    func: TypedFunc<(f64, f64), f64>,
    segments: Vec<((f64, f64), (f64, f64))>,
    dirty: bool,
}

fn preprocess_equation(input: &str) -> String {
    if let Some((lhs, rhs)) = input.split_once("=") {
        // lhs will never have another equals but rhs could
        if rhs.contains("=") {
            // TODO: Don't panic here
            panic!("Equation cannot have two equal signs");
        }
        format!("({lhs}) - ({rhs})")
    } else {
        input.to_string()
    }
}

impl Equation {
    pub fn new(input: &str) -> anyhow::Result<Self> {
        let processed_input = preprocess_equation(input);
        let (store, func) = compile_function(&processed_input)?;
        Ok(Equation {
            raw: input.to_string(),
            store: RefCell::new(store),
            func,
            segments: vec![],
            dirty: true,
        })
    }

    pub fn calc_xy(&self, x: f64, y: f64) -> f64 {
        let mut store = self.store.borrow_mut();
        self.func.call(&mut *store, (x, y)).unwrap()
    }

    pub fn recalculate_if_needed(&mut self, viewport: Viewport) {
        if self.dirty {
            self.segments.clear();
            let new_segments = marching_squares(|x, y| self.calc_xy(x, y), viewport, 300);
            self.segments.extend(new_segments);
            self.dirty = false;
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct GraphingCalculatorApp {
    viewport: Viewport,
    last_viewport: Viewport,
    equations: Vec<Equation>,
}

impl Default for GraphingCalculatorApp {
    fn default() -> Self {
        let equation = Equation::new("x^2+y^2=5").unwrap();
        Self {
            viewport: Viewport::DEFAULT_VIEWPORT,
            last_viewport: Viewport::DEFAULT_VIEWPORT,
            equations: vec![equation],
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

            if self.viewport != self.last_viewport {
                for equation in &mut self.equations {
                    equation.mark_dirty();
                }
            }

            let painter = ui.painter_at(graph_rect);

            // Draw background
            painter.rect_filled(graph_rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

            // Draw major axis lines
            draw_major_lines(&painter, self.viewport);

            // Draw x and y axis lines
            draw_axis_lines(&painter, self.viewport);

            // draw equations
            for equation in &mut self.equations {
                equation.recalculate_if_needed(self.viewport);
                for &((x0, y0), (x1, y1)) in &equation.segments {
                    painter.line_segment(
                        [
                            self.viewport.point_to_screen(graph_rect, x0, y0),
                            self.viewport.point_to_screen(graph_rect, x1, y1),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 0)),
                    );
                }
            }

            self.last_viewport = self.viewport;
        });
    }
}
