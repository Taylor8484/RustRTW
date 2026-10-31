//! The editor's panels and the marks over the canvas, in egui.
//!
//! The tool bar, the palette and a status line surround the canvas. What is
//! left is [`Studio::canvas_area`], and everything drawn there is projected
//! from the level, so it stays right when the camera tilts: the cell grid,
//! the pointer's cell, the selection and sprites for the pieces that cannot
//! be seen from above. The sprites are the original editor's.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui::{
    self, Color32, Pos2, Rect as EguiRect, Sense, Stroke, StrokeKind, TextureId, Vec2 as EguiVec,
    Widget, load::SizedTexture,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};
use wonderland_editor::canvas::{Block, Brush, Cell, Gesture, Tool};
use wonderland_formats::catalogue::{self, Group, Layer, PIECES, Piece, Sprite};

use crate::camera::GameCamera;
use crate::editor_scene::{Dialog, Editing, Leaving, Request, Studio};
use crate::flow::Phase;
use crate::level::GameData;
use crate::texture;

/// A palette sprite's size.
const ICON: f32 = 32.0;
const GROUPS: [(Group, &str); 9] = [
    (Group::Ground, "Ground"),
    (Group::Wall, "Walls"),
    (Group::Item, "Items"),
    (Group::Character, "Characters"),
    (Group::Box, "Boxes"),
    (Group::Monster, "Monsters"),
    (Group::Hazard, "Hazards"),
    (Group::Switch, "Switches"),
    (Group::Scenery, "Scenery"),
];
const GRID: Color32 = Color32::from_rgba_premultiplied(30, 30, 30, 30);
const EDGE: Color32 = Color32::from_rgb(255, 220, 90);
const HOVER: Color32 = Color32::WHITE;
const CHOSEN: Color32 = Color32::from_rgb(90, 200, 255);

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

fn brush(piece: &Piece, choices: &[usize]) -> Brush {
    Brush {
        layer: piece.layer,
        code: piece.code(choices),
    }
}

/// A piece seen edge on or not at all from above, which gets its sprite.
fn needs_marker(layer: Layer, code: i32) -> bool {
    const COIN: i32 = 5;
    const CLOCK: i32 = 7;
    const BONUS_COIN: i32 = 8;
    matches!(code, COIN | CLOCK | BONUS_COIN) && layer == Layer::Object
        || catalogue::find(layer, code).is_some_and(|p| p.group == Group::Setting)
}

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
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
    egui::Panel::left("palette")
        .default_size(190.0)
        .show(&mut root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                palette_panel(ui, palette, studio, &data, &mut images, &mut contexts);
            });
        });

    let area = root.available_rect_before_wrap();
    studio.canvas_area = Some(bevy::math::Rect::new(
        area.min.x, area.min.y, area.max.x, area.max.y,
    ));
    studio.typing = ctx.egui_wants_keyboard_input();

    if let Ok((camera, transform)) = camera.single() {
        let projection = Projection { camera, transform };
        let painter = root.painter_at(area);
        canvas_marks(
            &projection,
            &painter,
            palette,
            studio,
            &editing,
            &data,
            &mut images,
            &mut contexts,
        );
    }
    if let (Some(carried), Some(pointer)) = (studio.carrying, studio.pointer)
        && let Some(sprite) = catalogue::sprite(carried.layer, carried.code)
        && let Some((id, uv)) = palette.icon(sprite, &data, &mut images, &mut contexts)
        && !studio.on_canvas()
    {
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, "carried".into()));
        let rect =
            EguiRect::from_center_size(egui::pos2(pointer.x, pointer.y), EguiVec::splat(ICON));
        painter.image(id, rect, uv, Color32::WHITE);
    }
    dialogs(&ctx, studio);
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
        ui.separator();
        ask(ui, "Undo", "Ctrl+Z", Request::Undo);
        ask(ui, "Redo", "Ctrl+Y", Request::Redo);
        ui.separator();
        ask(ui, "Copy", "Ctrl+C", Request::Copy);
        ask(ui, "Cut", "Ctrl+X", Request::Cut);
        ask(ui, "Paste", "Ctrl+V, at the pointer", Request::Paste);
        ask(ui, "Delete", "Del", Request::Delete);
        ui.separator();
        ask(ui, "Test Play", "F5, or F6 from the pointer", Request::Play);
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
    });
    ui.small("Right button erases, Shift drags a box");
    if let Some((piece, choices)) = &mut palette.picked {
        ui.separator();
        ui.strong(piece.name);
        for (i, axis) in piece.axes.iter().enumerate() {
            egui::ComboBox::from_id_salt(axis.name)
                .selected_text(axis.values[choices[i]])
                .show_ui(ui, |ui| {
                    for (v, value) in axis.values.iter().enumerate() {
                        ui.selectable_value(&mut choices[i], v, *value);
                    }
                });
        }
        studio.canvas.tool = Tool::Brush(brush(piece, choices));
    }
    for (group, title) in GROUPS {
        egui::CollapsingHeader::new(title)
            .default_open(group == Group::Ground)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for piece in PIECES.iter().filter(|p| p.group == group) {
                        palette_item(ui, piece, palette, studio, data, images, contexts);
                    }
                });
            });
    }
}

fn palette_item(
    ui: &mut egui::Ui,
    piece: &'static Piece,
    palette: &mut Palette,
    studio: &mut Studio,
    data: &GameData,
    images: &mut Assets<Image>,
    contexts: &mut EguiContexts,
) {
    let zeroes = vec![0; piece.axes.len()];
    let picked = palette
        .picked
        .as_ref()
        .is_some_and(|(p, _)| std::ptr::eq(*p, piece));
    let choices = palette
        .picked
        .as_ref()
        .filter(|_| picked)
        .map_or(zeroes.clone(), |(_, c)| c.clone());
    let piece_brush = brush(piece, &choices);
    let sprite = catalogue::sprite(piece.layer, piece_brush.code);
    let icon = sprite.and_then(|s| palette.icon(s, data, images, contexts));
    let response = match icon {
        Some((id, uv)) => egui::Image::new(SizedTexture::new(id, EguiVec::splat(ICON)))
            .uv(uv)
            .sense(Sense::click_and_drag())
            .ui(ui),
        None => ui.add(egui::Button::new(piece.name).sense(Sense::click_and_drag())),
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
    response.on_hover_text(piece.name);
}

#[allow(clippy::too_many_arguments)]
fn canvas_marks(
    projection: &Projection,
    painter: &egui::Painter,
    palette: &mut Palette,
    studio: &Studio,
    editing: &Editing,
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
                if needs_marker(layer, code)
                    && let Some(sprite) = catalogue::sprite(layer, code)
                    && let Some((id, uv)) = palette.icon(sprite, data, images, contexts)
                    && let Some(rect) = projection.cell_rect((x as isize, y as isize), 0.5)
                {
                    painter.image(id, rect, uv, Color32::WHITE);
                }
            }
        }
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
        Some(Gesture::Stroke { .. }) => {}
        None => {
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
        && let Some(sprite) = catalogue::sprite(brush.layer, brush.code)
        && let Some((id, uv)) = palette.icon(sprite, data, images, contexts)
        && let Some(rect) = projection.cell_rect(cell, 1.0)
    {
        painter.image(id, rect, uv, Color32::from_white_alpha(170));
    }
}

fn dialogs(ctx: &egui::Context, studio: &mut Studio) {
    let window = |title: &'static str| {
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    };
    let mut ask = None;
    let mut close = false;
    match &mut studio.dialog {
        Dialog::None => {}
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
                                ask = Some(Request::OpenFile(file.clone()));
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
    }
    if close {
        studio.dialog = Dialog::None;
    }
    studio.request.extend(ask);
}
