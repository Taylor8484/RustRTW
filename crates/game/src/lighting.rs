//! Blitz's lighting: one directional light (`LevelLight`, created in
//! `Setup` at pitch 25, yaw -25) plus the ambient light, both driven per
//! tick by `logic::Lights`. Blitz lights per vertex with Direct3D 7's fixed
//! pipeline: ambient + light × max(0, N·L), each channel clamped to 0–1,
//! times the texture. The light colour may be negative (the level fades in
//! from -255), which darkens.
//!
//! Level surfaces are flat, so the factor is the same over a whole face and
//! the scene sets it as the material's colour (an unlit material, so Bevy's
//! own lighting stays out of it). Blitz multiplies in gamma space, so the
//! factor is given to Bevy as an sRGB colour.

use wonderland_logic::world::Lights;

use crate::coords::blitz_axes;

/// The direction `LevelLight` shines in, in Blitz space.
pub fn light_direction() -> [f32; 3] {
    blitz_axes(25.0, -25.0).0
}

/// The colour factor (0–1 per channel) of a face with Blitz-space `normal`.
pub fn factor(lights: &Lights, normal: [f32; 3]) -> [f32; 3] {
    let d = light_direction();
    let lambert = -(normal[0] * d[0] + normal[1] * d[1] + normal[2] * d[2]);
    let lambert = lambert.max(0.0);
    std::array::from_fn(|c| {
        let v = lights.ambient[c] as f32 + lights.light[c] as f32 * lambert;
        (v / 255.0).clamp(0.0, 1.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lights(light: i32, ambient: i32) -> Lights {
        Lights {
            light: [light; 3],
            light_target: [0; 3],
            light_speed: 0,
            ambient: [ambient; 3],
            ambient_target: [0; 3],
            ambient_speed: 0,
        }
    }

    #[test]
    fn floors_catch_part_of_the_light() {
        let up = [0.0, 1.0, 0.0];
        let f = factor(&lights(255, 0), up);
        // sin 25° of the light reaches a floor.
        assert!((f[0] - 25f32.to_radians().sin()).abs() < 1e-4);
        assert_eq!(factor(&lights(255, 255), up), [1.0; 3]);
        assert_eq!(factor(&lights(-255, 0), up), [0.0; 3]);
        // Faces turned away get the ambient only.
        assert_eq!(factor(&lights(255, 51), [0.0, -1.0, 0.0]), [0.2; 3]);
    }
}
