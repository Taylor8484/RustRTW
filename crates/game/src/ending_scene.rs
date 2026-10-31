//! Draws [`crate::ending`] in [`Phase::Menu`]: the rainbow tube, the deluxe
//! ending's floor and wall, and the figures. The sparkles are the menu
//! background's; everything else of it hides during an ending, as
//! `StartMenu(12)` hides it and the deluxe ending's floor and wall cover it.
//!
//! The endings run in full light, so nothing here needs relighting.
//!
//! TOW's ending plays as a level; only its sparkles are drawn here.

use bevy::prelude::*;

use crate::coords::vec;
use crate::ending::Figure;
use crate::flow::Phase;
use crate::hud::ScreenFilled;
use crate::level::GameData;
use crate::menu_scene::{MenuState, Part, mesh_of, plane, uv_transform};
use crate::model::{Geometry, ModelId, blitz_quat, flip, player_skins, sphere_segments};
use crate::objects::Models;
use crate::particles::frame;
use crate::scene::{LevelEntity, TextureCache};
use crate::sim::{Sim, SimStep};
use crate::style::TextureRef;
use crate::tow_ending::STARS;

/// The most figures on stage.
const FIGURES: usize = 4;

#[derive(Component)]
struct TowStar(usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Tube,
    Floor,
    Wall,
}

/// A figure's entity, its mesh, and what it shows now.
#[derive(Component)]
struct FigureSlot {
    index: usize,
    mesh: Handle<Mesh>,
    shown: Option<(ModelId, u32)>,
    texture: Option<TextureRef>,
}

pub struct EndingScenePlugin;

impl Plugin for EndingScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_stage)
            .add_systems(
                Update,
                (show_stage, pose_stage, pose_figures)
                    .chain()
                    .before(ScreenFilled)
                    .run_if(in_state(Phase::Menu)),
            )
            .add_systems(
                Update,
                pose_tow_stars.after(SimStep).run_if(in_state(Phase::Game)),
            );
    }
}

fn spawn_stage(
    mut commands: Commands,
    data: Res<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
) {
    let mut part = |commands: &mut Commands, stage: Stage, g: Geometry, path: &str, flags: u32| {
        let texture = TextureRef {
            path: path.into(),
            flags,
            rotation: 0,
        };
        let (image, see_through) = cache.get(&texture, &data, &mut images);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            cull_mode: None,
            alpha_mode: if see_through {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(mesh_of(&g))),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            Visibility::Hidden,
            stage,
        ));
    };
    let mut tube = sphere_segments(64);
    flip(&mut tube);
    part(&mut commands, Stage::Tube, tube, "graphics\\rainbow.jpg", 2);
    part(
        &mut commands,
        Stage::Floor,
        plane(),
        "graphics\\greenfloor1.jpg",
        1,
    );
    part(
        &mut commands,
        Stage::Wall,
        plane(),
        "graphics\\stars.jpg",
        1,
    );

    for index in 0..FIGURES {
        let mesh = meshes.add(mesh_of(&Geometry::default()));
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                unlit: true,
                ..default()
            })),
            Transform::IDENTITY,
            Visibility::Hidden,
            FigureSlot {
                index,
                mesh,
                shown: None,
                texture: None,
            },
        ));
    }
}

/// Swaps the stage and the background when an ending starts or stops.
#[allow(clippy::type_complexity)]
fn show_stage(
    state: Res<MenuState>,
    mut was: Local<Option<bool>>,
    mut parts: Query<(&Part, &mut Visibility), (Without<Stage>, Without<FigureSlot>)>,
    mut stage: Query<(&Stage, &mut Visibility), Without<FigureSlot>>,
    mut figures: Query<&mut Visibility, With<FigureSlot>>,
) {
    let ending = state.menu.ending.as_ref();
    let deluxe = ending.map(|e| e.deluxe);
    if *was == deluxe {
        return;
    }
    *was = deluxe;
    let shown = |on: bool| {
        if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for (part, mut v) in &mut parts {
        *v = shown(ending.is_none() || matches!(part, Part::Sparkle(_)));
    }
    for (s, mut v) in &mut stage {
        *v = shown(match s {
            Stage::Tube => deluxe == Some(false),
            Stage::Floor | Stage::Wall => deluxe == Some(true),
        });
    }
    for mut v in &mut figures {
        *v = Visibility::Hidden;
    }
}

fn pose_stage(
    state: Res<MenuState>,
    mut stage: Query<(&Stage, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(e) = &state.menu.ending else {
        return;
    };
    for (s, mut t, material) in &mut stage {
        let (new, uv) = match s {
            // `MenuEntity(60)`.
            Stage::Tube => (
                Transform::from_translation(vec([0.0, 0.0, 39.0]))
                    .with_rotation(
                        blitz_quat([90.0, 0.0, 0.0]) * blitz_quat([0.0, e.tube_turn, 0.0]),
                    )
                    .with_scale(Vec3::new(2.0, 40.0, 2.0)),
                uv_transform(1.0, e.rainbow),
            ),
            Stage::Floor => (Transform::IDENTITY, uv_transform(3.0, [0.0, 0.0])),
            Stage::Wall => (
                Transform::from_translation(vec([0.0, 0.0, 30.0]))
                    .with_rotation(blitz_quat([-90.0, 0.0, 0.0])),
                uv_transform(10.0, e.wall),
            ),
        };
        *t = new;
        if let Some(m) = materials.get(&material.0)
            && m.uv_transform != uv
            && let Some(mut m) = materials.get_mut(&material.0)
        {
            m.uv_transform = uv;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn pose_figures(
    state: Res<MenuState>,
    data: Res<GameData>,
    models: Option<Res<Models>>,
    mut slots: Query<(
        &mut FigureSlot,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
) {
    let (Some(e), Some(models)) = (&state.menu.ending, models) else {
        return;
    };
    for (mut slot, mut t, mut v, material) in &mut slots {
        let Some(f) = e.figures.get(slot.index) else {
            *v = Visibility::Hidden;
            continue;
        };
        let model = models.get(f.model);
        *v = Visibility::Inherited;
        *t = Transform {
            translation: vec(f.at),
            rotation: model.def.rotation() * f.turn,
            scale: Vec3::from_array(model.def.scale),
        };
        let key = (f.model, f.anim.time.to_bits());
        if slot.shown != Some(key)
            && let Some(mut m) = meshes.get_mut(&slot.mesh)
        {
            *m = mesh_of(&model.frame(f.anim.time));
            slot.shown = Some(key);
        }
        let texture = texture_of(f, &models);
        let Some(mut m) = materials.get_mut(&material.0) else {
            continue;
        };
        let mut see_through = false;
        if slot.texture != texture {
            slot.texture = texture.clone();
            m.base_color_texture = texture.as_ref().map(|t| {
                let (image, a) = cache.get(t, &data, &mut images);
                see_through = a;
                image
            });
            m.alpha_mode = if see_through {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }
        if f.alpha < 1.0 {
            m.alpha_mode = AlphaMode::Blend;
        }
        m.base_color = Color::srgba(1.0, 1.0, 1.0, f.alpha);
    }
}

/// TOW's ending's sparkles (flat `FireBallEntity` cubes): frame 3 of the
/// particle texture, always facing along the z axis.
#[allow(clippy::too_many_arguments)]
fn pose_tow_stars(
    mut commands: Commands,
    sim: Res<Sim>,
    data: Res<GameData>,
    mut stars: Query<(&TowStar, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
) {
    let Some(ending) = &sim.ending else {
        return;
    };
    if stars.is_empty() {
        let (r, uv) = frame(3);
        let (image, _) = cache.get(&r, &data, &mut images);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            uv_transform: uv,
            cull_mode: None,
            ..default()
        });
        let quad = meshes.add(Rectangle::new(2.0, 2.0));
        for i in 0..STARS {
            commands.spawn((
                Mesh3d(quad.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_scale(Vec3::ZERO),
                TowStar(i),
                LevelEntity,
            ));
        }
        return;
    }
    for (star, mut t) in &mut stars {
        let (at, roll, size) = ending.stars[star.0];
        *t = Transform::from_translation(vec(at))
            .with_rotation(Quat::from_rotation_z(roll.to_radians()))
            .with_scale(Vec3::splat(size));
    }
}

/// A figure's texture: its own, or the player's skin, or the model's.
fn texture_of(f: &Figure, models: &Models) -> Option<TextureRef> {
    let own = |path: &str| TextureRef {
        path: path.into(),
        flags: 1,
        rotation: 0,
    };
    match (f.texture, f.model) {
        (Some(path), _) => Some(own(path)),
        (None, ModelId::Player(k)) => Some(own(player_skins(k)[0])),
        _ => models.get(f.model).texture(),
    }
}
