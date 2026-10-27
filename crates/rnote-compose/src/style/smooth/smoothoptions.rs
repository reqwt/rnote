// Imports
use crate::Color;
use crate::style::PressureCurve;
use anyhow::Context;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{
    f64,
    ops::{AddAssign, MulAssign},
};

/// Options for shapes that can be drawn in a smooth style. Ensure the precursor struct used in deserialization matches this one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename = "smooth_options")]
pub struct SmoothOptions {
    /// Stroke width.
    #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
    pub stroke_width: f64,
    /// Stroke color. When set to None, the stroke outline is not drawn.
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// Fill color. When set to None, the fill is not drawn.
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// Pressure curve.
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
    /// Line style.
    #[serde(rename = "line_style")]
    pub line_style: LineStyle,
    /// Line cap.
    #[serde(rename = "line_cap")]
    pub line_cap: LineCap,
    /// Tip profile (circular or calligraphic chisel flat nib).
    #[serde(default, rename = "tip_profile")]
    pub tip_profile: TipProfile,
    /// The inner piet::StrokeStyle, computed using the stroke_width, line_style, and line_cap.
    #[serde(skip)]
    pub piet_stroke_style: piet::StrokeStyle,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        let stroke_width: f64 = 2.0;
        let line_style = LineStyle::default();
        let line_cap = LineCap::default();
        let tip_profile = TipProfile::default();
        Self {
            stroke_width,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
            line_style,
            line_cap,
            tip_profile,
            piet_stroke_style: Self::compute_piet_stroke_style(stroke_width, line_style, line_cap),
        }
    }
}

impl SmoothOptions {
    /// The ratio between the length of a dash and the width of the stroke
    const DASH_LENGTH_TO_WIDTH_RATIO: f64 = f64::consts::E;

    fn compute_piet_stroke_style(
        stroke_width: f64,
        line_style: LineStyle,
        line_cap: LineCap,
    ) -> piet::StrokeStyle {
        let mut dash_pattern = line_style.as_unscaled_vector();
        match line_cap {
            LineCap::Straight => dash_pattern
                .iter_mut()
                .for_each(|e| e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO)),
            LineCap::Rounded => dash_pattern.iter_mut().enumerate().for_each(|(idx, e)| {
                if !line_style.is_dotted() {
                    e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO);
                }
                // If the stroke has a rounded linecap, a half-disk with radius equal to the stroke width is added both ends of a stroke, this increases the length of each line by the width of the stroke, and is not taken into account by DashStroke, it has to be manually accounted for
                if idx % 2 == 1 {
                    e.add_assign(2.0 * stroke_width)
                }
            }),
        };
        let mut stroke_style = piet::StrokeStyle::new();
        stroke_style.set_dash_pattern(dash_pattern);
        stroke_style.set_line_cap(line_cap.into());
        stroke_style
    }

    /// Updates the inner piet::Strokestyle
    pub fn update_piet_stroke_style(&mut self) {
        self.piet_stroke_style =
            Self::compute_piet_stroke_style(self.stroke_width, self.line_style, self.line_cap);
    }

    /// Updates the line cap
    pub fn update_line_cap(&mut self, line_cap: LineCap) {
        // Dotted style requires a round LineCap
        if self.line_style.is_dotted() && line_cap != LineCap::Rounded {
            self.line_style = LineStyle::Solid;
        }
        self.line_cap = line_cap;
        self.update_piet_stroke_style();
    }

    /// Updates the line style
    pub fn update_line_style(&mut self, line_style: LineStyle) {
        // Dotted style requires a round LineCap
        if line_style.is_dotted() {
            self.line_cap = LineCap::Rounded;
        }
        self.line_style = line_style;
        self.update_piet_stroke_style();
    }
}

impl<'de> Deserialize<'de> for SmoothOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(default, rename = "smooth_options")]
        struct SmoothOptionsPrecursor {
            #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
            pub stroke_width: f64,
            #[serde(rename = "stroke_color")]
            pub stroke_color: Option<Color>,
            #[serde(rename = "fill_color")]
            pub fill_color: Option<Color>,
            #[serde(rename = "pressure_curve")]
            pub pressure_curve: PressureCurve,
            #[serde(rename = "line_style")]
            pub line_style: LineStyle,
            #[serde(rename = "line_cap")]
            pub line_cap: LineCap,
            #[serde(default, rename = "tip_profile")]
            pub tip_profile: TipProfile,
        }

        impl From<SmoothOptions> for SmoothOptionsPrecursor {
            fn from(value: SmoothOptions) -> Self {
                Self {
                    stroke_width: value.stroke_width,
                    stroke_color: value.stroke_color,
                    fill_color: value.fill_color,
                    pressure_curve: value.pressure_curve,
                    line_style: value.line_style,
                    line_cap: value.line_cap,
                    tip_profile: value.tip_profile,
                }
            }
        }

        impl Default for SmoothOptionsPrecursor {
            fn default() -> Self {
                SmoothOptions::default().into()
            }
        }

        let precursor = SmoothOptionsPrecursor::deserialize(deserializer)?;

        Ok(SmoothOptions {
            stroke_width: precursor.stroke_width,
            stroke_color: precursor.stroke_color,
            fill_color: precursor.fill_color,
            pressure_curve: precursor.pressure_curve,
            line_style: precursor.line_style,
            line_cap: precursor.line_cap,
            tip_profile: precursor.tip_profile,
            piet_stroke_style: Self::compute_piet_stroke_style(
                precursor.stroke_width,
                precursor.line_style,
                precursor.line_cap,
            ),
        })
    }
}

/// The profile of the pen tip / nib.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tip_profile")]
pub enum TipProfile {
    /// Traditional circular brush tip with uniform radius.
    #[serde(rename = "circular")]
    Circular,
    /// Calligraphic chisel flat nib with anisotropic elliptical offset.
    #[serde(rename = "chisel")]
    Chisel {
        /// Ratio between the major and minor axis (width vs thickness), e.g. 5.0 (5:1).
        #[serde(default = "TipProfile::default_aspect_ratio")]
        aspect_ratio: f64,
        /// Fixed calligraphic angle in radians when stylus tilt is not used/available (default 45 deg, PI/4).
        #[serde(default = "TipProfile::default_fixed_angle")]
        fixed_angle: f64,
        /// Whether to dynamically respond to stylus tilt and rotation.
        #[serde(default = "TipProfile::default_use_stylus_tilt")]
        use_stylus_tilt: bool,
    },
}

impl Default for TipProfile {
    fn default() -> Self {
        Self::Circular
    }
}

impl TipProfile {
    /// Default aspect ratio for chisel nib (5:1 width to thickness)
    pub const fn default_aspect_ratio() -> f64 {
        5.0
    }

    /// Default fixed angle for chisel nib (45 degrees)
    pub fn default_fixed_angle() -> f64 {
        std::f64::consts::FRAC_PI_4 // 45 degrees
    }

    /// Default preference for responding to stylus tilt and orientation
    pub const fn default_use_stylus_tilt() -> bool {
        true
    }

    /// Creates a default chisel tip profile
    pub fn chisel_default() -> Self {
        Self::Chisel {
            aspect_ratio: Self::default_aspect_ratio(),
            fixed_angle: Self::default_fixed_angle(),
            use_stylus_tilt: Self::default_use_stylus_tilt(),
        }
    }

    /// Returns true if the tip profile is a chisel nib
    pub fn is_chisel(&self) -> bool {
        matches!(self, Self::Chisel { .. })
    }
}

/// Line cap present at the start and end of a line
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive,
)]
#[serde(rename = "line_cap")]
pub enum LineCap {
    /// Straight line cap
    #[default]
    #[serde(rename = "straight")]
    Straight,
    /// Rounded line cap
    #[serde(rename = "rounded")]
    Rounded,
}

impl TryFrom<u32> for LineCap {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineCap try_from::<u32>() for value {value} failed"))
    }
}

impl From<LineCap> for piet::LineCap {
    fn from(value: LineCap) -> Self {
        match value {
            LineCap::Straight => piet::LineCap::Butt,
            LineCap::Rounded => piet::LineCap::Round,
        }
    }
}

/// The overall style of the line
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive,
)]
#[serde(rename = "line_style")]
pub enum LineStyle {
    /// Solid line style
    #[default]
    #[serde(rename = "solid")]
    Solid,
    /// Dotted line style, the dots are equidistant
    #[serde(rename = "dotted")]
    Dotted,
    /// Dashed line style, the dashes have less space between them
    #[serde(rename = "dashed_narrow")]
    DashedNarrow,
    /// Dashed line style, the dashes are equidistant
    #[serde(rename = "dashed_equidistant")]
    DashedEquidistant,
    /// Dashed line style, the dashes have more space between them
    #[serde(rename = "dashed_wide")]
    DashedWide,
}

impl LineStyle {
    /// Returns the baseline (meaning unscaled) dash pattern
    fn as_unscaled_vector(&self) -> Vec<f64> {
        match self {
            Self::Solid => Vec::new(),
            Self::Dotted => vec![0.0, 0.0], // LineCap must be set to 'Rounded'
            Self::DashedNarrow => vec![1.0, 0.618], // golden ratio, the longer segment is the dash itself
            Self::DashedEquidistant => vec![1.0, 1.0],
            Self::DashedWide => vec![1.0, 1.618], // golden ratio, the longer segment is the space between dashes
        }
    }
    /// Indicates whether or not the LineStyle is dotted
    pub fn is_dotted(&self) -> bool {
        match self {
            Self::Solid => false,
            Self::Dotted => true,
            Self::DashedNarrow => false,
            Self::DashedEquidistant => false,
            Self::DashedWide => false,
        }
    }
}

impl TryFrom<u32> for LineStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineStyle try_from::<u32>() for value {value} failed"))
    }
}
