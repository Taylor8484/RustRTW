//! The editor's panels and the marks over the canvas, in egui.
//!
//! The tool bar, the palette and a status line surround the canvas, and the
//! Pieces window shows the palette at a size to see. What is
//! left is [`Studio::canvas_area`], and everything drawn there is projected
//! from the level, so it stays right when the camera tilts: the cell grid,
//! the pointer's cell, the selection and pictures of the pieces that cannot
//! be seen from above, the links and the warnings. Pieces are drawn with the
//! pictures [`crate::editor_icons`] makes. Clicking a piece opens a popup beside it with its
//! choices, and a sign's text.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui::{
    self, Color32, Pos2, Rect as EguiRect, Sense, Stroke, StrokeKind, TextureId, Vec2 as EguiVec,
    Widget, load::SizedTexture,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};
use wonderland_editor::canvas::{Block, Brush, Cell, Gesture, Tool};
use wonderland_editor::proof::{Failure, Verdict};
use wonderland_editor::settings::{
    self, BACKGROUNDS, CUSTOM_BACKGROUND, CUSTOM_STYLE, Folder, HOUSE_STYLES, MUSIC, STYLES,
    Setting, TINTS,
};
use wonderland_editor::signs::{sign_lines, stored_sign};
use wonderland_editor::stage::{Entry, entries_of, expanded};
use wonderland_formats::catalogue::{self, Axis, Group, Layer, PIECES, Piece, Sprite};
use wonderland_logic::Ruleset;

use crate::camera::GameCamera;
use crate::editor_check::PackProof;
use crate::editor_icons::{EditorIconsPlugin, Icons};
use crate::editor_scene::{Dialog, Editing, Leaving, Request, Studio};
use crate::flow::Phase;
use crate::level::GameData;
use crate::texture;

/// A palette sprite's size.
const ICON: f32 = 40.0;
/// The Pieces window's size for them.
const BIG_ICON: f32 = 96.0;
/// A marker's share of its cell.
const MARKER: f32 = 0.7;
const GROUPS: [(Group, &str); 10] = [
    (Group::Ground, "Ground"),
    (Group::Wall, "Walls"),
    (Group::Item, "Items"),
    (Group::Character, "Characters"),
    (Group::Box, "Boxes"),
    (Group::Monster, "Monsters"),
    (Group::Hazard, "Hazards"),
    (Group::Switch, "Switches"),
    (Group::Sign, "Signs"),
    (Group::Scenery, "Scenery"),
];
const GRID: Color32 = Color32::from_rgba_premultiplied(30, 30, 30, 30);
const COVER: Color32 = Color32::from_rgb(27, 27, 27);
const EDGE: Color32 = Color32::from_rgb(255, 220, 90);
const HOVER: Color32 = Color32::WHITE;
const CHOSEN: Color32 = Color32::from_rgb(90, 200, 255);
const LINK: Color32 = Color32::from_rgb(120, 255, 140);
const FAINT_LINK: Color32 = Color32::from_rgba_premultiplied(50, 110, 60, 110);
const WARNING: Color32 = Color32::from_rgb(255, 80, 60);
const PROVEN: Color32 = Color32::from_rgb(120, 255, 140);
/// More choices than this go in a list instead of buttons.
const MAX_BUTTONS: usize = 5;
const SIGN: std::ops::Range<i32> = 1300..1320;
const POPUP_WIDTH: f32 = 200.0;

/// One of the original editor's sprite sheets, loaded for egui.
struct Sheet {
    id: TextureId,
    /// Frames across and down.
    size: [u32; 2],
    frame: u32,
}

impl Sheet {
    fn uv(&self, frame: u32) -> EguiRect {
        let across = (self.size[0] / self.frame).max(1);
        let down = (self.size[1] / self.frame).max(1);
        let min = egui::pos2((frame % across) as f32, (frame / across) as f32);
        EguiRect::from_min_size(
            egui::pos2(min.x / across as f32, min.y / down as f32),
            egui::vec2(1.0 / across as f32, 1.0 / down as f32),
        )
    }
}

/// The piece the palette has picked, and its choices.
#[derive(Resource, Default)]
struct Palette {
    sheets: HashMap<&'static str, Option<Sheet>>,
    picked: Option<(&'static Piece, Vec<usize>)>,
    hide_links: bool,
    /// The sheets of pictures of the pieces, as egui knows them.
    pictures: HashMap<AssetId<Image>, TextureId>,
}

impl Palette {
    /// The texture and part of it for a sprite.
    fn icon(
        &mut self,
        sprite: Sprite,
        data: &GameData,
        images: &mut Assets<Image>,
        contexts: &mut EguiContexts,
    ) -> Option<(TextureId, EguiRect)> {
        let sheet = self.sheets.entry(sprite.sheet).or_insert_with(|| {
            let bytes = data.assets.read(sprite.sheet).ok()?;
            let decoded = texture::decode(&bytes, 1).ok()?;
            let size = [decoded.width, decoded.height];
            let image = images.add(texture::to_image(decoded));
            Some(Sheet {
                id: contexts.add_image(EguiTextureHandle::Strong(image)),
                size,
                frame: sprite.size,
            })
        });
        sheet.as_ref().map(|s| (s.id, s.uv(sprite.frame)))
    }
}

/// What draws a piece: its picture from the game, or else its sprite from
/// the original editor.
fn piece_icon(
    (layer, code): (Layer, i32),
    palette: &mut Palette,
    icons: &Icons,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) -> Option<(TextureId, EguiRect)> {
    if let (Some(rect), Some(image)) = (icons.get(layer, code), icons.image.as_ref()) {
        let id = *palette
            .pictures
            .entry(image.id())
            .or_insert_with(|| contexts.add_image(EguiTextureHandle::Strong(image.clone())));
        let uv = EguiRect::from_min_max(
            egui::pos2(rect.min[0], rect.min[1]),
            egui::pos2(rect.max[0], rect.max[1]),
        );
        return Some((id, uv));
    }
    let sprite = catalogue::sprite(layer, code)?;
    palette.icon(sprite, data, images, contexts)
}

/// The choices a person makes: a sign's slot is handed out for them.
fn shown_axes(piece: &Piece) -> impl Iterator<Item = (usize, &Axis)> {
    let sign = piece.group == Group::Sign;
    piece.axes.iter().enumerate().filter(move |_| !sign)
}

/// One choice, as buttons or a list. Returns whether it changed.
fn axis_choice(ui: &mut egui::Ui, axis: &Axis, choice: &mut usize) -> bool {
    let before = *choice;
    if axis.values.len() <= MAX_BUTTONS {
        ui.horizontal_wrapped(|ui| {
            for (v, value) in axis.values.iter().enumerate() {
                ui.selectable_value(choice, v, *value);
            }
        });
    } else {
        egui::ComboBox::from_id_salt(axis.name)
            .selected_text(axis.values[*choice])
            .show_ui(ui, |ui| {
                for (v, value) in axis.values.iter().enumerate() {
                    ui.selectable_value(choice, v, *value);
                }
            });
    }
    *choice != before
}

fn brush(piece: &Piece, choices: &[usize]) -> Brush {
    Brush {
        layer: piece.layer,
        code: piece.code(choices),
    }
}

/// A piece seen edge on or not at all from above, which gets its sprite.
/// Settings on the outer wall are the settings panel's.
fn needs_marker(layer: Layer, code: i32, inside: bool) -> bool {
    const COIN: i32 = 5;
    const CLOCK: i32 = 7;
    const BONUS_COIN: i32 = 8;
    matches!(code, COIN | CLOCK | BONUS_COIN) && layer == Layer::Object
        || inside && catalogue::find(layer, code).is_some_and(|p| p.group == Group::Setting)
}

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorIconsPlugin);
        app.init_resource::<Palette>().add_systems(
            EguiPrimaryContextPass,
            panels.run_if(in_state(Phase::Editor)),
        );
    }
}

/// Where the level's points fall on the window.
struct Projection<'a> {
    camera: &'a Camera,
    transform: &'a GlobalTransform,
}

impl Projection<'_> {
    fn point(&self, x: f32, y: f32) -> Option<Pos2> {
        let corner = self
            .camera
            .logical_viewport_rect()
            .map_or(Vec2::ZERO, |r| r.min);
        let at = self
            .camera
            .world_to_viewport(self.transform, Vec3::new(x, 0.0, y))
            .ok()?
            + corner;
        Some(egui::pos2(at.x, at.y))
    }

    /// The corners of a block of cells.
    fn corners(&self, (x, y): (f32, f32), (w, h): (f32, f32)) -> Option<Vec<Pos2>> {
        [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
            .into_iter()
            .map(|(x, y)| self.point(x, y))
            .collect()
    }

    fn outline(&self, painter: &egui::Painter, at: Cell, size: (usize, usize), stroke: Stroke) {
        let (at, size) = ((at.0 as f32, at.1 as f32), (size.0 as f32, size.1 as f32));
        if let Some(points) = self.corners(at, size) {
            painter.add(egui::Shape::closed_line(points, stroke));
        }
    }

    /// A rectangle `share` the size of a cell, in its middle.
    fn cell_rect(&self, (x, y): Cell, share: f32) -> Option<EguiRect> {
        let (x, y) = (x as f32, y as f32);
        let centre = self.point(x + 0.5, y + 0.5)?;
        let side = self
            .point(x + 1.0, y + 0.5)?
            .distance(self.point(x, y + 0.5)?);
        Some(EguiRect::from_center_size(
            centre,
            egui::Vec2::splat(side * share),
        ))
    }
}

fn block_outline(
    projection: &Projection,
    painter: &egui::Painter,
    block: Block,
    offset: (isize, isize),
    colour: Color32,
) {
    projection.outline(
        painter,
        (block.x as isize + offset.0, block.y as isize + offset.1),
        (block.width, block.height),
        Stroke::new(2.0, colour),
    );
}

#[allow(clippy::too_many_arguments)]
fn panels(
    mut contexts: EguiContexts,
    mut palette: ResMut<Palette>,
    mut studio: ResMut<Studio>,
    editing: Option<Res<Editing>>,
    data: Res<GameData>,
    icons: Res<Icons>,
    mut images: ResMut<Assets<Image>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let Some(editing) = editing else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut().cloned() else {
        return;
    };
    let palette = &mut *palette;
    let studio = &mut *studio;
    let mut root = egui::Ui::new(
        ctx.clone(),
        "editor".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::top("tools").show(&mut root, |ui| {
        tool_bar(
            ui,
            studio,
            editing.0.level().file_name.as_str(),
            editing.0.is_dirty(),
        )
    });
    egui::Panel::bottom("status").show(&mut root, |ui| status_line(ui, studio, &editing));
    egui::Panel::right("level")
        .default_size(190.0)
        .show(&mut root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                level_panel(ui, studio, palette, &editing);
            });
        });
    egui::Panel::left("palette")
        .default_size(190.0)
        .show(&mut root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                palette_panel(
                    ui,
                    palette,
                    studio,
                    &icons,
                    &data,
                    &mut images,
                    &mut contexts,
                );
            });
        });

    let area = root.available_rect_before_wrap();
    studio.canvas_area = Some(bevy::math::Rect::new(
        area.min.x, area.min.y, area.max.x, area.max.y,
    ));
    studio.typing = ctx.egui_wants_keyboard_input();
    studio.over_window = studio.pointer.is_some_and(|p| {
        ctx.layer_id_at(egui::pos2(p.x, p.y))
            .is_some_and(|layer| layer.order != egui::Order::Background)
    });

    // The stage is on show behind this.
    if studio.busy {
        let painter = root.painter_at(area);
        painter.rect_filled(area, 0.0, COVER);
        painter.text(
            area.center(),
            egui::Align2::CENTER_CENTER,
            &studio.status,
            egui::FontId::proportional(16.0),
            Color32::WHITE,
        );
    } else if let Ok((camera, transform)) = camera.single() {
        let projection = Projection { camera, transform };
        let painter = root.painter_at(area);
        canvas_marks(
            &projection,
            &painter,
            palette,
            studio,
            &editing,
            &icons,
            &data,
            &mut images,
            &mut contexts,
        );
    }
    if let (Some(carried), Some(pointer)) = (studio.carrying, studio.pointer)
        && let Some((id, uv)) = piece_icon(
            (carried.layer, carried.code),
            palette,
            &icons,
            &data,
            &mut images,
            &mut contexts,
        )
        && !studio.on_canvas()
    {
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, "carried".into()));
        let rect =
            EguiRect::from_center_size(egui::pos2(pointer.x, pointer.y), EguiVec::splat(ICON));
        painter.image(id, rect, uv, Color32::WHITE);
    }
    if let Ok((camera, transform)) = camera.single() {
        let projection = Projection { camera, transform };
        popup(&ctx, &projection, studio, &editing);
    }
    pieces_window(
        &ctx,
        palette,
        studio,
        &icons,
        &data,
        &mut images,
        &mut contexts,
    );
    dialogs(&ctx, studio, editing.0.is_dirty());
}

/// `ticks` of play as minutes and seconds.
fn clock(ticks: u32) -> String {
    let seconds = i64::from(ticks) * i64::from(Ruleset::rtwp().tick_ms) / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// What a proof's verdict comes to, in a line.
fn verdict_line(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Proven(ticks) => format!("Proven beatable, in {}", clock(*ticks)),
        Verdict::Broke {
            failure: Failure::NoStart(e),
            ..
        } => format!("The solution cannot run: {e}"),
        Verdict::Broke { failure, tick, .. } => format!(
            "The solution breaks at {}: {}",
            clock(*tick),
            match failure {
                Failure::Lost => "the level is lost",
                _ => "it no longer reaches the exit",
            }
        ),
    }
}

fn proof_panel(ui: &mut egui::Ui, studio: &mut Studio, revision: u64) {
    ui.strong("Proof");
    let proof = &studio.proof;
    if proof.checking(revision) {
        ui.label("Checking the solution");
        return;
    }
    let mut go = None;
    match &proof.verdict {
        None => {
            ui.label("Not proven. Win a Test Play from the start.");
        }
        Some(verdict @ Verdict::Proven(_)) => {
            ui.colored_label(PROVEN, verdict_line(verdict));
        }
        Some(verdict @ Verdict::Broke { cell, .. }) => {
            let line = egui::RichText::new(verdict_line(verdict)).color(WARNING);
            match cell {
                Some(cell) => {
                    if ui.link(line).clicked() {
                        go = Some(Request::GoTo(*cell));
                    }
                }
                None => {
                    ui.label(line);
                }
            }
            ui.small("Win a Test Play from the start for a new solution.");
        }
    }
    studio.request.extend(go);
}

/// The choices of the one piece selected, beside it.
fn popup(ctx: &egui::Context, projection: &Projection, studio: &mut Studio, editing: &Editing) {
    let Some(block) = studio.canvas.selection() else {
        return;
    };
    if (block.width, block.height) != (1, 1) || studio.canvas.gesture().is_some() {
        return;
    }
    let spot = (block.x, block.y);
    let (layer, code) = editing.0.top(spot.0, spot.1);
    let Some(piece) = catalogue::find(layer, code) else {
        return;
    };
    let sign = layer == Layer::Tile && SIGN.contains(&code);
    let Some(at) = projection.point(spot.0 as f32 + 1.2, spot.1 as f32) else {
        return;
    };
    if piece.axes.is_empty() {
        return;
    }
    let mut choices = piece.choices(code);
    egui::Window::new(piece.name)
        .id("piece".into())
        .collapsible(false)
        .resizable(false)
        .fixed_pos(at)
        .default_width(POPUP_WIDTH)
        .max_width(POPUP_WIDTH)
        .show(ctx, |ui| {
            for (i, axis) in shown_axes(piece) {
                ui.label(axis.name);
                if axis_choice(ui, axis, &mut choices[i]) {
                    let brush = brush(piece, &choices);
                    studio.request.push(Request::SetPiece(spot, brush));
                }
            }
            if sign {
                let slot = (code - SIGN.start) as usize;
                let mut text = sign_lines(editing.0.sign_text(slot));
                ui.label("Sign text, up to four lines:");
                let response = ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_rows(4)
                        .desired_width(190.0),
                );
                if response.changed() {
                    let text = stored_sign(&text);
                    studio.request.push(Request::SignText(slot, text));
                }
                if response.lost_focus() {
                    studio.request.push(Request::Commit);
                }
            }
        });
}

fn on_off(ui: &mut egui::Ui, studio: &mut Studio, on: bool, label: &str, code: i32) {
    let mut now = on;
    if ui.checkbox(&mut now, label).changed() {
        studio
            .request
            .push(Request::Setting(Setting::Flag(code, now)));
    }
}

/// Picks one of `names` or none, as `Option` of its place.
fn optional_list(
    ui: &mut egui::Ui,
    id: &str,
    now: Option<usize>,
    names: &[String],
) -> Option<Option<usize>> {
    let shown = now
        .and_then(|i| names.get(i))
        .map_or("None", String::as_str);
    let mut picked = None;
    egui::ComboBox::from_id_salt(id)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if ui.selectable_label(now.is_none(), "None").clicked() {
                picked = Some(None);
            }
            for (i, name) in names.iter().enumerate() {
                if ui.selectable_label(now == Some(i), name).clicked() {
                    picked = Some(Some(i));
                }
            }
        });
    picked
}

fn level_panel(ui: &mut egui::Ui, studio: &mut Studio, palette: &mut Palette, editing: &Editing) {
    let editor = &editing.0;
    let level = editor.level();
    ui.strong("Level");
    let mut ask = |request| studio.request.push(Request::Setting(request));

    let mut time = level.time;
    ui.horizontal(|ui| {
        ui.label("Time");
        let drag = egui::DragValue::new(&mut time)
            .range(settings::MIN_TIME..=settings::MAX_TIME)
            .speed(settings::TIME_STEP);
        if ui.add(drag).changed() {
            ask(Setting::Time(time - time % settings::TIME_STEP));
        }
    });
    ui.label("Style");
    egui::ComboBox::from_id_salt("style")
        .selected_text(STYLES.get(level.style as usize).copied().unwrap_or("?"))
        .show_ui(ui, |ui| {
            for (i, name) in STYLES.iter().enumerate() {
                if ui
                    .selectable_label(level.style == i as i32, *name)
                    .clicked()
                {
                    ask(Setting::Style(i as i32));
                }
            }
        });
    ui.label("Background");
    let background = BACKGROUNDS.iter().find(|(b, _)| *b == level.background);
    egui::ComboBox::from_id_salt("background")
        .selected_text(background.map_or("?", |(_, name)| name))
        .show_ui(ui, |ui| {
            for (b, name) in BACKGROUNDS {
                if ui.selectable_label(level.background == b, name).clicked() {
                    ask(Setting::Background(b));
                }
            }
        });
    ui.label("Music");
    let track = level.music_track.unwrap_or(1);
    egui::ComboBox::from_id_salt("music")
        .selected_text(MUSIC.get((track - 1) as usize).copied().unwrap_or("?"))
        .show_ui(ui, |ui| {
            for (i, name) in MUSIC.iter().enumerate() {
                if ui.selectable_label(track == i as i32 + 1, *name).clicked() {
                    ask(Setting::Music(i as i32 + 1));
                }
            }
        });
    ui.separator();
    let flags = [
        (settings::SNOW, "Snow"),
        (settings::RAIN, "Rain"),
        (settings::LIGHTNING, "Lightning"),
        (settings::FLASHES, "Colour flashes"),
        (settings::HIDDEN_RAINBOW, "Hidden rainbow world"),
    ];
    for (code, label) in flags {
        on_off(ui, studio, editor.flag(code), label, code);
    }
    let mut ask = |request| studio.request.push(Request::Setting(request));
    let mut view3d = editor.view3d();
    if ui.checkbox(&mut view3d, "3D view").changed() {
        ask(Setting::View3d(view3d));
    }
    ui.label("Tint");
    let tints: Vec<String> = TINTS.iter().map(|t| (*t).to_owned()).collect();
    if let Some(tint) = optional_list(ui, "tint", editor.tint(), &tints) {
        ask(Setting::Tint(tint));
    }
    ui.label("House style");
    let houses: Vec<String> = (1..=HOUSE_STYLES).map(|n| n.to_string()).collect();
    if let Some(style) = optional_list(ui, "houses", editor.house_style(), &houses) {
        ask(Setting::HouseStyle(style));
    }

    ui.separator();
    for (folder, names) in std::mem::take(&mut studio.folders) {
        let (label, needs) = match folder {
            Folder::Houses => ("Custom houses", None),
            Folder::Models => ("Custom models", None),
            Folder::Textures => ("Custom textures", Some(level.style == CUSTOM_STYLE)),
            Folder::Backgrounds => (
                "Custom background",
                Some(level.background == CUSTOM_BACKGROUND),
            ),
        };
        if needs == Some(false) || names.is_empty() {
            studio.folders.push((folder, names));
            continue;
        }
        ui.label(label);
        let now = editor
            .folder(folder)
            .and_then(|d| names.iter().position(|n| n.eq_ignore_ascii_case(d)));
        if let Some(picked) = optional_list(ui, label, now, &names) {
            let dir = picked.map(|i| names[i].clone());
            studio
                .request
                .push(Request::Setting(Setting::Folder(folder, dir)));
        }
        studio.folders.push((folder, names));
    }

    ui.separator();
    proof_panel(ui, studio, editor.revision());
    ui.separator();
    ui.checkbox(&mut palette.hide_links, "Hide links");
    if studio.warnings.is_empty() {
        ui.label("No warnings");
    } else {
        ui.colored_label(WARNING, format!("{} warnings", studio.warnings.len()));
    }
    let mut go = None;
    for warning in &studio.warnings {
        match warning.cell {
            Some(cell) => {
                if ui.link(&warning.message).clicked() {
                    go = Some(Request::GoTo(cell));
                }
            }
            None => {
                ui.label(&warning.message);
            }
        }
    }
    studio.request.extend(go);
}

fn tool_bar(ui: &mut egui::Ui, studio: &mut Studio, name: &str, dirty: bool) {
    ui.horizontal(|ui| {
        let mut ask = |ui: &mut egui::Ui, label: &str, hint: &str, request: Request| {
            if ui.button(label).on_hover_text(hint).clicked() {
                studio.request.push(request);
            }
        };
        ask(ui, "New", "A new level", Request::Leave(Leaving::New));
        ask(
            ui,
            "Open",
            "Open a custom level",
            Request::Leave(Leaving::Open),
        );
        ask(ui, "Save", "Ctrl+S", Request::AskToSave);
        ask(ui, "Packs", "Level packs", Request::ListPacks);
        ui.separator();
        ask(ui, "Pieces", "P: every piece, bigger", Request::ShowPieces);
        ui.separator();
        ask(ui, "Undo", "Ctrl+Z", Request::Undo);
        ask(ui, "Redo", "Ctrl+Y", Request::Redo);
        ui.separator();
        ask(ui, "Copy", "Ctrl+C", Request::Copy);
        ask(ui, "Cut", "Ctrl+X", Request::Cut);
        ask(ui, "Paste", "Ctrl+V, at the pointer", Request::Paste);
        ask(ui, "Delete", "Del", Request::Delete);
        ask(
            ui,
            "Stamp",
            "Keep the selection as a stamp",
            Request::AskToStamp,
        );
        ui.separator();
        ask(ui, "Test Play", "F5, or F6 from the pointer", Request::Play);
        ask(
            ui,
            "Turn",
            "R: the brush or the piece selected",
            Request::Turn,
        );
        ask(ui, "Tilt", "T: the play camera", Request::Tilt);
        ask(ui, "Show All", "Home", Request::ShowAll);
        ui.separator();
        ask(ui, "Menu", "Esc", Request::Leave(Leaving::Menu));
        ui.separator();
        ui.label(format!("{name}{}", if dirty { " *" } else { "" }));
    });
}

fn status_line(ui: &mut egui::Ui, studio: &Studio, editing: &Editing) {
    ui.horizontal(|ui| {
        let level = editing.0.level();
        ui.label(format!("{} x {}", level.width, level.height));
        if let Some((x, y)) = studio.hover.filter(|_| studio.on_canvas()) {
            ui.separator();
            ui.label(format!("{x}, {y}"));
            let cell = usize::try_from(x).ok().zip(usize::try_from(y).ok());
            if let Some((x, y)) = cell.filter(|&(x, y)| x < level.width && y < level.height) {
                let (layer, code) = editing.0.top(x, y);
                if let Some(piece) = catalogue::find(layer, code) {
                    ui.label(piece.name);
                }
            }
        }
        ui.separator();
        ui.label(&studio.status);
    });
}

fn palette_panel(
    ui: &mut egui::Ui,
    palette: &mut Palette,
    studio: &mut Studio,
    icons: &Icons,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) {
    // Esc can put the brush down.
    if !matches!(studio.canvas.tool, Tool::Brush(_)) {
        palette.picked = None;
    }
    ui.horizontal(|ui| {
        let tool = &mut studio.canvas.tool;
        if ui
            .selectable_label(*tool == Tool::Select, "Select")
            .clicked()
        {
            *tool = Tool::Select;
            palette.picked = None;
        }
        if ui.selectable_label(*tool == Tool::Erase, "Erase").clicked() {
            *tool = Tool::Erase;
            palette.picked = None;
        }
        if ui
            .selectable_label(*tool == Tool::Link, "Link")
            .on_hover_text("L: drag from a button to a gate, or between teleporters")
            .clicked()
        {
            *tool = Tool::Link;
            palette.picked = None;
        }
    });
    ui.small("Right button erases, Shift drags a box");
    if let Some((piece, choices)) = &mut palette.picked {
        // R may have turned the brush.
        if let Tool::Brush(b) = studio.canvas.tool
            && piece.contains(b.code)
        {
            *choices = piece.choices(b.code);
        }
        ui.separator();
        ui.strong(piece.name);
        for (i, axis) in shown_axes(piece).filter(|(_, a)| !expanded(a.name)) {
            ui.label(axis.name);
            if axis_choice(ui, axis, &mut choices[i]) {
                studio.canvas.tool = Tool::Brush(brush(piece, choices));
            }
        }
    }
    if !studio.stamps.is_empty() {
        egui::CollapsingHeader::new("Stamps").show(ui, |ui| {
            for name in &studio.stamps {
                if ui.button(name).clicked() {
                    studio.request.push(Request::PickStamp(name.clone()));
                    palette.picked = None;
                }
            }
        });
    }
    for (group, title) in GROUPS {
        egui::CollapsingHeader::new(title)
            .default_open(group == Group::Ground)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for entry in group_entries(group) {
                        palette_item(
                            ui, &entry, ICON, palette, studio, icons, data, images, contexts,
                        );
                    }
                });
            });
    }
}

/// The palette's entries for `group`.
fn group_entries(group: Group) -> impl Iterator<Item = Entry> {
    PIECES
        .iter()
        .filter(move |p| p.group == group)
        .flat_map(entries_of)
}

/// The Pieces window: one group at a time, like the palette but bigger.
fn pieces_window(
    ctx: &egui::Context,
    palette: &mut Palette,
    studio: &mut Studio,
    icons: &Icons,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) {
    let Dialog::Pieces(mut shown) = studio.dialog else {
        return;
    };
    let mut close = false;
    egui::Window::new("Pieces")
        .collapsible(false)
        .resizable(false)
        .default_width(700.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (group, title) in GROUPS {
                    ui.selectable_value(&mut shown, group, title);
                }
            });
            ui.separator();
            let tool = studio.canvas.tool;
            egui::ScrollArea::vertical()
                .max_height(440.0)
                .show(ui, |ui| {
                    ui.set_width(680.0);
                    ui.horizontal_wrapped(|ui| {
                        for entry in group_entries(shown) {
                            palette_item(
                                ui, &entry, BIG_ICON, palette, studio, icons, data, images,
                                contexts,
                            );
                        }
                    });
                });
            ui.separator();
            close = ui.button("Cancel").clicked() || studio.canvas.tool != tool;
        });
    studio.dialog = if close || studio.carrying.is_some() {
        Dialog::None
    } else {
        Dialog::Pieces(shown)
    };
}

#[allow(clippy::too_many_arguments)]
fn palette_item(
    ui: &mut egui::Ui,
    entry: &Entry,
    size: f32,
    palette: &mut Palette,
    studio: &mut Studio,
    icons: &Icons,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) {
    let piece = entry.piece;
    // The entry's own choices, or those made since it was picked.
    let own = |(p, c): &(&'static Piece, Vec<usize>)| {
        std::ptr::eq(*p, piece)
            && piece
                .axes
                .iter()
                .enumerate()
                .all(|(i, a)| !expanded(a.name) || c[i] == entry.choices[i])
    };
    let picked = palette.picked.as_ref().is_some_and(own);
    let choices = palette
        .picked
        .as_ref()
        .filter(|_| picked)
        .map_or(entry.choices.clone(), |(_, c)| c.clone());
    let piece_brush = brush(piece, &choices);
    let own = (piece.layer, entry.code());
    let icon = piece_icon(own, palette, icons, data, images, contexts);
    let name = entry.name();
    let response = match icon {
        Some((id, uv)) => egui::Image::new(SizedTexture::new(id, EguiVec::splat(size)))
            .uv(uv)
            .sense(Sense::click_and_drag())
            .ui(ui),
        None => ui.add(egui::Button::new(name.as_str()).sense(Sense::click_and_drag())),
    };
    if picked {
        ui.painter().rect_stroke(
            response.rect,
            2.0,
            Stroke::new(2.0, CHOSEN),
            StrokeKind::Outside,
        );
    }
    if response.clicked() {
        palette.picked = Some((piece, choices));
        studio.canvas.tool = Tool::Brush(piece_brush);
    }
    if response.drag_started() {
        studio.carrying = Some(piece_brush);
    }
    response.on_hover_text(name);
}

#[allow(clippy::too_many_arguments)]
fn canvas_marks(
    projection: &Projection,
    painter: &egui::Painter,
    palette: &mut Palette,
    studio: &Studio,
    editing: &Editing,
    icons: &Icons,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) {
    let level = editing.0.level();
    let (width, height) = (level.width, level.height);
    if !studio.view.tilted {
        let grid = Stroke::new(1.0, GRID);
        for x in 1..width {
            if let (Some(a), Some(b)) = (
                projection.point(x as f32, 0.0),
                projection.point(x as f32, height as f32),
            ) {
                painter.line_segment([a, b], grid);
            }
        }
        for y in 1..height {
            if let (Some(a), Some(b)) = (
                projection.point(0.0, y as f32),
                projection.point(width as f32, y as f32),
            ) {
                painter.line_segment([a, b], grid);
            }
        }
    }
    projection.outline(painter, (0, 0), (width, height), Stroke::new(2.0, EDGE));

    for y in 0..height {
        for x in 0..width {
            for (layer, code) in [
                (Layer::Object, level.object(x, y)),
                (Layer::Tile, level.tile(x, y)),
            ] {
                let inside = (1..width - 1).contains(&x) && (1..height - 1).contains(&y);
                if needs_marker(layer, code, inside)
                    && let Some((id, uv)) =
                        piece_icon((layer, code), palette, icons, data, images, contexts)
                    && let Some(rect) = projection.cell_rect((x as isize, y as isize), MARKER)
                {
                    painter.image(id, rect, uv, Color32::WHITE);
                }
            }
        }
    }

    let middle = |(x, y): (usize, usize)| projection.point(x as f32 + 0.5, y as f32 + 0.5);
    let near = |(x, y): (usize, usize)| {
        let cell = (x as isize, y as isize);
        studio.hover == Some(cell) || studio.canvas.selection().is_some_and(|s| s.contains(cell))
    };
    for link in &studio.links {
        let bright = near(link.from) || near(link.to);
        if (bright || (!palette.hide_links && !link.conveyor))
            && let (Some(a), Some(b)) = (middle(link.from), middle(link.to))
        {
            let colour = if bright { LINK } else { FAINT_LINK };
            painter.line_segment([a, b], Stroke::new(if bright { 2.0 } else { 1.0 }, colour));
        }
    }
    for warning in &studio.warnings {
        if let Some((x, y)) = warning.cell {
            let cell = (x as isize, y as isize);
            projection.outline(painter, cell, (1, 1), Stroke::new(2.0, WARNING));
        }
    }
    if let Some(Verdict::Broke {
        cell: Some((x, y)), ..
    }) = studio.proof.verdict
        && !studio.proof.checking(editing.0.revision())
    {
        let cell = (x as isize, y as isize);
        projection.outline(painter, cell, (1, 1), Stroke::new(3.0, WARNING));
    }

    if let Some(block) = studio.canvas.selection() {
        block_outline(projection, painter, block, (0, 0), CHOSEN);
    }
    let mut ghost = studio
        .carrying
        .filter(|_| studio.on_canvas())
        .zip(studio.hover);
    match studio.canvas.gesture() {
        Some(Gesture::Band { start, end }) => {
            let at = (start.0.min(end.0), start.1.min(end.1));
            let size = (start.0.abs_diff(end.0) + 1, start.1.abs_diff(end.1) + 1);
            projection.outline(painter, at, size, Stroke::new(2.0, CHOSEN));
        }
        Some(Gesture::Piece { to, brush, .. }) => ghost = Some((brush, to)),
        Some(Gesture::Move { from, to }) => {
            if let Some(block) = studio.canvas.selection() {
                block_outline(
                    projection,
                    painter,
                    block,
                    (to.0 - from.0, to.1 - from.1),
                    HOVER,
                );
            }
        }
        Some(Gesture::Link { from, to }) => {
            let centre = |(x, y): Cell| projection.point(x as f32 + 0.5, y as f32 + 0.5);
            if let (Some(a), Some(b)) = (centre(from), centre(to)) {
                painter.line_segment([a, b], Stroke::new(2.0, LINK));
            }
        }
        Some(Gesture::Stroke { .. }) => {}
        None => {
            if let (Tool::Stamp, Some(size), Some(cell), true) = (
                studio.canvas.tool,
                studio.canvas.stamp_size(),
                studio.hover,
                studio.on_canvas(),
            ) {
                projection.outline(painter, cell, size, Stroke::new(2.0, HOVER));
            }
            if let (Tool::Brush(b), Some(cell), true) =
                (studio.canvas.tool, studio.hover, studio.on_canvas())
            {
                ghost = ghost.or(Some((b, cell)));
            }
        }
    }
    if let Some(cell) = studio.hover.filter(|_| studio.on_canvas()) {
        projection.outline(painter, cell, (1, 1), Stroke::new(2.0, HOVER));
    }
    if let Some((brush, cell)) = ghost
        && let Some((id, uv)) = piece_icon(
            (brush.layer, brush.code),
            palette,
            icons,
            data,
            images,
            contexts,
        )
        && let Some(rect) = projection.cell_rect(cell, 1.0)
    {
        painter.image(id, rect, uv, Color32::from_white_alpha(170));
    }
}

fn dialogs(ctx: &egui::Context, studio: &mut Studio, dirty: bool) {
    let window = |title: &'static str| {
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    };
    let mut ask = None;
    let mut close = false;
    match &mut studio.dialog {
        Dialog::None | Dialog::Pieces(_) => {}
        Dialog::Save {
            name,
            title,
            replacing,
        } => {
            window("Save Level").show(ctx, |ui| {
                ui.label("File name, letters and digits:");
                ui.text_edit_singleline(name);
                ui.label("Title:");
                ui.text_edit_singleline(title);
                if *replacing {
                    ui.colored_label(EDGE, format!("{name} is there already."));
                }
                ui.horizontal(|ui| {
                    if ui
                        .button(if *replacing { "Replace" } else { "Save" })
                        .clicked()
                    {
                        ask = Some(Request::Save {
                            name: name.clone(),
                            title: title.clone(),
                            replace: *replacing,
                        });
                    }
                    close = ui.button("Cancel").clicked();
                });
            });
        }
        Dialog::Open(files) => {
            window("Open Level").show(ctx, |ui| {
                if files.is_empty() {
                    ui.label("There are no custom levels yet.");
                }
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for file in files.iter() {
                            if ui.button(file).clicked() {
                                ask = Some(Request::OpenFile {
                                    pack: None,
                                    file: file.clone(),
                                });
                            }
                        }
                    });
                close = ui.button("Cancel").clicked();
            });
        }
        Dialog::Unsaved(then) => {
            let then = *then;
            window("Unsaved Changes").show(ctx, |ui| {
                ui.label("This level has changes that are not saved.");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        ask = Some(Request::AskToSave);
                    }
                    if ui.button("Don't Save").clicked() {
                        ask = Some(Request::Discard(then));
                    }
                    close = ui.button("Cancel").clicked();
                });
            });
        }
        Dialog::Stamp { name, replacing } => {
            window("Keep as a Stamp").show(ctx, |ui| {
                ui.label("Name, letters and digits:");
                ui.text_edit_singleline(name);
                if *replacing {
                    ui.colored_label(EDGE, format!("{name} is there already."));
                }
                ui.horizontal(|ui| {
                    if ui
                        .button(if *replacing { "Replace" } else { "Keep" })
                        .clicked()
                    {
                        ask = Some(Request::SaveStamp {
                            name: name.clone(),
                            replace: *replacing,
                        });
                    }
                    close = ui.button("Cancel").clicked();
                });
            });
        }
        Dialog::Packs { names, new } => {
            window("Level Packs").show(ctx, |ui| {
                ui.label("A pack is a folder of levels, played in order.");
                for name in names.iter() {
                    if ui.button(name).clicked() {
                        ask = Some(Request::OpenPack(name.clone()));
                    }
                }
                ui.separator();
                ui.label("New pack, letters and digits:");
                ui.text_edit_singleline(new);
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        ask = Some(Request::NewPack(new.clone()));
                    }
                    close = ui.button("Close").clicked();
                });
            });
        }
        Dialog::Pack { name, lines } => {
            window("Level Pack").show(ctx, |ui| {
                ui.strong(name.as_str());
                if lines.is_empty() {
                    ui.label("There are no levels in this pack yet.");
                }
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for (place, line) in lines.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.add_enabled(place > 0, egui::Button::new("Up")).clicked() {
                                    ask = Some(Request::MoveUp(name.clone(), place));
                                }
                                if ui.button("Take Out").clicked() {
                                    ask = Some(Request::TakeOut(name.clone(), place));
                                }
                                let open = egui::Button::new(line.file.as_str());
                                if ui
                                    .add_enabled(!dirty, open)
                                    .on_disabled_hover_text("Save this level first")
                                    .on_hover_text(&line.title)
                                    .clicked()
                                {
                                    ask = Some(Request::OpenFile {
                                        pack: Some(name.clone()),
                                        file: line.file.clone(),
                                    });
                                }
                                match &line.proof {
                                    PackProof::Checking => {
                                        ui.label("Checking");
                                    }
                                    PackProof::None => {
                                        ui.colored_label(EDGE, "Not proven");
                                    }
                                    PackProof::Done(verdict @ Verdict::Proven(_)) => {
                                        ui.colored_label(PROVEN, verdict_line(verdict));
                                    }
                                    PackProof::Done(verdict) => {
                                        ui.colored_label(WARNING, verdict_line(verdict));
                                    }
                                }
                                if line.warnings > 0 {
                                    let count = format!("{} warnings", line.warnings);
                                    ui.colored_label(WARNING, count);
                                }
                            });
                        }
                    });
                ui.small("Moving a level renames its file, so share the whole folder.");
                ui.horizontal(|ui| {
                    if ui
                        .button("Add This Level")
                        .on_hover_text("Moves the saved level on show into the pack")
                        .clicked()
                    {
                        ask = Some(Request::AddToPack(name.clone()));
                    }
                    if ui.button("All Packs").clicked() {
                        ask = Some(Request::ListPacks);
                    }
                    close = ui.button("Close").clicked();
                });
            });
        }
    }
    if close {
        studio.dialog = Dialog::None;
    }
    studio.request.extend(ask);
}
