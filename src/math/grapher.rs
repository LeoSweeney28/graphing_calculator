use crate::ui::Viewport;

/// https://en.wikipedia.org/wiki/Marching_squares#/media/File:Marching_squares_algorithm_schematic.svg
pub fn marching_squares(
    f: impl Fn(f64, f64) -> f64,
    viewport: Viewport,
    resolution: usize,
) -> Vec<((f64, f64), (f64, f64))> {
    let dx = viewport.width() / resolution as f64;
    let dy = viewport.height() / resolution as f64;

    let mut values: Vec<Vec<f64>> = Vec::with_capacity(resolution + 1);

    // precompute each value within the screen position
    for i in 0..resolution + 1 {
        let mut l = Vec::with_capacity(resolution + 1);
        for j in 0..resolution + 1 {
            let x = viewport.x_min + i as f64 * dx;
            let y = viewport.y_min + j as f64 * dy;
            l.push(f(x, y));
        }
        values.push(l);
    }

    let mut segments: Vec<((f64, f64), (f64, f64))> = vec![];

    let interpolate = |x0: f64, y0: f64, v0: f64, x1: f64, y1: f64, v1: f64| -> (f64, f64) {
        // linear interpolation
        let t = -v0 / (v1 - v0);
        (x0 + t * (x1 - x0), y0 + t * (y1 - y0))
    };

    for i in 0..resolution {
        for j in 0..resolution {
            let v00 = values[i][j]; // bottom left
            let v10 = values[i + 1][j]; // bottom right
            let v11 = values[i + 1][j + 1]; // top left
            let v01 = values[i][j + 1]; // top right

            let mut case_index = 0;
            if v00 >= 0.0 {
                case_index |= 0b0001;
            };
            if v10 >= 0.0 {
                case_index |= 0b0010;
            }
            if v11 >= 0.0 {
                case_index |= 0b0100;
            }
            if v01 >= 0.0 {
                case_index |= 0b1000;
            }

            if case_index == 0 || case_index == 15 {
                continue;
            };

            let x0 = viewport.x_min + i as f64 * dx;
            let x1 = x0 + dx;
            let y0 = viewport.y_min + j as f64 * dy;
            let y1 = y0 + dy;

            match case_index {
                1 | 14 => {
                    let p1 = interpolate(x0, y0, v00, x1, y0, v10); // bottom edge
                    let p2 = interpolate(x0, y0, v00, x0, y1, v01); // left edge
                    segments.push((p1, p2));
                }
                2 | 13 => {
                    let p1 = interpolate(x1, y0, v10, x1, y1, v11); // right edge
                    let p2 = interpolate(x0, y0, v00, x1, y0, v10); // bottom edge
                    segments.push((p1, p2));
                }
                3 | 12 => {
                    let p1 = interpolate(x0, y0, v00, x0, y1, v01); // left edge
                    let p2 = interpolate(x1, y0, v10, x1, y1, v11); // right edge
                    segments.push((p1, p2));
                }
                4 | 11 => {
                    let p1 = interpolate(x1, y0, v10, x1, y1, v11); // right edge
                    let p2 = interpolate(x0, y1, v01, x1, y1, v11); // top edge
                    segments.push((p1, p2));
                }
                6 | 9 => {
                    let p1 = interpolate(x0, y0, v00, x1, y0, v10); // bottom edge
                    let p2 = interpolate(x0, y1, v01, x1, y1, v11); // top edge
                    segments.push((p1, p2));
                }
                7 | 8 => {
                    let p1 = interpolate(x0, y0, v00, x0, y1, v01); // left edge
                    let p2 = interpolate(x0, y1, v01, x1, y1, v11); // top edge
                    segments.push((p1, p2));
                }
                5 | 10 => {
                    let p_bottom = interpolate(x0, y0, v00, x1, y0, v10);
                    let p_right = interpolate(x1, y0, v10, x1, y1, v11);
                    let p_top = interpolate(x0, y1, v01, x1, y1, v11);
                    let p_left = interpolate(x0, y0, v00, x0, y1, v01);

                    let v_center = (v00 + v10 + v11 + v01) / 4.0;

                    if v_center >= 0.0 {
                        segments.push((p_left, p_top)); // around top-left corner
                        segments.push((p_bottom, p_right)); // around bottom-right corner
                    } else {
                        segments.push((p_left, p_bottom)); // around bottom-left corner
                        segments.push((p_top, p_right)); // around top-right corner
                    }
                }
                _ => continue,
            }
        }
    }

    segments
}
