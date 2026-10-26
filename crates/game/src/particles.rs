//! Particles: `CreateParticle` and `ControlParticles` (`RTWP/particles.bb`).
//!
//! `logic` draws the random numbers the original spends on particles but
//! keeps no particles; it reports where effects happen
//! ([`Event::Effect`], [`Event::PlayerExploded`]). This plugin creates the
//! particles with its own random numbers, plus the ones that follow from
//! state alone (fireball trails, snow and rain), and moves them once per
//! tick as the original does.
//!
//! Sprites (type 0) face the camera. The texture is a frame of
//! `ParticleTexture` (`res21.dat`, 32x32 frames 16 to a row) or, from 100,
//! `ParticleTexture2` (`res22.dat`, 4 to a row). The water ripples (type 1,
//! flat squares) are not drawn.

use bevy::math::Affine2;
use bevy::prelude::*;
use wonderland_logic::{Effect, Event};

use crate::camera::GameCamera;
use crate::coords::vec;
use crate::level::GameData;
use crate::scene::{LevelEntity, TextureCache};
use crate::sim::{Sim, SimStep};
use crate::style::TextureRef;

/// One particle (the `particle` type).
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Particle {
    /// Blitz position.
    pub at: [f32; 3],
    pub velocity: [f32; 3],
    /// Gravity, taken off the vertical speed each tick.
    pub ddy: f32,
    /// Sprite roll and its change per tick, in degrees.
    pub roll: f32,
    pub spin: f32,
    pub timer: i32,
    pub lifetime: i32,
    pub size: f32,
}

impl Particle {
    /// One tick of `ControlParticles`; false when it is gone.
    pub fn tick(&mut self) -> bool {
        for i in 0..3 {
            self.at[i] += self.velocity[i];
        }
        self.velocity[1] -= self.ddy;
        if self.spin != 0.0 {
            self.roll += self.spin;
        }
        self.timer -= 1;
        self.timer > 0 && self.at[1] >= -0.1
    }

    pub fn alpha(&self) -> f32 {
        self.timer as f32 / self.lifetime.max(1) as f32
    }
}

/// A small generator for the renderer's own random numbers.
#[derive(Resource)]
struct Dice(u64);

impl Dice {
    fn next(&mut self) -> f32 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 40) as f32 / (1u64 << 24) as f32
    }

    /// `Rnd(a, b)`.
    fn rnd(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }

    /// `Rand(a, b)`.
    fn rand(&mut self, a: i32, b: i32) -> i32 {
        a + ((b - a + 1) as f32 * self.next()) as i32
    }
}

/// A particle to create: `CreateParticle(0, tex, size, ...)`.
struct Spec {
    tex: i32,
    size: f32,
    at: [f32; 3],
    velocity: [f32; 3],
    spin: f32,
    ddy: f32,
    timer: i32,
}

/// The particles one effect makes, as the original's `CreateParticle`
/// calls.
fn effect_particles(effect: Effect, at: [f32; 3], detail: i32, dice: &mut Dice) -> Vec<Spec> {
    let mut out = Vec::new();
    let mut burst = |out: &mut Vec<Spec>,
                     n: i32,
                     tex: i32,
                     size: f32,
                     h: f32,
                     up: (f32, f32),
                     spin: f32,
                     ddy: f32,
                     timer: i32| {
        for _ in 0..n {
            out.push(Spec {
                tex,
                size,
                at,
                velocity: [dice.rnd(-h, h), dice.rnd(up.0, up.1), dice.rnd(-h, h)],
                spin: dice.rnd(-spin, spin),
                ddy,
                timer,
            });
        }
    };
    match effect {
        Effect::Splinters(kind) => {
            // `KillBox`: wood (100), steel (0) or the rest (101).
            let tex = match kind {
                0 | 6 => 100,
                1 => 0,
                _ => 101,
            };
            burst(
                &mut out,
                7 * detail,
                tex,
                0.15,
                0.04,
                (0.01, 0.06),
                10.0,
                0.001,
                60 * detail,
            );
        }
        Effect::Burst(kind) => match kind {
            5 | 6 => burst(
                &mut out,
                3 * detail,
                0,
                0.2,
                0.04,
                (0.01, 0.06),
                10.0,
                0.001,
                60 * detail,
            ),
            12 => {
                for _ in 0..7 {
                    burst(
                        &mut out,
                        2 * detail,
                        0,
                        0.2,
                        0.14,
                        (0.01, 0.06),
                        10.0,
                        0.0005,
                        120 * detail,
                    );
                    burst(
                        &mut out,
                        2 * detail,
                        5,
                        0.1,
                        0.26,
                        (0.08, 0.2),
                        30.0,
                        0.002,
                        120 * detail,
                    );
                }
            }
            1 | 2 | 11 => burst(
                &mut out,
                14 * detail,
                5,
                0.1,
                0.16,
                (0.08, 0.2),
                30.0,
                0.004,
                60 * detail,
            ),
            _ => {
                burst(
                    &mut out,
                    7 * detail,
                    0,
                    0.2,
                    0.04,
                    (0.01, 0.06),
                    10.0,
                    0.001,
                    60 * detail,
                );
                burst(
                    &mut out,
                    7 * detail,
                    5,
                    0.2,
                    0.06,
                    (0.03, 0.09),
                    10.0,
                    0.002,
                    60 * detail,
                );
            }
        },
        Effect::ExitStar => out.push(Spec {
            tex: 3,
            size: 0.1,
            at,
            velocity: [
                dice.rnd(-0.01, 0.01),
                dice.rnd(0.02, 0.035),
                dice.rnd(-0.01, 0.01),
            ],
            spin: dice.rnd(-5.0, 5.0),
            ddy: 0.0005,
            timer: 180,
        }),
        Effect::CoinSparkle => out.push(Spec {
            tex: 3,
            size: 0.3,
            at,
            velocity: [0.0, 0.02, 0.0],
            spin: dice.rnd(4.0, 8.0),
            ddy: 0.0,
            timer: 120,
        }),
        Effect::BonusSparkle => out.push(Spec {
            tex: 9,
            size: 0.5,
            at,
            velocity: [0.0, 0.02, 0.0],
            spin: 0.0,
            ddy: 0.0,
            timer: 120,
        }),
    }
    out
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Dice(0x9E37_79B9_7F4A_7C15))
            .add_systems(
                Update,
                (spawn_particles, move_particles).chain().after(SimStep),
            );
    }
}

/// The texture and frame transform for particle texture `tex`.
fn frame(tex: i32) -> (TextureRef, Affine2) {
    let (path, flags, per_row, f) = if tex < 100 {
        ("res21.dat", 2, 16.0, tex)
    } else {
        ("res22.dat", 4, 4.0, tex - 100)
    };
    let f = f.max(0) as f32;
    let col = f % per_row;
    let row = (f / per_row).floor();
    let r = TextureRef {
        path: path.into(),
        flags,
        rotation: 0,
    };
    let t = Affine2::from_scale_angle_translation(
        Vec2::splat(1.0 / per_row),
        0.0,
        Vec2::new(col / per_row, row / per_row),
    );
    (r, t)
}

#[allow(clippy::too_many_arguments)]
fn spawn_particles(
    mut commands: Commands,
    sim: Res<Sim>,
    data: Res<GameData>,
    mut dice: ResMut<Dice>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
    existing: Query<(), With<Particle>>,
    mut quad: Local<Option<Handle<Mesh>>>,
) {
    let w = &sim.world;
    let detail = w.options.detail_level;
    let mut specs = Vec::new();
    for e in &sim.events {
        match *e {
            Event::Effect { effect, at } => {
                specs.extend(effect_particles(effect, at, detail, &mut dice));
            }
            Event::PlayerExploded(k) => {
                let at = w.players[k].position;
                for _ in 0..=30 * detail {
                    let angle = dice.rnd(0.0, 359.0).to_radians();
                    let speed = dice.rnd(0.03, 0.05);
                    specs.push(Spec {
                        tex: dice.rand(5, 8),
                        size: 0.2,
                        at,
                        velocity: [
                            speed * angle.sin(),
                            dice.rnd(-0.001, 0.001),
                            speed * angle.cos(),
                        ],
                        spin: 3.0,
                        ddy: 0.0,
                        timer: 200,
                    });
                }
            }
            _ => {}
        }
    }
    // Per tick: fireball trails (every 6 ticks of a fireball's move, twice
    // a tick) and the weather.
    for t in 0..sim.ticks as i32 {
        let timer = w.level_timer - sim.ticks as i32 + t;
        if timer % 3 == 0 {
            for f in w.fireballs() {
                if !f.dead {
                    specs.push(Spec {
                        tex: 0,
                        size: 0.2,
                        at: f.position,
                        velocity: [0.0, dice.rnd(-0.01, 0.01), dice.rnd(-0.01, 0.01)],
                        spin: dice.rnd(-1.0, 1.0),
                        ddy: 0.0,
                        timer: w.ruleset.base_speed * 2,
                    });
                }
            }
        }
        let d = 5 - detail;
        if timer % (1 + d * d) == 0 {
            let (width, height) = (w.level.grid.width() as f32, w.level.grid.height() as f32);
            if w.level.weather.snow {
                specs.push(Spec {
                    tex: 10,
                    size: 0.05,
                    at: [
                        dice.rnd(-2.0, width + 2.0),
                        dice.rnd(5.0, 10.0),
                        -dice.rnd(-2.0, height + 2.0),
                    ],
                    velocity: [0.01, -dice.rnd(0.02, 0.03), -0.01],
                    spin: dice.rnd(-0.5, 0.25),
                    ddy: 0.0,
                    timer: 10000,
                });
            }
            if w.level.weather.rain {
                let [cx, _, cz] = w.camera.position;
                specs.push(Spec {
                    tex: 11,
                    size: 0.2,
                    at: [
                        cx + dice.rnd(-10.0, 10.0),
                        dice.rnd(5.0, 10.0),
                        cz + dice.rnd(1.0, 10.0),
                    ],
                    velocity: [0.0, -0.15, 0.0],
                    spin: 0.0,
                    ddy: 0.0,
                    timer: 10000,
                });
            }
        }
    }
    if specs.is_empty() {
        return;
    }
    let quad = quad
        .get_or_insert_with(|| meshes.add(Rectangle::new(2.0, 2.0)))
        .clone();
    // `If NofParticles >= 100 * DetailLevel Then Return`.
    let room = usize::try_from(100 * detail)
        .unwrap_or(0)
        .saturating_sub(existing.iter().count());
    for s in specs.into_iter().take(room) {
        let (r, uv) = frame(s.tex);
        let (image, _) = cache.get(&r, &data, &mut images);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            uv_transform: uv,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(vec(s.at)).with_scale(Vec3::splat(s.size)),
            Particle {
                at: s.at,
                velocity: s.velocity,
                ddy: s.ddy,
                roll: 0.0,
                spin: s.spin,
                timer: s.timer,
                lifetime: s.timer,
                size: s.size,
            },
            LevelEntity,
        ));
    }
}

fn move_particles(
    mut commands: Commands,
    sim: Res<Sim>,
    camera: Query<&Transform, (With<GameCamera>, Without<Particle>)>,
    mut particles: Query<(
        Entity,
        &mut Particle,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let facing = camera.single().map_or(Quat::IDENTITY, |c| c.rotation);
    for (e, mut p, mut t, mat) in &mut particles {
        let mut alive = true;
        for _ in 0..sim.ticks {
            if !p.tick() {
                alive = false;
                break;
            }
        }
        if !alive {
            materials.remove(&mat.0);
            commands.entity(e).despawn();
            continue;
        }
        t.translation = vec(p.at);
        t.rotation = facing * Quat::from_rotation_z(p.roll.to_radians());
        if sim.ticks > 0
            && let Some(mut m) = materials.get_mut(&mat.0)
        {
            m.base_color = Color::srgba(1.0, 1.0, 1.0, p.alpha());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_fall_and_fade() {
        let mut p = Particle {
            at: [0.0, 1.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            ddy: 0.01,
            roll: 0.0,
            spin: 2.0,
            timer: 100,
            lifetime: 100,
            size: 0.2,
        };
        let mut ticks = 0;
        while p.tick() {
            ticks += 1;
        }
        // The speed is applied before gravity, so after n ticks it has
        // fallen 0.01 * n (n - 1) / 2: below -0.1 on tick 16.
        assert_eq!(ticks, 15);
        assert!(p.alpha() < 1.0);
    }

    #[test]
    fn frames_come_from_the_right_row() {
        let (r, t) = frame(5);
        assert_eq!(r.path, "res21.dat");
        let corner = t.transform_point2(Vec2::ZERO);
        assert!((corner.x - 5.0 / 16.0).abs() < 1e-6 && corner.y == 0.0);
        let (r, t) = frame(101);
        assert_eq!(r.path, "res22.dat");
        assert!((t.transform_point2(Vec2::ZERO).x - 0.25).abs() < 1e-6);
    }

    #[test]
    fn effects_scale_with_detail() {
        let mut dice = Dice(1);
        let n = |e, d, dice: &mut Dice| effect_particles(e, [0.0; 3], d, dice).len();
        assert_eq!(n(Effect::Splinters(0), 3, &mut dice), 21);
        assert_eq!(n(Effect::Burst(3), 2, &mut dice), 28);
        assert_eq!(n(Effect::ExitStar, 5, &mut dice), 1);
    }
}
