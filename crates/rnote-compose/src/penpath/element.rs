// Imports
use crate::Transformable;
use crate::ext::DPose2Ext;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// A pen input element.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename = "element")]
pub struct Element {
    #[serde(rename = "pos", with = "crate::serialize::glam_vector2_dp3")]
    /// The position of the element.
    pub pos: Vector2,
    #[serde(rename = "pressure", with = "crate::serialize::f64_dp3")]
    /// The pen pressure. The valid range is [0.0, 1.0].
    pub pressure: f64,
    #[serde(default, rename = "tilt_x", with = "crate::serialize::f64_dp3")]
    /// The pen tilt in the x-axis in degrees [-90.0, 90.0].
    pub tilt_x: f64,
    #[serde(default, rename = "tilt_y", with = "crate::serialize::f64_dp3")]
    /// The pen tilt in the y-axis in degrees [-90.0, 90.0].
    pub tilt_y: f64,
    #[serde(default, rename = "orientation", with = "crate::serialize::f64_dp3")]
    /// The pen orientation/rotation angle in radians.
    pub orientation: f64,
}

impl Default for Element {
    fn default() -> Self {
        Self::new(Vector2::ZERO, Self::PRESSURE_DEFAULT)
    }
}

impl Transformable for Element {
    fn translate(&mut self, offset: Vector2) {
        self.pos += offset;
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::from_rotation_wrt_center(angle, center);
        self.pos = pose.transform_point(self.pos);
        self.orientation = (self.orientation + angle).rem_euclid(2.0 * std::f64::consts::PI);
    }

    fn scale(&mut self, scale: Vector2) {
        self.pos *= scale;
    }
}

impl Element {
    /// The default fallback pen pressure, when it could not be retrieved from the input.
    pub const PRESSURE_DEFAULT: f64 = 0.5;
    /// The default fallback pen tilt.
    pub const TILT_DEFAULT: f64 = 0.0;
    /// The default fallback pen orientation in radians.
    pub const ORIENTATION_DEFAULT: f64 = 0.0;

    /// A new element from a position and pressure.
    pub fn new(pos: Vector2, pressure: f64) -> Self {
        Self {
            pos,
            pressure: pressure.clamp(0.0, 1.0),
            tilt_x: Self::TILT_DEFAULT,
            tilt_y: Self::TILT_DEFAULT,
            orientation: Self::ORIENTATION_DEFAULT,
        }
    }

    /// A new element with position, pressure, tilt and orientation.
    pub fn new_with_tilt(
        pos: Vector2,
        pressure: f64,
        tilt_x: f64,
        tilt_y: f64,
        orientation: f64,
    ) -> Self {
        Self {
            pos,
            pressure: pressure.clamp(0.0, 1.0),
            tilt_x,
            tilt_y,
            orientation,
        }
    }

    /// Helper to compute orientation angle in radians from tilt_x, tilt_y, or optional rotation.
    pub fn calc_orientation(tilt_x: f64, tilt_y: f64, rotation: Option<f64>) -> f64 {
        if let Some(rot) = rotation {
            rot.to_radians()
        } else if tilt_x.abs() > 1e-4 || tilt_y.abs() > 1e-4 {
            tilt_y.atan2(tilt_x)
        } else {
            0.0
        }
    }

    /// Sets the pressure, clamped to the range [0.0 - 1.0].
    pub fn set_pressure_clamped(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }

    /// Indicates if a element is out of valid bounds and should be filtered out.
    ///
    /// Returns true if element pos is not inside the bounds.
    pub fn filter_by_bounds(&self, filter_bounds: Aabb) -> bool {
        !filter_bounds.contains_local_point(self.pos)
    }

    /// Transforms the element position by the given transform.
    pub fn transform_by(&mut self, transform: DAffine2) {
        self.pos = transform.transform_point2(self.pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_defaults_and_backward_compatibility() {
        // Test deserialization from older json representation lacking tilt fields
        let json_old = r#"{"pos":[10.0, 20.0],"pressure":0.7}"#;
        let elem: Element = serde_json::from_str(json_old).expect("deserialization failed");
        assert_eq!(elem.pos, Vector2::new(10.0, 20.0));
        assert!((elem.pressure - 0.7).abs() < 1e-6);
        assert_eq!(elem.tilt_x, 0.0);
        assert_eq!(elem.tilt_y, 0.0);
        assert_eq!(elem.orientation, 0.0);

        // Test serialization and deserialization with tilt and orientation
        let elem_tilt = Element::new_with_tilt(Vector2::new(15.0, 25.0), 0.8, 30.0, -45.0, 1.25);
        let serialized = serde_json::to_string(&elem_tilt).expect("serialization failed");
        let deserialized: Element = serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(deserialized.pos, Vector2::new(15.0, 25.0));
        assert!((deserialized.pressure - 0.8).abs() < 1e-3);
        assert!((deserialized.tilt_x - 30.0).abs() < 1e-3);
        assert!((deserialized.tilt_y - (-45.0)).abs() < 1e-3);
        assert!((deserialized.orientation - 1.25).abs() < 1e-3);
    }

    #[test]
    fn test_orientation_calc() {
        // When rotation is supplied (e.g. 90 degrees), it is converted to radians
        let rot_deg = 90.0;
        let orient = Element::calc_orientation(0.0, 0.0, Some(rot_deg));
        assert!((orient - std::f64::consts::FRAC_PI_2).abs() < 1e-6);

        // From tilt_x and tilt_y (e.g. 45 degrees angle)
        let orient_tilt = Element::calc_orientation(10.0, 10.0, None);
        assert!((orient_tilt - std::f64::consts::FRAC_PI_4).abs() < 1e-6);
    }
}
