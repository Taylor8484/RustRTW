//! [`Phase::Editor`]: the canvas, and test play.
//!
//! The level shows in the real renderer from an orthographic camera straight
//! down, which [`View`] can tilt to the play camera. [`Phase::Editor`] keeps
//! a paused [`Sim`] of the level, tidied as for play, so that the game's own code draws it: a new
//! one is started whenever the level changes, and only the level's surfaces
//! are rebuilt, as the objects follow the [`Sim`].
//!
//! The pointer is turned into a cell by a ray to the ground, and
//! [`wonderland_editor::canvas`] decides what it does there.
//! [`crate::editor_ui`] draws the panels and overlays. It changes nothing
//! itself: it asks through [`Studio::request`], as the keys do.
//!
//! While [`Editing`] exists, [`crate::flow`] brings a won or abandoned level
//! back here and records nothing.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::camera::{OrthographicProjection, PerspectiveProjection, Projection, ScalingMode};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use wonderland_campaign::custom;
use wonderland_editor::canvas::{Brush, Canvas, Cell, Tool, turned};
use wonderland_editor::links::{Link, Spot, Warning, links, warnings};
use wonderland_editor::settings::{Folder, Setting};
use wonderland_editor::{Editor, level_name};
use wonderland_formats::catalogue::{self, Group, Layer};
use wonderland_formats::{Game, Level};
use wonderland_logic::Session;

use crate::camera::{FAR, GameCamera, fov, view_aspect};
use crate::coords::rotation;
use crate::flow::{Phase, begin_memory};
use crate::hud::{Screen, ScreenFilled};
use crate::level::GameData;
use crate::menu::MAIN;
use crate::menu_scene::MenuState;
use crate::music::{MusicRequest, extra_track};
use crate::scene::{LitFace, spawn_level};
use crate::sim::Sim;

/// The level being edited. Removed on leaving for the menus.
#[derive(Resource, Debug, Default)]
pub struct Editing(pub Editor);

/// Where the ortho camera hangs, well above the tallest object.
const CAMERA_HEIGHT: f32 = 60.0;
const MIN_SPAN: f32 = 8.0;
const MAX_SPAN: f32 = 120.0;
/// Fractions of the view per second, with an arrow key held.
const PAN_SPEED: f32 = 0.8;
/// A wheel notch's change of [`View::span`].
const ZOOM_STEP: f32 = 0.9;
/// The play camera's pitch, for a level with nobody to follow.
const TILT_PITCH: f32 = 60.0;
const TILT_HEIGHT: f32 = 9.0;

/// What the camera shows, in the level's cells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// The point on the ground in the middle of the view, as (x, y).
    pub centre: Vec2,
    /// How many cells fit top to bottom.
    pub span: f32,
    pub tilted: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            centre: Vec2::ZERO,
            span: 24.0,
            tilted: false,
        }
    }
}

impl View {
    /// Shows all of a `width` by `height` level.
    fn fit(&mut self, width: usize, height: usize) {
        self.centre = Vec2::new(width as f32, height as f32) / 2.0;
        // Room for the panels, whose size is not known here.
        self.span = (height.max(width * 3 / 4) as f32 * 1.25).clamp(MIN_SPAN, MAX_SPAN);
    }

    /// Changes the zoom by wheel `notches`, keeping `at` where it is.
    fn zoom(&mut self, notches: f32, at: Vec2) {
        let span = (self.span * ZOOM_STEP.powf(notches)).clamp(MIN_SPAN, MAX_SPAN);
        self.centre += (at - self.centre) * (1.0 - span / self.span);
        self.span = span;
    }
}

/// What to do once any changes are saved or given up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaving {
    New,
    Open,
    Menu,
}

/// A dialog over the canvas.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Dialog {
    #[default]
    None,
    Save {
        name: String,
        title: String,
        /// The file is there already, and this asks before replacing it.
        replacing: bool,
    },
    /// The custom levels to choose from.
    Open(Vec<String>),
    Unsaved(Leaving),
}

/// What the panels or the keys ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Delete,
    Play,
    PlayFromCursor,
    Tilt,
    ShowAll,
    /// Turns the brush, or the piece selected.
    Turn,
    /// Replaces the piece on a cell.
    SetPiece(Spot, Brush),
    Setting(Setting),
    /// Changes a sign's text, staying pending until [`Request::Commit`].
    SignText(usize, String),
    Commit,
    /// Brings a cell to the middle of the view.
    GoTo(Spot),
    Leave(Leaving),
    /// [`Request::Leave`] with any changes given up.
    Discard(Leaving),
    OpenFile(String),
    AskToSave,
    Save {
        name: String,
        title: String,
        replace: bool,
    },
}

/// The editor's screen. Kept through test play.
#[derive(Resource, Debug, Default)]
pub struct Studio {
    pub canvas: Canvas,
    pub view: View,
    /// The window position of the pointer.
    pub pointer: Option<Vec2>,
    /// The cell under the pointer, wherever it is.
    pub hover: Option<Cell>,
    /// The part of the window the panels leave, once they have been drawn.
    pub canvas_area: Option<Rect>,
    /// A piece being dragged from the palette.
    pub carrying: Option<Brush>,
    pub dialog: Dialog,
    pub status: String,
    /// A text box has the keyboard.
    pub typing: bool,
    pub request: Vec<Request>,
    /// What connects the level on show.
    pub links: Vec<Link>,
    /// What will not work in the level on show.
    pub warnings: Vec<Warning>,
    /// The art folders the game folder holds, by [`FOLDERS`].
    pub folders: Vec<(Folder, Vec<String>)>,
    /// The [`Editor::revision`] on show.
    drawn: Option<u64>,
    /// The view fits the level on show.
    fitted: bool,
}

impl Studio {
    /// Whether the pointer is over the canvas, with no dialog in the way.
    pub fn on_canvas(&self) -> bool {
        self.dialog == Dialog::None
            && match (self.pointer, self.canvas_area) {
                (Some(p), Some(area)) => area.contains(p),
                // Nothing drawn yet: no panels either.
                (_, None) => true,
                (None, _) => false,
            }
    }
}

/// The `CustomLevels` folder.
fn levels_dir(data: &GameData) -> PathBuf {
    data.assets.dir().join(custom::FOLDER)
}

/// The folders under `dir`, by name.
fn subfolders(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort_by_key(|n| n.to_ascii_uppercase());
    names
}

/// The level files in `dir`, by name.
fn level_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| custom::is_level_name(n))
        .collect();
    names.sort_by_key(|n| n.to_ascii_uppercase());
    names
}

/// The folders a level can name.
const FOLDERS: [Folder; 4] = [
    Folder::Houses,
    Folder::Models,
    Folder::Textures,
    Folder::Backgrounds,
];

/// A new map id. `SaveLevel` makes one from the clock on every save.
fn new_map_id() -> i32 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |t| t.as_secs());
    (seconds % i32::MAX as u64) as i32
}

/// Writes the level to `dir`, as `SaveLevel` writes to `customlevels`.
fn write_level(editor: &mut Editor, dir: &Path, name: &str, title: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    editor.rename(name);
    editor.set_title(title);
    let bytes = editor.save(new_map_id());
    let path = dir.join(format!("{name}.LV6"));
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Studio>()
            .add_systems(OnEnter(Phase::Editor), clear_screen)
            .add_systems(OnExit(Phase::Editor), leave_canvas)
            .add_systems(
                Update,
                (
                    track_pointer,
                    pointer_buttons,
                    editor_keys,
                    carry_out,
                    follow_level,
                    place_camera,
                )
                    .chain()
                    .before(ScreenFilled)
                    .distributive_run_if(resource_exists::<Editing>)
                    .run_if(in_state(Phase::Editor)),
            );
    }
}

fn clear_screen(
    mut screen: ResMut<Screen>,
    mut music: ResMut<MusicRequest>,
    mut studio: ResMut<Studio>,
    data: Res<GameData>,
) {
    screen.images.clear();
    screen.letters.clear();
    music.0 = Some(None);
    let list = |&folder: &Folder| {
        let dir = data.assets.resolve(folder.root());
        (folder, dir.map(|d| subfolders(&d)).unwrap_or_default())
    };
    studio.folders = FOLDERS.iter().map(list).collect();
}

/// Back to what the play camera expects, and the next visit redraws.
fn leave_canvas(mut studio: ResMut<Studio>, mut camera: Query<&mut Projection, With<GameCamera>>) {
    studio.drawn = None;
    for mut p in &mut camera {
        *p = Projection::Perspective(PerspectiveProjection {
            far: FAR,
            ..default()
        });
    }
}

/// The ground under a window position.
fn ground_at(camera: &Camera, transform: &GlobalTransform, pointer: Vec2) -> Option<Vec2> {
    let corner = camera.logical_viewport_rect().map_or(Vec2::ZERO, |r| r.min);
    let ray = camera.viewport_to_world(transform, pointer - corner).ok()?;
    let along = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let at = ray.get_point(along);
    Some(Vec2::new(at.x, at.z))
}

/// With no window (a test), `Studio`'s pointer stays as it was set.
fn track_pointer(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut studio: ResMut<Studio>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    studio.pointer = window.cursor_position();
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    studio.hover = studio
        .pointer
        .and_then(|p| ground_at(camera, transform, p))
        .map(|g| (g.x.floor() as isize, g.y.floor() as isize));
}

#[allow(clippy::too_many_arguments)]
fn pointer_buttons(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut motion: MessageReader<MouseMotion>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut editing: ResMut<Editing>,
    mut studio: ResMut<Studio>,
) {
    let notches: f32 = wheel.read().map(|w| w.y.signum()).sum();
    let moved: Vec2 = motion.read().map(|m| m.delta).sum();
    let studio = &mut *studio;
    let editor = &mut editing.0;
    let on_canvas = studio.on_canvas();

    if let (true, Some(pointer)) = (notches != 0.0 && on_canvas, studio.pointer)
        && let Ok((camera, transform)) = camera.single()
        && let Some(at) = ground_at(camera, transform, pointer)
    {
        studio.view.zoom(notches, at);
    }
    if buttons.pressed(MouseButton::Middle)
        && let Ok((camera, _)) = camera.single()
        && let Some(size) = camera.logical_viewport_size()
    {
        studio.view.centre -= moved * studio.view.span / size.y;
    }

    if let Some(cell) = studio.hover {
        studio.canvas.hover(editor, cell);
        if on_canvas {
            let band = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            if buttons.just_pressed(MouseButton::Left) {
                studio.canvas.press(editor, cell, false, band);
            } else if buttons.just_pressed(MouseButton::Right) {
                studio.canvas.press(editor, cell, true, false);
            }
        }
    }
    if buttons.just_released(MouseButton::Left) || buttons.just_released(MouseButton::Right) {
        match (studio.carrying.take(), studio.hover) {
            (Some(brush), Some(cell)) if on_canvas => studio.canvas.drop_piece(editor, cell, brush),
            (Some(_), _) => {}
            (None, _) => studio.canvas.release(editor),
        }
    }
    let shift = studio.canvas.take_shift();
    studio.view.centre += Vec2::new(shift.0 as f32, shift.1 as f32);
    studio.hover = studio.hover.map(|(x, y)| (x + shift.0, y + shift.1));
}

/// The keys. Esc lets go of one thing at a time before it leaves.
fn editor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    mut editing: ResMut<Editing>,
    mut studio: ResMut<Studio>,
) {
    let studio = &mut *studio;
    if studio.typing {
        return;
    }
    if studio.dialog != Dialog::None {
        if keys.just_pressed(KeyCode::Escape) {
            studio.dialog = Dialog::None;
        }
        return;
    }
    let held = |list: [KeyCode; 2]| keys.any_pressed(list);
    let ctrl = held([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = held([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let tap = |key| keys.just_pressed(key);

    let mut pan = Vec2::ZERO;
    for (key, way) in [
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::ArrowRight, Vec2::X),
        (KeyCode::ArrowUp, Vec2::NEG_Y),
        (KeyCode::ArrowDown, Vec2::Y),
    ] {
        if keys.pressed(key) {
            pan += way;
        }
    }
    let span = studio.view.span;
    studio.view.centre += pan * span * PAN_SPEED * time.delta_secs();

    let request = if tap(KeyCode::Escape) {
        if studio.canvas.gesture().is_some() {
            studio.canvas.cancel(&mut editing.0);
            None
        } else if studio.carrying.take().is_some() {
            None
        } else if studio.canvas.tool != Tool::Select {
            studio.canvas.tool = Tool::Select;
            None
        } else if studio.canvas.selection().is_some() {
            studio.canvas.deselect();
            None
        } else {
            Some(Request::Leave(Leaving::Menu))
        }
    } else if tap(KeyCode::F5) {
        Some(Request::Play)
    } else if tap(KeyCode::F6) {
        Some(Request::PlayFromCursor)
    } else if tap(KeyCode::KeyT) && !ctrl {
        Some(Request::Tilt)
    } else if tap(KeyCode::KeyR) && !ctrl {
        Some(Request::Turn)
    } else if tap(KeyCode::KeyL) && !ctrl {
        studio.canvas.tool = Tool::Link;
        None
    } else if tap(KeyCode::Home) {
        Some(Request::ShowAll)
    } else if tap(KeyCode::Delete) {
        Some(Request::Delete)
    } else if ctrl && tap(KeyCode::KeyZ) {
        Some(if shift { Request::Redo } else { Request::Undo })
    } else if ctrl && tap(KeyCode::KeyY) {
        Some(Request::Redo)
    } else if ctrl && tap(KeyCode::KeyC) {
        Some(Request::Copy)
    } else if ctrl && tap(KeyCode::KeyX) {
        Some(Request::Cut)
    } else if ctrl && tap(KeyCode::KeyV) {
        Some(Request::Paste)
    } else if ctrl && tap(KeyCode::KeyS) {
        Some(Request::AskToSave)
    } else {
        None
    };
    studio.request.extend(request);
}

/// Starts test play of `level`, with its music.
fn play(
    commands: &mut Commands,
    data: &GameData,
    level: Level,
    now: i64,
    next: &mut NextState<Phase>,
    music: &mut MusicRequest,
    status: &mut String,
) {
    match begin_memory(data, level, Session::default(), now) {
        Ok((level, sim)) => {
            if let Some(track) = level.level.music_track {
                music.0 = Some(Some(extra_track(track)));
            }
            commands.insert_resource(level);
            commands.insert_resource(sim);
            next.set(Phase::Game);
        }
        Err(e) => {
            error!("{e}");
            *status = e;
        }
    }
}

/// Reads a level the file menu picked.
fn read_level(path: &Path) -> Result<Level, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Level::parse(&bytes, Game::Rtwp).map_err(|e| format!("{}: {e}", path.display()))
}

/// Starts on another level, with nothing left of the last one.
fn replace_level(editing: &mut Editing, studio: &mut Studio, editor: Editor) {
    editing.0 = editor;
    studio.canvas = Canvas::default();
    studio.fitted = false;
}

#[allow(clippy::too_many_arguments)]
fn carry_out(
    mut commands: Commands,
    mut editing: ResMut<Editing>,
    mut studio: ResMut<Studio>,
    data: Res<GameData>,
    mut menu: ResMut<MenuState>,
    mut next: ResMut<NextState<Phase>>,
    mut music: ResMut<MusicRequest>,
    time: Res<Time<Real>>,
) {
    let studio = &mut *studio;
    let music = &mut *music;
    let now = time.elapsed().as_millis() as i64;
    for request in std::mem::take(&mut studio.request) {
        let editor = &mut editing.0;
        match request {
            Request::Undo | Request::Redo => {
                studio.canvas.cancel(editor);
                studio.canvas.deselect();
                if request == Request::Undo {
                    editor.undo();
                } else {
                    editor.redo();
                }
            }
            Request::Copy => {
                studio.canvas.copy(editor);
            }
            Request::Cut => studio.canvas.cut(editor),
            Request::Delete => studio.canvas.delete(editor),
            Request::Paste => {
                let at = studio.hover.unwrap_or((1, 1));
                if !studio.canvas.paste(editor, at) {
                    studio.status = "Nothing to paste there".to_owned();
                }
            }
            Request::Play => {
                studio.canvas.cancel(editor);
                let level = editor.playable();
                play(
                    &mut commands,
                    &data,
                    level,
                    now,
                    &mut next,
                    music,
                    &mut studio.status,
                );
            }
            Request::PlayFromCursor => {
                studio.canvas.cancel(editor);
                let Some((x, y)) = studio.hover.and_then(|(x, y)| {
                    let level = editor.level();
                    let x = usize::try_from(x).ok().filter(|&x| x < level.width)?;
                    let y = usize::try_from(y).ok().filter(|&y| y < level.height)?;
                    let tile = catalogue::find(Layer::Tile, level.tile(x, y));
                    tile.is_none_or(|p| p.group != Group::Wall)
                        .then_some((x, y))
                }) else {
                    studio.status = "Point at open ground to play from there".to_owned();
                    continue;
                };
                let level = editor.playable_from(x, y);
                play(
                    &mut commands,
                    &data,
                    level,
                    now,
                    &mut next,
                    music,
                    &mut studio.status,
                );
            }
            Request::Tilt => studio.view.tilted = !studio.view.tilted,
            Request::Turn => {
                if let Tool::Brush(brush) = studio.canvas.tool {
                    studio.canvas.tool = Tool::Brush(turned(brush));
                } else if let Some(block) = studio.canvas.selection()
                    && (block.width, block.height) == (1, 1)
                {
                    let (layer, code) = editor.top(block.x, block.y);
                    let brush = turned(Brush { layer, code });
                    editor.place(block.x, block.y, layer, brush.code);
                    editor.commit();
                }
            }
            Request::SetPiece((x, y), brush) => {
                editor.place(x, y, brush.layer, brush.code);
                editor.commit();
            }
            Request::Setting(setting) => editor.apply(setting),
            Request::SignText(slot, text) => editor.set_sign_text(slot, &text),
            Request::Commit => editor.commit(),
            Request::GoTo((x, y)) => {
                studio.view.centre = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            }
            Request::ShowAll => studio.fitted = false,
            Request::Leave(then) if editor.is_dirty() => studio.dialog = Dialog::Unsaved(then),
            Request::Leave(then) | Request::Discard(then) => {
                studio.dialog = Dialog::None;
                match then {
                    Leaving::New => replace_level(&mut editing, studio, Editor::new()),
                    Leaving::Open => {
                        studio.dialog = Dialog::Open(level_files(&levels_dir(&data)));
                    }
                    Leaving::Menu => {
                        commands.remove_resource::<Editing>();
                        commands.insert_resource(Studio::default());
                        menu.menu.hold_keys();
                        menu.menu.start(MAIN);
                        next.set(Phase::Menu);
                    }
                }
            }
            Request::OpenFile(name) => {
                studio.dialog = Dialog::None;
                match read_level(&levels_dir(&data).join(&name)) {
                    Ok(level) => replace_level(&mut editing, studio, Editor::open(level)),
                    Err(e) => studio.status = e,
                }
            }
            Request::AskToSave => {
                let level = editor.level();
                studio.dialog = Dialog::Save {
                    name: level_name(&level.file_name).unwrap_or_default(),
                    title: level.description.clone(),
                    replacing: false,
                };
            }
            Request::Save {
                name,
                title,
                replace,
            } => {
                let Some(name) = level_name(&name) else {
                    studio.status = format!(
                        "A level name is 1 to {} letters and digits",
                        wonderland_editor::MAX_NAME
                    );
                    continue;
                };
                let dir = levels_dir(&data);
                if !replace && dir.join(format!("{name}.LV6")).exists() {
                    studio.dialog = Dialog::Save {
                        name,
                        title,
                        replacing: true,
                    };
                    continue;
                }
                studio.canvas.cancel(editor);
                studio.dialog = Dialog::None;
                studio.status = match write_level(editor, &dir, &name, &title) {
                    Ok(()) => format!("Saved {name}"),
                    Err(e) => e,
                };
            }
        }
    }
}

/// Shows the level as it now stands, if it has changed.
fn follow_level(
    mut commands: Commands,
    editing: Res<Editing>,
    mut studio: ResMut<Studio>,
    data: Res<GameData>,
    time: Res<Time<Real>>,
    faces: Query<Entity, With<LitFace>>,
) {
    let editor = &editing.0;
    if !studio.fitted {
        studio.fitted = true;
        let level = editor.level();
        let (width, height) = (level.width, level.height);
        studio.view.fit(width, height);
    }
    if studio.drawn == Some(editor.revision()) {
        return;
    }
    studio.drawn = Some(editor.revision());
    studio.links = links(editor.level());
    studio.warnings = warnings(editor.level());
    let now = time.elapsed().as_millis() as i64;
    match begin_memory(&data, editor.playable(), Session::default(), now) {
        Ok((level, mut sim)) => {
            // A level fades in from dark, and this one never ticks.
            let lights = &mut sim.world.lights;
            lights.light = lights.light_target;
            lights.ambient = lights.ambient_target;
            // The Stinky tidying adds is not there to see or to drag.
            if !editor.has_character() {
                sim.world.players.iter_mut().for_each(|p| p.hidden = true);
            }
            commands.insert_resource(level);
            commands.insert_resource(sim);
            for e in &faces {
                commands.entity(e).despawn();
            }
            commands.run_system_cached(spawn_level);
        }
        Err(e) => {
            error!("{e}");
            studio.status = e;
        }
    }
}

fn place_camera(
    studio: Res<Studio>,
    sim: Option<Res<Sim>>,
    mut camera: Query<(&Camera, &mut Transform, &mut Projection), With<GameCamera>>,
) {
    let view = studio.view;
    let ground = Vec3::new(view.centre.x, 0.0, view.centre.y);
    for (cam, mut t, mut p) in &mut camera {
        if view.tilted {
            let aim = sim.as_ref().map(|s| s.world.camera);
            let turned = aim.map_or(rotation(TILT_PITCH, 0.0), |c| rotation(c.pitch, c.yaw));
            let forward = turned * Vec3::NEG_Z;
            let height = aim.map_or(TILT_HEIGHT, |c| c.position[1]);
            let (turned, forward, height) = if forward.y < -0.1 && height > 0.0 {
                (turned, forward, height)
            } else {
                let turned = rotation(TILT_PITCH, 0.0);
                (turned, turned * Vec3::NEG_Z, TILT_HEIGHT)
            };
            *t = Transform::from_translation(ground + forward * (height / forward.y))
                .with_rotation(turned);
            *p = Projection::Perspective(PerspectiveProjection {
                fov: fov(aim.map_or(2.0, |c| c.zoom), view_aspect(cam)),
                near: aim.map_or(0.1, |c| c.near),
                far: FAR,
                ..default()
            });
        } else {
            *t = Transform::from_translation(ground + Vec3::Y * CAMERA_HEIGHT)
                .looking_at(ground, Vec3::NEG_Z);
            *p = Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: view.span,
                },
                far: FAR,
                ..OrthographicProjection::default_3d()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bevy::state::app::StatesPlugin;
    use wonderland_campaign::profile::Profile;
    use wonderland_logic::{Completion, Event};

    use super::*;
    use crate::audio::MenuSounds;
    use crate::flow::FlowPlugin;
    use crate::game_menu::GameMenu;
    use crate::level::LevelData;
    use crate::menu::Menu;
    use crate::pad::Pad;
    use crate::scene::TextureCache;
    use crate::sim::Sim;

    fn phase(app: &App) -> Phase {
        *app.world().resource::<State<Phase>>().get()
    }

    /// Taps `key` and lets the phase change.
    fn tap(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(key);
        keys.clear();
        // A menu chooses on release, and the phase changes a frame later.
        app.update();
        app.update();
    }

    /// An app with no window, in the editor, with a player to record for.
    fn app(data: GameData) -> App {
        let mut menu = Menu::default();
        menu.profile = Some(Profile::new("ZED", 0));
        let menu = MenuState::of(menu, &data);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseWheel>()
            .add_message::<MouseMotion>()
            .init_resource::<Pad>()
            .init_resource::<MenuSounds>()
            .init_resource::<MusicRequest>()
            .init_resource::<Screen>()
            .init_resource::<TextureCache>()
            .init_resource::<Editing>()
            .insert_resource(data)
            .insert_resource(menu)
            .insert_state(Phase::Editor)
            .add_plugins((FlowPlugin, EditorPlugin));
        app.update();
        app
    }

    /// The real game data, if it is here.
    fn real_data() -> Option<GameData> {
        let dir = std::env::var_os("WL_ASSETS")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
            .join("RTWP");
        if !dir.is_dir() {
            eprintln!("skipping: no RTWP assets");
            return None;
        }
        Some(GameData::open(&dir).unwrap())
    }

    /// Clicks or drags the left button from `from` to `to`.
    fn drag(app: &mut App, from: Cell, to: Cell) {
        app.world_mut().resource_mut::<Studio>().hover = Some(from);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut().resource_mut::<Studio>().hover = Some(to);
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.clear();
        app.update();
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.release(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
    }

    fn request(app: &mut App, request: Request) {
        let mut studio = app.world_mut().resource_mut::<Studio>();
        studio.request.push(request);
        app.update();
    }

    #[test]
    fn the_view_keeps_the_ground_under_the_pointer_when_zooming() {
        let mut view = View::default();
        let at = Vec2::new(30.0, 4.0);
        let share = (at - view.centre) / view.span;
        view.zoom(3.0, at);
        assert!(view.span < 24.0);
        assert!(((at - view.centre) / view.span - share).length() < 1e-5);
    }

    #[test]
    fn levels_are_written_and_listed_as_the_original_names_them() {
        let dir = std::env::temp_dir().join(format!("wl-editor-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut editor = Editor::new();
        write_level(&mut editor, &dir, "MINE", "My Level").unwrap();
        std::fs::write(dir.join("notes.txt"), "").unwrap();
        assert_eq!(level_files(&dir), ["MINE.LV6"]);
        let level = read_level(&dir.join("MINE.LV6")).unwrap();
        assert_eq!(
            (level.file_name.as_str(), level.description.as_str()),
            ("MINE", "My Level")
        );
        assert!(!editor.is_dirty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_pointer_edits_the_level_and_the_preview_follows() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        let water = Brush {
            layer: Layer::Tile,
            code: 300,
        };
        app.world_mut().resource_mut::<Studio>().canvas.tool = Tool::Brush(water);
        drag(&mut app, (2, 2), (4, 2));
        let tile = |app: &App, x| app.world().resource::<Editing>().0.level().tile(x, 2);
        assert_eq!(
            (tile(&app, 2), tile(&app, 3), tile(&app, 4)),
            (300, 300, 300)
        );
        app.update();
        let shown = app.world().resource::<LevelData>();
        assert_eq!(shown.level.tile(3, 2), 300);

        // Past the edge the level grows and the picture stays put.
        let centre = app.world().resource::<Studio>().view.centre;
        drag(&mut app, (0, 5), (1, 5));
        assert_eq!(app.world().resource::<Editing>().0.level().width, 15);
        let moved = app.world().resource::<Studio>().view.centre - centre;
        assert_eq!(moved, Vec2::new(1.0, 0.0));

        request(&mut app, Request::Undo);
        request(&mut app, Request::Undo);
        let level = app.world().resource::<Editing>().0.level();
        assert_eq!((level.width, level.tile(3, 2)), (14, 100));

        // Unsaved changes stop the way out, until they are given up.
        drag(&mut app, (2, 2), (2, 2));
        request(&mut app, Request::Leave(Leaving::Menu));
        assert_eq!(
            app.world().resource::<Studio>().dialog,
            Dialog::Unsaved(Leaving::Menu)
        );
        tap(&mut app, KeyCode::Escape);
        assert_eq!(phase(&app), Phase::Editor);
        request(&mut app, Request::Discard(Leaving::Menu));
        app.update();
        assert_eq!(phase(&app), Phase::Menu);
    }

    #[test]
    fn a_new_level_shows_no_player_but_plays_with_one() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        app.update();
        let sim = app.world().resource::<Sim>();
        assert!(sim.world.players.iter().all(|p| p.hidden));
        assert!(sim.lights().ambient[0] > 0, "the level is not left dark");
        tap(&mut app, KeyCode::F5);
        let sim = app.world().resource::<Sim>();
        assert!(sim.world.players.iter().any(|p| p.exists && !p.hidden));
    }

    #[test]
    fn r_turns_the_brush_or_the_piece_selected() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        let cannon = Brush {
            layer: Layer::Tile,
            code: 800,
        };
        app.world_mut().resource_mut::<Studio>().canvas.tool = Tool::Brush(cannon);
        tap(&mut app, KeyCode::KeyR);
        let tool = app.world().resource::<Studio>().canvas.tool;
        assert_eq!(
            tool,
            Tool::Brush(Brush {
                code: 804,
                ..cannon
            })
        );

        drag(&mut app, (3, 3), (3, 3));
        app.world_mut().resource_mut::<Studio>().canvas.tool = Tool::Select;
        drag(&mut app, (3, 3), (3, 3));
        tap(&mut app, KeyCode::KeyR);
        let tile = app.world().resource::<Editing>().0.level().tile(3, 3);
        assert_eq!(tile, 808);
    }

    #[test]
    fn warnings_and_links_follow_the_level_and_test_play_starts_its_music() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        let mut editing = app.world_mut().resource_mut::<Editing>();
        editing.0.place(2, 2, Layer::Tile, 1200);
        editing.0.place(4, 2, Layer::Tile, 1200);
        editing.0.commit();
        app.update();
        let studio = app.world().resource::<Studio>();
        assert_eq!(studio.links.len(), 1);
        assert!(studio.warnings.iter().any(|w| w.message.contains("exit")));
        app.world_mut().resource_mut::<MusicRequest>().0 = None;
        tap(&mut app, KeyCode::F5);
        assert!(matches!(
            app.world().resource::<MusicRequest>().0,
            Some(Some(_))
        ));
    }

    #[test]
    fn esc_puts_things_down_one_at_a_time_before_leaving() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        let mut studio = app.world_mut().resource_mut::<Studio>();
        studio.canvas.tool = Tool::Erase;
        tap(&mut app, KeyCode::Escape);
        assert_eq!(app.world().resource::<Studio>().canvas.tool, Tool::Select);
        assert_eq!(phase(&app), Phase::Editor);
        tap(&mut app, KeyCode::Escape);
        assert_eq!(phase(&app), Phase::Menu);
    }

    #[test]
    fn play_from_the_cursor_starts_there_and_not_in_a_wall() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);
        app.world_mut().resource_mut::<Studio>().hover = Some((0, 0));
        tap(&mut app, KeyCode::F6);
        assert_eq!(phase(&app), Phase::Editor);
        assert!(!app.world().resource::<Studio>().status.is_empty());
        app.world_mut().resource_mut::<Studio>().hover = Some((6, 7));
        tap(&mut app, KeyCode::F6);
        assert_eq!(phase(&app), Phase::Game);
        let level = app.world().resource::<LevelData>();
        assert_eq!(level.level.object(6, 7), 1);
    }

    #[test]
    fn test_play_comes_back_to_the_editor() {
        let Some(data) = real_data() else {
            return;
        };
        let mut app = app(data);

        tap(&mut app, KeyCode::F5);
        assert_eq!(phase(&app), Phase::Game);
        let level = app.world().resource::<LevelData>();
        assert_eq!(level.level.object(1, 1), 1, "tidied: Stinky is added");
        let editing = app.world().resource::<Editing>();
        assert_eq!(editing.0.level().object(1, 1), 0, "the edit is untouched");

        // A lost level restarts from memory.
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.world.completion = Completion::Restart;
        app.update();
        assert_eq!(phase(&app), Phase::Game);
        let sim = app.world().resource::<Sim>();
        assert_eq!(sim.world.completion, Completion::Playing);

        // A win records nothing and comes back.
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.world.completion = Completion::Done;
        sim.events.push(Event::LevelWon {
            score: 900,
            perfect: true,
        });
        app.update();
        app.update();
        assert_eq!(phase(&app), Phase::Editor);
        let menu = &app.world().resource::<MenuState>().menu;
        assert_eq!(menu.profile.as_ref().unwrap().scores[0], 0);
        assert!(app.world().resource::<Screen>().letters.is_empty());

        // So does leaving through the in-game menu (Select Level, Enter).
        tap(&mut app, KeyCode::F5);
        assert_eq!(phase(&app), Phase::Game);
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.menu = Some(GameMenu::new());
        app.update();
        tap(&mut app, KeyCode::ArrowDown);
        tap(&mut app, KeyCode::Enter);
        assert_eq!(phase(&app), Phase::Editor);

        tap(&mut app, KeyCode::Escape);
        assert_eq!(phase(&app), Phase::Menu);
        assert!(!app.world().contains_resource::<Editing>());
    }
}
