//! The objects: players, boxes, monsters, items and the rest, drawn from
//! the `logic::World` each frame.
//!
//! [`draws`] turns the world into a list of [`Draw`]s (a model, where, how
//! turned and scaled, which MD2 frame), following the `Create*` and
//! `Control*` functions' entity calls. It is plain code, tested without a
//! GPU. The plugin keeps a pool of entities per model and gives the k-th
//! draw of a model to the k-th entity, so nothing is spawned while a level
//! runs once the pools have grown.
//!
//! Blitz lights objects per vertex, the same way as the level
//! ([`crate::lighting`]). Each entity has its own mesh whose vertex colours
//! hold the light factor for its normals as currently turned; they are
//! recomputed when the turn, frame or lights change. The materials are
//! unlit, so Bevy's lighting stays out of it.
//!
//! Visual state the logic does not keep (spins, pulses) is derived from
//! `level_timer`, which is how the original accumulates it for objects
//! created at level start.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::asset::RenderAssetUsages;
use bevy::math::Affine2;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use wonderland_logic::player::Skin;
use wonderland_logic::world::Lights;
use wonderland_logic::{Spawn, World};

use crate::coords::{triangle, vec};
use crate::flow::Phase;
use crate::level::{GameData, LevelData};
use crate::lighting::light_direction;
use crate::model::{Geometry, Model, ModelId, blitz_quat, load, player_skins};
use crate::scene::{LevelEntity, TextureCache};
use crate::sim::{Sim, SimStep, TickAnim};
use crate::style::{TextureRef, effective_background, effective_style};

/// How a draw's texture coordinates differ from its model's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Uv {
    Model,
    /// The model's 0–1 square mapped onto this part of the texture
    /// (`u0, v0, u1, v1`), for animation frames and atlases.
    Rect([f32; 4]),
    /// A button: the corner order of `CreateButton`'s quad.
    Button(i32),
}

/// One object to draw this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Draw {
    pub model: ModelId,
    /// Replaces the model's texture (`EntityTexture` on the copy).
    pub texture: Option<TextureRef>,
    /// Blitz position.
    pub position: [f32; 3],
    /// The full rotation in Bevy space, the template's included.
    pub rotation: Quat,
    pub scale: [f32; 3],
    /// MD2 frame.
    pub frame: f32,
    pub alpha: f32,
    pub color: [f32; 3],
    pub uv: Uv,
}

impl Draw {
    /// A copy of `model` at `position`, keeping the template's turn, scale,
    /// colour and alpha.
    fn copy(models: &Models, model: ModelId, position: [f32; 3]) -> Self {
        let def = &models.get(model).def;
        Self {
            model,
            texture: None,
            position,
            rotation: def.rotation(),
            scale: def.scale,
            frame: 0.0,
            alpha: def.alpha,
            color: def.color,
            uv: Uv::Model,
        }
    }

    /// `TurnEntity`: turns relative to the current rotation.
    fn turn(mut self, r: [f32; 3]) -> Self {
        self.rotation *= blitz_quat(r);
        self
    }

    /// `RotateEntity`: replaces the rotation.
    fn rotate(mut self, r: [f32; 3]) -> Self {
        self.rotation = blitz_quat(r);
        self
    }

    fn scale(mut self, s: [f32; 3]) -> Self {
        self.scale = s;
        self
    }

    fn texture(mut self, path: &str, flags: u32) -> Self {
        self.texture = Some(TextureRef {
            path: path.to_owned(),
            flags,
            rotation: 0,
        });
        self
    }
}

/// The loaded models.
#[derive(Resource)]
pub struct Models {
    map: HashMap<ModelId, Model>,
    missing: Model,
}

impl Models {
    /// Loads every model; ones that fail are reported and drawn as
    /// nothing.
    pub fn load_all(data: &GameData) -> Self {
        let mut map = HashMap::new();
        for id in crate::model::all_models() {
            match load(id, &data.assets) {
                Ok(m) => {
                    map.insert(id, m);
                }
                Err(e) => warn!("model {id:?}: {e}"),
            }
        }
        Self {
            map,
            missing: Model {
                def: crate::model::model_def(ModelId::Square),
                surfaces: Vec::new(),
                md2: false,
            },
        }
    }

    pub fn get(&self, id: ModelId) -> &Model {
        self.map.get(&id).unwrap_or(&self.missing)
    }
}

/// The level's breakaway texture names by style (`LoadStyleTextures`).
fn breakaway_texture(style: i32, kind: i32) -> String {
    let n = kind.rem_euclid(3) + 1;
    let prefix = match style {
        -1 => return "graphics\\rainbow.jpg".to_owned(),
        0 => "cave",
        1 => "sand",
        3 => "metal",
        4 | 7 => return "graphics\\castlebreak1.jpg".to_owned(),
        5 => "green",
        6 => "spooky",
        8 => "aztec",
        _ => "wood",
    };
    format!("graphics\\{prefix}break{n}.jpg")
}

/// `ScaleEntity`-free heading for monsters that face their move.
fn yaw_heading(h: i32) -> [f32; 3] {
    [0.0, h as f32, 0.0]
}

/// Everything to draw for `w`. `style` is the level style after the
/// fallbacks; `anim` the texture scroll state.
pub fn draws(w: &World, models: &Models, style: i32, anim: &TickAnim) -> Vec<Draw> {
    let mut out = Vec::new();
    let t = w.level_timer as f32;
    let d = |id: ModelId, p: [f32; 3]| Draw::copy(models, id, p);

    // Players and their shadows.
    for (k, p) in w.players.iter().enumerate().skip(1) {
        if !p.exists {
            continue;
        }
        if !p.hidden {
            let skin = player_skins(k)[match p.skin {
                Skin::Normal => 0,
                Skin::Dying => 1,
                Skin::C => 2,
                Skin::D => 3,
            }];
            let mut draw = d(ModelId::Player(k), p.position)
                .rotate([0.0, p.rotation as f32, 0.0])
                .texture(skin, 1);
            draw.frame = p.anim.time;
            if p.dying {
                // `TurnEntity 4, 6, 0` per tick from 100 down.
                let spun = (100 - p.dying_timer.max(0)) as f32;
                draw = draw.turn([4.0 * spun, 6.0 * spun, 0.0]);
            }
            if p.shrink > 0 {
                let s = p.shrink as f32 / 60.0;
                draw = draw.scale([s; 3]);
            }
            out.push(draw);
        }
        if p.shadow && !p.hidden {
            let [x, _, z] = p.position;
            out.push(d(ModelId::Shadow, [x - 0.25, 0.01, z + 0.25]).scale([0.55, 1.0, 0.55]));
        }
    }

    // Boxes.
    for b in &w.blocks {
        let p = b.position;
        let draw = match b.kind {
            0 => d(ModelId::WoodBox, p),
            1 => d(ModelId::SteelBox, p),
            2 => d(ModelId::Reflector, p),
            3 => d(ModelId::Reflector, p).rotate([0.0, 90.0, 0.0]),
            4 => d(ModelId::Boulder, p),
            5 => d(ModelId::GhostBox, p),
            6 => d(ModelId::BoomBox, p),
            7 => d(ModelId::Prism, p),
            8 => d(ModelId::StickyCube, p),
            9..=12 => d(ModelId::LinkSphere(b.kind as usize - 9), p),
            k if k >= 100 => {
                let mut draw = d(ModelId::PushCannon, p);
                if b.cannon_x != 0 {
                    draw = draw.turn([0.0, 0.0, 90.0 * (2 - b.cannon_x) as f32]);
                }
                if b.cannon_y != 0 {
                    draw = draw.turn([0.0, 0.0, 90.0 * (b.cannon_y + 1) as f32]);
                }
                draw
            }
            _ => continue,
        };
        out.push(draw);
    }
    for bx in &w.boxers {
        out.push(d(
            ModelId::Boxer,
            [bx.x as f32 + 0.5, 0.0, -(bx.y as f32 + 0.5)],
        ));
    }
    for tr in &w.transporters {
        let f = if tr.moving && tr.speed > 0 {
            1.0 - tr.moving_timer as f32 / tr.speed as f32
        } else {
            0.0
        };
        let x = tr.x as f32 + 0.5 + tr.moving_x as f32 * f;
        let y = tr.y as f32 + 0.5 + tr.moving_y as f32 * f;
        out.push(d(ModelId::Transporter, [x, 0.0, -y]));
    }

    // Monsters.
    for m in &w.monsters {
        if m.dead {
            continue;
        }
        let p = m.position;
        let h = m.data[1];
        let draw = match m.kind {
            0 => {
                let mut draw =
                    d(ModelId::Coily, p).turn([0.0, m.yaw as f32 + h as f32 / 8.0 * t, 0.0]);
                if m.dying {
                    let s = 0.4 * m.dying_timer as f32 / (w.ruleset.base_speed * 4) as f32;
                    draw = draw.scale([s, 0.38, s]);
                } else if m.speed > 0 {
                    let squash = 0.4 * m.data[0].abs() as f32 / (m.speed / 2).max(1) as f32;
                    draw = draw.scale([0.5, 0.5 - squash, 0.5]);
                }
                draw
            }
            1 | 2 => d(ModelId::ZBot, p).rotate([-90.0, 0.0, h as f32]),
            11 => {
                let wobble = 60.0 * (2.0 * h as f32).to_radians().sin();
                d(ModelId::ZBot, p).rotate([-50.0, wobble, h as f32])
            }
            3 | 4 => {
                let mut draw = d(ModelId::Bomb, p).rotate(yaw_heading(h));
                // `AnimateMD2 1, .4, 1, 24`.
                draw.frame = 1.0 + (t * 0.4) % 23.0;
                draw
            }
            5 | 6 => {
                let mut draw = d(ModelId::Ufo, p).rotate([-90.0, 0.0, h as f32]);
                if m.dying {
                    let spun = (w.ruleset.base_speed * 4 - m.dying_timer) as f32;
                    draw = draw.turn([-spun, spun, 4.0 * spun]);
                }
                draw
            }
            7 | 8 => {
                let mut draw = d(ModelId::Chomper, p).rotate([0.0, -90.0 - h as f32, 0.0]);
                if m.kind == 8 {
                    draw = draw.texture("graphics\\chomper2b.bmp", 1);
                }
                // `AnimateMD2 2, .3, 0, 6`: back and forth.
                let phase = (t * 0.3) % 12.0;
                draw.frame = if phase <= 6.0 { phase } else { 12.0 - phase };
                draw
            }
            9 => d(ModelId::Ghost, p).rotate([-90.0, 0.0, h as f32]),
            10 => d(ModelId::Ghost, p)
                .rotate([-90.0, 0.0, h as f32])
                .scale([0.6; 3])
                .texture("graphics\\rainbow.jpg", 1),
            12 => d(ModelId::Ufo, p)
                .rotate([-90.0, 0.0, h as f32 / 2.0])
                .scale([5.0; 3]),
            13..=20 => {
                let k = ((m.kind - 13) % 4 + 1) as usize;
                let tex = if m.kind <= 16 {
                    "graphics\\water.jpg"
                } else {
                    "graphics\\lava.jpg"
                };
                let mut draw = d(ModelId::Player(k), p)
                    .rotate([0.0, -90.0 - h as f32, 0.0])
                    .texture(tex, if m.kind <= 16 { 2 } else { 1 });
                if m.moving {
                    draw.frame = 2.0 + (t * 0.25) % 16.0;
                }
                draw
            }
            _ => continue,
        };
        out.push(draw);
        // Ground shadows (not ghosts, spirits or the mother ship), centred
        // under the monster.
        if !matches!(m.kind, 9 | 10 | 12) && !m.dying {
            let s = match m.kind {
                0 | 7 | 8 => 0.6,
                1 | 2 | 11 => 0.4,
                _ => 0.7,
            };
            let [x, _, z] = p;
            out.push(d(ModelId::Shadow, [x - s / 2.0, 0.01, z + s / 2.0]).scale([s, 1.0, s]));
        }
    }
    for wp in &w.warpers {
        let s = match wp.timer {
            1..40 => 1.0 + wp.timer as f32 / 20.0,
            40..200 => 1.0 + (200 - wp.timer) as f32 / 80.0,
            _ => 1.0,
        };
        out.push(
            d(
                ModelId::Warper,
                [wp.x as f32 + 0.5, 0.5, -(wp.y as f32 + 0.5)],
            )
            .scale([s; 3]),
        );
    }

    // Items.
    for it in &w.items {
        let (x, y) = (it.x as f32, it.y as f32);
        let centre = [x + 0.5, 0.5, -(y + 0.5)];
        match it.kind {
            0 => {
                out.push(d(ModelId::Coin, centre).turn([0.0, -2.0 * t, 0.0]));
                out.push(d(ModelId::CoinShadow, [x + 0.6, 0.01, -(y + 0.6)]).turn([
                    0.0,
                    -2.0 * t,
                    0.0,
                ]));
            }
            1 => {
                // Blitz reports the yaw in -180..180; the sine does not
                // care.
                let yaw = -t;
                let size = 0.4 + 0.1 * yaw.to_radians().sin();
                out.push(d(ModelId::Exit, centre).turn([0.0, yaw, 0.0]).scale([
                    size,
                    size / 2.0,
                    size,
                ]));
                out.push(
                    d(ModelId::Shadow, [x + 0.6 - size, 0.01, -y - 0.6 + size]).scale([
                        size * 2.0 - 0.2,
                        size,
                        size * 2.0 - 0.2,
                    ]),
                );
            }
            2 => {
                let [u, v] = anim.water;
                let yaw = it.data as f32 - v * 2000.0;
                let roll = if (u + it.data as f32 / 1000.0).rem_euclid(0.5) >= 0.4 {
                    10.0 * (u * 36000.0).to_radians().sin()
                } else {
                    0.0
                };
                out.push(d(ModelId::Clock, centre).rotate([0.0, yaw, roll]));
            }
            3 => {
                out.push(d(ModelId::BonusCoin, centre).turn([0.0, 0.0, -2.0 * t]));
                out.push(d(ModelId::CoinShadow, [x + 0.5, 0.01, -(y + 0.55)]));
            }
            _ => {}
        }
    }

    // Buttons.
    let base = w.ruleset.base_speed * 15;
    for b in &w.buttons {
        let tex = if b.active {
            "graphics\\buttonoffv5.bmp"
        } else {
            "graphics\\buttonv5.bmp"
        };
        let yaw = if b.kind % 3 == 2 && b.active {
            (b.timer * 90 / base) as f32
        } else {
            0.0
        };
        let mut draw = d(
            ModelId::Square,
            [b.x as f32 + 0.5, 0.01, -(b.y as f32) - 0.5],
        )
        .rotate([0.0, yaw, 0.0])
        .scale([0.8, 1.0, 0.8])
        .texture(tex, 4);
        draw.uv = Uv::Button(b.kind);
        out.push(draw);
    }

    // Laser gates, from the loader's list and the tiles.
    for s in &w.level.spawns {
        let Spawn::LaserGate { x, y, col, dir } = *s else {
            continue;
        };
        let open = w.level.grid.get(x, y) % 100 >= 50;
        let (fx, fz) = (x as f32 + 0.5, -(y as f32) - 0.5);
        if col == 0 {
            let sink = if open {
                if w.final_gate_anim == 0 {
                    continue;
                }
                -0.016 * (60 - w.final_gate_anim) as f32
            } else {
                0.0
            };
            let mut draw = d(ModelId::IronGate, [fx, sink, fz]);
            if dir == 1 {
                draw = draw.turn([0.0, 0.0, 90.0]);
            }
            out.push(draw);
            continue;
        }
        let colour: [u8; 3] = match col {
            1 => [255, 0, 255],
            2 => [255, 255, 0],
            3 => [0, 255, 0],
            4 => [0, 255, 255],
            5 => [255, 55, 55],
            6 => [50, 57, 255],
            7 => [255, 255, 200],
            _ => [255, 255, 255],
        };
        let yaw = if dir == 1 { 90.0 } else { 0.0 };
        if !open {
            let mut draw = d(ModelId::LaserBeams, [fx, 0.5, fz]).rotate([0.0, yaw, 0.0]);
            draw.color = colour.map(|c| f32::from(c) / 255.0);
            out.push(draw);
        }
        out.push(d(ModelId::LaserEmitters, [fx, 0.5, fz]).rotate([0.0, yaw, 0.0]));
    }

    // Hazards.
    for s in &w.spikes {
        out.push(d(
            ModelId::Spike,
            [s.x as f32 + 0.5, s.height, -(s.y as f32) - 0.5],
        ));
    }
    for c in &w.cannons {
        let mut draw = d(
            ModelId::Cannon,
            [c.x as f32 + 0.5, 0.0, -(c.y as f32) - 0.5],
        );
        if c.dx != 0 {
            draw = draw.turn([0.0, 0.0, 90.0 * (2 - c.dx) as f32]);
        }
        if c.dy != 0 {
            draw = draw.turn([0.0, 0.0, 90.0 * (c.dy + 1) as f32]);
        }
        out.push(draw);
    }
    for f in w.fireballs() {
        if !f.dead {
            out.push(d(ModelId::Fireball, f.position));
        }
    }
    let electro_base = w.ruleset.base_speed * 5 / 2;
    let g = &w.level.grid;
    let width = g.width() as i32;
    for (i, &e) in w.level.electros.iter().enumerate() {
        let Some(state) = usize::try_from(e).ok().and_then(|e| w.electros.get(e)) else {
            continue;
        };
        if !state.exists || state.timer > electro_base {
            continue;
        }
        let (x, y) = (i as i32 % width, i as i32 / width);
        let frame = (state.timer.rem_euclid(16) / 4) as f32;
        let mut draw =
            d(ModelId::Square, [x as f32, 0.01, -(y as f32)]).texture("graphics\\electro.bmp", 6);
        draw.uv = Uv::Rect([frame / 4.0, 0.0, (frame + 1.0) / 4.0, 1.0]);
        out.push(draw);
    }

    // Tile overlays: breakaway tiles and switchers.
    for (x, y) in g.cells() {
        let tile = g.get(x, y);
        if tile / 100 == 6 {
            let draw = d(ModelId::Square, [x as f32, 0.0, -(y as f32)])
                .texture(&breakaway_texture(style, tile - 600), 1);
            out.push(Draw {
                uv: Uv::Rect([0.0, 0.0, 1.0, 1.0]),
                ..draw
            });
        }
        let switcher = match tile {
            504..508 => Some(tile - 504),
            512..516 => Some(tile - 508),
            _ => None,
        };
        if let Some(kind) = switcher {
            let mut draw = d(ModelId::Square, [x as f32, 0.02, -(y as f32)]);
            draw.texture = Some(TextureRef {
                path: "graphics\\switcher.jpg".into(),
                flags: 1,
                rotation: [0, 270, 180, 90][(kind % 4) as usize],
            });
            draw.uv = Uv::Rect([0.0, 0.0, 1.0, 1.0]);
            out.push(draw);
        }
    }

    // Scenery.
    for f in &w.fish {
        out.push(d(ModelId::Fish, f.position()).turn([0.0, f.yaw, 0.0]));
    }
    for p in &w.pillars {
        let pos = [p.x as f32 + 0.5, 0.0, -(p.y as f32 + 0.5)];
        let draw = match p.kind {
            8..=11 => {
                let house = format!("graphics\\house{}.jpg", w.house_style.clamp(1, 4));
                let yaw = match p.kind {
                    8 => 90.0,
                    10 => -90.0,
                    11 => 180.0,
                    _ => 0.0,
                };
                d(ModelId::Pillar(8), pos)
                    .turn([0.0, yaw, 0.0])
                    .texture(&house, 1)
            }
            23 | 24 | 30 => {
                let tex = match p.kind {
                    23 => "graphics\\stinkystatue.jpg",
                    24 => "graphics\\rainbow.jpg",
                    _ => "graphics\\stinkystatue2.jpg",
                };
                d(ModelId::Player(1), [pos[0], 0.9, pos[2]])
                    .turn([0.0, 180.0, 0.0])
                    .texture(tex, 1)
            }
            22 => d(ModelId::Pillar(22), pos).turn([0.0, p.flower_yaw as f32, 0.0]),
            k => d(ModelId::Pillar(k.max(0) as usize), pos),
        };
        out.push(draw);
    }
    out
}

/// The level style after the original's fallbacks.
pub fn level_style(level: &LevelData) -> i32 {
    let s = &level.setup;
    effective_style(s.style, effective_background(s.background))
}

/// A pooled object entity.
#[derive(Component)]
struct Slot {
    mesh: Handle<Mesh>,
    /// What the mesh was last built from.
    key: u64,
    material: Option<MaterialKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialKey {
    texture: Option<(String, u32, i32)>,
    color: [u32; 3],
    alpha: u32,
}

#[derive(Resource, Default)]
struct Pools {
    by_model: HashMap<ModelId, Vec<Entity>>,
    materials: HashMap<MaterialKey, Handle<StandardMaterial>>,
}

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pools>()
            .add_systems(Startup, load_models)
            .add_systems(
                Update,
                sync_objects.after(SimStep).run_if(in_state(Phase::Game)),
            );
    }
}

fn load_models(mut commands: Commands, data: Res<GameData>) {
    commands.insert_resource(Models::load_all(&data));
}

/// The light factor for a Bevy-space normal, as an sRGB colour converted
/// to the linear values vertex colours hold.
pub fn vertex_colour(lights: &Lights, n: Vec3) -> [f32; 4] {
    let d = vec(light_direction());
    let lambert = (-n.dot(d)).max(0.0);
    let f = |c: usize| {
        let v = (lights.ambient[c] as f32 + lights.light[c] as f32 * lambert) / 255.0;
        v.clamp(0.0, 1.0)
    };
    let c = Color::srgb(f(0), f(1), f(2)).to_linear();
    [c.red, c.green, c.blue, 1.0]
}

fn button_uvs(kind: i32) -> [[f32; 2]; 4] {
    let u = (kind % 4) as f32 * 0.25;
    let v = (kind / 4) as f32 * 0.125;
    // `CreateButton`'s corners, matched to the flat square's vertices
    // (0, 0, 0), (1, 0, 0), (1, 0, -1), (0, 0, -1) after centring.
    [[u, 0.125 + v], [0.25 + u, 0.125 + v], [0.25 + u, v], [u, v]]
}

/// Builds the Bevy mesh for a draw: the model's geometry at its frame,
/// vertex colours lit for its rotation.
fn build_mesh(g: &Geometry, draw: &Draw, lights: &Lights) -> Mesh {
    let inv_scale = Vec3::from_array(draw.scale).recip();
    let centred = matches!(draw.uv, Uv::Button(_));
    let positions: Vec<[f32; 3]> = g
        .positions
        .iter()
        .map(|p| {
            let p = if centred {
                [p[0] - 0.5, p[1], p[2] + 0.5]
            } else {
                *p
            };
            vec(p).to_array()
        })
        .collect();
    let normals: Vec<Vec3> = g.normals.iter().map(|n| vec(*n)).collect();
    let colours: Vec<[f32; 4]> = normals
        .iter()
        .map(|n| {
            let world = (draw.rotation * (*n * inv_scale)).normalize_or_zero();
            vertex_colour(lights, world)
        })
        .collect();
    let uvs: Vec<[f32; 2]> = match draw.uv {
        Uv::Model => g.uvs.clone(),
        Uv::Rect([u0, v0, u1, v1]) => g
            .positions
            .iter()
            .map(|p| [u0 + (u1 - u0) * p[0], v0 + (v1 - v0) * -p[2]])
            .collect(),
        Uv::Button(kind) => {
            let corners = button_uvs(kind);
            (0..g.positions.len()).map(|i| corners[i % 4]).collect()
        }
    };
    let indices: Vec<u32> = g.triangles.iter().flat_map(|&t| triangle(t)).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals.iter().map(|n| n.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

fn mesh_key(draw: &Draw, lights: &Lights) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    draw.frame.to_bits().hash(&mut h);
    for v in draw.rotation.to_array() {
        // Small rounding keeps a still object from rebuilding.
        ((v * 4096.0).round() as i32).hash(&mut h);
    }
    for v in draw.scale {
        v.to_bits().hash(&mut h);
    }
    format!("{:?}", draw.uv).hash(&mut h);
    lights.light.hash(&mut h);
    lights.ambient.hash(&mut h);
    h.finish()
}

#[allow(clippy::too_many_arguments)]
fn sync_objects(
    mut commands: Commands,
    sim: Res<Sim>,
    level: Res<LevelData>,
    data: Res<GameData>,
    anim: Res<TickAnim>,
    models: Option<Res<Models>>,
    mut pools: ResMut<Pools>,
    mut slots: Query<(
        &mut Slot,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
) {
    let Some(models) = models else {
        return;
    };
    let world = &sim.world;
    let lights = world.lights;
    let list = draws(world, &models, level_style(&level), &anim);

    let mut grouped: HashMap<ModelId, Vec<&Draw>> = HashMap::new();
    for d in &list {
        grouped.entry(d.model).or_default().push(d);
    }
    // Hide the unused tail of every pool.
    let pools = &mut *pools;
    for (model, entities) in &pools.by_model {
        let used = grouped.get(model).map_or(0, Vec::len);
        for &e in &entities[used.min(entities.len())..] {
            if let Ok((_, _, mut v, _)) = slots.get_mut(e) {
                *v = Visibility::Hidden;
            }
        }
    }
    for (model_id, group) in grouped {
        let model = models.get(model_id);
        let pool = pools.by_model.entry(model_id).or_default();
        for (k, draw) in group.into_iter().enumerate() {
            let transform = Transform {
                translation: vec(draw.position),
                rotation: draw.rotation,
                scale: Vec3::from_array(draw.scale),
            };
            // The material: texture, colour and alpha.
            let tex = draw.texture.clone().or_else(|| model.texture());
            let mkey = MaterialKey {
                texture: tex
                    .as_ref()
                    .map(|t| (t.path.to_lowercase(), t.flags, t.rotation)),
                color: draw.color.map(f32::to_bits),
                alpha: draw.alpha.to_bits(),
            };
            let material = match pools.materials.get(&mkey) {
                Some(m) => m.clone(),
                None => {
                    let (image, see_through) = match &tex {
                        Some(t) => {
                            let (i, a) = cache.get(t, &data, &mut images);
                            (Some(i), a)
                        }
                        None => (None, false),
                    };
                    let masked = tex
                        .as_ref()
                        .is_some_and(|t| t.flags & 4 != 0 && t.flags & 2 == 0);
                    let [r, g, b] = draw.color;
                    let m = materials.add(StandardMaterial {
                        base_color: Color::srgba(r, g, b, draw.alpha),
                        base_color_texture: image,
                        unlit: true,
                        alpha_mode: if draw.alpha < 1.0 || (see_through && !masked) {
                            AlphaMode::Blend
                        } else if masked {
                            AlphaMode::Mask(0.5)
                        } else {
                            AlphaMode::Opaque
                        },
                        uv_transform: match tex.as_ref().map_or(0, |t| t.rotation) {
                            0 => Affine2::IDENTITY,
                            r => crate::scene::uv_transform([0.0, 0.0], r),
                        },
                        ..default()
                    });
                    pools.materials.insert(mkey.clone(), m.clone());
                    m
                }
            };
            let key = mesh_key(draw, &lights);
            if let Some(&e) = pool.get(k)
                && let Ok((mut slot, mut t, mut v, mut mat)) = slots.get_mut(e)
            {
                *t = transform;
                *v = Visibility::Visible;
                if slot.material.as_ref() != Some(&mkey) {
                    mat.0 = material;
                    slot.material = Some(mkey);
                }
                if slot.key != key {
                    let g = model.frame(draw.frame);
                    if let Some(mut m) = meshes.get_mut(&slot.mesh) {
                        *m = build_mesh(&g, draw, &lights);
                    }
                    slot.key = key;
                }
                continue;
            }
            let g = model.frame(draw.frame);
            let mesh = meshes.add(build_mesh(&g, draw, &lights));
            let e = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    transform,
                    Visibility::Visible,
                    Slot {
                        mesh,
                        key,
                        material: Some(mkey),
                    },
                    LevelEntity,
                ))
                .id();
            pool.push(e);
        }
    }
}

/// Forgets the pools, after the level's entities were despawned.
pub fn clear_pools(world: &mut bevy::ecs::world::World) {
    if let Some(mut p) = world.get_resource_mut::<Pools>() {
        p.by_model.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderland_logic::{Input, Options, Ruleset, Session};

    fn models() -> Models {
        // Empty models still carry their definitions.
        let mut map = HashMap::new();
        for id in crate::model::all_models() {
            map.insert(
                id,
                Model {
                    def: crate::model::model_def(id),
                    surfaces: vec![crate::model::flat_square()],
                    md2: false,
                },
            );
        }
        Models {
            map,
            missing: Model {
                def: crate::model::model_def(ModelId::Square),
                surfaces: Vec::new(),
                md2: false,
            },
        }
    }

    fn world(tiles: &[i32], objects: &[i32]) -> World {
        let l = wonderland_formats::Level {
            magic: "Stinky & Loof Level File v5".into(),
            version: 5,
            file_name: "t.lv5".into(),
            map_id: Some(1),
            description: String::new(),
            houses: None,
            models: None,
            textures: None,
            backgrounds: None,
            time: 100,
            style: 1,
            background: 10,
            width: tiles.len(),
            height: 1,
            tiles: tiles.to_vec(),
            objects: objects.to_vec(),
            signs: vec![String::new(); 20],
            music_track: Some(1),
        };
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    #[test]
    fn players_items_and_boxes_are_drawn() {
        // Player, wooden box, coin, exit.
        let mut w = world(&[100; 4], &[1, 9, 5, 6]);
        w.step(Input::default());
        let list = draws(&w, &models(), 1, &TickAnim::default());
        let count = |id: ModelId| list.iter().filter(|d| d.model == id).count();
        assert_eq!(count(ModelId::Player(1)), 1);
        assert_eq!(count(ModelId::WoodBox), 1);
        assert_eq!(count(ModelId::Coin), 1);
        assert_eq!(count(ModelId::CoinShadow), 1);
        assert_eq!(count(ModelId::Exit), 1);
        let p = list.iter().find(|d| d.model == ModelId::Player(1)).unwrap();
        assert_eq!(p.position, w.players[1].position);
        assert_eq!(p.texture.as_ref().unwrap().path, "graphics\\stinky2.bmp");
    }

    #[test]
    fn breakaway_textures_follow_the_style() {
        assert_eq!(breakaway_texture(0, 1), "graphics\\cavebreak2.jpg");
        assert_eq!(breakaway_texture(4, 2), "graphics\\castlebreak1.jpg");
        assert_eq!(breakaway_texture(-1, 0), "graphics\\rainbow.jpg");
    }

    #[test]
    fn lit_faces_brighten_towards_the_light() {
        let lights = Lights {
            light: [255; 3],
            light_target: [0; 3],
            light_speed: 0,
            ambient: [0; 3],
            ambient_target: [0; 3],
            ambient_speed: 0,
        };
        let towards = -vec(light_direction());
        assert!(vertex_colour(&lights, towards)[0] > 0.99);
        assert_eq!(vertex_colour(&lights, -towards)[0], 0.0);
    }
}
