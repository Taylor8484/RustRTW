//! Draws and runs [`menu`](crate::menu) in [`Phase::Menu`]: the 3D
//! background `MenuSetup` builds for style 1, the camera, the logos and
//! selector bar, the level selector's squares, and what the menus ask
//! for (sounds, music, saving the options, starting a level, quitting).
//!
//! The background: a star-textured cylinder round the camera (`res30.dat`,
//! not fogged), two scrolling water planes, mountains from the heightmap
//! `res9.dat` (`LoadTerrain`, drawn here as a full grid rather than
//! Blitz's level-of-detail mesh), a sun that rises and changes colour, and
//! 50 sparkles. Everything but the sun and the sparkles is lit by the
//! level light, which `MenuBkgUpdate` brings up from black, so the scene
//! starts dark and comes up like a sunrise. The fog `MenuSetup` turns on
//! (linear, 90 to 128) stays on in the game as well, as in the original.
//!
//! `WL_MENU=<ticks>[,<menu>]` starts with `MenuTimer` already at `ticks`
//! (and in menu `menu`), for checking the later parts of the sunrise or
//! another menu in a screenshot; `WL_PLAYER=<name>` loads that player
//! first.

use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::camera::Projection;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::Affine2;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use wonderland_campaign::custom::{self, CustomLevels};
use wonderland_campaign::profile::Players;
use wonderland_campaign::worlds::{LEVELS, Worlds};
use wonderland_formats::Game;
use wonderland_formats::level::Level;
use wonderland_logic::world::Lights;
use wonderland_logic::{Clock, Session};

use crate::audio::MenuSounds;
use crate::camera::{FAR, GameCamera, fov, view_aspect};
use crate::coords::{rotation, triangle, vec};
use crate::flow::{Phase, Playlist, begin, record};
use crate::hud::{Screen, ScreenFilled, ScreenImage, camera_rect};
use crate::level::GameData;
use crate::lighting::factor;
use crate::menu::{Action, LevelCursor, Menu, MenuKeys, Selector, Square, exit_sound};
use crate::model::{Geometry, cylinder_segments, flip, sphere_segments};
use crate::music::{MusicRequest, extra_track, world_track};
use crate::objects::vertex_colour;
use crate::particles::frame;
use crate::scene::TextureCache;
use crate::sim::Sim;
use crate::style::TextureRef;

/// `CameraFogRange`.
pub const FOG_RANGE: (f32, f32) = (90.0, 128.0);

/// The menus and the clock their ticks run on.
#[derive(Resource)]
pub struct MenuState {
    pub menu: Menu,
    clock: Clock,
    /// Set once the game is closing: when to exit and what to open then.
    quitting: Option<(i64, Option<&'static str>)>,
}

/// Part of the menu background.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Sky,
    /// The lower (0) and upper (1) water plane.
    Water(usize),
    Terrain,
    Sun,
    Sparkle(usize),
}

/// A lit background mesh: its Blitz-space normals, and the lights and
/// turn its vertex colours were last worked out for.
#[derive(Component)]
struct Lit {
    normals: Vec<[f32; 3]>,
    mesh: Handle<Mesh>,
    done: Option<([i32; 3], [i32; 3], f32)>,
}

pub struct MenuScenePlugin;

impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_menu, spawn_background))
            .add_systems(OnEnter(Phase::Menu), show_background::<true>)
            .add_systems(OnExit(Phase::Menu), show_background::<false>)
            .add_systems(
                Update,
                (
                    (menu_frame, pose_background, menu_camera)
                        .chain()
                        .before(ScreenFilled)
                        .run_if(in_state(Phase::Menu)),
                    (fog_colour, quit_when_done),
                ),
            );
    }
}

fn now_ms(time: &Time<Real>) -> i64 {
    time.elapsed().as_millis() as i64
}

fn setup_menu(mut commands: Commands, data: Res<GameData>, time: Res<Time<Real>>) {
    let mut menu = Menu::new();
    menu.players = Some(Players::new(data.assets.dir()));
    menu.custom = Some(CustomLevels::new(data.assets.dir()));
    menu.level_titles = level_titles(&data, &menu.worlds);
    // `WL_PLAYER=<name>`: that player's progress, for screenshots.
    if let (Ok(name), Some(players)) = (std::env::var("WL_PLAYER"), &menu.players) {
        let entry = (players.list().unwrap_or_default().into_iter())
            .find(|e| e.file_stem.eq_ignore_ascii_case(&name));
        match entry.map(|e| players.load(&e)) {
            Some(Ok(p)) => {
                menu.level = p.first_level();
                menu.pack = (p.world / 10).clamp(0, 3);
                menu.profile = Some(p);
            }
            Some(Err(e)) => warn!("{e}"),
            None => warn!("no player {name}"),
        }
    }
    // `WL_MENU=<timer>[,<menu>]`, for screenshots.
    let spec = std::env::var("WL_MENU").unwrap_or_default();
    let mut spec = spec.split(',').map(|v| v.trim().parse::<i32>().ok());
    if let Some(Some(t)) = spec.next() {
        // As if the title had been running that long.
        for timer in (-500..t).map(|t| t + 1) {
            menu.timer = timer;
            menu.background.tick(timer, menu.current);
        }
    }
    if let Some(Some(m)) = spec.next() {
        menu.end();
        menu.start(m);
    }
    commands.insert_resource(MenuState {
        menu,
        clock: Clock::new(&data.ruleset, now_ms(&time)),
        quitting: None,
    });
}

/// Each world level's title (`MapName$` in its file), by level number.
fn level_titles(data: &GameData, worlds: &Worlds) -> Vec<String> {
    (0..LEVELS as i32)
        .map(|l| {
            let path = worlds.level_path(l)?;
            let bytes = data.assets.read(&path).ok()?;
            match Level::parse(&bytes, Game::Rtwp) {
                Ok(level) => Some(level.description),
                Err(e) => {
                    warn!("{path}: {e}");
                    None
                }
            }
        })
        .map(Option::unwrap_or_default)
        .collect()
}

/// The menu keys held now.
fn read_keys(keys: &ButtonInput<KeyCode>) -> MenuKeys {
    let any = |k: &[KeyCode]| k.iter().any(|&k| keys.pressed(k));
    MenuKeys {
        up: any(&[KeyCode::ArrowUp, KeyCode::Numpad8]),
        down: any(&[KeyCode::ArrowDown, KeyCode::Numpad2]),
        left: any(&[KeyCode::ArrowLeft, KeyCode::Numpad4]),
        right: any(&[KeyCode::ArrowRight, KeyCode::Numpad6]),
        enter: any(&[KeyCode::Enter, KeyCode::NumpadEnter]),
        escape: keys.pressed(KeyCode::Escape),
        space: keys.pressed(KeyCode::Space),
        one: keys.pressed(KeyCode::Digit1),
        zero: keys.pressed(KeyCode::Digit0),
        delete: keys.pressed(KeyCode::Delete),
        backspace: keys.pressed(KeyCode::Backspace),
        page_up: keys.pressed(KeyCode::PageUp),
        page_down: keys.pressed(KeyCode::PageDown),
        f1: keys.pressed(KeyCode::F1),
        f2: keys.pressed(KeyCode::F2),
    }
}

/// The logos, the selector bar, and the level selector's squares and
/// cursor.
fn menu_images(menu: &Menu) -> Vec<ScreenImage> {
    let mut out = Vec::new();
    // Masked and unlit (`EntityFX 1`).
    let sign = |path: &str, rect, uv, turn| ScreenImage {
        texture: TextureRef {
            path: path.into(),
            flags: 4,
            rotation: 0,
        },
        rect,
        uv,
        colour: [1.0; 4],
        turn,
    };
    for sq in &menu.squares {
        let (file, uv) = match sq.square {
            Square::NotWon => ("graphics\\levelsigns2.bmp", [0.0, 0.0, 0.5, 0.5]),
            Square::Won => ("graphics\\levelsigns2.bmp", [0.5, 0.0, 1.0, 0.5]),
            Square::Back => ("graphics\\levelsigns2.bmp", [0.0, 0.5, 0.5, 1.0]),
            Square::Forward => ("graphics\\levelsigns2.bmp", [0.5, 0.5, 1.0, 1.0]),
            Square::Perfect => ("graphics\\levelperfect.bmp", [0.0, 0.0, 1.0, 1.0]),
        };
        out.push(sign(
            file,
            camera_rect(sq.x, sq.y, 4.0, 0.15, 0.15),
            uv,
            0.0,
        ));
    }
    if let Some(LevelCursor { x, y, turn }) = menu.cursor {
        out.push(sign(
            "graphics\\levelsigns2b.bmp",
            camera_rect(x, y, 3.99, 0.16, 0.16),
            [0.0, 0.0, 1.0, 1.0],
            turn,
        ));
    }
    if let Some(Selector {
        x,
        y,
        z,
        half_width,
    }) = menu.selector
    {
        out.push(ScreenImage {
            texture: TextureRef {
                path: "graphics\\menubar.jpg".into(),
                flags: 1,
                rotation: 0,
            },
            rect: camera_rect(x, y, z, half_width, Selector::HALF_HEIGHT),
            uv: [0.0, 0.0, 1.0, 1.0],
            colour: [1.0, 1.0, 1.0, 0.6],
            turn: 0.0,
        });
    }
    if menu.logos {
        // Flat squares at z = 3 facing the camera, lit like any mesh.
        let f = factor(
            &lights(&menu.background.light, &menu.background.ambient),
            [0.0, 0.0, -1.0],
        );
        for (file, x) in [("res51.dat", -0.5), ("res52.dat", 0.5)] {
            out.push(ScreenImage {
                texture: TextureRef {
                    path: file.into(),
                    flags: 4,
                    rotation: 0,
                },
                rect: camera_rect(x, 0.5, 3.0, 0.5, 0.5),
                uv: [0.01, 0.01, 0.99, 0.99],
                colour: [f[0], f[1], f[2], 0.85],
                turn: 0.0,
            });
        }
    }
    out
}

fn lights(light: &[i32; 3], ambient: &[i32; 3]) -> Lights {
    Lights {
        light: *light,
        light_target: *light,
        light_speed: 0,
        ambient: *ambient,
        ambient_target: *ambient,
        ambient_speed: 0,
    }
}

#[allow(clippy::too_many_arguments)]
fn menu_frame(
    mut commands: Commands,
    mut state: ResMut<MenuState>,
    mut data: ResMut<GameData>,
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    time: Res<Time<Real>>,
    mut screen: ResMut<Screen>,
    mut sounds: ResMut<MenuSounds>,
    mut next: ResMut<NextState<Phase>>,
    sim: Option<Res<Sim>>,
    mut music: ResMut<MusicRequest>,
) {
    let now = now_ms(&time);
    let ticks = state.clock.advance(now).ticks;
    let st = &mut *state;
    for k in typed.read() {
        if k.state == ButtonState::Pressed
            && let Some(text) = &k.text
        {
            text.bytes()
                .filter(u8::is_ascii)
                .for_each(|c| st.menu.type_char(c));
        }
    }
    st.menu.millisecs = now as i32;
    let actions = if st.quitting.is_some() {
        Vec::new()
    } else {
        st.menu.frame(read_keys(&keys), ticks, &mut data.config)
    };
    for a in actions {
        match a {
            Action::Sound(s) => sounds.0.push(s),
            Action::TitleMusic => music.0 = Some(Some(crate::music::TITLE)),
            Action::LevelMusic(world) => music.0 = Some(Some(world_track(world))),
            Action::StopMusic => music.0 = Some(None),
            // `TweenTime=MilliSecs()-TweenPeriod`.
            Action::Entered(_) => st.clock = Clock::new(&data.ruleset, now),
            Action::OptionsChanged => data.apply_config(),
            Action::Warn(e) => warn!("{e}"),
            Action::SaveOptions => {
                if let Err(e) = data.save_config() {
                    warn!("cannot save the options: {e}");
                }
            }
            Action::StartLevel(level) => {
                let session = sim
                    .as_ref()
                    .map_or_else(Session::default, |s| s.world.session.clone());
                let record = record(&st.menu);
                let custom = level == custom::LEVEL;
                let path = if custom {
                    st.menu.custom_path()
                } else {
                    st.menu.worlds.level_path(level)
                };
                match path
                    .ok_or_else(|| format!("no level {level}"))
                    .and_then(|path| begin(&data, &path, session, now, record))
                {
                    Ok((level, sim)) => {
                        // `ExtraLevelMusicTrack` changed: its music.
                        if custom && let Some(t) = st.menu.custom_music(level.level.music_track) {
                            music.0 = Some(Some(extra_track(t)));
                        }
                        commands.insert_resource(level);
                        commands.insert_resource(sim);
                        commands.remove_resource::<Playlist>();
                        next.set(Phase::Game);
                    }
                    Err(e) => {
                        error!("{e}");
                        st.menu.start(if custom {
                            crate::menu::CUSTOM_LEVELS
                        } else {
                            crate::menu::LEVEL_SELECT
                        });
                    }
                }
            }
            Action::Quit { url } => {
                // `ClearWorld`, then a random goodbye and two seconds for it.
                sounds.0.push(exit_sound((now % 100) as u32));
                st.quitting = Some((now + 2000, url));
            }
        }
    }
    screen.letters = st.menu.text.shown();
    screen.images = if st.quitting.is_some() {
        Vec::new()
    } else {
        menu_images(&st.menu)
    };
}

fn quit_when_done(
    state: Option<Res<MenuState>>,
    time: Res<Time<Real>>,
    mut exit: MessageWriter<AppExit>,
    mut background: Query<&mut Visibility, With<Part>>,
) {
    let Some((at, url)) = state.and_then(|s| s.quitting) else {
        return;
    };
    for mut v in &mut background {
        *v = Visibility::Hidden;
    }
    if now_ms(&time) < at {
        return;
    }
    if let Some(url) = url {
        open_url(url);
    }
    exit.write(AppExit::Success);
}

/// `ExecFile(url)`: the system's browser.
fn open_url(url: &str) {
    let result = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(e) = result {
        warn!("cannot open {url}: {e}");
    }
}

/// `StartMenu`'s camera: at the origin looking along +z, `CameraZoom 2`,
/// `CameraRange .98, 140`.
fn menu_camera(mut camera: Query<(&Camera, &mut Transform, &mut Projection), With<GameCamera>>) {
    for (cam, mut t, mut p) in &mut camera {
        let aspect = view_aspect(cam);
        *t = Transform::IDENTITY;
        if let Projection::Perspective(p) = &mut *p {
            p.fov = fov(2.0, aspect);
            p.near = 0.98;
            p.far = FAR;
        }
    }
}

/// Fog on the camera from the start, coloured as the menus last left it.
fn fog_colour(
    mut commands: Commands,
    state: Option<Res<MenuState>>,
    mut cameras: Query<(Entity, Option<&mut DistanceFog>), With<GameCamera>>,
) {
    let Some(state) = state else {
        return;
    };
    let [r, g, b] = state
        .menu
        .background
        .fog
        .map(|c| c.clamp(0, 255) as f32 / 255.0);
    let color = Color::srgb(r, g, b);
    for (e, fog) in &mut cameras {
        match fog {
            Some(mut f) => {
                if f.color != color {
                    f.color = color;
                }
            }
            None => {
                commands.entity(e).insert(DistanceFog {
                    color,
                    falloff: FogFalloff::Linear {
                        start: FOG_RANGE.0,
                        end: FOG_RANGE.1,
                    },
                    ..default()
                });
            }
        }
    }
}

/// A Bevy mesh from Blitz geometry, coloured white.
fn mesh_of(g: &Geometry) -> Mesh {
    let positions: Vec<[f32; 3]> = g.positions.iter().map(|p| vec(*p).to_array()).collect();
    let normals: Vec<[f32; 3]> = g.normals.iter().map(|n| vec(*n).to_array()).collect();
    let indices: Vec<u32> = g.triangles.iter().flat_map(|&t| triangle(t)).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, g.uvs.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; g.positions.len()])
    .with_inserted_indices(Indices::U32(indices))
}

/// `CreatePlane`, cut to a square that reaches past the far plane, with
/// Blitz's one texture repeat per unit.
pub fn plane() -> Geometry {
    let s = 160.0;
    Geometry {
        positions: vec![[-s, 0.0, s], [s, 0.0, s], [s, 0.0, -s], [-s, 0.0, -s]],
        normals: vec![[0.0, 1.0, 0.0]; 4],
        uvs: vec![[-s, -s], [s, -s], [s, s], [-s, s]],
        triangles: vec![[0, 1, 2], [0, 2, 3]],
    }
}

/// `LoadTerrain` of a square heightmap: one vertex per pixel, heights 0–1
/// from the red channel, the image's top row at the far edge (z = size -
/// 1), one texture repeat per unit. `y_scale` is the `ScaleEntity` the
/// normals are worked out for.
pub fn terrain(heights: &[u8], size: usize, y_scale: f32) -> Geometry {
    let h = |x: usize, z: usize| heights[(size - 1 - z) * size + x] as f32 / 255.0;
    let mut g = Geometry::default();
    for z in 0..size {
        for x in 0..size {
            g.positions.push([x as f32, h(x, z), z as f32]);
            let dx = h((x + 1).min(size - 1), z) - h(x.saturating_sub(1), z);
            let dz = h(x, (z + 1).min(size - 1)) - h(x, z.saturating_sub(1));
            // The normal of the scaled surface, back in unscaled space so
            // it can be scaled like the positions.
            let n = Vec3::new(-dx / 2.0 * y_scale, 1.0, -dz / 2.0 * y_scale).normalize();
            g.normals.push(n.to_array());
            g.uvs.push([x as f32, -(z as f32)]);
        }
    }
    let row = size as u32;
    for z in 0..row - 1 {
        for x in 0..row - 1 {
            let a = z * row + x;
            // Clockwise seen from above, Blitz's front.
            g.triangles.push([a, a + row, a + row + 1]);
            g.triangles.push([a, a + row + 1, a + 1]);
        }
    }
    g
}

/// The heightmap's red channel and size.
fn load_heightmap(data: &GameData) -> Option<(Vec<u8>, usize)> {
    let bytes = data.assets.read("res9.dat").ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgb8();
    let size = img.width() as usize;
    (img.height() as usize == size).then(|| (img.pixels().map(|p| p[0]).collect(), size))
}

#[allow(clippy::too_many_arguments)]
fn spawn_background(
    mut commands: Commands,
    data: Res<GameData>,
    phase: Res<State<Phase>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<TextureCache>,
) {
    // `OnEnter` of the first phase may already have run.
    let shown = if *phase.get() == Phase::Menu {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let tex = |path: &str| TextureRef {
        path: path.into(),
        flags: 1,
        rotation: 0,
    };
    let mut lit = |commands: &mut Commands,
                   part: Part,
                   g: Geometry,
                   texture: TextureRef,
                   alpha: f32,
                   fog: bool| {
        let normals = g.normals.clone();
        let mesh = meshes.add(mesh_of(&g));
        let (image, _) = cache.get(&texture, &data, &mut images);
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, alpha),
            base_color_texture: Some(image),
            unlit: true,
            fog_enabled: fog,
            alpha_mode: if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            shown,
            part,
            Lit {
                normals,
                mesh,
                done: None,
            },
        ));
    };

    let mut sky = cylinder_segments(64, true);
    flip(&mut sky);
    lit(&mut commands, Part::Sky, sky, tex("res30.dat"), 1.0, false);
    lit(
        &mut commands,
        Part::Water(0),
        plane(),
        tex("graphics\\titlewater.jpg"),
        1.0,
        true,
    );
    lit(
        &mut commands,
        Part::Water(1),
        plane(),
        tex("graphics\\titlewater.jpg"),
        0.5,
        true,
    );
    match load_heightmap(&data) {
        Some((heights, size)) => lit(
            &mut commands,
            Part::Terrain,
            terrain(&heights, size, 25.0),
            tex("graphics\\titlemountains.jpg"),
            1.0,
            true,
        ),
        None => warn!("res9.dat: cannot load the menu's heightmap"),
    }

    // The sun: full bright and not fogged (`EntityFX 9`).
    let sun = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        unlit: true,
        fog_enabled: false,
        ..default()
    });
    let sphere = sphere_segments(32);
    let mut sun_mesh = mesh_of(&sphere);
    sun_mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
    commands.spawn((
        Mesh3d(meshes.add(sun_mesh)),
        MeshMaterial3d(sun),
        Transform::IDENTITY,
        shown,
        Part::Sun,
    ));

    // The sparkles: frame 3 of the particle texture, `ScaleSprite .6`.
    let (r, uv) = frame(3);
    let (image, _) = cache.get(&r, &data, &mut images);
    let sparkle = materials.add(StandardMaterial {
        base_color_texture: Some(image),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        uv_transform: uv,
        cull_mode: None,
        ..default()
    });
    let quad = meshes.add(Rectangle::new(1.2, 1.2));
    for i in 0..50 {
        commands.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(sparkle.clone()),
            Transform::IDENTITY,
            shown,
            Part::Sparkle(i),
        ));
    }
}

fn show_background<const SHOW: bool>(mut parts: Query<&mut Visibility, With<Part>>) {
    for mut v in &mut parts {
        *v = if SHOW {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// `ScaleTexture` then `PositionTexture`, in the convention of
/// [`crate::scene::uv_transform`].
fn uv_transform(scale: f32, position: [f32; 2]) -> Affine2 {
    Affine2::from_translation(-Vec2::from(position)) * Affine2::from_scale(Vec2::splat(1.0 / scale))
}

/// Places the background as `MenuBkgUpdate` left it and relights it.
#[allow(clippy::type_complexity)]
fn pose_background(
    state: Res<MenuState>,
    mut parts: Query<(
        &Part,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Option<&mut Lit>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let bg = &state.menu.background;
    let turn = rotation(0.0, bg.turn);
    let l = lights(&bg.light, &bg.ambient);
    for (part, mut t, material, lit) in &mut parts {
        let (new, uv) = match *part {
            Part::Sky => (
                Transform::from_rotation(turn).with_scale(Vec3::splat(128.0)),
                Some(uv_transform(0.125, bg.tex[0])),
            ),
            Part::Water(0) => (
                Transform::from_translation(vec([0.0, -5.2, 0.0])).with_rotation(turn),
                Some(uv_transform(20.0, bg.tex[1])),
            ),
            Part::Water(_) => (
                Transform::from_translation(vec([0.0, bg.water_y, 0.0])).with_rotation(turn),
                Some(uv_transform(10.0, bg.tex[2])),
            ),
            Part::Terrain => (
                Transform::from_translation(vec(bg.terrain_at))
                    .with_rotation(turn)
                    .with_scale(Vec3::new(1.0, 25.0, 1.0)),
                Some(uv_transform(10.0, [0.0, 0.0])),
            ),
            Part::Sun => {
                if let Some(mut m) = materials.get_mut(&material.0) {
                    let [r, g, b] = bg.sun_colour.map(|c| c.clamp(0, 255) as f32 / 255.0);
                    let c = Color::srgb(r, g, b);
                    if m.base_color != c {
                        m.base_color = c;
                    }
                }
                (
                    Transform::from_translation(vec(bg.sun_at)).with_scale(Vec3::splat(5.0)),
                    None,
                )
            }
            Part::Sparkle(i) => {
                let (at, roll) = bg.sparkles[i];
                // Sprites face the camera, which looks straight ahead.
                (
                    Transform::from_translation(vec(at))
                        .with_rotation(Quat::from_rotation_z(roll.to_radians())),
                    None,
                )
            }
        };
        if *t != new {
            *t = new;
        }
        if let Some(uv) = uv
            && let Some(m) = materials.get(&material.0)
            && m.uv_transform != uv
            && let Some(mut m) = materials.get_mut(&material.0)
        {
            m.uv_transform = uv;
        }
        // Relight when the lights change or the turn has moved on a
        // little.
        if let Some(mut lit) = lit {
            let stale = lit.done.is_none_or(|(light, ambient, done_turn)| {
                light != bg.light || ambient != bg.ambient || (bg.turn - done_turn).abs() > 0.25
            });
            if stale && let Some(mut mesh) = meshes.get_mut(&lit.mesh) {
                let colours: Vec<[f32; 4]> = lit
                    .normals
                    .iter()
                    .map(|n| {
                        let n = (turn * (vec(*n) * t.scale.recip())).normalize_or_zero();
                        vertex_colour(&l, n)
                    })
                    .collect();
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
                lit.done = Some((bg.light, bg.ambient, bg.turn));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_faces_up_and_follows_the_heights() {
        // A ramp rising towards +x.
        let size = 4;
        let heights: Vec<u8> = (0..size * size).map(|i| (i % size * 85) as u8).collect();
        let g = terrain(&heights, size, 1.0);
        assert_eq!(g.positions.len(), 16);
        assert_eq!(g.triangles.len(), 18);
        assert!((g.positions[3][1] - 1.0).abs() < 1e-6);
        // Slopes down towards -x, so the normal leans that way.
        assert!(g.normals[5][0] < 0.0 && g.normals[5][1] > 0.0);
        // Wound like the real meshes: (b - a) x (c - a) along the normal.
        for t in &g.triangles {
            let [a, b, c] = t.map(|i| Vec3::from(g.positions[i as usize]));
            assert!((b - a).cross(c - a).y > 0.0);
        }
    }

    #[test]
    fn logos_light_up_with_the_sunrise() {
        let mut m = Menu::new();
        let dark = menu_images(&m);
        assert_eq!(dark.len(), 2);
        assert_eq!(dark[0].colour[0], 0.0);
        m.background.tick(6000, 0);
        let lit = menu_images(&m);
        assert!(lit[0].colour[0] > 0.5 && lit[0].colour[3] == 0.85);
    }
}
