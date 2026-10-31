//! The level scene: the mesh `ConstructLevel` builds, the backdrop, Blitz's
//! lighting on them, and the texture scrolling `UpdateGame` does.
//!
//! Each level surface is split by face direction, because the lighting is
//! per face: every piece gets its own unlit material whose colour is the
//! light factor ([`crate::lighting`]), updated each frame.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::math::Affine2;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::coords::{triangle, vec};
use crate::flow::Phase;
use crate::level::{GameData, LevelData};
use crate::level_mesh::{Quad, Tex, construct_background, construct_level};
use crate::lighting;
use crate::objects::Models;
use crate::sim::{Sim, SimStep, TickAnim};
use crate::style::{
    LevelTextures, TextureRef, effective_background, effective_style, level_textures,
};
use crate::texture;

/// Marks everything spawned for the current level, so a new level can
/// clear it.
#[derive(Component)]
pub struct LevelEntity;

/// A face lit like a Blitz vertex: its material's colour follows the
/// lights.
#[derive(Component)]
pub struct LitFace {
    /// Blitz-space normal.
    pub normal: [f32; 3],
    pub alpha: f32,
}

/// A material whose texture scrolls each tick.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum UvAnim {
    /// `PositionTexture WaterTexture` (and the lava texture).
    Water,
    /// `PositionTexture ConveyorTextureN(set)`, rotated by `RotateTexture`.
    Conveyor { set: usize, rotation: i32 },
}

/// A sky plane drifting with the water texture (`PositionEntity
/// BackGroundPlane, WaterTexU * drift, y, WaterTexV * drift`).
#[derive(Component)]
pub struct Drift {
    pub y: f32,
    pub drift: f32,
}

/// Loaded level textures, by file and flags.
#[derive(Resource, Default)]
pub struct TextureCache {
    images: HashMap<(String, u32), (Handle<Image>, bool)>,
}

impl TextureCache {
    /// The image for `r` and whether it has see-through pixels. A missing
    /// or broken file is reported once and drawn white.
    pub fn get(
        &mut self,
        r: &TextureRef,
        data: &GameData,
        images: &mut Assets<Image>,
    ) -> (Handle<Image>, bool) {
        let key = (r.path.to_lowercase(), r.flags);
        if let Some(found) = self.images.get(&key) {
            return found.clone();
        }
        let decoded = data
            .assets
            .read(&r.path)
            .map_err(|e| e.to_string())
            .and_then(|d| texture::decode(&d, r.flags).map_err(|e| e.to_string()));
        let entry = match decoded {
            Ok(d) => {
                let alpha = d.has_alpha;
                (images.add(texture::to_image(d)), alpha)
            }
            Err(e) => {
                warn!("texture {}: {e}", r.path);
                (Handle::default(), false)
            }
        };
        self.images.insert(key, entry.clone());
        entry
    }
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextureCache>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(
                Update,
                (update_lit_faces, animate_uvs, drift_planes)
                    .after(SimStep)
                    .run_if(in_state(Phase::Game)),
            );
    }
}

fn texture_ref(t: Tex, tex: &LevelTextures) -> (TextureRef, Option<UvAnim>) {
    match t {
        Tex::Floor(i) => (tex.floor[i].clone(), None),
        Tex::FloorSign => (tex.floor_sign.clone(), None),
        Tex::Trampoline => (tex.trampoline.clone(), None),
        Tex::WallTop => (tex.wall_top.clone(), None),
        Tex::WallSide(i) => (tex.wall_side[i].clone(), None),
        Tex::Water => (tex.water.clone(), Some(UvAnim::Water)),
        Tex::Lava => (tex.lava.clone(), Some(UvAnim::Water)),
        Tex::Ice(i) => (tex.ice[i].clone(), None),
        Tex::Conveyor { set, dir } => {
            let r = tex.conveyor[dir].clone();
            let rotation = r.rotation;
            (r, Some(UvAnim::Conveyor { set, rotation }))
        }
        Tex::Teleport(i) => (tex.teleport[i].clone(), None),
        Tex::Background => (
            tex.background
                .as_ref()
                .map(|b| b.0.clone())
                .unwrap_or_else(|| tex.wall_top.clone()),
            None,
        ),
    }
}

/// A Bevy mesh from Blitz quads.
pub fn quads_mesh(quads: &[&Quad]) -> Mesh {
    let mut positions = Vec::with_capacity(quads.len() * 4);
    let mut normals = Vec::with_capacity(quads.len() * 4);
    let mut uvs = Vec::with_capacity(quads.len() * 4);
    let mut indices = Vec::with_capacity(quads.len() * 6);
    for q in quads {
        let base = positions.len() as u32;
        let n = vec(q.normal()).to_array();
        for k in 0..4 {
            positions.push(vec(q.positions[k]).to_array());
            normals.push(n);
            uvs.push(q.uvs[k]);
        }
        for t in q.triangles {
            indices.extend(triangle(t).map(|i| base + i));
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Splits quads by face direction, keeping first-seen order.
fn by_normal(quads: &[Quad]) -> Vec<([f32; 3], Vec<&Quad>)> {
    let mut groups: Vec<([f32; 3], Vec<&Quad>)> = Vec::new();
    for q in quads {
        let n = q.normal();
        let key = n.map(|v| (v * 1000.0).round());
        match groups
            .iter_mut()
            .find(|(g, _)| g.map(|v| (v * 1000.0).round()) == key)
        {
            Some((_, list)) => list.push(q),
            None => groups.push((n, vec![q])),
        }
    }
    groups
}

#[allow(clippy::too_many_arguments)]
fn spawn_surface(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    cache: &mut TextureCache,
    data: &GameData,
    textures: &LevelTextures,
    t: Tex,
    alpha: f32,
    quads: &[Quad],
) {
    let (r, anim) = texture_ref(t, textures);
    let (image, see_through) = cache.get(&r, data, images);
    for (normal, group) in by_normal(quads) {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            unlit: true,
            alpha_mode: if alpha < 1.0 || see_through {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });
        let mut e = commands.spawn((
            Mesh3d(meshes.add(quads_mesh(&group))),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            LitFace { normal, alpha },
            LevelEntity,
        ));
        if let Some(a) = anim {
            e.insert(a);
        }
    }
}

/// Builds the level and its backdrop from [`LevelData`].
#[allow(clippy::too_many_arguments)]
pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
    data: Res<GameData>,
    level: Res<LevelData>,
    models: Option<ResMut<Models>>,
) {
    if let Some(mut models) = models {
        models.set_level(&level.level, &data.assets);
    }
    let setup = &level.setup;
    let background = effective_background(setup.background);
    let style = effective_style(setup.style, background);
    let textures = level_textures(
        setup.style,
        setup.background,
        setup.weather.snow,
        data.teleport_option,
    );

    for s in construct_level(&setup.grid, background) {
        spawn_surface(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut cache,
            &data,
            &textures,
            s.texture,
            s.alpha,
            &s.quads,
        );
    }

    let (width, height) = (setup.grid.width(), setup.grid.height());
    let bg = construct_background(background, style, width, height);
    for s in &bg.surfaces {
        spawn_surface(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut cache,
            &data,
            &textures,
            s.texture,
            s.alpha,
            &s.quads,
        );
    }

    // CreatePlane is endless; the camera sees 140 units at most.
    let (cx, cz) = (width as f32 / 2.0, -(height as f32) / 2.0);
    let half = 1000.0;
    for p in &bg.planes {
        let (r, anim) = texture_ref(p.texture, &textures);
        let (image, see_through) = cache.get(&r, &data, &mut images);
        let corners = [[-half, -half], [half, -half], [-half, half], [half, half]];
        let quad = Quad {
            positions: corners.map(|[x, z]| [cx + x, 0.0, cz + z]),
            uvs: corners.map(|[x, z]| [(cx + x) / p.scale, -(cz + z) / p.scale]),
            triangles: [[0, 2, 1], [1, 2, 3]],
        };
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            alpha_mode: if p.alpha < 1.0 || see_through {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });
        let mut e = commands.spawn((
            Mesh3d(meshes.add(quads_mesh(&[&quad]))),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, p.y, 0.0),
            LitFace {
                normal: [0.0, 1.0, 0.0],
                alpha: p.alpha,
            },
            Drift {
                y: p.y,
                drift: p.drift,
            },
            LevelEntity,
        ));
        if let Some(a) = anim {
            e.insert(a);
        }
    }

    if let Some(centre) = bg.warp_sphere {
        let (r, _) = texture_ref(Tex::Background, &textures);
        let (image, _) = cache.get(&r, &data, &mut images);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            // FlipMesh: seen from inside.
            cull_mode: Some(bevy::render::render_resource::Face::Front),
            uv_transform: Affine2::from_scale(Vec2::splat(4.0)),
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.0).mesh().uv(64, 32))),
            MeshMaterial3d(material),
            Transform::from_translation(vec(centre)).with_scale(Vec3::new(200.0, 25.0, 200.0)),
            // The flipped sphere's normals point inwards; the lighting is
            // taken as a floor's.
            LitFace {
                normal: [0.0, 1.0, 0.0],
                alpha: 1.0,
            },
            LevelEntity,
        ));
    }
}

/// `LightColor`/`AmbientLight` from `World::lights`, per face.
fn update_lit_faces(
    sim: Res<Sim>,
    faces: Query<(&LitFace, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lights = sim.lights();
    for (face, handle) in &faces {
        let [r, g, b] = lighting::factor(&lights, face.normal);
        let colour = Color::srgba(r, g, b, face.alpha);
        if let Some(mut m) = materials.get_mut(&handle.0)
            && m.base_color != colour
        {
            m.base_color = colour;
        }
    }
}

/// The texture matrix for Blitz's `PositionTexture` then `RotateTexture`:
/// the texture moves by `position` and turns by `rotation` degrees, so a
/// conveyor's stripes run the way the belt carries.
pub fn uv_transform(position: [f32; 2], rotation: i32) -> Affine2 {
    Affine2::from_translation(-Vec2::from(position))
        * Affine2::from_angle((rotation as f32).to_radians())
}

fn animate_uvs(
    anim: Res<TickAnim>,
    faces: Query<(&UvAnim, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !anim.is_changed() {
        return;
    }
    for (a, handle) in &faces {
        let t = match *a {
            UvAnim::Water => uv_transform(anim.water, 0),
            UvAnim::Conveyor { set, rotation } => {
                uv_transform([0.0, 1.0 - anim.conveyor[set]], rotation)
            }
        };
        if let Some(mut m) = materials.get_mut(&handle.0) {
            m.uv_transform = t;
        }
    }
}

fn drift_planes(anim: Res<TickAnim>, mut planes: Query<(&Drift, &mut Transform)>) {
    for (d, mut t) in &mut planes {
        t.translation = vec([anim.water[0] * d.drift, d.y, anim.water[1] * d.drift]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conveyor_stripes_move_with_the_belt() {
        // The position is `1 - ConveyorTex`, so it shrinks as the belt
        // runs.
        let flow = |rotation: i32| {
            let a = uv_transform([0.0, 0.01], rotation);
            let b = uv_transform([0.0, 0.0], rotation);
            // Find where texture point (0.5, 0.5) shows in each.
            let at = |m: Affine2| m.inverse().transform_point2(Vec2::splat(0.5));
            let d = at(b) - at(a);
            (d / d.length()).round()
        };
        // Tile 500 carries north (towards v = 0), 501 east (+u), 502
        // south, 503 west.
        assert_eq!(flow(0), Vec2::new(0.0, -1.0));
        assert_eq!(flow(270), Vec2::new(1.0, 0.0));
        assert_eq!(flow(180), Vec2::new(0.0, 1.0));
        assert_eq!(flow(90), Vec2::new(-1.0, 0.0));
    }

    #[test]
    fn faces_split_by_direction() {
        let surfaces = construct_level(
            &{
                let mut g = wonderland_logic::Grid::new(4, 3, 1);
                for (x, t) in [100, 200, 100].into_iter().enumerate() {
                    g.set(x as i32, 0, t);
                }
                g
            },
            0,
        );
        let light = surfaces
            .iter()
            .find(|s| s.texture == Tex::WallSide(0))
            .unwrap();
        assert_eq!(by_normal(&light.quads).len(), 2);
    }
}
