use eframe::egui::{self, Key};
use mexpr::{Memory, Store, TypedFunc, compile_function};
use std::cell::RefCell;

use crate::{
    math::grapher::marching_squares_from_values,
    ui::{
        grid::{draw_axis_lines, draw_major_lines},
        viewport::Viewport,
    },
};

const MARCHING_RESOLUTION: usize = 300;
const INPUT_STRIDE_BYTES: usize = 16;
const OUTPUT_STRIDE_BYTES: usize = 8;
const WASM_PAGE_SIZE: usize = 65_536;

pub struct Equation {
    store: RefCell<Store<()>>,
    memory: Memory,
    batch_func: TypedFunc<(i32, i32), i32>,
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
        let (store, _func, batch_func, memory) = compile_function(&processed_input)?;
        Ok(Equation {
            store: RefCell::new(store),
            memory,
            batch_func,
            segments: vec![],
            dirty: true,
        })
    }

    pub fn recalculate_if_needed(&mut self, viewport: Viewport) {
        if self.dirty {
            let resolution = MARCHING_RESOLUTION;
            let side = resolution + 1;
            let point_count = side * side;
            let point_count_i32 = i32::try_from(point_count).expect("resolution too large");

            let input_bytes = point_count * INPUT_STRIDE_BYTES;
            let output_bytes = point_count * OUTPUT_STRIDE_BYTES;
            let needed_bytes = input_bytes + output_bytes;

            let dx = viewport.width() / resolution as f64;
            let dy = viewport.height() / resolution as f64;

            let values = {
                let mut store = self.store.borrow_mut();

                let current_size = self.memory.data_size(&*store);
                if current_size < needed_bytes {
                    let additional = needed_bytes - current_size;
                    let pages = additional.div_ceil(WASM_PAGE_SIZE);
                    self.memory
                        .grow(&mut *store, pages as u64)
                        .expect("failed to grow wasm memory");
                }

                {
                    let data = self.memory.data_mut(&mut *store);
                    for i in 0..side {
                        let x = viewport.x_min + i as f64 * dx;
                        for j in 0..side {
                            let y = viewport.y_min + j as f64 * dy;
                            let idx = i * side + j;
                            let base = idx * INPUT_STRIDE_BYTES;

                            data[base..base + 8].copy_from_slice(&x.to_le_bytes());
                            data[base + 8..base + INPUT_STRIDE_BYTES]
                                .copy_from_slice(&y.to_le_bytes());
                        }
                    }
                }

                let out_ptr = self
                    .batch_func
                    .call(&mut *store, (0, point_count_i32))
                    .expect("batch wasm call failed") as usize;

                let data = self.memory.data(&*store);
                let out_end = out_ptr + output_bytes;
                assert!(
                    out_end <= data.len(),
                    "batch output out of wasm memory bounds"
                );

                let mut out = Vec::with_capacity(point_count);
                for idx in 0..point_count {
                    let base = out_ptr + idx * OUTPUT_STRIDE_BYTES;
                    let mut buf = [0_u8; 8];
                    buf.copy_from_slice(&data[base..base + OUTPUT_STRIDE_BYTES]);
                    out.push(f64::from_le_bytes(buf));
                }
                out
            };

            self.segments = marching_squares_from_values(viewport, resolution, &values);
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
