//! On-screen text: the HUD (`MainGameLoop`'s `DisplayText` calls) and the
//! signs `logic` keeps (`StartSign`, `ControlSigns` in `RTWP/text.bb`).
//!
//! The original draws each letter as a sprite in front of the camera on a
//! grid 50 letters across, each letter twice as tall as wide, row 0 at the
//! top of a 4:3 screen. The font is five 16-glyph strips (`res15.dat` to
//! `res20.dat`, 16x32 glyphs from ASCII 32), with alpha from brightness
//! (texture flag 2). Here each letter is a UI image node.
//!
//! The game and the menus both fill [`Screen`]: the menus add their
//! selector bar and logos as [`ScreenImage`]s, drawn under the letters.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use wonderland_logic::World;
use wonderland_logic::sign::LEVEL_NAME;

use crate::camera::GameCamera;
use crate::flow::Phase;
use crate::level::GameData;
use crate::pad::{Device, Icon, Pad, icon_png, sign_prompt};
use crate::scene::TextureCache;
use crate::shot::{headless, headless_size};
use crate::sim::{Sim, SimStep};
use crate::style::TextureRef;

/// One letter on the 50-column grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Letter {
    pub ascii: u8,
    /// Column, 0–49 (may be half a column off for centred lines).
    pub x: f32,
    /// Row from the top.
    pub y: f32,
    pub alpha: f32,
    /// Which part of a screen wider than 4:3 the column counts from:
    /// the centred 4:3 grid, or the screen's left or right edge (for the
    /// lines in the corners).
    pub edge: Edge,
}

/// See [`Letter::edge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Edge {
    #[default]
    Grid,
    Left,
    Right,
}

/// The letters moved to count from `edge`.
pub fn pinned(mut letters: Vec<Letter>, edge: Edge) -> Vec<Letter> {
    for l in &mut letters {
        l.edge = edge;
    }
    letters
}

/// `DisplayText`: a line from column `x`.
pub fn display_text(text: &str, x: f32, y: f32, alpha: f32) -> Vec<Letter> {
    text.bytes()
        .enumerate()
        .map(|(i, ascii)| Letter {
            ascii,
            x: x + i as f32,
            y,
            alpha,
            edge: Edge::Grid,
        })
        .collect()
}

/// `StartSign`'s layout: lines split at `#`, each centred on column 25,
/// the first on row `y`. Ported loop for loop, so a trailing `#` is shown
/// as a letter, as in the original.
pub fn sign_layout(text: &str, y: f32, alpha: f32) -> Vec<Letter> {
    let mut out = Vec::new();
    let mut text: Vec<u8> = text.bytes().collect();
    if text.is_empty() {
        return out;
    }
    let mut y = y;
    let mut j = 1usize;
    let centred = |line: &[u8], y: f32, out: &mut Vec<Letter>| {
        let start = 25.0 - line.len() as f32 / 2.0;
        for (i, &ascii) in line.iter().enumerate() {
            out.push(Letter {
                ascii,
                x: start + i as f32,
                y,
                alpha,
                edge: Edge::Grid,
            });
        }
    };
    while j < text.len() {
        if text[j - 1] == b'#' || j == text.len() {
            centred(&text[..j - 1], y, &mut out);
            y += 1.0;
            text = text[j..].to_vec();
            j = 0;
        }
        j += 1;
    }
    centred(&text[..j.min(text.len())], y, &mut out);
    out
}

/// `M:SS`, as `MainGameLoop` writes the time left.
pub fn time_string(seconds: i32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// The HUD lines for this tick: coins, bonus coins and the time. The coin
/// count blinks once every coin is taken, and so does a completed bonus.
pub fn hud_letters(w: &World) -> Vec<Letter> {
    let mut out = Vec::new();
    let blink_on = w.level_timer % 40 < 20;
    let (coins_word, of) = match w.options.language {
        1 => ("Taler: ", " von "),
        2 => ("Sous:  ", " de "),
        _ => ("Coins: ", " of "),
    };
    if w.coins > 0 || blink_on {
        let taken = w.total_coins - w.coins;
        out.extend(pinned(
            display_text(&format!("{coins_word}{taken}"), 0.0, 0.0, 1.0),
            Edge::Left,
        ));
        let x = if taken >= 10 { 9.0 } else { 8.0 };
        out.extend(pinned(
            display_text(&format!("{of}{}", w.total_coins), x, 0.0, 1.0),
            Edge::Left,
        ));
    }
    if w.total_bonus_coins > 0 && (w.bonus_coins > 0 || blink_on) {
        let text = format!(
            "Bonus: {} of {}",
            w.total_bonus_coins - w.bonus_coins,
            w.total_bonus_coins
        );
        out.extend(pinned(display_text(&text, 0.0, 1.0, 1.0), Edge::Left));
    }
    let time = time_string(w.map_time);
    out.extend(pinned(
        display_text(&time, 50.0 - time.len() as f32, 0.0, 1.0),
        Edge::Right,
    ));
    out
}

/// The level name's alpha: in over ticks 0–200, out over 600–800
/// (`ControlSigns`).
pub fn level_name_alpha(level_timer: i32) -> f32 {
    let t = level_timer.min(800);
    if t < 200 {
        t as f32 / 200.0
    } else if t > 600 {
        1.0 - (t - 600) as f32 / 200.0
    } else {
        // The last value set before tick 200.
        199.0 / 200.0
    }
}

/// Every letter on screen: the signs, then the HUD.
pub fn all_letters(w: &World, device: Device) -> Vec<Letter> {
    let mut out = Vec::new();
    for s in w.signs_shown() {
        let alpha = if s.tag == LEVEL_NAME {
            level_name_alpha(w.level_timer)
        } else {
            s.alpha
        };
        out.extend(sign_layout(&sign_prompt(&s.text, device), s.y, alpha));
    }
    out.extend(hud_letters(w));
    out
}

/// A picture in front of the camera, in letter-grid units: columns
/// across, rows down from row 0.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenImage {
    pub texture: TextureRef,
    /// Left, top, right and bottom.
    pub rect: [f32; 4],
    /// The part of the texture shown, 0–1 (left, top, right, bottom).
    pub uv: [f32; 4],
    /// sRGB colour and alpha it is multiplied by.
    pub colour: [f32; 4],
    /// Turned counter-clockwise about its centre, in degrees.
    pub turn: f32,
}

/// Where a rectangle in front of a zoom-2 camera lands on the letter grid:
/// centre `(x, y)` and half-size `(hx, hy)` at depth `z`, in camera space.
/// Letters sit at depth 1, where the grid is 1 unit across.
pub fn camera_rect(x: f32, y: f32, z: f32, hx: f32, hy: f32) -> [f32; 4] {
    let col = |x: f32| (x / z + 0.5) / 0.02;
    let row = |y: f32| (0.375 - y / z) / 0.04;
    [col(x - hx), row(y + hy), col(x + hx), row(y - hy)]
}

/// What is drawn over the 3D view this frame.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Screen {
    pub images: Vec<ScreenImage>,
    pub letters: Vec<Letter>,
}

/// The root UI node the letters hang under.
#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct Glyph;

/// What the screen was last drawn from.
#[derive(Resource, Default)]
struct Drawn {
    screen: Screen,
    size: Vec2,
    fonts: Vec<Handle<Image>>,
    device: Device,
    icons: HashMap<(Device, Icon), Handle<Image>>,
}

/// Runs after the game or the menus have filled [`Screen`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreenFilled;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Drawn>()
            .init_resource::<Screen>()
            .add_systems(Startup, spawn_root)
            .add_systems(
                Update,
                (
                    game_screen
                        .after(SimStep)
                        .before(ScreenFilled)
                        .run_if(in_state(Phase::Game)),
                    draw_screen.after(ScreenFilled),
                ),
            );
    }
}

fn game_screen(sim: Res<Sim>, pad: Res<Pad>, mut screen: ResMut<Screen>) {
    screen.images.clear();
    screen.letters = match &sim.ending {
        Some(e) => e.text.shown(),
        None => all_letters(&sim.world, pad.device),
    };
    if let Some(menu) = &sim.menu {
        screen.letters.extend(menu.text.shown());
    }
}

fn spawn_root(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        HudRoot,
    ));
}

/// The picture of `icon` on `device`, decoded the first time it is shown.
fn icon_handle(
    device: Device,
    icon: Icon,
    drawn: &mut Drawn,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    if let Some(handle) = drawn.icons.get(&(device, icon)) {
        return Some(handle.clone());
    }
    let png = image::load_from_memory(icon_png(device, icon)?).ok()?;
    let image = Image::new(
        Extent3d {
            width: png.width(),
            height: png.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        png.into_rgba8().into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    let handle = images.add(image);
    drawn.icons.insert((device, icon), handle.clone());
    Some(handle)
}

#[allow(clippy::too_many_arguments)]
fn draw_screen(
    mut commands: Commands,
    screen: Res<Screen>,
    pad: Res<Pad>,
    data: Res<GameData>,
    mut drawn: ResMut<Drawn>,
    mut cache: ResMut<TextureCache>,
    mut images: ResMut<Assets<Image>>,
    window: Query<(), With<PrimaryWindow>>,
    root: Query<(Entity, Has<UiTargetCamera>), With<HudRoot>>,
    camera: Query<(Entity, &Camera), With<GameCamera>>,
    glyphs: Query<Entity, With<Glyph>>,
) {
    // The UI is laid out in the camera's viewport (see `camera`).
    let viewport = camera
        .single()
        .ok()
        .and_then(|(_, c)| c.logical_viewport_size());
    let size = match viewport {
        Some(s) if window.single().is_ok() || headless() => s,
        _ if headless() => {
            let (w, h) = headless_size();
            Vec2::new(w as f32, h as f32)
        }
        _ => return,
    };
    if *screen == drawn.screen && size == drawn.size && pad.device == drawn.device {
        return;
    }
    let Ok((root, targeted)) = root.single() else {
        return;
    };
    // The UI goes where the game camera draws, which is an image when
    // headless.
    if !targeted && let Ok((cam, _)) = camera.single() {
        commands.entity(root).insert(UiTargetCamera(cam));
    }
    if drawn.fonts.is_empty() {
        drawn.fonts = (15..=20)
            .map(|n| {
                let r = TextureRef {
                    path: format!("res{n}.dat"),
                    flags: 2,
                    rotation: 0,
                };
                cache.get(&r, &data, &mut images).0
            })
            .collect();
    }
    for g in &glyphs {
        commands.entity(g).despawn();
    }
    // Columns are a fiftieth of the width; rows twice that, with row 0
    // where the top of a 4:3 picture would be.
    // On a screen wider than 4:3 the grid stays in the centred 4:3 area.
    let width = size.x.min(size.y * 4.0 / 3.0);
    let left = (size.x - width) / 2.0;
    let cell = width / 50.0;
    let top = size.y / 2.0 - 0.375 * width;
    let pictures: Vec<(Handle<Image>, Option<Vec2>, &ScreenImage)> = screen
        .images
        .iter()
        .map(|i| {
            let handle = cache.get(&i.texture, &data, &mut images).0;
            let px = images.get(&handle).map(|im| im.size_f32());
            (handle, px, i)
        })
        .collect();
    let icons = Icon::ALL.map(|icon| icon_handle(pad.device, icon, &mut drawn, &mut images));
    commands.entity(root).with_children(|ui| {
        for (handle, px, i) in pictures {
            let [l, t, r, b] = i.rect;
            let mut image = ImageNode::new(handle).with_mode(NodeImageMode::Stretch);
            if let Some(px) = px {
                let [u0, v0, u1, v1] = i.uv;
                image.rect = Some(Rect::new(u0 * px.x, v0 * px.y, u1 * px.x, v1 * px.y));
            }
            let [cr, cg, cb, ca] = i.colour;
            image.color = Color::srgba(cr, cg, cb, ca);
            ui.spawn((
                image,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left + l * cell),
                    top: Val::Px(top + t * cell * 2.0),
                    width: Val::Px((r - l) * cell),
                    height: Val::Px((b - t) * cell * 2.0),
                    ..default()
                },
                UiTransform::from_rotation(Rot2::degrees(-i.turn)),
                Glyph,
            ));
        }
        let node = |x: f32, y: f32, width: f32, l: &Letter| Node {
            position_type: PositionType::Absolute,
            left: Val::Px(
                match l.edge {
                    Edge::Grid => left,
                    Edge::Left => 0.0,
                    Edge::Right => 2.0 * left,
                } + x * cell,
            ),
            top: Val::Px(top + y * cell * 2.0),
            width: Val::Px(width),
            height: Val::Px(cell * 2.0),
            ..default()
        };
        for l in &screen.letters {
            if let Some(icon) = Icon::from_byte(l.ascii) {
                if let Some(handle) = &icons[icon as usize - 1] {
                    let mut image = ImageNode::new(handle.clone());
                    image.color = Color::srgba(1.0, 1.0, 1.0, l.alpha);
                    ui.spawn((image, node(l.x, l.y, cell * 2.0, l), Glyph));
                }
                continue;
            }
            if l.ascii <= b' ' {
                continue;
            }
            let index = (l.ascii - 32) as usize;
            let Some(font) = drawn.fonts.get(index / 16) else {
                continue;
            };
            let f = (index % 16) as f32;
            let mut image = ImageNode::new(font.clone());
            image.rect = Some(Rect::new(f * 16.0, 0.0, f * 16.0 + 16.0, 32.0));
            image.color = Color::srgba(1.0, 1.0, 1.0, l.alpha);
            ui.spawn((image, node(l.x, l.y, cell, l), Glyph));
        }
    });
    drawn.device = pad.device;
    drawn.screen = screen.clone();
    drawn.size = size;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(letters: &[Letter], y: f32) -> (String, f32) {
        let row: Vec<&Letter> = letters.iter().filter(|l| l.y == y).collect();
        (
            row.iter().map(|l| l.ascii as char).collect(),
            row.first().map_or(-1.0, |l| l.x),
        )
    }

    #[test]
    fn signs_split_and_centre_lines() {
        let l = sign_layout("AB#CDEF", 7.0, 1.0);
        assert_eq!(line(&l, 7.0), ("AB".into(), 24.0));
        assert_eq!(line(&l, 8.0), ("CDEF".into(), 23.0));
        // Odd lengths sit half a column over.
        let l = sign_layout("ABC", 0.0, 1.0);
        assert_eq!(line(&l, 0.0), ("ABC".into(), 23.5));
        // A blank line between, as in the level-complete text.
        let l = sign_layout("X# #Y", 2.0, 1.0);
        assert_eq!(line(&l, 3.0), (" ".into(), 24.5));
        assert_eq!(line(&l, 4.0).0, "Y");
        // The loop stops one short, so a trailing # is shown.
        assert_eq!(line(&sign_layout("AB#", 0.0, 1.0), 0.0).0, "AB#");
    }

    #[test]
    fn time_reads_minutes_and_seconds() {
        assert_eq!(time_string(125), "2:05");
        assert_eq!(time_string(0), "0:00");
    }

    #[test]
    fn level_name_fades() {
        assert_eq!(level_name_alpha(0), 0.0);
        assert_eq!(level_name_alpha(100), 0.5);
        assert!(level_name_alpha(700) < 0.6);
    }
}
