//! The palette's icons: the real pieces, as they look in the game.
//!
//! Game art cannot ship with the source, so the first visit to the editor
//! makes them. [`wonderland_editor::stage`] lays one of each piece out in a
//! level, well apart, and a second camera looks at it from about the play
//! camera's angle and draws into one [`Image`]. Pipelines build in the
//! background, and a piece drawn before its pipeline is ready is missing, so
//! the picture is read back once none are waiting. Each piece is found in it
//! by what differs from the backdrop and cut out at one size, into a small
//! sheet, so that a little coin and a house both fill their icons. A piece
//! with no icon keeps its sprite from the original editor.

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
use wonderland_editor::stage::{PITCH, ROW_PITCH, stage};
use wonderland_formats::catalogue::Layer;

use crate::camera::FAR;
use crate::editor_scene::{Studio, begin_stage};
use crate::flow::Phase;
use crate::level::GameData;
use crate::scene::spawn_level;

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
const BEHIND: [u8; 3] = [209, 214, 227];
/// How far a colour is from [`BEHIND`] for it to be part of a piece.
const TOLERANCE: i32 = 12;
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

/// An RGBA picture.
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

    /// Whether a pixel is part of a piece rather than the backdrop.
    fn inked(&self, x: usize, y: usize) -> bool {
        let p = self.pixel(x, y);
        (0..3).any(|c| (i32::from(p[c]) - i32::from(BEHIND[c])).abs() > TOLERANCE)
    }

    /// The bounds of what is joined to anything in `seed`, staying in `area`:
    /// a neighbour that reaches into the area is not joined to the piece.
    fn bounds(&self, area: Area, seed: Area) -> Option<Area> {
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
        low.map(|(a, b)| Area {
            x: a,
            y: b,
            width: high.0 - a + 1,
            height: high.1 - b + 1,
        })
    }

    /// Draws the square of `side` around the middle of `piece` into the
    /// [`ICON`]-sized block of `sheet` at (`to_x`, `to_y`), averaging the
    /// pixels each icon pixel covers.
    fn cut(&self, piece: Area, side: usize, sheet: &mut [u8], at: (usize, usize)) {
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
                let mut sum = [0u32; 3];
                let mut count = 0;
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        let (px, py) = (left + sx as isize, top + sy as isize);
                        let colour: [u8; 3] = if px >= 0
                            && py >= 0
                            && (px as usize) < self.width
                            && (py as usize) < self.height
                        {
                            let p = self.pixel(px as usize, py as usize);
                            [p[0], p[1], p[2]]
                        } else {
                            BEHIND
                        };
                        (0..3).for_each(|c| sum[c] += u32::from(colour[c]));
                        count += 1;
                    }
                }
                let to = ((at.1 + y) * sheet_width + at.0 + x) * 4;
                for c in 0..3 {
                    sheet[to + c] = (sum[c] / count) as u8;
                }
                sheet[to + 3] = 255;
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
        let Some(piece) = picture.bounds(area, seed) else {
            continue;
        };
        let at = (i % SHEET_COLUMNS * ICON, i / SHEET_COLUMNS * ICON);
        picture.cut(piece, side_for(piece, area), &mut sheet, at);
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

/// The icons, and how far along they are.
#[derive(Resource, Debug, Default)]
pub struct Icons {
    /// The sheet of icons.
    pub image: Option<Handle<Image>>,
    rects: HashMap<(bool, i32), IconRect>,
    stage: Option<Stage>,
    done: bool,
}

#[derive(Debug)]
struct Stage {
    camera: Entity,
    places: Vec<Place>,
    target: Handle<Image>,
    frames: u32,
    settled: u32,
    /// The picture, once it has been asked for and has come.
    asked: bool,
    picture: Arc<Mutex<Option<Image>>>,
}

impl Icons {
    /// Whether the level on show is the stage, or soon will be.
    pub fn staging(&self) -> bool {
        !self.done
    }

    pub fn get(&self, layer: Layer, code: i32) -> Option<IconRect> {
        self.done
            .then(|| self.rects.get(&(layer == Layer::Tile, code)))
            .flatten()
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

/// Puts the stage up and the camera on it, the first time.
fn begin(
    mut commands: Commands,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    mut studio: ResMut<Studio>,
    data: Res<GameData>,
    device: Option<Res<RenderDevice>>,
) {
    if icons.done || icons.stage.is_some() {
        return;
    }
    let (level, cells) = stage();
    let (width, height) = (level.width as f32, level.height as f32);
    let (sin, cos) = PITCH_DEGREES.to_radians().sin_cos();
    let high = height * sin + TALLEST * cos;
    let most = device.map_or(u32::MAX, |d| d.limits().max_texture_dimension_2d) as f32;
    let scale = SCALE.min(most / width.max(high));
    let centre = Vec3::new(width / 2.0, 0.0, height / 2.0);
    let (shown, sim) = match begin_stage(&data, level) {
        Ok(started) => started,
        Err(e) => {
            error!("{e}");
            icons.done = true;
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
    let [r, g, b] = BEHIND;
    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::srgb_u8(r, g, b)),
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
        asked: false,
        picture: Arc::default(),
    });
    studio.status = "Making the palette's pictures".to_owned();
    studio.busy = true;
    commands.insert_resource(shown);
    commands.insert_resource(sim);
    commands.run_system_cached(spawn_level);
}

/// Reads the picture back when it has all it is going to have, and makes the
/// sheet from it.
fn watch(
    mut commands: Commands,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    mut studio: ResMut<Studio>,
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
    if let Some(rgba) = taken
        .and_then(|i| i.try_into_dynamic().ok())
        .map(|i| i.to_rgba8())
    {
        let picture = Picture {
            width: rgba.width() as usize,
            height: rgba.height() as usize,
            pixels: rgba.as_raw(),
        };
        let (sheet, height, rects) = cut_out(&picture, &stage.places);
        icons.rects = rects;
        icons.image = Some(images.add(Image::new(
            Extent3d {
                width: (SHEET_COLUMNS * ICON) as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            sheet,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )));
    }
    commands.entity(stage.camera).despawn();
    images.remove(&stage.target);
    icons.stage = None;
    icons.done = true;
    studio.busy = false;
    studio.status.clear();
    studio.redraw();
}

#[cfg(test)]
mod tests {
    use super::*;

    const INK: [u8; 4] = [200, 20, 20, 255];

    /// A 40 by 30 picture of backdrop, with a block of ink.
    fn picture(ink: Area) -> Vec<u8> {
        let mut pixels = vec![];
        for y in 0..30 {
            for x in 0..40 {
                let inked = (ink.x..ink.x + ink.width).contains(&x)
                    && (ink.y..ink.y + ink.height).contains(&y);
                let [r, g, b] = BEHIND;
                pixels.extend(if inked { INK } else { [r, g, b, 255] });
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
        assert_eq!(p.bounds(ALL, ALL), Some(ink));
        let elsewhere = Area {
            x: 20,
            y: 20,
            width: 5,
            height: 5,
        };
        assert_eq!(p.bounds(ALL, elsewhere), None);
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
        assert_eq!(p.bounds(ALL, seed), Some(ink));
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
        // The middle of the icon is the piece; its corner is backdrop.
        let at = |x: usize, y: usize| &sheet[(y * SHEET_COLUMNS * ICON + x) * 4..][..3];
        assert_eq!(at(ICON / 2, ICON / 2), &INK[..3]);
        assert_eq!(at(1, 1), &BEHIND[..]);
    }
}
