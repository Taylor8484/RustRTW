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
//! back here and records nothing. A test play from the level's start keeps
//! its keys, and a win there becomes the level's [`Solution`], which
//! [`crate::editor_check`] then checks after every edit.
//!
//! Levels are saved in `CustomLevels`, or in a pack, a folder there, with the
//! solution beside the level. Stamps are in a folder of their own.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::camera::{OrthographicProjection, PerspectiveProjection, Projection, ScalingMode};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use wonderland_campaign::custom;
use wonderland_campaign::pack::{self, Pack, solution_path};
use wonderland_editor::canvas::{Brush, Canvas, Cell, Tool, turned};
use wonderland_editor::links::{Link, Spot, Warning, links, warnings};
use wonderland_editor::proof::{Recording, Solution, fair};
use wonderland_editor::settings::{Folder, Setting};
use wonderland_editor::{Clip, Editor, level_name, stamp};
use wonderland_formats::Level;
use wonderland_formats::catalogue::{self, Group, Layer};
use wonderland_logic::{Completion, Ruleset};

use crate::camera::{FAR, GameCamera, fov, view_aspect};
use crate::coords::rotation;
use crate::editor_check::{
    Answers, PackLine, PackProof, Proof, check_pack, pack_lines, read_level, read_solution,
};
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
    /// Naming a stamp of the selection.
    Stamp {
        name: String,
        replacing: bool,
    },
    /// The packs to choose from, and the name for a new one.
    Packs {
        names: Vec<String>,
        new: String,
    },
    /// One pack's levels.
    Pack {
        name: String,
        lines: Vec<PackLine>,
    },
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
    /// Opens a level of `CustomLevels`, or of a pack there.
    OpenFile {
        pack: Option<String>,
        file: String,
    },
    AskToSave,
    Save {
        name: String,
        title: String,
        replace: bool,
    },
    AskToStamp,
    SaveStamp {
        name: String,
        replace: bool,
    },
    PickStamp(String),
    ListPacks,
    NewPack(String),
    OpenPack(String),
    /// Moves the saved level on show into a pack.
    AddToPack(String),
    /// Moves a level of a pack up one place.
    MoveUp(String, usize),
    /// Moves a level of a pack back out to `CustomLevels`.
    TakeOut(String, usize),
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
    /// The stamps on disk, by name.
    pub stamps: Vec<String>,
    /// The way to win the level on show.
    pub proof: Proof,
    /// The file the level on show was opened from or saved to, without its
    /// extension, and the pack that is in.
    pub file: Option<String>,
    pub pack: Option<String>,
    /// The check of the pack in the dialog.
    pack_check: Option<Answers<(usize, PackProof)>>,
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

/// The folder the level on show belongs in.
fn level_dir(data: &GameData, studio: &Studio) -> PathBuf {
    let dir = levels_dir(data);
    studio.pack.as_ref().map_or(dir.clone(), |p| dir.join(p))
}

fn stamps_dir(data: &GameData) -> PathBuf {
    data.assets.dir().join(stamp::FOLDER)
}

/// The stamps in `dir`, by name.
fn stamp_names(dir: &Path) -> Vec<String> {
    let extension = format!(".{}", stamp::EXTENSION);
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(&extension).map(str::to_owned))
        .collect();
    names.sort_by_key(|n| n.to_ascii_uppercase());
    names
}

fn stamp_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.{}", stamp::EXTENSION))
}

/// Writes the selection's cells as a stamp.
fn write_stamp(dir: &Path, name: &str, clip: &Clip) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = stamp_path(dir, name);
    std::fs::write(&path, clip.to_bytes()).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads a stamp the palette picked.
fn read_stamp(dir: &Path, name: &str) -> Result<Clip, String> {
    let path = stamp_path(dir, name);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Clip::parse(&bytes).ok_or_else(|| format!("{name} is not a stamp"))
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

/// Writes the level's solution beside it, or takes away an old one, which
/// is for whatever the file held before.
fn write_solution(dir: &Path, name: &str, solution: Option<&Solution>) -> Result<(), String> {
    let path = solution_path(&dir.join(format!("{name}.LV6")));
    let done = match solution {
        Some(solution) => std::fs::write(&path, solution.to_bytes()),
        None => match std::fs::remove_file(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            done => done,
        },
    };
    done.map_err(|e| format!("{}: {e}", path.display()))
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Studio>()
            .add_systems(OnEnter(Phase::Editor), (clear_screen, take_win))
            .add_systems(OnExit(Phase::Editor), leave_canvas)
            .add_systems(
                Update,
                (
                    track_pointer,
                    pointer_buttons,
                    editor_keys,
                    carry_out,
                    follow_level,
                    follow_proof,
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
    studio.stamps = stamp_names(&stamps_dir(&data));
}

/// A test play from the start that won gives the level its solution, as long
/// as it wins again. Saved at once if the level on disk is the one played.
fn take_win(
    sim: Option<ResMut<Sim>>,
    editing: Option<Res<Editing>>,
    data: Res<GameData>,
    mut studio: ResMut<Studio>,
) {
    let (Some(mut sim), Some(editing)) = (sim, editing) else {
        return;
    };
    let Some(recording) = sim.recording.take() else {
        return;
    };
    if sim.world.completion != Completion::Done {
        return;
    }
    let editor = &editing.0;
    let level = editor.playable();
    let options = fair(data.options);
    let Some(solution) = Solution::new(&level, &Ruleset::rtwp(), options, &recording) else {
        studio.status = "That win could not be repeated, so it proves nothing".to_owned();
        return;
    };
    studio.status = "Proven beatable".to_owned();
    if let (false, Some(file)) = (editor.is_dirty(), &studio.file)
        && let Err(e) = write_solution(&level_dir(&data, &studio), file, Some(&solution))
    {
        studio.status = e;
    }
    studio.proof.won(solution, editor.revision());
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

/// Starts test play of `level`, with its music. A play of the level as it
/// stands keeps its `recording`, which a win makes the solution.
#[allow(clippy::too_many_arguments)]
fn play(
    commands: &mut Commands,
    data: &GameData,
    level: Level,
    recording: Option<Recording>,
    now: i64,
    next: &mut NextState<Phase>,
    music: &mut MusicRequest,
    status: &mut String,
) {
    match begin_memory(data, level, now) {
        Ok((level, mut sim)) => {
            sim.recording = recording;
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

/// Starts on another level, with nothing left of the last one.
fn replace_level(editing: &mut Editing, studio: &mut Studio, editor: Editor) {
    editing.0 = editor;
    studio.canvas = Canvas::default();
    studio.fitted = false;
    studio.proof = Proof::default();
    studio.file = None;
    studio.pack = None;
}

/// Shows the pack `name` and starts checking its levels.
fn show_pack(studio: &mut Studio, data: &GameData, name: String) {
    let pack = Pack::new(&levels_dir(data), &name);
    match pack.levels() {
        Ok(files) => {
            let lines = pack_lines(&pack.dir, files.clone());
            let check = check_pack(pack.dir, files, fair(data.options));
            studio.pack_check = Some(check);
            studio.dialog = Dialog::Pack { name, lines };
        }
        Err(e) => studio.status = e.to_string(),
    }
}

/// Moves the level on show into a pack, which it must be saved for.
fn add_to_pack(
    editor: &Editor,
    studio: &mut Studio,
    data: &GameData,
    name: &str,
) -> Result<(), String> {
    if studio.pack.is_some() {
        return Err("This level is in a pack already".to_owned());
    }
    let file = studio.file.clone().filter(|_| !editor.is_dirty());
    let file = file.ok_or("Save the level before adding it to a pack")?;
    let dir = levels_dir(data);
    let path = dir.join(format!("{file}.LV6"));
    let added = Pack::new(&dir, name)
        .add(&path)
        .map_err(|e| e.to_string())?;
    let stem = added.rsplit_once('.').map_or(added.as_str(), |(s, _)| s);
    studio.file = Some(stem.to_owned());
    studio.pack = Some(name.to_owned());
    Ok(())
}

/// The file of the level on show as a place in pack `name`, if it is there.
fn open_place(studio: &Studio, data: &GameData, name: &str) -> Option<usize> {
    let file = studio
        .file
        .as_ref()
        .filter(|_| studio.pack.as_deref() == Some(name))?;
    let levels = Pack::new(&levels_dir(data), name).levels().ok()?;
    levels
        .iter()
        .position(|l| l.strip_suffix(".LV6") == Some(file))
}

/// Keeps track of the level on show now that it is at `place` in pack `name`.
fn set_open_place(studio: &mut Studio, data: &GameData, name: &str, place: usize) {
    let levels = Pack::new(&levels_dir(data), name)
        .levels()
        .unwrap_or_default();
    if let Some(file) = levels.get(place).and_then(|l| l.strip_suffix(".LV6")) {
        studio.file = Some(file.to_owned());
    }
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
                    Some(Recording::default()),
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
                    None,
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
            Request::OpenFile { pack, file } => {
                studio.dialog = Dialog::None;
                let dir = levels_dir(&data);
                let dir = pack.as_ref().map_or(dir.clone(), |p| dir.join(p));
                let path = dir.join(&file);
                match read_level(&path) {
                    Ok(level) => {
                        replace_level(&mut editing, studio, Editor::open(level));
                        studio.proof = Proof::new(read_solution(&path));
                        studio.pack = pack;
                        let stem = file.rsplit_once('.').map(|(stem, _)| stem.to_owned());
                        studio.file = stem.filter(|s| level_name(s).as_ref() == Some(s));
                    }
                    Err(e) => studio.status = e,
                }
            }
            Request::AskToSave => {
                let level = editor.level();
                let name = studio.file.clone();
                studio.dialog = Dialog::Save {
                    name: name.or(level_name(&level.file_name)).unwrap_or_default(),
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
                let dir = level_dir(&data, studio);
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
                let solution = studio.proof.solution.as_ref();
                let written = write_level(editor, &dir, &name, &title)
                    .and_then(|()| write_solution(&dir, &name, solution));
                studio.status = match written {
                    Ok(()) => {
                        let saved = format!("Saved {name}");
                        studio.file = Some(name);
                        saved
                    }
                    Err(e) => e,
                };
            }
            Request::AskToStamp => {
                if studio.canvas.selection().is_some() {
                    studio.dialog = Dialog::Stamp {
                        name: String::new(),
                        replacing: false,
                    };
                } else {
                    studio.status = "Select the cells to keep as a stamp".to_owned();
                }
            }
            Request::SaveStamp { name, replace } => {
                let Some(name) = level_name(&name) else {
                    studio.status = format!(
                        "A stamp name is 1 to {} letters and digits",
                        wonderland_editor::MAX_NAME
                    );
                    continue;
                };
                let dir = stamps_dir(&data);
                if !replace && stamp_path(&dir, &name).exists() {
                    studio.dialog = Dialog::Stamp {
                        name,
                        replacing: true,
                    };
                    continue;
                }
                studio.dialog = Dialog::None;
                let Some(clip) = studio.canvas.selected_clip(editor) else {
                    continue;
                };
                studio.status = match write_stamp(&dir, &name, &clip) {
                    Ok(()) => format!("Kept {name} as a stamp"),
                    Err(e) => e,
                };
                studio.stamps = stamp_names(&dir);
            }
            Request::PickStamp(name) => match read_stamp(&stamps_dir(&data), &name) {
                Ok(clip) => {
                    studio.canvas.cancel(editor);
                    studio.canvas.pick_stamp(clip);
                }
                Err(e) => studio.status = e,
            },
            Request::ListPacks => {
                let dir = levels_dir(&data);
                studio.pack_check = None;
                studio.dialog = Dialog::Packs {
                    names: pack::packs(&dir).unwrap_or_default(),
                    new: String::new(),
                };
            }
            Request::NewPack(name) => {
                let Some(name) = level_name(&name) else {
                    studio.status = format!(
                        "A pack name is 1 to {} letters and digits",
                        wonderland_editor::MAX_NAME
                    );
                    continue;
                };
                let dir = levels_dir(&data).join(&name);
                match std::fs::create_dir_all(&dir) {
                    Ok(()) => show_pack(studio, &data, name),
                    Err(e) => studio.status = format!("{}: {e}", dir.display()),
                }
            }
            Request::OpenPack(name) => show_pack(studio, &data, name),
            Request::AddToPack(name) => {
                if let Err(e) = add_to_pack(editor, studio, &data, &name) {
                    studio.status = e;
                }
                show_pack(studio, &data, name);
            }
            Request::MoveUp(name, place) => {
                let open = open_place(studio, &data, &name);
                match Pack::new(&levels_dir(&data), &name).move_up(place) {
                    Ok(_) => {
                        let now = match open {
                            Some(p) if p == place => Some(p - 1),
                            Some(p) if p + 1 == place => Some(p + 1),
                            other => other,
                        };
                        if let Some(now) = now {
                            set_open_place(studio, &data, &name, now);
                        }
                    }
                    Err(e) => studio.status = e.to_string(),
                }
                show_pack(studio, &data, name);
            }
            Request::TakeOut(name, place) => {
                let open = open_place(studio, &data, &name);
                let dir = levels_dir(&data);
                let pack = Pack::new(&dir, &name);
                let leaving = pack.levels().unwrap_or_default().get(place).cloned();
                match pack.take_out(place, &dir) {
                    Ok(()) => match open {
                        Some(p) if p == place => {
                            let file = leaving.unwrap_or_default();
                            let file = pack::unplaced(&file);
                            let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
                            studio.file = Some(stem.to_owned());
                            studio.pack = None;
                        }
                        Some(p) if p > place => set_open_place(studio, &data, &name, p - 1),
                        _ => {}
                    },
                    Err(e) => studio.status = e.to_string(),
                }
                show_pack(studio, &data, name);
            }
        }
    }
}

/// Keeps the level's proof and the pack dialog's up to date.
fn follow_proof(editing: Res<Editing>, data: Res<GameData>, mut studio: ResMut<Studio>) {
    let studio = &mut *studio;
    let settled = studio.canvas.gesture().is_none();
    studio.proof.follow(&editing.0, fair(data.options), settled);
    let Some(answers) = &studio.pack_check else {
        return;
    };
    let (answer, more) = answers.next();
    if let (Some((i, proof)), Dialog::Pack { lines, .. }) = (answer, &mut studio.dialog)
        && let Some(line) = lines.get_mut(i)
    {
        line.proof = proof;
    }
    if !more || matches!(studio.dialog, Dialog::None) {
        studio.pack_check = None;
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
    match begin_memory(&data, editor.playable(), now) {
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
    use wonderland_editor::proof::Verdict;
    use wonderland_logic::{Event, Input};

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

    /// A game folder with no game in it, which saving can write to.
    fn temp_data(tag: &str) -> Option<GameData> {
        let dir = std::env::temp_dir().join(format!("wl-studio-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        GameData::open(&dir).ok()
    }

    /// Stinky, then a clear run to the exit, which F5 tests.
    fn corridor(app: &mut App) {
        let mut editing = app.world_mut().resource_mut::<Editing>();
        editing.0.place(2, 5, Layer::Object, 1);
        editing.0.place(6, 5, Layer::Object, 6);
        editing.0.commit();
        app.update();
    }

    /// Plays the corridor to the end, as if right were held throughout.
    fn win(app: &mut App) {
        tap(app, KeyCode::F5);
        assert_eq!(phase(app), Phase::Game);
        let mut sim = app.world_mut().resource_mut::<Sim>();
        let recording = sim.recording.as_mut().expect("from the start");
        let right = Input {
            right: true,
            ..Input::default()
        };
        (0..2000).for_each(|_| recording.push(right));
        sim.world.completion = Completion::Done;
        app.update();
        app.update();
        assert_eq!(phase(app), Phase::Editor);
    }

    /// Runs the app until the check of the level on show is in.
    fn settle(app: &mut App) -> Option<Verdict> {
        for _ in 0..500 {
            app.update();
            let revision = app.world().resource::<Editing>().0.revision();
            let proof = &app.world().resource::<Studio>().proof;
            if !proof.checking(revision) {
                return proof.verdict.clone();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the check did not finish");
    }

    #[test]
    fn a_win_proves_the_level_and_an_edit_rechecks_it() {
        let Some(data) = temp_data("proof") else {
            return;
        };
        let dir = levels_dir(&data);
        let mut app = app(data);
        corridor(&mut app);
        assert_eq!(settle(&mut app), None);
        win(&mut app);
        let Some(Verdict::Proven(ticks)) = settle(&mut app) else {
            panic!("won");
        };

        // A wall in the way breaks it, and undoing mends it.
        let mut editing = app.world_mut().resource_mut::<Editing>();
        editing.0.place(4, 5, Layer::Tile, 200);
        editing.0.commit();
        let Some(Verdict::Broke { cell, .. }) = settle(&mut app) else {
            panic!("the way is shut");
        };
        assert_eq!(cell, Some((4, 5)));
        request(&mut app, Request::Undo);
        assert_eq!(settle(&mut app), Some(Verdict::Proven(ticks)));

        // The solution goes with the level, and comes back with it.
        request(
            &mut app,
            Request::Save {
                name: "HALL".into(),
                title: "Hall".into(),
                replace: false,
            },
        );
        assert!(dir.join("HALL.WIN").exists());
        request(&mut app, Request::Leave(Leaving::New));
        assert_eq!(settle(&mut app), None);
        let file = "HALL.LV6".to_owned();
        request(&mut app, Request::OpenFile { pack: None, file });
        assert_eq!(settle(&mut app), Some(Verdict::Proven(ticks)));

        // Saving over it with no solution takes the old one away.
        app.world_mut().resource_mut::<Studio>().proof = Proof::default();
        request(
            &mut app,
            Request::Save {
                name: "HALL".into(),
                title: "Hall".into(),
                replace: true,
            },
        );
        assert!(!dir.join("HALL.WIN").exists());
    }

    #[test]
    fn a_win_on_a_saved_level_is_kept_at_once() {
        let Some(data) = temp_data("kept") else {
            return;
        };
        let dir = levels_dir(&data);
        let mut app = app(data);
        corridor(&mut app);
        request(
            &mut app,
            Request::Save {
                name: "HALL".into(),
                title: String::new(),
                replace: false,
            },
        );
        assert!(!dir.join("HALL.WIN").exists());
        win(&mut app);
        assert!(dir.join("HALL.WIN").exists());
    }

    #[test]
    fn a_play_from_the_pointer_proves_nothing_and_losing_starts_the_keys_again() {
        let Some(data) = temp_data("pointer") else {
            return;
        };
        let mut app = app(data);
        corridor(&mut app);
        app.world_mut().resource_mut::<Studio>().hover = Some((4, 5));
        tap(&mut app, KeyCode::F6);
        assert!(app.world().resource::<Sim>().recording.is_none());
        request_win(&mut app);
        assert_eq!(settle(&mut app), None);

        tap(&mut app, KeyCode::F5);
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.recording.as_mut().unwrap().push(Input::default());
        sim.world.session.rng = wonderland_logic::BlitzRng::new(99);
        sim.world.completion = Completion::Restart;
        app.update();
        let level = app.world().resource::<LevelData>().level.clone();
        let data = app.world().resource::<GameData>();
        let (_, fresh) = begin_memory(data, level, 0).unwrap();
        let sim = app.world().resource::<Sim>();
        assert_eq!(sim.recording, Some(Recording::default()));
        assert_eq!(sim.world.session, fresh.world.session);
    }

    fn request_win(app: &mut App) {
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.world.completion = Completion::Done;
        app.update();
        app.update();
    }

    #[test]
    fn a_selection_becomes_a_stamp_for_the_palette() {
        let Some(data) = temp_data("stamp") else {
            return;
        };
        let mut app = app(data);
        request(&mut app, Request::AskToStamp);
        let studio = app.world().resource::<Studio>();
        assert_eq!(studio.dialog, Dialog::None, "nothing is selected");
        assert!(!studio.status.is_empty());

        let mut editing = app.world_mut().resource_mut::<Editing>();
        editing.0.place(3, 3, Layer::Tile, 300);
        editing.0.commit();
        drag(&mut app, (3, 3), (3, 3));
        request(&mut app, Request::AskToStamp);
        let name = "pool".to_owned();
        let replace = false;
        request(&mut app, Request::SaveStamp { name, replace });
        assert_eq!(app.world().resource::<Studio>().stamps, ["POOL"]);

        let name = "POOL".to_owned();
        request(&mut app, Request::SaveStamp { name, replace });
        assert!(matches!(
            app.world().resource::<Studio>().dialog,
            Dialog::Stamp {
                replacing: true,
                ..
            }
        ));
        app.world_mut().resource_mut::<Studio>().dialog = Dialog::None;

        request(&mut app, Request::PickStamp("POOL".into()));
        assert_eq!(app.world().resource::<Studio>().canvas.tool, Tool::Stamp);
        drag(&mut app, (8, 8), (8, 8));
        assert_eq!(app.world().resource::<Editing>().0.level().tile(8, 8), 300);
    }

    /// The names in the pack dialog.
    fn pack_files(app: &App) -> Vec<String> {
        match &app.world().resource::<Studio>().dialog {
            Dialog::Pack { lines, .. } => lines.iter().map(|l| l.file.clone()).collect(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_saved_level_moves_into_a_pack_and_keeps_its_way_back() {
        let Some(data) = temp_data("pack") else {
            return;
        };
        let dir = levels_dir(&data);
        let mut app = app(data);
        corridor(&mut app);
        win(&mut app);
        request(&mut app, Request::AddToPack("MINE".into()));
        assert!(!app.world().resource::<Studio>().status.is_empty());

        for name in ["ONE", "TWO"] {
            request(
                &mut app,
                Request::Save {
                    name: name.into(),
                    title: name.into(),
                    replace: false,
                },
            );
            if name == "ONE" {
                request(&mut app, Request::NewPack("MINE".into()));
                request(&mut app, Request::AddToPack("MINE".into()));
                request(&mut app, Request::Leave(Leaving::New));
                corridor(&mut app);
            }
        }
        request(&mut app, Request::AddToPack("MINE".into()));
        assert_eq!(pack_files(&app), ["01ONE.LV6", "02TWO.LV6"]);
        assert_eq!(
            app.world().resource::<Studio>().file.as_deref(),
            Some("02TWO")
        );

        // The pack's proofs come in, and the one on show follows the order.
        for _ in 0..500 {
            app.update();
            let done = matches!(&app.world().resource::<Studio>().dialog,
                Dialog::Pack { lines, .. } if lines[0].proof != PackProof::Checking);
            if done {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let Dialog::Pack { lines, .. } = &app.world().resource::<Studio>().dialog else {
            panic!("the dialog stays");
        };
        assert!(matches!(
            lines[0].proof,
            PackProof::Done(Verdict::Proven(_))
        ));

        request(&mut app, Request::MoveUp("MINE".into(), 1));
        assert_eq!(pack_files(&app), ["01TWO.LV6", "02ONE.LV6"]);
        let studio = app.world().resource::<Studio>();
        assert_eq!(studio.file.as_deref(), Some("01TWO"));
        assert!(dir.join("MINE/02ONE.WIN").exists());

        // Saving now goes to the pack, not beside it.
        request(&mut app, Request::AskToSave);
        match &app.world().resource::<Studio>().dialog {
            Dialog::Save { name, .. } => assert_eq!(name, "01TWO"),
            other => panic!("{other:?}"),
        }
        request(
            &mut app,
            Request::Save {
                name: "01TWO".into(),
                title: String::new(),
                replace: true,
            },
        );
        assert!(!dir.join("01TWO.LV6").exists());

        request(&mut app, Request::TakeOut("MINE".into(), 0));
        assert_eq!(pack_files(&app), ["01ONE.LV6"]);
        let studio = app.world().resource::<Studio>();
        assert_eq!(
            (studio.file.as_deref(), studio.pack.as_deref()),
            (Some("TWO"), None)
        );
        assert!(dir.join("TWO.LV6").exists());
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
