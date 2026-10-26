//! Fish and pillars: `CreateFish`, `ControlFish` and `CreatePillar`
//! (`RTWP/gameobject.bb`). They are scenery, but both draw random numbers
//! when created, so they are created here in load order.

use crate::World;

/// One fish circling in the water (the `Fish*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fish {
    pub x: i32,
    pub y: i32,
    /// `FishSpeed`: degrees per tick, `Rnd(0.5, 1.5)`, negated half the
    /// time (the fish then faces the other way).
    pub speed: f32,
    /// `FishTimer`: the angle round its circle.
    pub timer: f32,
    /// The entity's yaw, turned by `speed` each tick (plus 180 for a
    /// negative speed).
    pub yaw: f32,
}

impl Fish {
    /// The entity position (x, up, -row) `ControlFish` sets.
    pub fn position(&self) -> [f32; 3] {
        let t = self.timer.to_radians();
        [
            self.x as f32 + 0.75 + 1.5 * t.sin(),
            -0.5,
            -(self.y as f32 + 0.4) + 1.5 * t.cos(),
        ]
    }
}

/// One pillar, house or statue (the `Pillar*` arrays).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pillar {
    pub x: i32,
    pub y: i32,
    /// `PillarType`: the mesh (`PillarMesh(kind)`); 8–11 are houses facing
    /// four ways, 23, 24 and 30 player statues.
    pub kind: i32,
    /// The yaw a flower (22) is turned by, `Rand(-180, 180)`; 0 otherwise.
    pub flower_yaw: i32,
}

impl World {
    /// `CreateFish`.
    pub(crate) fn create_fish(&mut self, x: i32, y: i32) {
        let mut speed = self.session.rng.rnd(0.5, 1.5);
        let mut yaw = 0.0;
        if self.session.rng.rand(0, 100) < 50 {
            speed = -speed;
            yaw = 180.0;
        }
        self.fish.push(Fish {
            x,
            y,
            speed,
            timer: 0.0,
            yaw,
        });
    }

    /// `ControlFish`. The original tests the speed of the slot past the
    /// last fish (`FishSpeed(NofFish)`), which is 0 unless an earlier level
    /// had more fish, so every fish takes the second branch: a fish with a
    /// positive speed circles backwards and its timer never wraps.
    pub(crate) fn control_fish(&mut self) {
        for f in &mut self.fish {
            f.timer -= f.speed;
            if f.timer >= 360.0 {
                f.timer -= 360.0;
            }
            f.yaw += f.speed;
        }
    }

    /// `CreatePillar`. Kinds 12–15 set the house style instead.
    pub(crate) fn create_pillar(&mut self, x: i32, y: i32, kind: i32) {
        if (12..16).contains(&kind) {
            self.house_style = kind - 11;
            return;
        }
        let flower_yaw = if kind == 22 {
            self.session.rng.rand(-180, 180)
        } else {
            0
        };
        self.pillars.push(Pillar {
            x,
            y,
            kind,
            flower_yaw,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::setup::tests::level;
    use crate::{BlitzRng, Options, Ruleset, Session, World};

    #[test]
    fn fish_and_flowers_draw_in_load_order() {
        // v5 codes: 48 fish; 50 + kind for pillars (72 = flower 22,
        // 63 = house style 2).
        let l = level(5, 4, &[100, 300, 100, 100], &[1, 48, 72, 63]);
        let w = World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        let mut rng = BlitzRng::default();
        let mut speed = rng.rnd(0.5, 1.5);
        if rng.rand(0, 100) < 50 {
            speed = -speed;
        }
        assert_eq!(w.fish[0].speed, speed);
        assert_eq!(w.pillars.len(), 1);
        assert_eq!(w.pillars[0].flower_yaw, rng.rand(-180, 180));
        assert_eq!(w.house_style, 2);
    }
}
