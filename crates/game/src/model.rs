//! The object models `Setup` loads (`RTWP/setup.bb`), as Blitz holds them
//! once the `RotateMesh`/`PositionMesh`/`ScaleMesh`/`FitMesh` calls are
//! baked in, plus the template entity's own scale and rotation, which
//! `CopyEntity` passes on to every copy. Geometry stays in Blitz
//! coordinates; [`crate::objects`] converts it when it builds Bevy meshes.
//!
//! The textures named here replace any the mesh file names, as
//! `EntityTexture` does.

use bevy::math::{Mat3, Quat, Vec3};
use wonderland_formats::GameAssets;
use wonderland_formats::mesh::{self, md2};

use crate::style::TextureRef;

/// A Blitz rotation `(pitch, yaw, roll)` in degrees as a matrix in Blitz
/// space: yaw × pitch × roll, from `geom.h` in the Blitz3D source.
pub fn blitz_matrix(r: [f32; 3]) -> Mat3 {
    let (sp, cp) = r[0].to_radians().sin_cos();
    let (sy, cy) = r[1].to_radians().sin_cos();
    let (sr, cr) = r[2].to_radians().sin_cos();
    let yaw = Mat3::from_cols(Vec3::new(cy, 0.0, sy), Vec3::Y, Vec3::new(-sy, 0.0, cy));
    let pitch = Mat3::from_cols(Vec3::X, Vec3::new(0.0, cp, sp), Vec3::new(0.0, -sp, cp));
    let roll = Mat3::from_cols(Vec3::new(cr, sr, 0.0), Vec3::new(-sr, cr, 0.0), Vec3::Z);
    yaw * pitch * roll
}

/// A Blitz rotation as a Bevy rotation (Bevy space is Blitz space with z
/// negated).
pub fn blitz_quat(r: [f32; 3]) -> Quat {
    let flip = Mat3::from_diagonal(Vec3::new(1.0, 1.0, -1.0));
    Quat::from_mat3(&(flip * blitz_matrix(r) * flip))
}

/// What a model is made of.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// `LoadMesh` of a `.3ds` or `.x` file.
    File(&'static str),
    /// `LoadMD2`.
    Md2(&'static str),
    /// `CreateSphere(8)`: radius 1.
    Sphere,
    /// `CreateCube()`: -1 to 1.
    Cube,
    /// `CreateCylinder(6, solid)`: radius 1, y from -1 to 1.
    Cylinder { solid: bool },
    /// `CreateFlatSquare`: (0, 0, 0) to (1, 0, -1), facing up.
    FlatSquare,
}

/// A mesh operation baked into the vertices, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bake {
    Rotate([f32; 3]),
    Position([f32; 3]),
    Scale([f32; 3]),
    /// `FitMesh x, y, z, w, h, d, False`.
    Fit([f32; 3], [f32; 3]),
}

/// One model as `Setup` prepares it.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDef {
    pub shape: Shape,
    pub texture: Option<(&'static str, u32)>,
    pub bake: Vec<Bake>,
    /// `ScaleEntity` on the template.
    pub scale: [f32; 3],
    /// `RotateEntity` then `TurnEntity` calls on the template, in order.
    pub turns: Vec<[f32; 3]>,
    /// `EntityColor`, 0–1.
    pub color: [f32; 3],
    /// `EntityAlpha`.
    pub alpha: f32,
}

impl ModelDef {
    fn new(shape: Shape, texture: Option<(&'static str, u32)>) -> Self {
        Self {
            shape,
            texture,
            bake: Vec::new(),
            scale: [1.0; 3],
            turns: Vec::new(),
            color: [1.0; 3],
            alpha: 1.0,
        }
    }

    fn bake(mut self, b: Bake) -> Self {
        self.bake.push(b);
        self
    }

    fn scale(mut self, s: [f32; 3]) -> Self {
        self.scale = s;
        self
    }

    fn turn(mut self, r: [f32; 3]) -> Self {
        self.turns.push(r);
        self
    }

    fn color(mut self, c: [u8; 3]) -> Self {
        self.color = c.map(|c| f32::from(c) / 255.0);
        self
    }

    fn alpha(mut self, a: f32) -> Self {
        self.alpha = a;
        self
    }

    /// The template's rotation as a Bevy rotation.
    pub fn rotation(&self) -> Quat {
        self.turns
            .iter()
            .fold(Quat::IDENTITY, |q, &r| q * blitz_quat(r))
    }
}

/// Every model the objects use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModelId {
    /// `PlayerMesh(1–4)`; the skin is chosen per draw.
    Player(usize),
    WoodBox,
    SteelBox,
    Reflector,
    Boulder,
    GhostBox,
    BoomBox,
    Prism,
    StickyCube,
    /// Link spheres: red, yellow, green, blue (box types 9–12).
    LinkSphere(usize),
    PushCannon,
    Boxer,
    Transporter,
    Cannon,
    Fireball,
    IronGate,
    /// The three laser beams of a coloured gate, and their emitters.
    LaserBeams,
    LaserEmitters,
    Coily,
    ZBot,
    Bomb,
    Ufo,
    Chomper,
    Ghost,
    Warper,
    Coin,
    CoinShadow,
    Exit,
    Clock,
    BonusCoin,
    Spike,
    Fish,
    /// `PillarMesh(n)`.
    Pillar(usize),
    /// A round shadow (`CreateFlatSquare(RoundShadowTexture)`).
    Shadow,
    /// A flat square for overlays (breakaway tiles, switchers, buttons,
    /// electro tiles), textured per draw.
    Square,
}

/// The skin textures of player `k` (1–4): normal, dying, C and D.
pub fn player_skins(k: usize) -> [&'static str; 4] {
    match k {
        1 => [
            "graphics\\stinky2.bmp",
            "graphics\\stinky2b.bmp",
            "graphics\\stinky2c.bmp",
            "graphics\\stinky2d.bmp",
        ],
        2 => [
            "graphics\\loof.bmp",
            "graphics\\loofb.bmp",
            "graphics\\loofc.bmp",
            "graphics\\loofd.bmp",
        ],
        3 => [
            "graphics\\qookie2a.bmp",
            "graphics\\qookie2b.bmp",
            "graphics\\qookie2c.bmp",
            "graphics\\qookie2d.bmp",
        ],
        _ => [
            "graphics\\peegue2a.bmp",
            "graphics\\peegue2b.bmp",
            "graphics\\peegue2c.bmp",
            "graphics\\peegue2d.bmp",
        ],
    }
}

/// `ModelDef` for `id`, following `Setup`.
pub fn model_def(id: ModelId) -> ModelDef {
    use Bake::*;
    use Shape::*;
    let t = |path: &'static str, flags: u32| Some((path, flags));
    let box3ds = File("graphics\\box.3ds");
    match id {
        ModelId::Player(k) => {
            let md2 = if k == 4 {
                "graphics\\peegueb.md2"
            } else {
                "graphics\\stinky.md2"
            };
            let d = ModelDef::new(Md2(md2), t(player_skins(k)[0], 1));
            match k {
                2 => d.scale([0.9, 0.7, 0.95]),
                3 => d.scale([1.1, 0.82, 1.1]),
                4 => d.scale([1.05, 0.9, 1.05]),
                _ => d,
            }
        }
        // `ScaleMesh WoodBoxMesh, 1.1, 1.1, 1.1` sits in the boom box's
        // lines, but scales the wooden box.
        ModelId::WoodBox => {
            ModelDef::new(box3ds, t("graphics\\woodbox.bmp", 1)).bake(Scale([1.1; 3]))
        }
        ModelId::SteelBox => ModelDef::new(box3ds, t("graphics\\steelbox.bmp", 1)),
        ModelId::Reflector => ModelDef::new(
            File("graphics\\reflector.3ds"),
            t("graphics\\reflector.bmp", 2),
        )
        .bake(Rotate([-90.0, 0.0, 0.0])),
        ModelId::Boulder => ModelDef::new(Sphere, t("res23.dat", 1)).scale([0.45; 3]),
        ModelId::GhostBox => ModelDef::new(box3ds, t("graphics\\plasmabox.jpg", 2)),
        ModelId::BoomBox => {
            ModelDef::new(File("graphics\\barrel.3ds"), t("graphics\\barrel.jpg", 1))
                .bake(Rotate([-90.0, 0.0, 0.0]))
        }
        ModelId::Prism => ModelDef::new(File("graphics\\prism.3ds"), t("graphics\\prism.jpg", 1))
            .bake(Rotate([-90.0, 0.0, 0.0]))
            .scale([1.1; 3]),
        ModelId::StickyCube => ModelDef::new(Cube, t("res54.dat", 1))
            .scale([0.45; 3])
            .alpha(0.5),
        ModelId::LinkSphere(c) => {
            let tex = ["res58.dat", "res57.dat", "res56.dat", "res55.dat"][c.min(3)];
            ModelDef::new(Sphere, t(tex, 1)).scale([0.45; 3])
        }
        ModelId::PushCannon => {
            ModelDef::new(File("graphics\\scouge.3ds"), t("res53.dat", 1)).turn([-90.0, -90.0, 0.0])
        }
        ModelId::Boxer => ModelDef::new(box3ds, t("graphics\\boxer.bmp", 1))
            .bake(Position([0.0, 0.4, 0.0]))
            .scale([1.3; 3]),
        ModelId::Transporter => ModelDef::new(box3ds, t("graphics\\steelbox.bmp", 1))
            .bake(Position([0.0, -1.0, 0.0]))
            .bake(Scale([1.0, 0.1, 1.0])),
        ModelId::Cannon => {
            ModelDef::new(File("graphics\\scouge.3ds"), t("graphics\\scouge3.bmp", 1))
                .turn([-90.0, -90.0, 0.0])
        }
        ModelId::Fireball => ModelDef::new(Sphere, None)
            .color([255, 240, 200])
            .alpha(0.5)
            .scale([0.2; 3]),
        ModelId::IronGate => ModelDef::new(File("graphics\\gate.3ds"), None)
            .bake(Position([0.0, 0.1, 0.0]))
            .turn([-90.0, 0.0, 0.0])
            .scale([1.0, 1.0, 1.3])
            .color([128, 128, 128]),
        ModelId::LaserBeams => ModelDef::new(Cylinder { solid: false }, None).alpha(0.5),
        ModelId::LaserEmitters => {
            ModelDef::new(Cylinder { solid: true }, t("graphics\\steelbox.bmp", 1))
        }
        ModelId::Coily => ModelDef::new(Md2("graphics\\coily.md2"), t("graphics\\coily.bmp", 1))
            .scale([0.3, 0.2, 0.3]),
        ModelId::ZBot => {
            ModelDef::new(File("graphics\\zbot.3ds"), t("graphics\\zbot.bmp", 1)).scale([0.6; 3])
        }
        ModelId::Bomb => ModelDef::new(Md2("graphics\\bomb.md2"), t("graphics\\bomb.bmp", 1))
            .turn([0.0, 180.0, 0.0]),
        ModelId::Ufo => ModelDef::new(File("graphics\\ufo.3ds"), t("graphics\\ufo.bmp", 1)),
        ModelId::Chomper => ModelDef::new(
            Md2("graphics\\chomper.md2"),
            t("graphics\\chomper2a.bmp", 1),
        ),
        ModelId::Ghost => ModelDef::new(File("graphics\\ghost.3ds"), t("graphics\\ghost2.bmp", 2))
            .scale([1.0, 1.0, 0.8])
            .bake(Rotate([0.0, 0.0, -90.0])),
        ModelId::Warper => ModelDef::new(Sphere, t("graphics\\lava.jpg", 1))
            .bake(Scale([0.5; 3]))
            .alpha(0.7),
        ModelId::Coin => ModelDef::new(File("graphics\\coin.3ds"), t("graphics\\coin2.bmp", 2))
            .scale([0.8, 0.8, 0.5]),
        ModelId::CoinShadow => ModelDef::new(FlatSquare, t("graphics\\coinshadow.bmp", 2))
            .bake(Position([-0.5, 0.0, 0.5]))
            .scale([0.6, 0.6, 0.1])
            .alpha(0.5),
        ModelId::Exit => ModelDef::new(Sphere, t("graphics\\rainbow.jpg", 2)).scale([0.5; 3]),
        ModelId::Clock => ModelDef::new(File("graphics\\clock.3ds"), t("graphics\\clock.bmp", 1))
            .bake(Rotate([-90.0, 0.0, 0.0])),
        ModelId::BonusCoin => ModelDef::new(File("graphics\\coin.3ds"), t("graphics\\coin.jpg", 1))
            .scale([0.8, 0.8, 0.5]),
        ModelId::Spike => ModelDef::new(File("graphics\\spikes.3ds"), t("graphics\\spikes.jpg", 1))
            .bake(Rotate([-90.0, 0.0, 0.0])),
        ModelId::Fish => ModelDef::new(File("graphics\\fish.x"), t("graphics\\fish.bmp", 1))
            .bake(Fit([-0.5; 3], [1.0; 3])),
        ModelId::Pillar(n) => pillar_def(n),
        ModelId::Shadow => ModelDef::new(FlatSquare, t("graphics\\beachballshadow.bmp", 2)),
        ModelId::Square => ModelDef::new(FlatSquare, None),
    }
}

/// `PillarMesh(n)`. Kinds without a mesh of their own (9–11 use the
/// house; 23, 24 and 30 the player) are mapped by the caller.
fn pillar_def(n: usize) -> ModelDef {
    use Bake::*;
    use Shape::*;
    let t = |path: &'static str| Some((path, 1));
    let up = Rotate([-90.0, 0.0, 0.0]);
    match n {
        0 => ModelDef::new(File("graphics\\pillar.3ds"), t("graphics\\pillar1.jpg"))
            .turn([-90.0, 0.0, 0.0]),
        1 => ModelDef::new(File("graphics\\pyramid.3ds"), t("graphics\\pillar2.jpg"))
            .turn([-90.0, 0.0, 0.0]),
        2 => ModelDef::new(File("graphics\\spike.3ds"), t("graphics\\pillar2.jpg"))
            .turn([-90.0, 0.0, 0.0]),
        3 => ModelDef::new(File("graphics\\fountain.3ds"), t("graphics\\fountain.jpg"))
            .bake(up)
            .bake(Position([0.0, 0.5, 0.0]))
            .bake(Scale([1.0, 0.8, 1.0])),
        4 => ModelDef::new(File("graphics\\pyramid2.3ds"), t("graphics\\pyramid.jpg"))
            .bake(up)
            .bake(Position([0.0, 0.5, 0.0]))
            .bake(Scale([3.0; 3])),
        5 => ModelDef::new(File("graphics\\pyramid2.3ds"), t("graphics\\pyramid.jpg"))
            .bake(up)
            .bake(Position([0.0, 0.5, 0.0])),
        6 => ModelDef::new(File("graphics\\box.3ds"), t("graphics\\woodbox.bmp"))
            .bake(Position([0.0, 0.5, 0.0]))
            .bake(Scale([3.0; 3])),
        7 => ModelDef::new(File("graphics\\lamp.3ds"), t("graphics\\lamp.jpg"))
            .bake(up)
            .bake(Position([0.0, 0.5, 0.0]))
            .bake(Scale([1.5; 3])),
        // The house; its texture follows `HouseStyle`.
        8 => ModelDef::new(File("graphics\\house.3ds"), t("graphics\\house1.jpg"))
            .bake(up)
            .bake(Position([0.0, 1.0, 0.0]))
            .bake(Scale([1.5; 3])),
        16 => ModelDef::new(File("graphics\\tree.3ds"), t("graphics\\tree.jpg"))
            .turn([-90.0, 0.0, 0.0])
            .turn([0.0, 0.0, 90.0])
            .bake(Scale([1.0, 1.0, 1.5])),
        17 => ModelDef::new(File("graphics\\tree.3ds"), t("graphics\\tree.jpg"))
            .turn([-90.0, 0.0, 0.0])
            .turn([0.0, 0.0, 90.0])
            .bake(Scale([0.4, 0.4, 1.6])),
        20 => ModelDef::new(File("graphics\\tree.3ds"), t("graphics\\shroom.jpg"))
            .turn([-90.0, 0.0, 0.0])
            .bake(Scale([0.3; 3])),
        21 => ModelDef::new(File("graphics\\tree.3ds"), t("graphics\\shroom.jpg"))
            .turn([-90.0, 0.0, 0.0]),
        18 => ModelDef::new(File("graphics\\pine.3ds"), t("graphics\\pine1.jpg")),
        19 => ModelDef::new(File("graphics\\pine.3ds"), t("graphics\\pine2.jpg")),
        22 => ModelDef::new(File("graphics\\flower.3ds"), t("graphics\\flower.jpg"))
            .bake(up)
            .bake(Scale([0.02; 3])),
        25 => ModelDef::new(File("graphics\\snowman.3ds"), t("graphics\\snowman.jpg")),
        // Kinds the original never loads a mesh for show nothing.
        _ => ModelDef::new(FlatSquare, None).alpha(0.0),
    }
}

/// One surface's vertices in Blitz space.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Blitz winding.
    pub triangles: Vec<[u32; 3]>,
}

/// A loaded model: its surfaces (for an MD2, one per frame of the single
/// surface) with the bakes applied.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub def: ModelDef,
    /// Static surfaces, or an MD2's frames.
    pub surfaces: Vec<Geometry>,
    pub md2: bool,
}

impl Model {
    /// The geometry at MD2 frame `time`, interpolated like
    /// `MD2Model::render`; static models return their first surface.
    pub fn frame(&self, time: f32) -> Geometry {
        if !self.md2 || self.surfaces.len() < 2 {
            return self.surfaces.first().cloned().unwrap_or_default();
        }
        let n = self.surfaces.len();
        let t = time.max(0.0);
        let a = (t.floor() as usize).min(n - 1);
        let b = (a + 1).min(n - 1);
        let f = t - t.floor();
        let (ga, gb) = (&self.surfaces[a], &self.surfaces[b]);
        let lerp = |x: &[[f32; 3]], y: &[[f32; 3]]| -> Vec<[f32; 3]> {
            x.iter()
                .zip(y)
                .map(|(p, q)| std::array::from_fn(|i| p[i] + (q[i] - p[i]) * f))
                .collect()
        };
        Geometry {
            positions: lerp(&ga.positions, &gb.positions),
            normals: lerp(&ga.normals, &gb.normals),
            uvs: ga.uvs.clone(),
            triangles: ga.triangles.clone(),
        }
    }

    /// The texture the template entity is given.
    pub fn texture(&self) -> Option<TextureRef> {
        self.def.texture.map(|(path, flags)| TextureRef {
            path: path.to_owned(),
            flags,
            rotation: 0,
        })
    }
}

fn from_mesh(m: &mesh::Mesh) -> Vec<Geometry> {
    m.surfaces
        .iter()
        .map(|s| Geometry {
            positions: s.vertices.iter().map(|v| v.position).collect(),
            normals: s.vertices.iter().map(|v| v.normal).collect(),
            uvs: s.vertices.iter().map(|v| v.uv).collect(),
            triangles: s.triangles.clone(),
        })
        .collect()
}

/// `CreateSphere(8)`: 16 segments around, 8 down, u around, v from the
/// top. Normals point out.
pub fn sphere() -> Geometry {
    let (around, down) = (16u32, 8u32);
    let mut g = Geometry::default();
    for j in 0..=down {
        let theta = std::f32::consts::PI * j as f32 / down as f32;
        for i in 0..=around {
            let phi = std::f32::consts::TAU * i as f32 / around as f32;
            let p = [
                theta.sin() * phi.sin(),
                theta.cos(),
                theta.sin() * phi.cos(),
            ];
            g.positions.push(p);
            g.normals.push(p);
            g.uvs
                .push([i as f32 / around as f32, j as f32 / down as f32]);
        }
    }
    let row = around + 1;
    for j in 0..down {
        for i in 0..around {
            let a = j * row + i;
            g.triangles.push([a, a + row + 1, a + 1]);
            g.triangles.push([a, a + row, a + row + 1]);
        }
    }
    g
}

/// `CreateCube()`: six faces, each with the whole texture.
pub fn cube() -> Geometry {
    let mut g = Geometry::default();
    // Each face: normal, then the right and up directions seen from
    // outside.
    let faces = [
        ([0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        ([0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
    ];
    for (n, r, u) in faces {
        let base = g.positions.len() as u32;
        for (su, sv) in [(-1.0, 1.0), (1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
            let p: [f32; 3] = std::array::from_fn(|i| n[i] + r[i] * su + u[i] * sv);
            g.positions.push(p);
            g.normals.push(n);
            g.uvs.push([(su + 1.0) / 2.0, (1.0 - sv) / 2.0]);
        }
        g.triangles.push([base, base + 1, base + 2]);
        g.triangles.push([base, base + 2, base + 3]);
    }
    g
}

/// `CreateCylinder(6, solid)`.
pub fn cylinder(solid: bool) -> Geometry {
    let segs = 6u32;
    let mut g = Geometry::default();
    for i in 0..=segs {
        let phi = std::f32::consts::TAU * i as f32 / segs as f32;
        let (s, c) = phi.sin_cos();
        for y in [1.0, -1.0] {
            g.positions.push([s, y, c]);
            g.normals.push([s, 0.0, c]);
            g.uvs
                .push([i as f32 / segs as f32, if y > 0.0 { 0.0 } else { 1.0 }]);
        }
    }
    for i in 0..segs {
        let a = i * 2;
        g.triangles.push([a, a + 3, a + 2]);
        g.triangles.push([a, a + 1, a + 3]);
    }
    if solid {
        for (y, flip) in [(1.0f32, false), (-1.0, true)] {
            let centre = g.positions.len() as u32;
            g.positions.push([0.0, y, 0.0]);
            g.normals.push([0.0, y, 0.0]);
            g.uvs.push([0.5, 0.5]);
            for i in 0..=segs {
                let phi = std::f32::consts::TAU * i as f32 / segs as f32;
                let (s, c) = phi.sin_cos();
                g.positions.push([s, y, c]);
                g.normals.push([0.0, y, 0.0]);
                g.uvs.push([0.5 + s / 2.0, 0.5 - c / 2.0]);
            }
            for i in 0..segs {
                let (a, b) = (centre + 1 + i, centre + 2 + i);
                g.triangles
                    .push(if flip { [centre, b, a] } else { [centre, a, b] });
            }
        }
    }
    g
}

/// `CreateFlatSquare`.
pub fn flat_square() -> Geometry {
    Geometry {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
        normals: vec![[0.0, 1.0, 0.0]; 4],
        uvs: vec![[0.01, -0.99], [0.99, -0.99], [0.99, 0.01], [0.01, 0.01]],
        triangles: vec![[0, 1, 2], [0, 2, 3]],
    }
}

/// The three beams of a coloured laser gate (`CreateLaserGate`).
fn laser_beams() -> Geometry {
    let mut out = Geometry::default();
    for offset in [
        [0.0, 0.25, 0.0],
        [0.0, -0.125, 0.2165],
        [0.0, -0.125, -0.2165],
    ] {
        let mut c = cylinder(false);
        apply(&mut c, Bake::Scale([0.05, 0.5, 0.05]));
        apply(&mut c, Bake::Rotate([0.0, 0.0, 90.0]));
        apply(&mut c, Bake::Position(offset));
        append(&mut out, &c);
    }
    out
}

/// The six emitters at the ends of the beams.
fn laser_emitters() -> Geometry {
    let mut out = Geometry::default();
    for z in [0.0, 0.2165, -0.2165] {
        let y = if z == 0.0 { 0.25 } else { -0.125 };
        for x in [-0.47, 0.47] {
            let mut c = cylinder(true);
            apply(&mut c, Bake::Scale([0.08, 0.03, 0.08]));
            apply(&mut c, Bake::Rotate([0.0, 0.0, 90.0]));
            apply(&mut c, Bake::Position([x, y, z]));
            append(&mut out, &c);
        }
    }
    out
}

fn append(out: &mut Geometry, g: &Geometry) {
    let base = out.positions.len() as u32;
    out.positions.extend_from_slice(&g.positions);
    out.normals.extend_from_slice(&g.normals);
    out.uvs.extend_from_slice(&g.uvs);
    out.triangles
        .extend(g.triangles.iter().map(|t| t.map(|i| i + base)));
}

fn bounds(surfaces: &[Geometry]) -> Option<(Vec3, Vec3)> {
    let mut it = surfaces.iter().flat_map(|g| g.positions.iter());
    let first = Vec3::from_array(*it.next()?);
    Some(it.fold((first, first), |(lo, hi), p| {
        let p = Vec3::from_array(*p);
        (lo.min(p), hi.max(p))
    }))
}

/// Applies one bake to a surface. `Fit` needs the whole model's bounds,
/// so [`load`] handles it.
fn apply(g: &mut Geometry, b: Bake) {
    match b {
        Bake::Rotate(r) => {
            let m = blitz_matrix(r);
            for p in &mut g.positions {
                *p = (m * Vec3::from_array(*p)).to_array();
            }
            for n in &mut g.normals {
                *n = (m * Vec3::from_array(*n)).to_array();
            }
        }
        Bake::Position(o) => {
            for p in &mut g.positions {
                *p = std::array::from_fn(|i| p[i] + o[i]);
            }
        }
        Bake::Scale(s) => {
            for p in &mut g.positions {
                *p = std::array::from_fn(|i| p[i] * s[i]);
            }
            for n in &mut g.normals {
                let v = Vec3::new(n[0] / s[0], n[1] / s[1], n[2] / s[2]).normalize_or_zero();
                *n = v.to_array();
            }
        }
        Bake::Fit(..) => {}
    }
}

/// Loads model `id` from the game data.
pub fn load(id: ModelId, assets: &GameAssets) -> Result<Model, String> {
    let def = model_def(id);
    let (mut surfaces, md2) = match def.shape {
        Shape::File(path) => {
            let data = assets.read(path).map_err(|e| format!("{path}: {e}"))?;
            let m = if path.to_lowercase().ends_with(".x") {
                mesh::x::parse(&data)
            } else {
                mesh::tds::parse(&data)
            }
            .map_err(|e| format!("{path}: {e}"))?;
            (from_mesh(&m), false)
        }
        Shape::Md2(path) => {
            let data = assets.read(path).map_err(|e| format!("{path}: {e}"))?;
            let m = md2::parse(&data).map_err(|e| format!("{path}: {e}"))?;
            let frames = (0..m.frames.len())
                .flat_map(|f| from_mesh(&m.frame_mesh(f)))
                .collect();
            (frames, true)
        }
        Shape::Sphere => (vec![sphere()], false),
        Shape::Cube => (vec![cube()], false),
        Shape::Cylinder { .. } if id == ModelId::LaserBeams => (vec![laser_beams()], false),
        Shape::Cylinder { .. } if id == ModelId::LaserEmitters => (vec![laser_emitters()], false),
        Shape::Cylinder { solid } => (vec![cylinder(solid)], false),
        Shape::FlatSquare => (vec![flat_square()], false),
    };
    for &b in &def.bake {
        if let Bake::Fit(at, size) = b {
            // Every frame of an MD2 would fit separately; none is fitted.
            let Some((lo, hi)) = bounds(&surfaces) else {
                continue;
            };
            let extent = (hi - lo).max(Vec3::splat(1e-6));
            let scale = Vec3::from_array(size) / extent;
            let to = Vec3::from_array(at);
            for g in &mut surfaces {
                apply(&mut g.clone(), b);
                for p in &mut g.positions {
                    let v = (Vec3::from_array(*p) - lo) * scale + to;
                    *p = v.to_array();
                }
                for n in &mut g.normals {
                    *n = (Vec3::from_array(*n) / scale)
                        .normalize_or_zero()
                        .to_array();
                }
            }
        } else {
            for g in &mut surfaces {
                apply(g, b);
            }
        }
    }
    Ok(Model { def, surfaces, md2 })
}

/// Every model, for loading up front.
pub fn all_models() -> Vec<ModelId> {
    let mut v = vec![
        ModelId::WoodBox,
        ModelId::SteelBox,
        ModelId::Reflector,
        ModelId::Boulder,
        ModelId::GhostBox,
        ModelId::BoomBox,
        ModelId::Prism,
        ModelId::StickyCube,
        ModelId::PushCannon,
        ModelId::Boxer,
        ModelId::Transporter,
        ModelId::Cannon,
        ModelId::Fireball,
        ModelId::IronGate,
        ModelId::LaserBeams,
        ModelId::LaserEmitters,
        ModelId::Coily,
        ModelId::ZBot,
        ModelId::Bomb,
        ModelId::Ufo,
        ModelId::Chomper,
        ModelId::Ghost,
        ModelId::Warper,
        ModelId::Coin,
        ModelId::CoinShadow,
        ModelId::Exit,
        ModelId::Clock,
        ModelId::BonusCoin,
        ModelId::Spike,
        ModelId::Fish,
        ModelId::Shadow,
        ModelId::Square,
    ];
    v.extend((1..=4).map(ModelId::Player));
    v.extend((0..4).map(ModelId::LinkSphere));
    v.extend(
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 16, 17, 18, 19, 20, 21, 22, 25]
            .into_iter()
            .map(ModelId::Pillar),
    );
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn blitz_rotations_match_the_camera_conventions() {
        // Positive yaw turns the forward (+z) vector left (-x).
        let m = blitz_matrix([0.0, 90.0, 0.0]);
        assert!(close(m * Vec3::Z, Vec3::NEG_X));
        // Positive pitch looks down.
        let m = blitz_matrix([90.0, 0.0, 0.0]);
        assert!(close(m * Vec3::Z, Vec3::NEG_Y));
        // The same rotations in Bevy space, against `coords::rotation`.
        let q = blitz_quat([30.0, 40.0, 0.0]);
        let c = crate::coords::rotation(30.0, 40.0);
        assert!(close(q * Vec3::NEG_Z, c * Vec3::NEG_Z));
        assert!(close(q * Vec3::Y, c * Vec3::Y));
    }

    #[test]
    fn primitives_wind_like_blitz_meshes() {
        // In the game's own meshes, (b - a) x (c - a) points along the
        // vertex normals (checked on box.3ds, coin.3ds and stinky.md2).
        for (name, g) in [
            ("sphere", sphere()),
            ("cube", cube()),
            ("cylinder", cylinder(true)),
            ("square", flat_square()),
            ("beams", laser_beams()),
        ] {
            for t in &g.triangles {
                let [a, b, c] = t.map(|i| Vec3::from_array(g.positions[i as usize]));
                let n: Vec3 = t
                    .iter()
                    .map(|&i| Vec3::from_array(g.normals[i as usize]))
                    .sum();
                let area = (b - a).cross(c - a);
                if area.length() > 1e-6 {
                    assert!(area.dot(n) > 0.0, "{name}: {t:?}");
                }
            }
        }
    }

    #[test]
    fn every_model_loads_from_the_real_data() {
        let dir = crate::level::default_assets_dir();
        let Ok(assets) = GameAssets::open(&dir) else {
            eprintln!("skipping: no game data at {}", dir.display());
            return;
        };
        for id in all_models() {
            let m = load(id, &assets).unwrap_or_else(|e| panic!("{id:?}: {e}"));
            assert!(!m.surfaces.is_empty(), "{id:?}");
            if let Some(t) = m.texture() {
                assert!(assets.exists(&t.path), "{id:?}: {}", t.path);
            }
        }
        let p = load(ModelId::Player(1), &assets).unwrap();
        assert_eq!(p.surfaces.len(), 147);
    }
}
