//! The palette's icons: the real pieces, as they look in the game.
//!
//! Game art cannot ship with the source, so the first visit to the editor
//! makes them. [`wonderland_editor::stage`] lays one of each piece out in a
//! level, well apart, and a second camera looks at it from about the play
//! camera's angle and draws into one [`Image`]. Pipelines build in the
//! background, and a piece drawn before its pipeline is ready is missing, so
//! the picture is read back once none are waiting, once over black and once
//! over white: what changes between them is what can be seen through, which
//! gives each pixel its true opacity, even in glass and ghosts. Each piece is
//! found by what is left and cut out at one size, into a small sheet, so that
//! a little coin and a house both fill their icons. A piece with no icon
//! keeps its sprite from the original editor.
//!
//! The stage wears the [`Look`] of the level being edited, so the palette's
//! walls and floors are that level's. A new look is photographed when the
//! level takes it on, and the sheets made are kept until the game closes.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bevy::asset::RenderAssetUsages;
use bevy::camera::{OrthographicProjection, Projection, RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::ExtractSchedule;
use bevy::render::RenderApp;
use bevy::render::render_resource::{Extent3d, PipelineCache, TextureDimension, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use wonderland_editor::Editor;
use wonderland_editor::settings::SNOW;
use wonderland_editor::stage::{PITCH, ROW_PITCH, expanded, stage};
use wonderland_formats::Level;
use wonderland_formats::catalogue::{self, Layer};
use wonderland_formats::level::CustomDir;

use crate::camera::FAR;
use crate::editor_scene::{Editing, Studio, begin_stage};
use crate::flow::Phase;
use crate::level::GameData;
use crate::scene::{LitFace, spawn_level};

/// Pixels to a cell of the stage, if the graphics card takes a picture that
/// big.
const SCALE: f32 = 80.0;
/// How far the camera looks down, in degrees.
const PITCH_DEGREES: f32 = 50.0;
/// The most a piece stands above its cell, in cells.
const TALLEST: f32 = 3.0;
/// How far above its cell, up the picture, a piece may start: some fly clear
/// of their shadows.
const HOVERS: f32 = 2.4;
/// The opacity a pixel needs to count as part of a piece.
const FAINT: u8 = 12;
/// Pixels of an icon, and of room around the piece in the source.
const ICON: usize = 128;
const MARGIN: usize = 6;
/// The fewest source pixels an icon shows, so that nothing is blown up.
const SMALLEST: usize = 48;
/// Icons across the finished sheet.
const SHEET_COLUMNS: usize = 16;
/// Frames to let the level settle before believing the pipelines are done.
const SETTLE: u32 = 10;
/// Frames to give up after, keeping the sprites.
const MOST_FRAMES: u32 = 1200;

/// The backdrop that forces the rainbow style.
const RAINBOW: i32 = 10;
/// Tiles whose texture has the piece painted on floor.
const TELEPORTER: (bool, i32) = (true, 1200);
const SIGN: (bool, i32) = (true, 1300);
/// The sign in its tile, as half sizes.
const SIGN_SIZE: [f32; 2] = [0.51, 0.5];

/// Render pipelines still building.
#[derive(Resource, Clone, Default)]
struct Waiting(Arc<AtomicUsize>);

fn count_waiting(cache: Res<PipelineCache>, waiting: Res<Waiting>) {
    let count = cache.waiting_pipelines().count();
    waiting.0.store(count, Ordering::Relaxed);
}

/// The part of the sheet a piece's icon is, as UV corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

/// A block of pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Area {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

/// A piece found in a picture: its bounds, and which pixels of `area` it has.
struct Shape {
    bounds: Area,
    area: Area,
    seen: Vec<bool>,
}

impl Shape {
    /// Keeps the pixels `inside` likes, given their place in the bounds from
    /// -1 to 1, for a tile whose texture has floor painted around the piece.
    fn keep(&mut self, inside: impl Fn(f32, f32) -> bool) {
        let Area {
            x: left,
            y: top,
            width,
            height,
        } = self.bounds;
        let place = |at: usize, low: usize, size: usize| {
            ((at - low) as f32 + 0.5) / size as f32 * 2.0 - 1.0
        };
        let (mut low, mut high) = ((usize::MAX, usize::MAX), (0, 0));
        for y in 0..self.area.height {
            for x in 0..self.area.width {
                let (px, py) = (self.area.x + x, self.area.y + y);
                let i = y * self.area.width + x;
                if !self.seen[i] {
                    continue;
                }
                if inside(place(px, left, width), place(py, top, height)) {
                    low = (low.0.min(px), low.1.min(py));
                    high = (high.0.max(px), high.1.max(py));
                } else {
                    self.seen[i] = false;
                }
            }
        }
        if low.0 != usize::MAX {
            self.bounds = Area {
                x: low.0,
                y: low.1,
                width: high.0 - low.0 + 1,
                height: high.1 - low.1 + 1,
            };
        }
    }

    fn has(&self, x: usize, y: usize) -> bool {
        let Area {
            x: left,
            y: top,
            width,
            height,
        } = self.area;
        (left..left + width).contains(&x)
            && (top..top + height).contains(&y)
            && self.seen[(y - top) * width + (x - left)]
    }
}

/// An sRGB level as light.
fn to_linear(level: u8) -> f32 {
    let v = f32::from(level) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(light: f32) -> u8 {
    let v = if light <= 0.0031308 {
        light * 12.92
    } else {
        1.055 * light.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

/// The picture with no backdrop, from the same picture over black and over
/// white, RGBA with plain alpha. Blending happens in linear light, so a pixel
/// is `colour * a` over black and `colour * a + 1 - a` over white.
fn lift(dark: &[u8], light: &[u8]) -> Vec<u8> {
    let table: Vec<f32> = (0..=255).map(to_linear).collect();
    let mut out = Vec::with_capacity(dark.len());
    for (d, l) in dark.chunks_exact(4).zip(light.chunks_exact(4)) {
        let (d, l) = (&d[..3], &l[..3]);
        let through: f32 = (0..3)
            .map(|c| table[usize::from(l[c])] - table[usize::from(d[c])])
            .sum();
        let alpha = (1.0 - through / 3.0).clamp(0.0, 1.0);
        for &level in d {
            out.push(to_srgb(table[usize::from(level)] / alpha.max(1.0 / 255.0)));
        }
        out.push((alpha * 255.0).round() as u8);
    }
    out
}

/// An RGBA picture with no backdrop.
struct Picture<'a> {
    pixels: &'a [u8],
    width: usize,
    height: usize,
}

impl Picture<'_> {
    fn pixel(&self, x: usize, y: usize) -> &[u8] {
        let at = (y * self.width + x) * 4;
        &self.pixels[at..at + 4]
    }

    /// Whether a pixel is part of a piece.
    fn inked(&self, x: usize, y: usize) -> bool {
        self.pixel(x, y)[3] >= FAINT
    }

    /// What is joined to anything in `seed`, staying in `area`: a neighbour
    /// that reaches into the area is not joined to the piece.
    fn find(&self, area: Area, seed: Area) -> Option<Shape> {
        let right = (area.x + area.width).min(self.width);
        let bottom = (area.y + area.height).min(self.height);
        let index = |x: usize, y: usize| (y - area.y) * area.width + (x - area.x);
        let mut seen = vec![false; area.width * area.height];
        let mut stack = vec![];
        for y in seed.y.max(area.y)..(seed.y + seed.height).min(bottom) {
            for x in seed.x.max(area.x)..(seed.x + seed.width).min(right) {
                if self.inked(x, y) {
                    seen[index(x, y)] = true;
                    stack.push((x, y));
                }
            }
        }
        let (mut low, mut high): (Option<(usize, usize)>, (usize, usize)) = (None, (0, 0));
        while let Some((x, y)) = stack.pop() {
            let (a, b) = low.unwrap_or((x, y));
            low = Some((a.min(x), b.min(y)));
            high = (high.0.max(x), high.1.max(y));
            let beside = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            for (nx, ny) in beside {
                let inside = (area.x..right).contains(&nx) && (area.y..bottom).contains(&ny);
                if inside && !seen[index(nx, ny)] && self.inked(nx, ny) {
                    seen[index(nx, ny)] = true;
                    stack.push((nx, ny));
                }
            }
        }
        low.map(|(a, b)| Shape {
            bounds: Area {
                x: a,
                y: b,
                width: high.0 - a + 1,
                height: high.1 - b + 1,
            },
            area,
            seen,
        })
    }

    /// Draws the square of `side` around the middle of `shape` into the
    /// [`ICON`]-sized block of `sheet` at `at`, averaging the pixels each icon
    /// pixel covers. The sheet has premultiplied alpha, as egui wants.
    fn cut(&self, shape: &Shape, side: usize, sheet: &mut [u8], at: (usize, usize)) {
        let piece = shape.bounds;
        let left = (piece.x + piece.width / 2) as isize - (side / 2) as isize;
        let top = (piece.y + piece.height / 2) as isize - (side / 2) as isize;
        let sheet_width = SHEET_COLUMNS * ICON;
        for y in 0..ICON {
            let (y0, y1) = (
                y * side / ICON,
                ((y + 1) * side / ICON).max(y * side / ICON + 1),
            );
            for x in 0..ICON {
                let (x0, x1) = (
                    x * side / ICON,
                    ((x + 1) * side / ICON).max(x * side / ICON + 1),
                );
                let mut sum = [0u32; 4];
                let mut count = 0;
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        count += 1;
                        let (px, py) = (left + sx as isize, top + sy as isize);
                        if px < 0 || py < 0 || !shape.has(px as usize, py as usize) {
                            continue;
                        }
                        let p = self.pixel(px as usize, py as usize);
                        let alpha = u32::from(p[3]);
                        (0..3).for_each(|c| sum[c] += u32::from(p[c]) * alpha / 255);
                        sum[3] += alpha;
                    }
                }
                let to = ((at.1 + y) * sheet_width + at.0 + x) * 4;
                for c in 0..4 {
                    sheet[to + c] = (sum[c] / count) as u8;
                }
            }
        }
    }
}

/// Where a piece is in the picture.
#[derive(Debug, Clone, Copy)]
struct Place {
    key: (bool, i32),
    /// Its share of the picture.
    area: Area,
    /// Around its cell, where some of it is sure to be.
    seed: Area,
}

/// The side of the square that shows `piece`, in `area`'s pixels.
fn side_for(piece: Area, area: Area) -> usize {
    (piece.width.max(piece.height) + MARGIN * 2)
        .max(SMALLEST)
        .min(area.width.max(area.height))
}

/// Cuts each piece of `places` out of `picture`: a sheet of pixels with
/// [`SHEET_COLUMNS`] across, and where each icon is. A piece that cannot be
/// found has no icon.
fn cut_out(
    picture: &Picture,
    places: &[Place],
) -> (Vec<u8>, usize, HashMap<(bool, i32), IconRect>) {
    let rows = places.len().div_ceil(SHEET_COLUMNS).max(1);
    let (width, height) = (SHEET_COLUMNS * ICON, rows * ICON);
    let mut sheet = vec![0; width * height * 4];
    let mut rects = HashMap::new();
    for (i, &Place { key, area, seed }) in places.iter().enumerate() {
        let Some(mut shape) = picture.find(area, seed) else {
            continue;
        };
        match key {
            TELEPORTER => shape.keep(|x, y| x * x + y * y <= 0.84),
            SIGN => shape.keep(|x, y| x.abs() <= SIGN_SIZE[0] && y.abs() <= SIGN_SIZE[1]),
            _ => {}
        }
        let at = (i % SHEET_COLUMNS * ICON, i / SHEET_COLUMNS * ICON);
        picture.cut(&shape, side_for(shape.bounds, area), &mut sheet, at);
        let uv = |x: usize, y: usize| [x as f32 / width as f32, y as f32 / height as f32];
        rects.insert(
            key,
            IconRect {
                min: uv(at.0, at.1),
                max: uv(at.0 + ICON, at.1 + ICON),
            },
        );
    }
    (sheet, height, rects)
}

/// What of a level changes how its pieces look.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Look {
    style: i32,
    /// The rainbow backdrop forces a style of its own. No other backdrop
    /// changes a piece, but the water and lava ones leave those tiles
    /// without a top, so the stage keeps its own.
    rainbow: bool,
    houses: Option<CustomDir>,
    models: Option<CustomDir>,
    textures: Option<CustomDir>,
    snow: bool,
}

impl Look {
    fn of(editor: &Editor) -> Self {
        let level = editor.level();
        Self {
            style: level.style,
            rainbow: level.background == RAINBOW,
            houses: level.houses.clone(),
            models: level.models.clone(),
            textures: level.textures.clone(),
            snow: editor.flag(SNOW),
        }
    }

    fn dress(&self, level: &mut Level) {
        level.style = self.style;
        if self.rainbow {
            level.background = RAINBOW;
        }
        level.houses.clone_from(&self.houses);
        level.models.clone_from(&self.models);
        level.textures.clone_from(&self.textures);
    }
}

/// The icons of one look.
#[derive(Debug, Clone)]
struct Sheet {
    image: Handle<Image>,
    rects: HashMap<(bool, i32), IconRect>,
}

/// The icons, and how far along they are.
#[derive(Resource, Debug, Default)]
pub struct Icons {
    /// The sheet of icons on show.
    pub image: Option<Handle<Image>>,
    rects: HashMap<(bool, i32), IconRect>,
    stage: Option<Stage>,
    /// The look on show, or being made.
    look: Option<Look>,
    /// Every look made so far.
    made: Vec<(Look, Sheet)>,
}

#[derive(Debug)]
struct Stage {
    camera: Entity,
    places: Vec<Place>,
    target: Handle<Image>,
    frames: u32,
    settled: u32,
    /// The picture over black, once it has come.
    dark: Option<Image>,
    /// Whether the picture in hand has been asked for.
    asked: bool,
    picture: Arc<Mutex<Option<Image>>>,
}

impl Icons {
    /// Whether the level on show is the stage.
    pub fn staging(&self) -> bool {
        self.stage.is_some()
    }

    fn show(&mut self, sheet: Sheet) {
        self.image = Some(sheet.image);
        self.rects = sheet.rects;
    }

    /// The icon for a code: its own, or else the one for its piece, as only
    /// a palette entry's own code is photographed.
    pub fn get(&self, layer: Layer, code: i32) -> Option<IconRect> {
        let key = |code| (layer == Layer::Tile, code);
        let own = self.rects.get(&key(code));
        own.or_else(|| {
            let piece = catalogue::find(layer, code)?;
            let choices: Vec<usize> = (piece.axes.iter().zip(piece.choices(code)))
                .map(|(axis, choice)| if expanded(axis.name) { choice } else { 0 })
                .collect();
            self.rects.get(&key(piece.code(&choices)))
        })
        .copied()
    }
}

pub struct EditorIconsPlugin;

impl Plugin for EditorIconsPlugin {
    fn build(&self, app: &mut App) {
        let waiting = Waiting::default();
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .insert_resource(waiting.clone())
                .add_systems(ExtractSchedule, count_waiting);
        }
        app.insert_resource(waiting)
            .init_resource::<Icons>()
            .add_systems(
                Update,
                (begin, watch)
                    .chain()
                    .before(crate::editor_scene::EditorSteps)
                    .run_if(in_state(Phase::Editor)),
            );
    }
}

/// Puts the stage up and the camera on it, when the level takes on a look
/// with no sheet yet.
#[allow(clippy::too_many_arguments)]
fn begin(
    mut commands: Commands,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    mut studio: ResMut<Studio>,
    data: Res<GameData>,
    device: Option<Res<RenderDevice>>,
    editing: Option<Res<Editing>>,
    faces: Query<Entity, With<LitFace>>,
) {
    let Some(editing) = editing else {
        return;
    };
    let look = Look::of(&editing.0);
    if icons.stage.is_some() || icons.look.as_ref() == Some(&look) {
        return;
    }
    icons.look = Some(look.clone());
    let made = icons.made.iter().find(|(l, _)| *l == look);
    if let Some(sheet) = made.map(|(_, sheet)| sheet.clone()) {
        icons.show(sheet);
        return;
    }
    let (mut level, cells) = stage();
    look.dress(&mut level);
    let (width, height) = (level.width as f32, level.height as f32);
    let (sin, cos) = PITCH_DEGREES.to_radians().sin_cos();
    let high = height * sin + TALLEST * cos;
    let most = device.map_or(u32::MAX, |d| d.limits().max_texture_dimension_2d) as f32;
    let scale = SCALE.min(most / width.max(high));
    let centre = Vec3::new(width / 2.0, 0.0, height / 2.0);
    let (shown, sim) = match begin_stage(&data, level, look.snow) {
        Ok(started) => started,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    let target = images.add(Image::new_target_texture(
        (width * scale) as u32,
        (high * scale) as u32,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let forward = Vec3::new(0.0, -sin, -cos);
    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            RenderTarget::Image(target.clone().into()),
            Tonemapping::None,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width,
                    height: high,
                },
                far: FAR,
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(centre - forward * 80.0).looking_at(centre, Vec3::Y),
        ))
        .id();

    // Each piece's share of the picture: its cell's stretch of ground and
    // what stands on it, which reaches up the picture.
    let (across, down) = (PITCH as f32 * scale, ROW_PITCH as f32 * sin * scale);
    let places = cells
        .iter()
        .map(|&(layer, code, x, y)| {
            let ground = (y as f32 + 0.5) * sin + TALLEST * cos;
            let bottom = (ground + ROW_PITCH as f32 * sin / 4.0) * scale;
            let middle = (x as f32 + 0.5) * scale;
            let area = Area {
                x: (middle - across / 2.0) as usize,
                y: (bottom - down).max(0.0) as usize,
                width: across as usize,
                height: down as usize,
            };
            let seed = Area {
                x: (middle - scale) as usize,
                y: ((ground - HOVERS) * scale).max(0.0) as usize,
                width: (scale * 2.0) as usize,
                height: ((HOVERS + 0.6) * scale) as usize,
            };
            Place {
                key: (layer == Layer::Tile, code),
                area,
                seed,
            }
        })
        .collect();
    icons.stage = Some(Stage {
        camera,
        places,
        target,
        frames: 0,
        settled: 0,
        dark: None,
        asked: false,
        picture: Arc::default(),
    });
    studio.status = "Making the palette's pictures".to_owned();
    studio.busy = true;
    commands.insert_resource(shown);
    commands.insert_resource(sim);
    for e in &faces {
        commands.entity(e).despawn();
    }
    commands.run_system_cached(spawn_level);
}

/// Reads the picture back when it has all it is going to have, and makes the
/// sheet from it.
fn watch(
    mut commands: Commands,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    mut studio: ResMut<Studio>,
    mut cameras: Query<&mut Camera>,
    waiting: Res<Waiting>,
) {
    let icons = &mut *icons;
    let Some(stage) = &mut icons.stage else {
        return;
    };
    stage.frames += 1;
    if waiting.0.load(Ordering::Relaxed) == 0 {
        stage.settled += 1;
    } else {
        stage.settled = 0;
    }
    if stage.settled >= SETTLE && !stage.asked {
        stage.asked = true;
        let picture = stage.picture.clone();
        commands
            .spawn(Screenshot::image(stage.target.clone()))
            .observe(move |captured: On<ScreenshotCaptured>| {
                if let Ok(mut slot) = picture.lock() {
                    *slot = Some(captured.image.clone());
                }
            });
    }
    let taken = stage.picture.lock().ok().and_then(|mut slot| slot.take());
    if taken.is_none() && stage.frames < MOST_FRAMES {
        return;
    }
    if stage.dark.is_none() && stage.frames < MOST_FRAMES {
        stage.dark = taken;
        stage.asked = false;
        stage.settled = 0;
        if let Ok(mut camera) = cameras.get_mut(stage.camera) {
            camera.clear_color = ClearColorConfig::Custom(Color::WHITE);
        }
        return;
    }
    let rgba = |image: Option<Image>| {
        let image = image?.try_into_dynamic().ok()?;
        Some(image.to_rgba8())
    };
    if let (Some(dark), Some(light)) = (rgba(stage.dark.take()), rgba(taken)) {
        let pixels = lift(dark.as_raw(), light.as_raw());
        let picture = Picture {
            width: dark.width() as usize,
            height: dark.height() as usize,
            pixels: &pixels,
        };
        let (sheet, height, rects) = cut_out(&picture, &stage.places);
        let image = images.add(Image::new(
            Extent3d {
                width: (SHEET_COLUMNS * ICON) as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            sheet,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));
        let sheet = Sheet { image, rects };
        if let Some(look) = icons.look.clone() {
            icons.made.push((look, sheet.clone()));
        }
        icons.image = Some(sheet.image);
        icons.rects = sheet.rects;
    }
    commands.entity(stage.camera).despawn();
    images.remove(&stage.target);
    icons.stage = None;
    studio.busy = false;
    studio.status.clear();
    studio.redraw();
}

#[cfg(test)]
mod tests {
    use wonderland_editor::settings::Setting;

    use super::*;

    const INK: [u8; 4] = [200, 20, 20, 255];

    /// A 40 by 30 picture of backdrop, with a block of ink.
    fn picture(ink: Area) -> Vec<u8> {
        let mut pixels = vec![];
        for y in 0..30 {
            for x in 0..40 {
                let inked = (ink.x..ink.x + ink.width).contains(&x)
                    && (ink.y..ink.y + ink.height).contains(&y);
                pixels.extend(if inked { INK } else { [0; 4] });
            }
        }
        pixels
    }

    const ALL: Area = Area {
        x: 0,
        y: 0,
        width: 40,
        height: 30,
    };

    #[test]
    fn a_piece_is_found_by_what_differs_from_the_backdrop() {
        let ink = Area {
            x: 10,
            y: 5,
            width: 7,
            height: 3,
        };
        let pixels = picture(ink);
        let p = Picture {
            pixels: &pixels,
            width: 40,
            height: 30,
        };
        assert_eq!(p.find(ALL, ALL).map(|s| s.bounds), Some(ink));
        let elsewhere = Area {
            x: 20,
            y: 20,
            width: 5,
            height: 5,
        };
        assert!(p.find(ALL, elsewhere).is_none());
    }

    #[test]
    fn a_neighbour_reaching_into_the_area_is_left_out() {
        let ink = Area {
            x: 10,
            y: 5,
            width: 7,
            height: 3,
        };
        let mut pixels = picture(ink);
        // Something else, along the bottom of the area.
        for x in 0..40 {
            pixels[(25 * 40 + x) * 4..][..4].copy_from_slice(&INK);
        }
        let p = Picture {
            pixels: &pixels,
            width: 40,
            height: 30,
        };
        let seed = Area {
            x: 12,
            y: 6,
            width: 2,
            height: 2,
        };
        assert_eq!(p.find(ALL, seed).map(|s| s.bounds), Some(ink));
    }

    #[test]
    fn a_small_piece_is_not_blown_up_and_a_big_one_stays_in_its_share() {
        let tiny = Area {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let share = Area {
            x: 0,
            y: 0,
            width: 400,
            height: 300,
        };
        assert_eq!(side_for(tiny, share), SMALLEST);
        let big = Area {
            width: 1000,
            ..tiny
        };
        assert_eq!(side_for(big, share), 400);
    }

    #[test]
    fn a_cut_fills_its_icon_and_an_empty_place_has_none() {
        let ink = Area {
            x: 12,
            y: 10,
            width: 16,
            height: 16,
        };
        let pixels = picture(ink);
        let p = Picture {
            pixels: &pixels,
            width: 40,
            height: 30,
        };
        let nothing = Area {
            x: 30,
            y: 0,
            width: 10,
            height: 8,
        };
        let places = [
            Place {
                key: (true, 1),
                area: ALL,
                seed: ALL,
            },
            Place {
                key: (true, 2),
                area: nothing,
                seed: nothing,
            },
        ];
        let (sheet, height, rects) = cut_out(&p, &places);
        assert_eq!(height, ICON);
        assert_eq!(rects.len(), 1);
        let rect = rects[&(true, 1)];
        assert_eq!(rect.min, [0.0, 0.0]);
        // The middle of the icon is the piece; its corner is clear.
        let at = |x: usize, y: usize| &sheet[(y * SHEET_COLUMNS * ICON + x) * 4..][..4];
        assert_eq!(at(ICON / 2, ICON / 2), &INK[..]);
        assert_eq!(at(1, 1), &[0; 4][..]);
    }

    #[test]
    fn a_neighbour_in_the_square_is_not_drawn() {
        let ink = Area {
            x: 12,
            y: 10,
            width: 16,
            height: 16,
        };
        let mut pixels = picture(ink);
        pixels[(2 * 40 + 2) * 4..][..4].copy_from_slice(&INK);
        let p = Picture {
            pixels: &pixels,
            width: 40,
            height: 30,
        };
        let seed = Area {
            x: 18,
            y: 16,
            width: 2,
            height: 2,
        };
        let places = [Place {
            key: (true, 1),
            area: ALL,
            seed,
        }];
        let (sheet, ..) = cut_out(&p, &places);
        let side = side_for(ink, ALL);
        assert!(side > 26);
        let inked = sheet.chunks_exact(4).filter(|p| p[3] > 0).count();
        let all = ICON * ICON;
        assert!(inked < all, "{inked} of {all}");
        // The stray pixel is at the far corner, outside the piece's shape.
        let corner = (0..ICON / 8).flat_map(|y| (0..ICON / 8).map(move |x| (x, y)));
        for (x, y) in corner {
            assert_eq!(sheet[(y * SHEET_COLUMNS * ICON + x) * 4 + 3], 0);
        }
    }

    #[test]
    fn a_style_or_a_folder_is_a_new_look_and_a_plain_backdrop_is_not() {
        let mut editor = Editor::new();
        let look = Look::of(&editor);
        editor.apply(Setting::Background(3));
        assert_eq!(Look::of(&editor), look);
        editor.apply(Setting::Style(look.style + 1));
        assert_ne!(Look::of(&editor), look);
        let mut dressed = Editor::new().level().clone();
        Look::of(&editor).dress(&mut dressed);
        assert_eq!(dressed.style, look.style + 1);
    }

    #[test]
    fn painted_floor_is_trimmed_off() {
        let ink = Area {
            x: 10,
            y: 10,
            width: 20,
            height: 10,
        };
        let pixels = picture(ink);
        let p = Picture {
            pixels: &pixels,
            width: 40,
            height: 30,
        };
        let mut shape = p.find(ALL, ALL).unwrap();
        shape.keep(|x, _| x >= 0.0);
        assert_eq!((shape.bounds.x, shape.bounds.width), (20, 10));
        assert!(shape.has(25, 15) && !shape.has(15, 15));
    }

    #[test]
    fn lifting_finds_opaque_clear_and_see_through_pixels() {
        let half = to_srgb(0.5);
        // Solid red, nothing, and a half see-through white.
        let dark = [200, 20, 20, 255, 0, 0, 0, 255, half, half, half, 255];
        let light = [200, 20, 20, 255, 255, 255, 255, 255, 255, 255, 255, 255];
        let out = lift(&dark, &light);
        assert_eq!(&out[..4], &[200, 20, 20, 255]);
        assert_eq!(out[7], 0);
        assert!((127..=129).contains(&out[11]), "{}", out[11]);
        assert!(out[8] >= 253, "{}", out[8]);
    }
}
