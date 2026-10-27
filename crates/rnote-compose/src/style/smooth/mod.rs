// Modules
mod smoothoptions;

// Re-exports
pub use smoothoptions::{LineCap, LineStyle, SmoothOptions, TipProfile};

// Imports
use super::Composer;
use crate::PenPath;
use crate::ext::Vector2Ext;
use crate::penpath::{self, Element, Segment};
use crate::shapes::{
    Arrow, CubicBezier, Ellipse, Line, Polygon, Polyline, QuadraticBezier, Rectangle, Shapeable,
};
use kurbo::Shape;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;

impl Composer<SmoothOptions> for Line {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let line = self.outline_path();

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                line,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Arrow {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.internal_compute_bounds(Some(options.stroke_width))
            .loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();

        if let Some(stroke_color) = options.stroke_color {
            let arrow = self.to_kurbo(Some(options.stroke_width));
            cx.stroke_styled(
                arrow,
                &Into::<piet::Color>::into(stroke_color),
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }

        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Rectangle {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let shape = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                shape,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Ellipse {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let ellipse = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&ellipse, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                ellipse,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for QuadraticBezier {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let quadbez = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&quadbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                quadbez,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for CubicBezier {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let cubbez = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&cubbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                cubbez,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Polyline {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };
        let n_points = self.path.len();
        let single_pos = self.path.iter().all(|p| *p == self.start);

        // Single element/position polylines need special treatment to be rendered
        if n_points == 0 || single_pos {
            cx.fill(
                kurbo::Circle::new(self.start.to_kurbo_point(), options.stroke_width),
                &Into::<piet::Color>::into(color),
            );
        } else {
            let style = options
                .piet_stroke_style
                .clone()
                .line_cap(piet::LineCap::Butt)
                .line_join(piet::LineJoin::Bevel);
            cx.stroke_styled(
                self.outline_path(),
                &Into::<piet::Color>::into(color),
                options.stroke_width,
                &style,
            );
        }
    }
}

impl Composer<SmoothOptions> for Polygon {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };
        let n_points = self.path.len();
        let single_pos = self.path.iter().all(|p| *p == self.start);

        // Single element/position polylines need special treatment to be rendered
        if n_points == 0 || single_pos {
            cx.fill(
                kurbo::Circle::new(self.start.to_kurbo_point(), options.stroke_width),
                &Into::<piet::Color>::into(color),
            );
        } else {
            let outline_path = self.outline_path();
            if let Some(fill_color) = options.fill_color {
                cx.fill(&outline_path, &Into::<piet::Color>::into(fill_color));
            }
            let style = options
                .piet_stroke_style
                .clone()
                .line_cap(piet::LineCap::Butt)
                .line_join(piet::LineJoin::Bevel);

            cx.stroke_styled(
                &outline_path,
                &Into::<piet::Color>::into(color),
                options.stroke_width,
                &style,
            );
        }
    }
}

impl Composer<SmoothOptions> for PenPath {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };

        let mut full_path = kurbo::BezPath::new();
        let mut single_pos = true;
        let mut prev = self.start;

        cx.save().unwrap();

        for seg in self.segments.iter() {
            if seg.end().pos == self.start.pos {
                continue;
            } else {
                single_pos = false;
            }

            let bez_path = {
                match seg {
                    Segment::LineTo { end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let bez_path = compose_lines_variable_width(
                            &[Line {
                                start: prev.pos,
                                end: end.pos,
                            }],
                            width_start,
                            width_end,
                            prev,
                            *end,
                            options,
                        );

                        prev = *end;
                        bez_path
                    }
                    Segment::QuadBezTo { cp, end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let quadbez = QuadraticBezier {
                            start: prev.pos,
                            cp: *cp,
                            end: end.pos,
                        };
                        let n_splits = penpath::no_subsegments_for_segment_len(
                            quadbez.outline_path().perimeter(0.25),
                        )
                        .max(2);
                        let lines = quadbez.approx_with_lines(n_splits);
                        let bez_path = compose_lines_variable_width(
                            &lines,
                            width_start,
                            width_end,
                            prev,
                            *end,
                            options,
                        );

                        prev = *end;
                        bez_path
                    }
                    Segment::CubBezTo { cp1, cp2, end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let cubbez = CubicBezier {
                            start: prev.pos,
                            cp1: *cp1,
                            cp2: *cp2,
                            end: end.pos,
                        };
                        let n_splits = penpath::no_subsegments_for_segment_len(
                            cubbez.outline_path().perimeter(0.25),
                        )
                        .max(2);
                        let lines = cubbez.approx_with_lines(n_splits);
                        let bez_path = compose_lines_variable_width(
                            &lines,
                            width_start,
                            width_end,
                            prev,
                            *end,
                            options,
                        );

                        prev = *end;
                        bez_path
                    }
                }
            };

            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(bez_path.clone(), &stroke_brush, 0.2);

            full_path.extend(bez_path);
        }

        cx.fill(full_path, &Into::<piet::Color>::into(color));

        // Single element/position strokes need special treatment to be rendered
        if single_pos {
            let start_width = options
                .pressure_curve
                .apply(options.stroke_width, self.start.pressure);
            match options.tip_profile {
                TipProfile::Circular => {
                    cx.fill(
                        kurbo::Circle::new(self.start.pos.to_kurbo_point(), start_width * 0.5),
                        &Into::<piet::Color>::into(color),
                    );
                }
                TipProfile::Chisel {
                    aspect_ratio,
                    fixed_angle,
                    use_stylus_tilt,
                } => {
                    let theta = if use_stylus_tilt
                        && (self.start.orientation.abs() > 1e-4
                            || self.start.tilt_x.abs() > 1e-4
                            || self.start.tilt_y.abs() > 1e-4)
                    {
                        if self.start.orientation.abs() > 1e-4 {
                            self.start.orientation
                        } else {
                            Element::calc_orientation(self.start.tilt_x, self.start.tilt_y, None)
                        }
                    } else {
                        fixed_angle
                    };

                    let a = start_width * 0.5;
                    let b = (a / aspect_ratio.max(1.0)).max(0.1);
                    let (sin_t, cos_t) = theta.sin_cos();
                    let u = Vector2::new(cos_t, sin_t) * a;
                    let v = Vector2::new(-sin_t, cos_t) * b;
                    let p = self.start.pos;

                    let mut dab_path = kurbo::BezPath::new();
                    dab_path.move_to((p - u - v).to_kurbo_point());
                    dab_path.line_to((p + u - v).to_kurbo_point());
                    dab_path.line_to((p + u + v).to_kurbo_point());
                    dab_path.line_to((p - u + v).to_kurbo_point());
                    dab_path.close_path();

                    cx.fill(dab_path, &Into::<piet::Color>::into(color));
                }
            }
        }

        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for crate::Shape {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        match self {
            crate::Shape::Arrow(arrow) => arrow.composed_bounds(options),
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
            crate::Shape::Polyline(polyline) => polyline.composed_bounds(options),
            crate::Shape::Polygon(polygon) => polygon.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match self {
            crate::Shape::Arrow(arrow) => arrow.draw_composed(cx, options),
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.draw_composed(cx, options),
            crate::Shape::CubicBezier(cubbez) => cubbez.draw_composed(cx, options),
            crate::Shape::Polyline(polyline) => polyline.draw_composed(cx, options),
            crate::Shape::Polygon(polygon) => polygon.draw_composed(cx, options),
        }
    }
}

/// Composes lines with variable width. Must be drawn with only a fill.
fn compose_lines_variable_width(
    lines: &[Line],
    start_width: f64,
    end_width: f64,
    start_elem: Element,
    end_elem: Element,
    options: &SmoothOptions,
) -> kurbo::BezPath {
    // The lines variable is ghosted here, to make sure we can only use the filtered
    let lines = lines
        .iter()
        .filter(|line| (line.end - line.start).length() > 0.0)
        .collect::<Vec<&Line>>();
    let n_lines = lines.len();
    if n_lines == 0 {
        return kurbo::BezPath::new();
    }

    let compute_offset = |dir_orth_unit: Vector2, half_width: f64, t: f64| -> Vector2 {
        match options.tip_profile {
            TipProfile::Circular => dir_orth_unit * half_width,
            TipProfile::Chisel {
                aspect_ratio,
                fixed_angle,
                use_stylus_tilt,
            } => {
                // Determine nib angle
                let theta = if use_stylus_tilt
                    && (start_elem.orientation.abs() > 1e-4
                        || end_elem.orientation.abs() > 1e-4
                        || start_elem.tilt_x.abs() > 1e-4
                        || end_elem.tilt_x.abs() > 1e-4
                        || start_elem.tilt_y.abs() > 1e-4
                        || end_elem.tilt_y.abs() > 1e-4)
                {
                    let angle_start = if start_elem.orientation.abs() > 1e-4 {
                        start_elem.orientation
                    } else {
                        Element::calc_orientation(start_elem.tilt_x, start_elem.tilt_y, None)
                    };
                    let angle_end = if end_elem.orientation.abs() > 1e-4 {
                        end_elem.orientation
                    } else {
                        Element::calc_orientation(end_elem.tilt_x, end_elem.tilt_y, None)
                    };
                    let diff = (angle_end - angle_start).rem_euclid(2.0 * std::f64::consts::PI);
                    let diff = if diff > std::f64::consts::PI {
                        diff - 2.0 * std::f64::consts::PI
                    } else {
                        diff
                    };
                    angle_start + diff * t
                } else {
                    fixed_angle
                };

                let a = half_width; // major semi-axis
                let b = (half_width / aspect_ratio.max(1.0)).max(0.1); // minor semi-axis

                // Unit normal vector (dir_orth_unit) in world coordinates
                let nx = dir_orth_unit.x;
                let ny = dir_orth_unit.y;

                // Rotate normal into nib coordinate frame (by -theta)
                let (sin_t, cos_t) = theta.sin_cos();
                let n_prime_x = nx * cos_t + ny * sin_t;
                let n_prime_y = -nx * sin_t + ny * cos_t;

                // Support vector on ellipse in nib coordinate frame:
                // S'(n') = (a^2 * n'_x, b^2 * n'_y) / sqrt(a^2 * n'_x^2 + b^2 * n'_y^2)
                let a2 = a * a;
                let b2 = b * b;
                let denom = (a2 * n_prime_x * n_prime_x + b2 * n_prime_y * n_prime_y).sqrt();

                if denom < 1e-6 {
                    dir_orth_unit * half_width
                } else {
                    let delta_prime_x = (a2 * n_prime_x) / denom;
                    let delta_prime_y = (b2 * n_prime_y) / denom;

                    // Rotate offset back into world coordinates (by +theta)
                    let delta_x = delta_prime_x * cos_t - delta_prime_y * sin_t;
                    let delta_y = delta_prime_x * sin_t + delta_prime_y * cos_t;

                    Vector2::new(delta_x, delta_y)
                }
            }
        }
    };

    let (pos_offset_coords, neg_offset_coords): (Vec<_>, Vec<_>) = lines
        .iter()
        .enumerate()
        .flat_map(|(i, line)| {
            let t_start = f64::from(i as i32) / f64::from(n_lines as u32);
            let t_end = f64::from(i as i32 + 1) / f64::from(n_lines as u32);

            let line_start_width = start_width + (end_width - start_width) * t_start;
            let line_end_width = start_width + (end_width - start_width) * t_end;

            let dir_orth_unit = (line.end - line.start).orth_unit();

            let offset_start = compute_offset(dir_orth_unit, line_start_width * 0.5, t_start);
            let offset_end = compute_offset(dir_orth_unit, line_end_width * 0.5, t_end);

            [
                (line.start + offset_start, line.start - offset_start),
                (line.end + offset_end, line.end - offset_end),
            ]
        })
        .unzip();

    let first_line = lines.first().unwrap();
    let last_line = lines.last().unwrap();
    let start_dir_unit = (first_line.end - first_line.start).normalize();
    let end_dir_unit = (last_line.end - last_line.start).normalize();
    let start_pos_offset_coord = pos_offset_coords.first().unwrap().to_owned();
    let end_pos_offset_coord = pos_offset_coords.last().unwrap().to_owned();
    let start_neg_offset_coord = neg_offset_coords.first().unwrap().to_owned();
    let end_neg_offset_coord = neg_offset_coords.last().unwrap().to_owned();

    let mut bez_path = kurbo::BezPath::new();

    // Start cap
    if start_width > 0.0 && start_pos_offset_coord != start_neg_offset_coord {
        if options.tip_profile.is_chisel() || options.line_cap == LineCap::Straight {
            bez_path.move_to(start_neg_offset_coord.to_kurbo_point());
            bez_path.line_to(start_pos_offset_coord.to_kurbo_point());
        } else {
            bez_path.move_to(start_neg_offset_coord.to_kurbo_point());
            bez_path.curve_to(
                (start_neg_offset_coord - start_dir_unit * start_width * (2.0 / 3.0)).to_kurbo_point(),
                (start_pos_offset_coord - start_dir_unit * start_width * (2.0 / 3.0)).to_kurbo_point(),
                start_pos_offset_coord.to_kurbo_point(),
            );
        }
    } else {
        bez_path.move_to(start_pos_offset_coord.to_kurbo_point());
    }

    // Positive offset path
    bez_path.extend(
        pos_offset_coords
            .into_iter()
            .map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())),
    );

    // End cap
    if end_width > 0.0 && end_pos_offset_coord != end_neg_offset_coord {
        if options.tip_profile.is_chisel() || options.line_cap == LineCap::Straight {
            bez_path.line_to(end_pos_offset_coord.to_kurbo_point());
            bez_path.line_to(end_neg_offset_coord.to_kurbo_point());
        } else {
            bez_path.curve_to(
                (end_pos_offset_coord + end_dir_unit * end_width * (2.0 / 3.0)).to_kurbo_point(),
                (end_neg_offset_coord + end_dir_unit * end_width * (2.0 / 3.0)).to_kurbo_point(),
                end_neg_offset_coord.to_kurbo_point(),
            );
        }
    } else {
        bez_path.line_to(end_neg_offset_coord.to_kurbo_point());
    }

    // Negative offset path (needs to be reversed)
    bez_path.extend(
        neg_offset_coords
            .into_iter()
            .rev()
            .map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())),
    );
    bez_path.close_path();

    bez_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chisel_perpendicular_vs_parallel() {
        let mut options = SmoothOptions::default();
        options.stroke_width = 10.0;
        options.tip_profile = TipProfile::Chisel {
            aspect_ratio: 5.0,
            fixed_angle: 0.0, // horizontal nib
            use_stylus_tilt: false,
        };

        let start_elem = Element::new(Vector2::ZERO, 1.0);
        let end_elem = Element::new(Vector2::new(0.0, 100.0), 1.0);

        // Vertical stroke (perpendicular to horizontal nib angle 0)
        let vertical_line = Line {
            start: Vector2::new(0.0, 0.0),
            end: Vector2::new(0.0, 100.0),
        };
        let path_vertical = compose_lines_variable_width(
            &[vertical_line],
            10.0,
            10.0,
            start_elem,
            end_elem,
            &options,
        );
        let v_bounds = path_vertical.bounding_box();
        let v_width = v_bounds.width();
        // Since vertical stroke is perpendicular to horizontal nib, width across X should be ~ 10.0
        assert!((v_width - 10.0).abs() < 1e-1, "Vertical stroke width was {v_width}, expected ~10.0");

        // Horizontal stroke (parallel to horizontal nib angle 0)
        let horizontal_line = Line {
            start: Vector2::new(0.0, 0.0),
            end: Vector2::new(100.0, 0.0),
        };
        let path_horizontal = compose_lines_variable_width(
            &[horizontal_line],
            10.0,
            10.0,
            start_elem,
            Element::new(Vector2::new(100.0, 0.0), 1.0),
            &options,
        );
        let h_bounds = path_horizontal.bounding_box();
        let h_height = h_bounds.height();
        // Since horizontal stroke is parallel to horizontal nib, thickness across Y should be ~ 10.0 / 5.0 = 2.0
        assert!((h_height - 2.0).abs() < 1e-1, "Horizontal stroke thickness was {h_height}, expected ~2.0");

        // Verify the 5:1 ratio: perpendicular width is 5x parallel thickness
        assert!((v_width / h_height - 5.0).abs() < 0.2);
    }

    #[test]
    fn test_chisel_stylus_tilt_responsive() {
        let mut options = SmoothOptions::default();
        options.stroke_width = 10.0;
        options.tip_profile = TipProfile::Chisel {
            aspect_ratio: 4.0,
            fixed_angle: 0.0,
            use_stylus_tilt: true,
        };

        // When stylus is oriented vertically (orientation = PI/2, e.g. 90 deg)
        let start_elem = Element::new_with_tilt(
            Vector2::ZERO,
            1.0,
            0.0,
            10.0,
            std::f64::consts::FRAC_PI_2,
        );
        let end_elem = Element::new_with_tilt(
            Vector2::new(100.0, 0.0),
            1.0,
            0.0,
            10.0,
            std::f64::consts::FRAC_PI_2,
        );

        // Horizontal stroke is now PERPENDICULAR to the vertically-held nib!
        let line = Line {
            start: Vector2::new(0.0, 0.0),
            end: Vector2::new(100.0, 0.0),
        };
        let path = compose_lines_variable_width(
            &[line],
            10.0,
            10.0,
            start_elem,
            end_elem,
            &options,
        );
        let bounds = path.bounding_box();
        // The stroke should be full width (~10.0) along Y because nib is vertical!
        assert!((bounds.height() - 10.0).abs() < 1e-1, "Height was {}, expected ~10.0", bounds.height());
    }
}
