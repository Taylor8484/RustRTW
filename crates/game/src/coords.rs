//! Blitz to Bevy coordinates.
//!
//! Blitz is left-handed with y up and the camera looking along +z; Bevy is
//! right-handed with y up and the camera looking along -z. Negating z maps
//! one onto the other and shows the same picture, so a face Blitz draws
//! clockwise on screen is still clockwise. Blitz's front faces are
//! clockwise and Bevy's counter-clockwise, so every triangle's index order
//! is reversed ([`triangle`]).

use bevy::math::{Quat, Vec3};
use bevy::transform::components::Transform;

/// A Blitz position or direction in Bevy space.
pub fn vec(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[1], -p[2])
}

/// A Blitz triangle's indices in Bevy's winding.
pub fn triangle(t: [u32; 3]) -> [u32; 3] {
    [t[0], t[2], t[1]]
}

/// Blitz's `(pitch, yaw)` in degrees as forward and up vectors in Blitz
/// space. Positive pitch looks down; positive yaw turns left.
pub fn blitz_axes(pitch: f32, yaw: f32) -> ([f32; 3], [f32; 3]) {
    let (sp, cp) = pitch.to_radians().sin_cos();
    let (sy, cy) = yaw.to_radians().sin_cos();
    let forward = [-cp * sy, -sp, cp * cy];
    let up = [-sp * sy, cp, sp * cy];
    (forward, up)
}

/// The rotation of a Blitz entity turned by `(pitch, yaw)`, for Bevy.
pub fn rotation(pitch: f32, yaw: f32) -> Quat {
    let (f, u) = blitz_axes(pitch, yaw);
    Transform::IDENTITY.looking_to(vec(f), vec(u)).rotation
}

/// `CameraZoom` as Bevy's vertical field of view. Blitz keeps the
/// horizontal view fixed (zoom 1 is 90° across) and derives the vertical
/// one from the viewport's shape.
pub fn vertical_fov(zoom: f32, aspect: f32) -> f32 {
    2.0 * (1.0 / (zoom * aspect)).atan()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn camera_axes() {
        // The level-start camera looks straight down, the top of the
        // screen towards row 0 (Blitz +z, Bevy -z).
        let q = rotation(90.0, 0.0);
        assert!(close(q * Vec3::NEG_Z, Vec3::NEG_Y));
        assert!(close(q * Vec3::Y, Vec3::NEG_Z));
        // Yaw 0 looks along Blitz +z, which is Bevy -z.
        let q = rotation(0.0, 0.0);
        assert!(close(q * Vec3::NEG_Z, Vec3::NEG_Z));
        // Positive yaw turns left (to -x).
        let q = rotation(0.0, 90.0);
        assert!(close(q * Vec3::NEG_Z, Vec3::NEG_X));
    }

    #[test]
    fn zoom_one_is_ninety_degrees_across() {
        let v = vertical_fov(1.0, 1.0);
        assert!((v - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert!(vertical_fov(2.0, 4.0 / 3.0) < v);
    }
}
