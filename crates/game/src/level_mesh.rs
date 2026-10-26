//! The level mesh: `ConstructLevel` and the `LevelAdd*`/`ShapeAdd*`
//! functions below it (`RTWP/level.bb`), plus the backgrounds it builds.
//!
//! Everything here is in Blitz coordinates (x right, y up, z = -row, a
//! tile spanning `x..x+1` and `-y-1..-y`) with Blitz's winding, so the code
//! reads like the original; [`crate::coords`] converts to Bevy once.
//!
//! Each tile class gets its surfaces in the original order: floors (with
//! object tiles drawn as floor 100), walls, the deep hole, fake walls,
//! water, ice, conveyors, lava and teleporters. Flat surfaces are covered
//! by the largest rectangles the original's scan finds, and wall sides run
//! as far along a row or column as the same condition holds, so the texture
//! coordinates match. Breakaway tiles, switchers, electro tiles and the
//! other object tiles are objects (step 3.2).
//!
//! `ConstructLevel` also turns pillars, x-floors (104), box factories and
//! warpers into walls (200) and fake walls into floor once the mesh is
//! built. That rewrite is gameplay, not geometry, and is not done here.

use wonderland_logic::Grid;

/// Which level texture a surface uses (see [`crate::style::LevelTextures`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tex {
    /// `FloorTexture` to `FloorTexture4`.
    Floor(usize),
    FloorSign,
    Trampoline,
    WallTop,
    /// `WallTextureSide1a` (0) to `3b` (5).
    WallSide(usize),
    Water,
    Lava,
    /// `IceTexture` (0) and the four curves.
    Ice(usize),
    /// `ConveyorTexture1–4(set)`: `set` 0 for tiles 500–503 and 1 for
    /// 508–511, which scroll separately.
    Conveyor {
        set: usize,
        dir: usize,
    },
    Teleport(usize),
    /// `BackGroundTexture`.
    Background,
}

/// Two triangles over four corners, as `AddVertex`/`AddTriangle` made them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub positions: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
    /// Corner indices in Blitz's winding (clockwise seen from the front).
    pub triangles: [[u32; 3]; 2],
}

impl Quad {
    /// The face normal from the first triangle, as `UpdateNormals` gives
    /// it for a flat quad.
    pub fn normal(&self) -> [f32; 3] {
        let [a, b, c] = self.triangles[0].map(|i| self.positions[i as usize]);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len == 0.0 {
            [0.0, 1.0, 0.0]
        } else {
            n.map(|v| v / len)
        }
    }
}

/// One `CreateSurface`: a texture, a brush alpha and its quads.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelSurface {
    pub texture: Tex,
    /// `BrushAlpha`: 0.8 for the water and lava tops, else 1.
    pub alpha: f32,
    pub quads: Vec<Quad>,
}

/// A `CreatePlane` backdrop: an endless plane at height `y`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub y: f32,
    pub texture: Tex,
    /// `ScaleTexture`: world units per texture repeat.
    pub scale: f32,
    /// `EntityAlpha`.
    pub alpha: f32,
    /// The sky planes drift with the water texture: `WaterTexu * drift`.
    pub drift: f32,
}

/// What `ConstructLevel` puts around the level.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Background {
    pub planes: Vec<Plane>,
    /// The edge walls of backgrounds 2 and 4.
    pub surfaces: Vec<LevelSurface>,
    /// Background 7: the inside of a sphere scaled (200, 25, 200) around
    /// this point, with the texture scaled by 0.25.
    pub warp_sphere: Option<[f32; 3]>,
}

/// A working copy of `MapTile` for the scans. Reads past the level return
/// -1; the original reads stale tiles there, but every such read is
/// `And`ed with a bounds test that is already false.
struct Tiles {
    width: i32,
    height: i32,
    tiles: Vec<i32>,
}

impl Tiles {
    fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width() as i32, grid.height() as i32);
        let tiles = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| grid.get(x, y))
            .collect();
        Self {
            width,
            height,
            tiles,
        }
    }

    fn get(&self, x: i32, y: i32) -> i32 {
        if (0..self.width).contains(&x) && (0..self.height).contains(&y) {
            self.tiles[(y * self.width + x) as usize]
        } else {
            -1
        }
    }

    fn set(&mut self, x: i32, y: i32, t: i32) {
        self.tiles[(y * self.width + x) as usize] = t;
    }

    fn any(&self, t: i32) -> bool {
        self.tiles.contains(&t)
    }

    /// Rewrites every tile matching `f` to `to`, returning the originals.
    fn rewrite(&mut self, to: i32, f: impl Fn(i32) -> bool) -> Vec<i32> {
        let saved = self.tiles.clone();
        for t in &mut self.tiles {
            if f(*t) {
                *t = to;
            }
        }
        saved
    }

    /// "Finally go through and undelete".
    fn undelete(&mut self) {
        for t in &mut self.tiles {
            if *t > 10000 {
                *t -= 10000;
            }
        }
    }
}

/// `ConstructLevel`'s surfaces for the level in `grid`, before any object
/// has changed a tile. `background` is the effective `MapBackground`.
pub fn construct_level(grid: &Grid, background: i32) -> Vec<LevelSurface> {
    let mut m = Tiles::new(grid);
    let mut out = Vec::new();

    // Object tiles with floor underneath draw as floor 100.
    let saved = m.rewrite(100, |t| {
        matches!(t / 100, 8 | 10 | 11 | 22) || (504..508).contains(&t) || t == 104
    });
    for (tile, i) in [(100, 0), (101, 1), (102, 2), (103, 3)] {
        add_floor(&mut m, &mut out, tile, Tex::Floor(i));
    }
    add_floor(&mut m, &mut out, 1300, Tex::FloorSign);
    add_floor(&mut m, &mut out, 1800, Tex::Trampoline);
    m.tiles = saved;

    for (tile, height) in [(200, 0.9), (202, 1.35), (203, 1.8)] {
        add_wall(
            &mut m,
            &mut out,
            tile,
            [Tex::WallTop, Tex::WallSide(0), Tex::WallSide(1)],
            height,
        );
    }
    add_deep_hole(&mut m, &mut out, 201, Tex::WallSide(3), Tex::WallSide(2));
    for (tile, height) in [(1500, 0.9), (1501, 1.35), (1502, 1.8)] {
        add_wall(
            &mut m,
            &mut out,
            tile,
            [Tex::WallTop, Tex::WallSide(4), Tex::WallSide(5)],
            height,
        );
    }

    // Water breakaways and transporters have water underneath.
    let saved = m.rewrite(300, |t| (t / 100 == 6 && t < 603) || t == 1700);
    add_liquid(&mut m, &mut out, 300, Tex::Water, -2.0, background != 5);
    m.tiles = saved;

    for i in 0..5 {
        add_floor(&mut m, &mut out, 400 + i as i32, Tex::Ice(i));
    }
    for dir in 0..4 {
        add_floor(
            &mut m,
            &mut out,
            500 + dir as i32,
            Tex::Conveyor { set: 0, dir },
        );
    }
    for dir in 0..4 {
        add_floor(
            &mut m,
            &mut out,
            508 + dir as i32,
            Tex::Conveyor { set: 1, dir },
        );
    }

    // And lava under the second breakaways and transporters.
    let saved = m.rewrite(900, |t| {
        (t / 100 == 6 && (603..606).contains(&t)) || t == 1702
    });
    add_liquid(&mut m, &mut out, 900, Tex::Lava, -1.0, background != 6);
    m.tiles = saved;

    for i in 0..8 {
        add_floor(&mut m, &mut out, 1200 + i as i32, Tex::Teleport(i));
    }
    out
}

/// `LevelAddFloor`.
fn add_floor(m: &mut Tiles, out: &mut Vec<LevelSurface>, tile: i32, texture: Tex) {
    if !m.any(tile) {
        return;
    }
    out.push(surface(texture, 1.0, shape_flat(m, tile, 0.0)));
}

/// `LevelAddWall`: the top, then the light sides (left and back) and the
/// dark sides (right and front).
fn add_wall(m: &mut Tiles, out: &mut Vec<LevelSurface>, tile: i32, tex: [Tex; 3], height: f32) {
    if !m.any(tile) {
        return;
    }
    out.push(surface(tex[0], 1.0, shape_flat(m, tile, height)));
    out.push(surface(
        tex[1],
        1.0,
        shape_walls(m, tile, height, 0, &LIGHT),
    ));
    out.push(surface(tex[2], 1.0, shape_walls(m, tile, height, 0, &DARK)));
}

/// `LevelAddWater` and `LevelAddLava`: the bed (unless the backdrop is the
/// same liquid), the sides, and the see-through top at -0.2.
fn add_liquid(
    m: &mut Tiles,
    out: &mut Vec<LevelSurface>,
    tile: i32,
    top: Tex,
    depth: f32,
    bed: bool,
) {
    if !m.any(tile) {
        return;
    }
    if bed {
        out.push(surface(Tex::WallSide(2), 1.0, shape_flat(m, tile, depth)));
    }
    out.push(surface(
        Tex::WallSide(2),
        1.0,
        shape_walls(m, tile, depth, 0, &ALL),
    ));
    if bed {
        out.push(surface(top, 0.8, shape_flat(m, tile, -0.2)));
    }
}

/// `LevelAddDeepHole`: sides 50 deep, none along the level edge.
fn add_deep_hole(m: &mut Tiles, out: &mut Vec<LevelSurface>, tile: i32, light: Tex, dark: Tex) {
    if !m.any(tile) {
        return;
    }
    out.push(surface(light, 1.0, shape_walls(m, tile, -50.0, 1, &LIGHT)));
    out.push(surface(dark, 1.0, shape_walls(m, tile, -50.0, 1, &DARK)));
}

fn surface(texture: Tex, alpha: f32, quads: Vec<Quad>) -> LevelSurface {
    LevelSurface {
        texture,
        alpha,
        quads,
    }
}

/// `ShapeAddFlat`: covers the tiles with rectangles, each as wide as the
/// first row allows and then as deep as every column allows.
fn shape_flat(m: &mut Tiles, tile: i32, height: f32) -> Vec<Quad> {
    let mut quads = Vec::new();
    for j in 0..m.height {
        for i in 0..m.width {
            if m.get(i, j) != tile {
                continue;
            }
            let mut i3 = i;
            while i3 + 1 < m.width && m.get(i3 + 1, j) == tile {
                i3 += 1;
            }
            let mut j2 = j;
            let j3 = loop {
                let stop = (i..=i3).any(|i2| j2 + 1 == m.height || m.get(i2, j2 + 1) != tile);
                if stop {
                    break j2;
                }
                j2 += 1;
            };
            let (w, d) = ((i3 - i + 1) as f32, (j3 - j + 1) as f32);
            let (i, j, i3f, j3f) = (i as f32, j as f32, i3 as f32, j3 as f32);
            quads.push(Quad {
                positions: [
                    [-0.001 + i, height, -j + 0.001],
                    [1.001 + i3f, height, -j + 0.001],
                    [-0.001 + i, height, -j3f - 1.001],
                    [1.001 + i3f, height, -j3f - 1.001],
                ],
                uvs: [[0.0, 0.0], [w, 0.0], [0.0, d], [w, d]],
                triangles: [[0, 1, 2], [3, 2, 1]],
            });
            let (i, j, i3) = (i as i32, j as i32, i3f as i32);
            for i2 in i..=i3 {
                for j2 in j..=j3 {
                    let t = m.get(i2, j2);
                    m.set(i2, j2, t + 10000);
                }
            }
        }
    }
    m.undelete();
    quads
}

/// One of the four wall scans in `ShapeAddWalls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
    Back,
    Front,
}

const LIGHT: [Side; 2] = [Side::Left, Side::Back];
const DARK: [Side; 2] = [Side::Right, Side::Front];
const ALL: [Side; 4] = [Side::Left, Side::Right, Side::Back, Side::Front];

/// `ShapeAddWalls` (all sides), `ShapeAddWallsLight` (left and back) and
/// `ShapeAddWallsDark` (right and front): vertical faces from height 0 to
/// `height` wherever the tile meets a different one. `edge` 1 leaves out
/// the faces on the level's outer edge.
fn shape_walls(m: &mut Tiles, tile: i32, height: f32, edge: i32, sides: &[Side]) -> Vec<Quad> {
    let mut quads = Vec::new();
    for &side in sides {
        wall_side(m, tile, height, edge, side, &mut quads);
        m.undelete();
    }
    quads
}

fn wall_side(m: &mut Tiles, tile: i32, h: f32, edge: i32, side: Side, quads: &mut Vec<Quad>) {
    let (w, ht) = (m.width, m.height);
    let other = |t: i32| t != tile && t != tile + 10000;
    // The scan's outer and inner loops, and the neighbour that makes a face.
    let (outer, inner, dn): (i32, (i32, i32), (i32, i32)) = match side {
        Side::Left => (ht, (edge, w - 1), (-1, 0)),
        Side::Right => (ht, (0, w - 1 - edge), (1, 0)),
        Side::Back => (w, (edge, ht - 1), (0, -1)),
        Side::Front => (w, (0, ht - 1 - edge), (0, 1)),
    };
    let vertical = matches!(side, Side::Left | Side::Right);
    for a in 0..outer {
        for b in inner.0..=inner.1 {
            let (i, j) = if vertical { (b, a) } else { (a, b) };
            if m.get(i, j) != tile {
                continue;
            }
            let at_edge = match side {
                Side::Left => i == 0,
                Side::Right => i == w - 1,
                Side::Back => j == 0,
                Side::Front => j == ht - 1,
            };
            if !at_edge && !other(m.get(i + dn.0, j + dn.1)) {
                continue;
            }
            // Run along the column (left/right) or row (back/front).
            let (limit, step) = if vertical { (ht, (0, 1)) } else { (w, (1, 0)) };
            let mut k = 1;
            let pos = |k: i32| (i + step.0 * k, j + step.1 * k);
            if at_edge {
                loop {
                    let (x, y) = pos(k);
                    let along = if vertical { y } else { x };
                    if !(m.get(x, y) == tile && along < limit) {
                        break;
                    }
                    k += 1;
                }
            } else {
                loop {
                    let (x, y) = pos(k);
                    let along = if vertical { y } else { x };
                    if !(m.get(x, y) == tile
                        && other(m.get(x + dn.0, y + dn.1))
                        && along < limit - 1)
                    {
                        break;
                    }
                    k += 1;
                }
            }
            let len = k as f32;
            // v at the bottom: 1, or the depth for the deep hole.
            let v0 = if h == -50.0 { h } else { 1.0 };
            let (i_f, j_f) = (i as f32, j as f32);
            let (positions, triangles) = match side {
                Side::Left | Side::Right => {
                    let x = if side == Side::Left { i_f } else { i_f + 1.0 };
                    let z1 = -j_f - len;
                    let p = [[x, h, -j_f], [x, 0.0, -j_f], [x, h, z1], [x, 0.0, z1]];
                    let t = match (side, h > 0.0) {
                        (Side::Left, true) => [[0, 2, 1], [1, 2, 3]],
                        (Side::Left, false) => [[0, 3, 1], [0, 2, 3]],
                        (_, true) => [[0, 1, 2], [1, 3, 2]],
                        (_, false) => [[3, 0, 1], [2, 0, 3]],
                    };
                    (p, t)
                }
                Side::Back | Side::Front => {
                    let z = if side == Side::Back { -j_f } else { -j_f - 1.0 };
                    let x1 = i_f + len;
                    let p = [[i_f, h, z], [i_f, 0.0, z], [x1, h, z], [x1, 0.0, z]];
                    let t = match (side, h > 0.0) {
                        (Side::Back, true) => [[1, 2, 0], [3, 2, 1]],
                        (Side::Back, false) => [[1, 3, 2], [1, 2, 0]],
                        (_, true) => [[2, 1, 0], [2, 3, 1]],
                        (_, false) => [[1, 0, 3], [3, 0, 2]],
                    };
                    (p, t)
                }
            };
            quads.push(Quad {
                positions,
                uvs: [[0.0, 0.0], [0.0, v0], [len, 0.0], [len, v0]],
                triangles,
            });
            for kk in 0..k {
                let (x, y) = pos(kk);
                let t = m.get(x, y);
                m.set(x, y, t + 10000);
            }
        }
    }
}

/// The backdrop `ConstructLevel` adds for `background` (after fallbacks).
/// `style` is the effective `MapStyle`.
pub fn construct_background(
    background: i32,
    style: i32,
    width: usize,
    height: usize,
) -> Background {
    let mut bg = Background::default();
    let (w, h) = (width as f32, height as f32);
    if matches!(background, 0 | 1 | 3 | 8 | 9 | 10 | 11) {
        let drift = if matches!(background, 0 | 9 | 10) {
            40.0
        } else {
            0.0
        };
        let alpha = if background == 0 { 0.5 } else { 1.0 };
        bg.planes.push(Plane {
            y: -50.0,
            texture: Tex::Background,
            scale: 50.0,
            alpha,
            drift,
        });
    }
    if background == 0 {
        bg.planes.push(Plane {
            y: -60.0,
            texture: Tex::Background,
            scale: 50.0,
            alpha: 1.0,
            drift: 20.0,
        });
    }
    let plane = |y: f32, texture: Tex| Plane {
        y,
        texture,
        scale: 1.0,
        alpha: 1.0,
        drift: 0.0,
    };
    match background {
        5 => {
            bg.planes.push(plane(-2.001, Tex::WallSide(2)));
            bg.planes.push(plane(-0.2, Tex::Water));
        }
        6 => {
            bg.planes.push(plane(-2.1, Tex::WallSide(2)));
            bg.planes.push(plane(-0.2, Tex::Lava));
        }
        7 => bg.warp_sphere = Some([(width / 2) as f32, 0.0, -((height / 8) as f32)]),
        _ => {}
    }

    let quad = |p: [[f32; 3]; 4], uv: [[f32; 2]; 4], t: [[u32; 3]; 2]| Quad {
        positions: p,
        uvs: uv,
        triangles: t,
    };
    if background == 2 {
        // Slanted edge walls.
        let (left_tex, right_tex) = if style == 4 {
            (Tex::WallTop, Tex::WallTop)
        } else {
            (Tex::WallSide(3), Tex::WallSide(2))
        };
        let left = vec![
            quad(
                [
                    [1.0, 0.0, 10.0],
                    [1.0, 0.0, -10.0 - h],
                    [-9.0, 10.0, -10.0 - h],
                    [-9.0, 10.0, 10.0],
                ],
                [[20.0 + h, 14.46], [0.0, 14.46], [0.0, 0.0], [20.0 + h, 0.0]],
                [[2, 3, 0], [2, 0, 1]],
            ),
            quad(
                [
                    [-10.0, 10.0, 9.0],
                    [w + 10.0, 10.0, 9.0],
                    [-10.0, 0.0, -1.0],
                    [w + 10.0, 0.0, -1.0],
                ],
                [[0.0, 0.0], [20.0 + w, 0.0], [0.0, 14.46], [20.0 + w, 14.46]],
                [[0, 1, 3], [0, 3, 2]],
            ),
        ];
        let right = vec![
            quad(
                [
                    [w + 9.0, 10.0, 10.0],
                    [w + 9.0, 10.0, -h - 10.0],
                    [w - 1.0, 0.0, 10.0],
                    [w - 1.0, 0.0, -h - 10.0],
                ],
                [[0.0, 0.0], [20.0 + h, 0.0], [0.0, 14.46], [20.0 + h, 14.46]],
                [[0, 1, 3], [0, 3, 2]],
            ),
            quad(
                [
                    [w + 10.0, 10.0, -h - 9.0],
                    [-10.0, 10.0, -h - 9.0],
                    [w + 10.0, 0.0, -h + 1.0],
                    [-10.0, 0.0, -h + 1.0],
                ],
                [[0.0, 0.0], [20.0 + w, 0.0], [0.0, 14.46], [20.0 + w, 14.46]],
                [[0, 1, 3], [0, 3, 2]],
            ),
        ];
        bg.surfaces.push(surface(left_tex, 1.0, left));
        bg.surfaces.push(surface(right_tex, 1.0, right));
    }
    if background == 4 {
        // Flat edge walls at wall height.
        let y = 0.9;
        let t = [[0, 1, 2], [1, 3, 2]];
        let quads = vec![
            quad(
                [
                    [-10.0, y, 10.0],
                    [w + 10.0, y, 10.0],
                    [-10.0, y, 0.0],
                    [w + 10.0, y, 0.0],
                ],
                [[0.0, 0.0], [w + 20.0, 0.0], [0.0, 10.0], [w + 20.0, 10.0]],
                t,
            ),
            quad(
                [[-10.0, y, 0.0], [0.0, y, 0.0], [-10.0, y, -h], [0.0, y, -h]],
                [[0.0, 0.0], [10.0, 0.0], [0.0, h], [10.0, h]],
                t,
            ),
            quad(
                [
                    [w, y, 0.0],
                    [w + 10.0, y, 0.0],
                    [w, y, -h],
                    [w + 10.0, y, -h],
                ],
                [[0.0, 0.0], [10.0, 0.0], [0.0, h], [10.0, h]],
                t,
            ),
            quad(
                [
                    [-10.0, y, -h],
                    [w + 10.0, y, -h],
                    [-10.0, y, -h - 10.0],
                    [w + 10.0, y, -h - 10.0],
                ],
                [[0.0, 0.0], [20.0 + w, 0.0], [0.0, 10.0], [20.0 + w, 10.0]],
                t,
            ),
        ];
        bg.surfaces.push(surface(Tex::WallTop, 1.0, quads));
    }
    bg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(width: usize, tiles: &[i32]) -> Grid {
        let height = tiles.len() / width;
        let mut g = Grid::new(16, width, height);
        for (k, &t) in tiles.iter().enumerate() {
            g.set((k % width) as i32, (k / width) as i32, t);
        }
        g
    }

    fn find(s: &[LevelSurface], tex: Tex) -> Vec<&LevelSurface> {
        s.iter().filter(|x| x.texture == tex).collect()
    }

    #[test]
    fn floors_merge_into_rectangles() {
        // 3x2 floor with a wall in the bottom-right corner.
        let s = construct_level(&grid(3, &[100, 100, 100, 100, 100, 200]), 0);
        let floor = &find(&s, Tex::Floor(0))[0].quads;
        // The first row cannot extend down (the wall), so it is 3x1, then
        // the remaining two tiles are 2x1.
        assert_eq!(floor.len(), 2);
        assert_eq!(floor[0].uvs[3], [3.0, 1.0]);
        assert_eq!(floor[0].positions[0], [-0.001, 0.0, 0.001]);
        assert_eq!(floor[0].positions[3], [3.001, 0.0, -1.001]);
        assert_eq!(floor[1].uvs[3], [2.0, 1.0]);
        assert_eq!(floor[0].normal(), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn object_tiles_draw_as_floor() {
        let s = construct_level(&grid(4, &[100, 1000, 1101, 2200]), 0);
        assert_eq!(find(&s, Tex::Floor(0))[0].quads.len(), 1);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn walls_face_outwards() {
        let s = construct_level(&grid(3, &[100, 200, 100]), 0);
        let top = &find(&s, Tex::WallTop)[0].quads;
        assert_eq!(top[0].positions[0][1], 0.9);
        let light = &find(&s, Tex::WallSide(0))[0].quads;
        let dark = &find(&s, Tex::WallSide(1))[0].quads;
        // Left face at x = 1 facing -x; back face facing +z (up the map).
        let normals: Vec<_> = light.iter().map(Quad::normal).collect();
        assert!(normals.contains(&[-1.0, 0.0, 0.0]));
        assert!(normals.contains(&[0.0, 0.0, 1.0]));
        let normals: Vec<_> = dark.iter().map(Quad::normal).collect();
        assert!(normals.contains(&[1.0, 0.0, 0.0]));
        assert!(normals.contains(&[0.0, 0.0, -1.0]));
    }

    #[test]
    fn water_sides_face_into_the_pool() {
        let s = construct_level(&grid(3, &[100, 300, 100]), 0);
        let water = find(&s, Tex::Water);
        assert_eq!(water[0].alpha, 0.8);
        assert_eq!(water[0].quads[0].positions[0][1], -0.2);
        let sides = find(&s, Tex::WallSide(2));
        // Bed, then the four sides.
        assert_eq!(sides[0].quads[0].positions[0][1], -2.0);
        let normals: Vec<_> = sides[1].quads.iter().map(Quad::normal).collect();
        assert_eq!(normals.len(), 4);
        assert!(normals.contains(&[1.0, 0.0, 0.0]), "{normals:?}");
        // The water backdrop replaces the bed and the top.
        let s = construct_level(&grid(3, &[100, 300, 100]), 5);
        assert!(find(&s, Tex::Water).is_empty());
        assert_eq!(find(&s, Tex::WallSide(2)).len(), 1);
    }

    #[test]
    fn breakaways_over_water_and_lava() {
        let s = construct_level(&grid(4, &[601, 1700, 604, 1702]), 0);
        assert_eq!(find(&s, Tex::Water)[0].quads.len(), 1);
        assert_eq!(find(&s, Tex::Lava)[0].quads.len(), 1);
    }

    #[test]
    fn wall_runs_follow_the_column() {
        // A 1x3 wall column between floors: one left face 3 tiles long.
        let s = construct_level(&grid(3, &[100, 200, 100, 100, 200, 100, 100, 200, 100]), 0);
        let light = &find(&s, Tex::WallSide(0))[0].quads;
        let left: Vec<_> = light
            .iter()
            .filter(|q| q.normal() == [-1.0, 0.0, 0.0])
            .collect();
        // The run stops short of the last row (`j2 < MapHeight - 1`), so
        // the last tile gets its own face.
        assert_eq!(left.len(), 2);
        assert_eq!(left[0].uvs[2][0] + left[1].uvs[2][0], 3.0);
    }

    #[test]
    fn deep_holes_skip_the_level_edge() {
        let s = construct_level(&grid(2, &[201, 100]), 0);
        let light = &find(&s, Tex::WallSide(3))[0].quads;
        // No left face at x = 0 and no back face at row 0.
        assert!(light.is_empty());
        // A right face only: the front scan skips the last (only) row.
        let dark = &find(&s, Tex::WallSide(2))[0].quads;
        assert_eq!(dark.len(), 1);
        assert_eq!(dark[0].uvs[1][1], -50.0);
    }

    #[test]
    fn backgrounds() {
        let b = construct_background(0, 2, 10, 10);
        assert_eq!(b.planes.len(), 2);
        assert_eq!(b.planes[0].alpha, 0.5);
        assert_eq!(
            construct_background(4, 2, 10, 10).surfaces[0].quads.len(),
            4
        );
        assert_eq!(
            construct_background(2, 4, 10, 10).surfaces[0].texture,
            Tex::WallTop
        );
        assert!(construct_background(7, 2, 10, 10).warp_sphere.is_some());
    }
}
