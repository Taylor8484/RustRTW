//! `.3ds` meshes (3D Studio). Named `tds` because a module name cannot start
//! with a digit.
//!
//! Ported from Blitz3D's own loader, as `LoadMesh` uses it. Sources, all in
//! <https://github.com/blitz-research/blitz3d> (`master`, which may differ in
//! small ways from the v1.83 the games shipped with; the 3ds loader has not
//! changed in any way that matters here as far as the public history shows):
//!
//! - `blitz3d/loader_3ds.cpp`: `Loader_3DS::load`, `parseFile`, `parseScene`,
//!   `parseObject`, `parseTriMesh`, `parseVertList`, `parseMapList`,
//!   `parseFaceList`, `parseFaceMat`, `parseMaterial`, `parseColor`,
//!   `parseKeyFramer` and `parseMeshInfo`.
//! - `blitz3d/meshloader.cpp`: `MeshLoader::addTriangle` and `endMesh`, which
//!   split faces into one surface per brush.
//! - `blitz3d/surface.cpp`: `Surface::updateNormals`.
//! - `blitz3d/meshmodel.cpp`: `MeshModel::transform`, `add`, `findSurface`.
//! - `blitz3d/entity.cpp`: `setParent`, `setWorldTform`, `setLocalTform`
//!   (the world transform of a mesh survives a round trip through
//!   position/scale/rotation, which is lossy for shear and mirroring).
//! - `bbruntime/bbblitz3d.cpp`: `loadEntity`, `bbLoadMesh`, `collapseMesh`
//!   and `blitz3d_open`, which installs the default 3ds loader matrix.
//! - `blitz3d/geom.h`: `Matrix`, `Transform`, `Quat`, `matrixQuat`, and the
//!   fuzzy `Vector::operator<` that `updateNormals` keys its map with.
//!
//! # Chunks
//!
//! Read: `MAIN` (4D4D), `SCENE` (3D3D), `OBJECT` (4000, name then children),
//! `TRIMESH` (4100), `VERTLIST` (4110), `FACELIST` (4120, face indices, then
//! its sub-chunks), `FACEMAT` (4130), `MAPLIST` (4140), `TRMATRIX` (4160),
//! `MATERIAL` (AFFF) with `MATNAME` (A000), `DIFFUSE` (A020, colour as
//! `RGBF` 0010 or `RGBB` 0011) and `TEXTURE` (A200 → `MAPFILE` A300), and
//! from the keyframer (B000) the object nodes (B002) with `NODE_ID` (B030),
//! `NODE_HDR` (B010), `PIVOT` (B013). `$$$DUMMY` nodes become empty
//! hierarchy nodes.
//!
//! Skipped, as Blitz skips them: smoothing groups (4150), face flags, lights,
//! cameras, ambient/specular/bump/opacity maps, the gamma-corrected colour
//! chunks, `BOUNDBOX`, `INSTANCE_NAME`, `KFHDR`, `CURR_TIME` and every
//! animation track (`LoadMesh` passes `HINT_COLLAPSE`).
//!
//! # Conventions
//!
//! Blitz installs `LoaderMatrix "3ds"` = `[1 0 0; 0 0 1; 0 1 0]` by default:
//! y and z are swapped, which has determinant -1, so Blitz also swaps the
//! second and third index of every face to keep it front-facing. The output
//! here is exactly that: positions are `(x, z, y)` of the file, triangles are
//! `[a, c, b]`, and `v` is flipped (`1 - v`). Each object's vertices are put
//! into its local space by the inverse of its `TRMATRIX`, shifted by its
//! keyframer pivot, and `LoadMesh` then moves them to world space again with
//! the node's world transform, so a plain file comes out as it was authored.
//!
//! # Collapse
//!
//! `LoadMesh` builds the hierarchy (object parents come from the keyframer's
//! node ids) and `collapseMesh` flattens it depth first, children before the
//! node itself. Triangles with an equal brush (colour and texture) share one
//! surface across all objects. A surface's vertices are the ones its
//! triangles use, in order of first use; a vertex used by two surfaces is
//! duplicated.
//!
//! # Normals
//!
//! `Surface::updateNormals` runs per object, before the world transform: each
//! non-degenerate triangle's face normal (`(v1-v0) x (v2-v0)`, unit length)
//! is summed onto every vertex at the same position (positions closer than
//! 1e-6 on each axis count as equal, so seams are smoothed), and the sum is
//! normalized. The world transform then multiplies the normals by the
//! cofactor matrix without renormalizing, exactly as `MeshModel::transform`
//! does. Where Blitz would divide zero by zero and store NaN (a vertex whose
//! triangles are all degenerate) this stores a zero normal.
//!
//! # Differences from Blitz
//!
//! - Corrupt input (a face index out of range, a truncated chunk) is an error
//!   where Blitz reads out of bounds.
//! - Surfaces are ordered by Blitz's `map<Brush, ...>` order within an
//!   object, which is a `memcmp` of the render state. Colour bytes are
//!   compared as Blitz does; the texture pointer that follows is an address
//!   at run time, so textured brushes with the same colour are ordered by
//!   file name here.
//! - Two materials with the same colour and texture (case-insensitive) share
//!   a surface, as their brushes compare equal in Blitz. The surface keeps the
//!   first one's name.

use std::collections::BTreeMap;

use crate::blitz::BlitzReader;
use crate::mesh::{Material, Mesh, Surface, Vertex};
use crate::{Error, Result};

const EPSILON: f32 = 0.000001;

const CHUNK_RGBF: u16 = 0x0010;
const CHUNK_RGBB: u16 = 0x0011;
const CHUNK_MAIN: u16 = 0x4D4D;
const CHUNK_SCENE: u16 = 0x3D3D;
const CHUNK_OBJECT: u16 = 0x4000;
const CHUNK_TRIMESH: u16 = 0x4100;
const CHUNK_VERTLIST: u16 = 0x4110;
const CHUNK_FACELIST: u16 = 0x4120;
const CHUNK_FACEMAT: u16 = 0x4130;
const CHUNK_MAPLIST: u16 = 0x4140;
const CHUNK_TRMATRIX: u16 = 0x4160;
const CHUNK_MATERIAL: u16 = 0xAFFF;
const CHUNK_MATNAME: u16 = 0xA000;
const CHUNK_DIFFUSE: u16 = 0xA020;
const CHUNK_TEXTURE: u16 = 0xA200;
const CHUNK_MAPFILE: u16 = 0xA300;
const CHUNK_KEYFRAMER: u16 = 0xB000;
const CHUNK_OBJECT_NODE: u16 = 0xB002;
const CHUNK_NODE_HDR: u16 = 0xB010;
const CHUNK_PIVOT: u16 = 0xB013;
const CHUNK_NODE_ID: u16 = 0xB030;

/// "No node" in the keyframer's ids.
const NO_NODE: u16 = 65535;

/// Parses a `.3ds` file the way `LoadMesh` loads it.
pub fn parse(data: &[u8]) -> Result<Mesh> {
    let mut r = BlitzReader::new(data);
    let id = r.read_short()?;
    let len = r.read_int()?;
    if id != CHUNK_MAIN {
        return Err(Error::Invalid(format!("not a 3ds file (chunk {id:#06x})")));
    }
    // A main chunk longer than the file is read up to the end of the file.
    let end = usize::try_from(len).unwrap_or(0).min(data.len());
    let mut loader = Loader::new();
    for c in chunks(data, 6, end)? {
        match c.id {
            CHUNK_SCENE => loader.scene(data, &c)?,
            CHUNK_KEYFRAMER => loader.keyframer(data, &c)?,
            _ => {}
        }
    }
    Ok(loader.collapse())
}

// ---------------------------------------------------------------------------
// Chunks

#[derive(Clone, Copy)]
struct Chunk {
    id: u16,
    /// First byte after the 6-byte header.
    body: usize,
    /// First byte after the chunk.
    end: usize,
}

/// The chunks laid end to end in `data[pos..end]` (`nextChunk`).
fn chunks(data: &[u8], mut pos: usize, end: usize) -> Result<Vec<Chunk>> {
    let mut out = Vec::new();
    while pos < end {
        let mut r = BlitzReader::new(data.get(pos..end).unwrap_or_default());
        let id = r.read_short()?;
        let len = r.read_int()?;
        let chunk_end = usize::try_from(len)
            .ok()
            .filter(|&l| l >= 6)
            .and_then(|l| pos.checked_add(l))
            .filter(|&e| e <= end)
            .ok_or_else(|| {
                Error::Invalid(format!("chunk {id:#06x} at {pos} has bad length {len}"))
            })?;
        out.push(Chunk {
            id,
            body: pos + 6,
            end: chunk_end,
        });
        pos = chunk_end;
    }
    Ok(out)
}

fn body<'a>(data: &'a [u8], c: &Chunk) -> BlitzReader<'a> {
    BlitzReader::new(&data[c.body..c.end])
}

/// A NUL-terminated string (`parseString`), decoded as Latin-1.
fn read_cstring(r: &mut BlitzReader) -> Result<String> {
    let mut s = String::new();
    loop {
        match r.read_byte()? {
            0 => return Ok(s),
            b => s.push(char::from(b)),
        }
    }
}

fn read_vec3(r: &mut BlitzReader) -> Result<V3> {
    Ok([r.read_float()?, r.read_float()?, r.read_float()?])
}

// ---------------------------------------------------------------------------
// Blitz's geom.h, in f32 with the same operation order

type V3 = [f32; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: V3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalized(a: V3) -> V3 {
    let l = length(a);
    [a[0] / l, a[1] / l, a[2] / l]
}

/// `Vector::operator!=`: equal to within [`EPSILON`] on every axis.
fn differs(a: V3, b: V3) -> bool {
    (a[0] - b[0]).abs() > EPSILON || (a[1] - b[1]).abs() > EPSILON || (a[2] - b[2]).abs() > EPSILON
}

/// `Vector::operator<`, the (not quite transitive) ordering of `map<Vector,..>`.
fn fuzzy_less(a: V3, b: V3) -> bool {
    if (a[0] - b[0]).abs() > EPSILON {
        return a[0] < b[0];
    }
    if (a[1] - b[1]).abs() > EPSILON {
        return a[1] < b[1];
    }
    (a[2] - b[2]).abs() > EPSILON && a[2] < b[2]
}

/// Columns `i`, `j`, `k`, as in Blitz's `Matrix`.
#[derive(Clone, Copy)]
struct Mat {
    i: V3,
    j: V3,
    k: V3,
}

impl Mat {
    const IDENTITY: Mat = Mat {
        i: [1.0, 0.0, 0.0],
        j: [0.0, 1.0, 0.0],
        k: [0.0, 0.0, 1.0],
    };

    fn from_quat(w: f32, v: V3) -> Mat {
        let (xx, yy, zz) = (v[0] * v[0], v[1] * v[1], v[2] * v[2]);
        let (xy, xz, yz) = (v[0] * v[1], v[0] * v[2], v[1] * v[2]);
        let (wx, wy, wz) = (w * v[0], w * v[1], w * v[2]);
        Mat {
            i: [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
            j: [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
            k: [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
        }
    }

    fn mul_vec(&self, q: V3) -> V3 {
        let (i, j, k) = (self.i, self.j, self.k);
        [
            i[0] * q[0] + j[0] * q[1] + k[0] * q[2],
            i[1] * q[0] + j[1] * q[1] + k[1] * q[2],
            i[2] * q[0] + j[2] * q[1] + k[2] * q[2],
        ]
    }

    fn mul(&self, q: &Mat) -> Mat {
        Mat {
            i: self.mul_vec(q.i),
            j: self.mul_vec(q.j),
            k: self.mul_vec(q.k),
        }
    }

    fn determinant(&self) -> f32 {
        let (i, j, k) = (self.i, self.j, self.k);
        i[0] * (j[1] * k[2] - j[2] * k[1]) - i[1] * (j[0] * k[2] - j[2] * k[0])
            + i[2] * (j[0] * k[1] - j[1] * k[0])
    }

    /// `Matrix::operator-`.
    fn inverse(&self) -> Mat {
        let (i, j, k) = (self.i, self.j, self.k);
        let t = 1.0 / self.determinant();
        Mat {
            i: [
                t * (j[1] * k[2] - j[2] * k[1]),
                -t * (i[1] * k[2] - i[2] * k[1]),
                t * (i[1] * j[2] - i[2] * j[1]),
            ],
            j: [
                -t * (j[0] * k[2] - j[2] * k[0]),
                t * (i[0] * k[2] - i[2] * k[0]),
                -t * (i[0] * j[2] - i[2] * j[0]),
            ],
            k: [
                t * (j[0] * k[1] - j[1] * k[0]),
                -t * (i[0] * k[1] - i[1] * k[0]),
                t * (i[0] * j[1] - i[1] * j[0]),
            ],
        }
    }

    fn cofactor(&self) -> Mat {
        let (i, j, k) = (self.i, self.j, self.k);
        Mat {
            i: [
                j[1] * k[2] - j[2] * k[1],
                -(j[0] * k[2] - j[2] * k[0]),
                j[0] * k[1] - j[1] * k[0],
            ],
            j: [
                -(i[1] * k[2] - i[2] * k[1]),
                i[0] * k[2] - i[2] * k[0],
                -(i[0] * k[1] - i[1] * k[0]),
            ],
            k: [
                i[1] * j[2] - i[2] * j[1],
                -(i[0] * j[2] - i[2] * j[0]),
                i[0] * j[1] - i[1] * j[0],
            ],
        }
    }

    fn orthogonalized(&self) -> Mat {
        let k = normalized(self.k);
        let i = normalized(cross(self.j, k));
        let j = cross(k, i);
        Mat { i, j, k }
    }

    /// `matrixQuat`: a `(w, xyz)` quaternion for the (orthogonalized) matrix.
    fn to_quat(self) -> (f32, V3) {
        let m = self.orthogonalized();
        let t = m.i[0] + m.j[1] + m.k[2];
        let (w, x, y, z);
        if t > EPSILON {
            let t = (t + 1.0).sqrt() * 2.0;
            x = (m.k[1] - m.j[2]) / t;
            y = (m.i[2] - m.k[0]) / t;
            z = (m.j[0] - m.i[1]) / t;
            w = t / 4.0;
        } else if m.i[0] > m.j[1] && m.i[0] > m.k[2] {
            let t = (m.i[0] - m.j[1] - m.k[2] + 1.0).sqrt() * 2.0;
            x = t / 4.0;
            y = (m.j[0] + m.i[1]) / t;
            z = (m.i[2] + m.k[0]) / t;
            w = (m.k[1] - m.j[2]) / t;
        } else if m.j[1] > m.k[2] {
            let t = (m.j[1] - m.k[2] - m.i[0] + 1.0).sqrt() * 2.0;
            x = (m.j[0] + m.i[1]) / t;
            y = t / 4.0;
            z = (m.k[1] + m.j[2]) / t;
            w = (m.i[2] - m.k[0]) / t;
        } else {
            let t = (m.k[2] - m.j[1] - m.i[0] + 1.0).sqrt() * 2.0;
            x = (m.i[2] + m.k[0]) / t;
            y = (m.k[1] + m.j[2]) / t;
            z = t / 4.0;
            w = (m.j[0] - m.i[1]) / t;
        }
        (w, [x, y, z])
    }
}

/// Blitz's `Transform`: a matrix and a translation.
#[derive(Clone, Copy)]
struct Tform {
    m: Mat,
    v: V3,
}

impl Tform {
    const IDENTITY: Tform = Tform {
        m: Mat::IDENTITY,
        v: [0.0; 3],
    };

    fn apply(&self, q: V3) -> V3 {
        let r = self.m.mul_vec(q);
        [r[0] + self.v[0], r[1] + self.v[1], r[2] + self.v[2]]
    }

    fn mul(&self, q: &Tform) -> Tform {
        let r = self.m.mul_vec(q.v);
        Tform {
            m: self.m.mul(&q.m),
            v: [r[0] + self.v[0], r[1] + self.v[1], r[2] + self.v[2]],
        }
    }

    fn inverse(&self) -> Tform {
        let m = self.m.inverse();
        Tform {
            m,
            v: m.mul_vec([-self.v[0], -self.v[1], -self.v[2]]),
        }
    }
}

/// `blitz3d_open`'s loader matrix for `"3ds"`: swap y and z.
const LOADER_MATRIX: Tform = Tform {
    m: Mat {
        i: [1.0, 0.0, 0.0],
        j: [0.0, 0.0, 1.0],
        k: [0.0, 1.0, 0.0],
    },
    v: [0.0; 3],
};

// ---------------------------------------------------------------------------
// Brushes and the meshes being built

/// The part of Blitz's `Brush` a 3ds file can set.
#[derive(Clone, Debug)]
struct Brush {
    name: String,
    color: V3,
    texture: Option<String>,
}

impl Brush {
    fn plain() -> Brush {
        Brush {
            name: String::new(),
            color: [1.0; 3],
            texture: None,
        }
    }

    /// The order of Blitz's `map<Brush, ..>` (`memcmp` of the render state),
    /// which also decides which brushes are equal.
    fn key(&self) -> BrushKey {
        let mut color = [0u8; 12];
        for (chunk, c) in color.chunks_exact_mut(4).zip(self.color) {
            chunk.copy_from_slice(&c.to_le_bytes());
        }
        (color, self.texture.as_ref().map(|t| t.to_lowercase()))
    }
}

type BrushKey = ([u8; 12], Option<String>);

#[derive(Clone, Default)]
struct BSurface {
    brush: Option<Brush>,
    vertices: Vec<Vertex>,
    triangles: Vec<[u32; 3]>,
}

/// A `MeshModel`: surfaces in creation order.
#[derive(Clone, Default)]
struct BMesh {
    surfaces: Vec<(BrushKey, BSurface)>,
}

impl BMesh {
    fn surface_for(&mut self, brush: &Brush) -> &mut BSurface {
        let key = brush.key();
        let idx = match self.surfaces.iter().position(|(k, _)| *k == key) {
            Some(idx) => idx,
            None => {
                self.surfaces.push((
                    key,
                    BSurface {
                        brush: Some(brush.clone()),
                        ..BSurface::default()
                    },
                ));
                self.surfaces.len() - 1
            }
        };
        &mut self.surfaces[idx].1
    }

    /// `MeshModel::transform`.
    fn transform(&mut self, t: &Tform) {
        let co = t.m.cofactor();
        for (_, s) in &mut self.surfaces {
            for v in &mut s.vertices {
                v.position = t.apply(v.position);
                v.normal = co.mul_vec(v.normal);
            }
        }
    }

    /// `MeshModel::add`.
    fn add(&mut self, src: &BMesh) {
        for (_, s) in &src.surfaces {
            let brush = s.brush.as_ref().expect("surfaces carry a brush");
            let dest = self.surface_for(brush);
            let base = dest.vertices.len() as u32;
            dest.triangles
                .extend(s.triangles.iter().map(|t| t.map(|i| i + base)));
            dest.vertices.extend_from_slice(&s.vertices);
        }
    }

    /// `Surface::updateNormals` on every surface.
    fn update_normals(&mut self) {
        for (_, s) in &mut self.surfaces {
            // `map<Vector, Vector>` keyed by the fuzzy ordering, kept sorted.
            let mut map: Vec<(V3, V3)> = Vec::new();
            fn slot(map: &mut Vec<(V3, V3)>, key: V3) -> usize {
                let idx = map.partition_point(|(k, _)| fuzzy_less(*k, key));
                if idx < map.len() && !fuzzy_less(key, map[idx].0) {
                    idx
                } else {
                    map.insert(idx, (key, [0.0; 3]));
                    idx
                }
            }
            for t in &s.triangles {
                let p = t.map(|i| s.vertices[i as usize].position);
                let mut n = cross(sub(p[1], p[0]), sub(p[2], p[0]));
                if length(n) <= EPSILON {
                    continue;
                }
                n = normalized(n);
                for q in p {
                    let idx = slot(&mut map, q);
                    for (a, b) in map[idx].1.iter_mut().zip(n) {
                        *a += b;
                    }
                }
            }
            for v in &mut s.vertices {
                let idx = slot(&mut map, v.position);
                let sum = map[idx].1;
                v.normal = if length(sum) > 0.0 {
                    normalized(sum)
                } else {
                    [0.0; 3]
                };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The entity hierarchy

/// An `Entity`/`MeshModel`: Blitz keeps a node's transform as position,
/// scale and rotation, so setting a world transform is lossy.
struct Node {
    parent: Option<usize>,
    children: Vec<usize>,
    pos: V3,
    scale: V3,
    rot: (f32, V3),
    mesh: Option<BMesh>,
}

impl Node {
    fn new(parent: Option<usize>) -> Node {
        Node {
            parent,
            children: Vec::new(),
            pos: [0.0; 3],
            scale: [1.0; 3],
            rot: (1.0, [0.0; 3]),
            mesh: None,
        }
    }
}

struct Loader {
    /// Node 0 is the root.
    nodes: Vec<Node>,
    materials: BTreeMap<String, Brush>,
    /// `name_map`: object name to node.
    names: BTreeMap<String, usize>,
    /// `id_map`: keyframer node id to node.
    ids: BTreeMap<u16, usize>,
}

impl Loader {
    fn new() -> Loader {
        Loader {
            nodes: vec![Node::new(None)],
            materials: BTreeMap::new(),
            names: BTreeMap::new(),
            ids: BTreeMap::new(),
        }
    }

    fn local_tform(&self, n: usize) -> Tform {
        let node = &self.nodes[n];
        let mut m = Mat::from_quat(node.rot.0, node.rot.1);
        for (axis, s) in [&mut m.i, &mut m.j, &mut m.k].into_iter().zip(node.scale) {
            for c in axis.iter_mut() {
                *c *= s;
            }
        }
        Tform { m, v: node.pos }
    }

    fn world_tform(&self, n: usize) -> Tform {
        match self.nodes[n].parent {
            Some(p) => self.world_tform(p).mul(&self.local_tform(n)),
            None => self.local_tform(n),
        }
    }

    /// `Entity::setWorldTform`.
    fn set_world_tform(&mut self, n: usize, t: &Tform) {
        let local = match self.nodes[n].parent {
            Some(p) => self.world_tform(p).inverse().mul(t),
            None => *t,
        };
        let node = &mut self.nodes[n];
        node.pos = local.v;
        node.scale = [length(local.m.i), length(local.m.j), length(local.m.k)];
        node.rot = local.m.to_quat();
    }

    /// `Entity::setParent`: appends to the new parent's children.
    fn set_parent(&mut self, n: usize, p: usize) {
        if self.nodes[n].parent == Some(p) {
            return;
        }
        if let Some(old) = self.nodes[n].parent {
            self.nodes[old].children.retain(|&c| c != n);
        }
        self.nodes[n].parent = Some(p);
        self.nodes[p].children.push(n);
    }

    fn add_node(&mut self, parent: usize) -> usize {
        self.nodes.push(Node::new(Some(parent)));
        let n = self.nodes.len() - 1;
        self.nodes[parent].children.push(n);
        n
    }

    fn scene(&mut self, data: &[u8], scene: &Chunk) -> Result<()> {
        for c in chunks(data, scene.body, scene.end)? {
            match c.id {
                CHUNK_OBJECT => self.object(data, &c)?,
                CHUNK_MATERIAL => self.material(data, &c)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn object(&mut self, data: &[u8], c: &Chunk) -> Result<()> {
        let mut r = body(data, c);
        let name = read_cstring(&mut r)?;
        let start = c.body + r.position();
        for sub in chunks(data, start, c.end)? {
            if sub.id == CHUNK_TRIMESH {
                let node = self.add_node(0);
                self.names.insert(name.clone(), node);
                self.trimesh(data, &sub, node)?;
            }
        }
        Ok(())
    }

    fn material(&mut self, data: &[u8], c: &Chunk) -> Result<()> {
        let mut name = String::new();
        let mut color = [1.0; 3];
        let mut texture = String::new();
        for sub in chunks(data, c.body, c.end)? {
            match sub.id {
                CHUNK_MATNAME => name = read_cstring(&mut body(data, &sub))?,
                CHUNK_DIFFUSE => color = parse_color(data, &sub)?,
                CHUNK_TEXTURE => {
                    for m in chunks(data, sub.body, sub.end)? {
                        if m.id == CHUNK_MAPFILE {
                            texture = read_cstring(&mut body(data, &m))?;
                        }
                    }
                }
                _ => {}
            }
        }
        let mut brush = Brush {
            name: name.clone(),
            color,
            texture: None,
        };
        if !texture.is_empty() {
            brush.texture = Some(texture);
            brush.color = [1.0; 3];
        }
        if !name.is_empty() {
            self.materials.insert(name, brush);
        }
        Ok(())
    }

    fn trimesh(&mut self, data: &[u8], c: &Chunk, node: usize) -> Result<()> {
        let mut verts: Vec<Vertex> = Vec::new();
        let mut faces: Vec<([u32; 3], Brush)> = Vec::new();
        let mut tform = Tform::IDENTITY;
        for sub in chunks(data, c.body, c.end)? {
            match sub.id {
                CHUNK_VERTLIST => {
                    let mut r = body(data, &sub);
                    for _ in 0..r.read_short()? {
                        let p = LOADER_MATRIX.apply(read_vec3(&mut r)?);
                        verts.push(Vertex {
                            position: p,
                            ..Vertex::default()
                        });
                    }
                }
                CHUNK_MAPLIST => {
                    let mut r = body(data, &sub);
                    let count = usize::from(r.read_short()?);
                    if count > verts.len() {
                        return Err(Error::Invalid(format!(
                            "map list has {count} entries for {} vertices",
                            verts.len()
                        )));
                    }
                    for v in &mut verts[..count] {
                        let u = r.read_float()?;
                        let w = r.read_float()?;
                        v.uv = [u, 1.0 - w];
                    }
                }
                CHUNK_FACELIST => self.facelist(data, &sub, &mut faces)?,
                CHUNK_TRMATRIX => {
                    let mut r = body(data, &sub);
                    let m = Mat {
                        i: read_vec3(&mut r)?,
                        j: read_vec3(&mut r)?,
                        k: read_vec3(&mut r)?,
                    };
                    let t = Tform {
                        m,
                        v: read_vec3(&mut r)?,
                    };
                    tform = LOADER_MATRIX.mul(&t).mul(&LOADER_MATRIX.inverse());
                }
                _ => {}
            }
        }

        self.set_world_tform(node, &tform);

        let inv = tform.inverse();
        for v in &mut verts {
            v.position = inv.apply(v.position);
        }

        // MeshLoader::addTriangle / endMesh: one surface per brush, in brush
        // order; vertices numbered by first use within each surface.
        let mut by_brush: BTreeMap<BrushKey, (Brush, Vec<[u32; 3]>)> = BTreeMap::new();
        for (idx, brush) in faces {
            by_brush
                .entry(brush.key())
                .or_insert_with(|| (brush.clone(), Vec::new()))
                .1
                .push(idx);
        }
        let mut mesh = BMesh::default();
        for (key, (brush, tris)) in by_brush {
            let mut surf = BSurface {
                brush: Some(brush),
                ..BSurface::default()
            };
            let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
            for tri in tris {
                let mut out = [0u32; 3];
                for (o, n) in out.iter_mut().zip(tri) {
                    let v = *verts.get(n as usize).ok_or_else(|| {
                        Error::Invalid(format!(
                            "face uses vertex {n} of {} in its mesh",
                            verts.len()
                        ))
                    })?;
                    *o = *remap.entry(n).or_insert_with(|| {
                        surf.vertices.push(v);
                        (surf.vertices.len() - 1) as u32
                    });
                }
                surf.triangles.push(out);
            }
            mesh.surfaces.push((key, surf));
        }
        mesh.update_normals();
        self.nodes[node].mesh = Some(mesh);
        Ok(())
    }

    fn facelist(
        &mut self,
        data: &[u8],
        c: &Chunk,
        faces: &mut Vec<([u32; 3], Brush)>,
    ) -> Result<()> {
        let mut r = body(data, c);
        let count = r.read_short()?;
        let first = faces.len();
        for _ in 0..count {
            let a = r.read_short()?;
            let b = r.read_short()?;
            let c = r.read_short()?;
            let _flags = r.read_short()?;
            // The determinant of the loader matrix is negative: swap two
            // vertices to keep the face pointing the same way.
            faces.push(([u32::from(a), u32::from(c), u32::from(b)], Brush::plain()));
        }
        let start = c.body + r.position();
        for sub in chunks(data, start, c.end)? {
            if sub.id != CHUNK_FACEMAT {
                continue;
            }
            let mut r = body(data, &sub);
            let name = read_cstring(&mut r)?;
            // `materials_map[name]` inserts a default brush for unknown names.
            let brush = self
                .materials
                .entry(name.clone())
                .or_insert_with(Brush::plain)
                .clone();
            for _ in 0..r.read_short()? {
                let f = usize::from(r.read_short()?);
                let slot = faces.get_mut(f).ok_or_else(|| {
                    Error::Invalid(format!("material {name:?} assigned to missing face {f}"))
                })?;
                slot.1 = brush.clone();
            }
        }
        debug_assert!(faces.len() >= first);
        Ok(())
    }

    fn keyframer(&mut self, data: &[u8], c: &Chunk) -> Result<()> {
        for sub in chunks(data, c.body, c.end)? {
            if sub.id == CHUNK_OBJECT_NODE {
                self.mesh_info(data, &sub)?;
            }
        }
        Ok(())
    }

    /// `parseMeshInfo` with `HINT_COLLAPSE`: only the hierarchy and pivot.
    fn mesh_info(&mut self, data: &[u8], c: &Chunk) -> Result<()> {
        let mut name = String::new();
        let mut pivot = [0.0f32; 3];
        let mut id = NO_NODE;
        let mut parent = NO_NODE;
        for sub in chunks(data, c.body, c.end)? {
            let mut r = body(data, &sub);
            match sub.id {
                CHUNK_NODE_ID => id = r.read_short()?,
                CHUNK_NODE_HDR => {
                    name = read_cstring(&mut r)?;
                    let _flags1 = r.read_short()?;
                    let _flags2 = r.read_short()?;
                    parent = r.read_short()?;
                }
                CHUNK_PIVOT => pivot = LOADER_MATRIX.apply(read_vec3(&mut r)?),
                _ => {}
            }
        }

        let mut p = 0;
        if parent != NO_NODE {
            match self.ids.get(&parent) {
                Some(&n) => p = n,
                None => return Ok(()),
            }
        }
        let node = if name == "$$$DUMMY" {
            self.add_node(p)
        } else {
            let Some(n) = self.names.remove(&name) else {
                return Ok(());
            };
            if differs(pivot, [0.0; 3]) {
                let t = Tform {
                    m: Mat::IDENTITY,
                    v: [-pivot[0], -pivot[1], -pivot[2]],
                };
                if let Some(mesh) = self.nodes[n].mesh.as_mut() {
                    mesh.transform(&t);
                }
            }
            let world = self.world_tform(n);
            self.set_parent(n, p);
            self.set_world_tform(n, &world);
            n
        };
        if id != NO_NODE {
            self.ids.insert(id, node);
        }
        Ok(())
    }

    /// `bbLoadMesh`'s `collapseMesh`: children first, each mesh moved to world
    /// space and added to one mesh.
    fn collapse(&self) -> Mesh {
        fn walk(l: &Loader, n: usize, out: &mut BMesh) {
            for &c in &l.nodes[n].children {
                walk(l, c, out);
            }
            if let Some(mesh) = &l.nodes[n].mesh {
                let mut mesh = mesh.clone();
                mesh.transform(&l.world_tform(n));
                out.add(&mesh);
            }
        }
        let mut out = BMesh::default();
        walk(self, 0, &mut out);
        Mesh {
            surfaces: out
                .surfaces
                .into_iter()
                .map(|(_, s)| {
                    let brush = s.brush.expect("surfaces carry a brush");
                    Surface {
                        vertices: s.vertices,
                        triangles: s.triangles,
                        material: Material {
                            name: brush.name,
                            color: brush.color,
                            alpha: 1.0,
                            texture: brush.texture,
                        },
                    }
                })
                .collect(),
        }
    }
}

/// `parseColor`: the last `RGBF`/`RGBB` child; black if there is none.
fn parse_color(data: &[u8], c: &Chunk) -> Result<V3> {
    let mut v = [0.0; 3];
    for sub in chunks(data, c.body, c.end)? {
        let mut r = body(data, &sub);
        match sub.id {
            CHUNK_RGBF => v = read_vec3(&mut r)?,
            CHUNK_RGBB => {
                let mut rgb = [0.0; 3];
                for c in &mut rgb {
                    *c = f32::from(r.read_byte()?) / 255.0;
                }
                v = rgb;
            }
            _ => {}
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blitz::BlitzWriter;

    fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
        let mut w = BlitzWriter::new();
        w.write_short(id);
        w.write_int(body.len() as i32 + 6);
        w.write_bytes(body);
        w.into_bytes()
    }

    fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
        parts.concat()
    }

    fn floats(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    fn cstr(s: &str) -> Vec<u8> {
        let mut v = s.as_bytes().to_vec();
        v.push(0);
        v
    }

    fn material(name: &str, rgb: [u8; 3], texture: Option<&str>) -> Vec<u8> {
        let mut parts = vec![
            chunk(CHUNK_MATNAME, &cstr(name)),
            chunk(CHUNK_DIFFUSE, &chunk(CHUNK_RGBB, &rgb)),
        ];
        if let Some(t) = texture {
            parts.push(chunk(CHUNK_TEXTURE, &chunk(CHUNK_MAPFILE, &cstr(t))));
        }
        chunk(CHUNK_MATERIAL, &cat(&parts))
    }

    /// Two triangles sharing an edge in the file's xy plane (z up), one
    /// untextured red and one textured. Optionally with a mesh matrix that
    /// moves the object's origin to (10, 20, 30).
    fn sample(with_matrix: bool) -> Vec<u8> {
        let mut verts = 4u16.to_le_bytes().to_vec();
        verts.extend(floats(&[
            0.0, 0.0, 0.0, //
            1.0, 0.0, 0.0, //
            1.0, 2.0, 0.0, //
            0.0, 2.0, 3.0,
        ]));
        let mut faces = 2u16.to_le_bytes().to_vec();
        for f in [[0u16, 1, 2, 0], [0, 2, 3, 0]] {
            faces.extend(f.iter().flat_map(|i| i.to_le_bytes()));
        }
        let mat = |name: &str, face: u16| {
            let mut b = cstr(name);
            b.extend(1u16.to_le_bytes());
            b.extend(face.to_le_bytes());
            chunk(CHUNK_FACEMAT, &b)
        };
        faces.extend(cat(&[mat("red", 0), mat("tex", 1)]));
        let mut uvs = 4u16.to_le_bytes().to_vec();
        uvs.extend(floats(&[0.0, 0.25, 1.0, 0.25, 1.0, 1.0, 0.0, 1.0]));
        let mut parts = vec![
            chunk(CHUNK_VERTLIST, &verts),
            chunk(CHUNK_MAPLIST, &uvs),
            chunk(CHUNK_FACELIST, &faces),
        ];
        if with_matrix {
            parts.push(chunk(
                CHUNK_TRMATRIX,
                &floats(&[1., 0., 0., 0., 1., 0., 0., 0., 1., 10., 20., 30.]),
            ));
        }
        let object = chunk(
            CHUNK_OBJECT,
            &[cstr("obj"), chunk(CHUNK_TRIMESH, &cat(&parts))].concat(),
        );
        let scene = chunk(
            CHUNK_SCENE,
            &cat(&[
                material("red", [255, 0, 0], None),
                material("tex", [0, 255, 0], Some("wall.jpg")),
                object,
            ]),
        );
        chunk(CHUNK_MAIN, &scene)
    }

    #[test]
    fn axes_winding_uvs_and_material_split() {
        let mesh = parse(&sample(false)).unwrap();
        assert_eq!(mesh.surfaces.len(), 2);
        let red = mesh
            .surfaces
            .iter()
            .find(|s| s.material.name == "red")
            .unwrap();
        let tex = mesh
            .surfaces
            .iter()
            .find(|s| s.material.name == "tex")
            .unwrap();
        assert_eq!(red.material.color, [1.0, 0.0, 0.0]);
        assert_eq!(red.material.texture, None);
        assert_eq!(tex.material.texture.as_deref(), Some("wall.jpg"));
        assert_eq!(tex.material.color, [1.0; 3]);

        // File (x, y, z) becomes (x, z, y); the triangle [0, 1, 2] becomes
        // [0, 2, 1] and its vertices are numbered by first use (0, 2, 1);
        // v becomes 1 - v.
        let p = |s: &Surface| s.vertices.iter().map(|v| v.position).collect::<Vec<_>>();
        assert_eq!(p(red), [[0., 0., 0.], [1., 0., 2.], [1., 0., 0.]]);
        assert_eq!(red.triangles, [[0, 1, 2]]);
        assert_eq!(p(tex), [[0., 0., 0.], [0., 3., 2.], [1., 0., 2.]]);
        assert_eq!(tex.triangles, [[0, 1, 2]]);
        assert_eq!(red.vertices[0].uv, [0.0, 0.75]);
        assert_eq!(red.vertices[1].uv, [1.0, 0.0]);
        assert_eq!(red.vertices[2].uv, [1.0, 0.75]);
        assert_eq!(tex.vertices[1].uv, [0.0, 0.0]);

        // Red: (1,0,2) x (1,0,0) = (0, 2, 0), so the normal is +y (up).
        for v in &red.vertices {
            assert_eq!(v.normal, [0.0, 1.0, 0.0]);
        }
    }

    #[test]
    fn mesh_matrix_does_not_move_the_geometry() {
        // Vertices are in world space; the matrix only places the pivot.
        let a = parse(&sample(false)).unwrap();
        let b = parse(&sample(true)).unwrap();
        let (lo_a, hi_a) = a.bounds().unwrap();
        let (lo_b, hi_b) = b.bounds().unwrap();
        for i in 0..3 {
            assert!((lo_a[i] - lo_b[i]).abs() < 1e-4 && (hi_a[i] - hi_b[i]).abs() < 1e-4);
        }
        assert_eq!(a.vertex_count(), b.vertex_count());
    }

    fn tri_object(name: &str, x: f32) -> Vec<u8> {
        let mut verts = 3u16.to_le_bytes().to_vec();
        verts.extend(floats(&[x, 0.0, 0.0, x + 1.0, 0.0, 0.0, x, 1.0, 0.0]));
        let mut faces = 1u16.to_le_bytes().to_vec();
        faces.extend([0u16, 1, 2, 0].iter().flat_map(|i| i.to_le_bytes()));
        let mesh = cat(&[chunk(CHUNK_VERTLIST, &verts), chunk(CHUNK_FACELIST, &faces)]);
        chunk(
            CHUNK_OBJECT,
            &[cstr(name), chunk(CHUNK_TRIMESH, &mesh)].concat(),
        )
    }

    fn node(id: u16, name: &str, parent: u16, pivot: Option<[f32; 3]>) -> Vec<u8> {
        let mut hdr = cstr(name);
        hdr.extend([0u8; 4]);
        hdr.extend(parent.to_le_bytes());
        let mut parts = vec![
            chunk(CHUNK_NODE_ID, &id.to_le_bytes()),
            chunk(CHUNK_NODE_HDR, &hdr),
        ];
        if let Some(p) = pivot {
            parts.push(chunk(CHUNK_PIVOT, &floats(&p)));
        }
        chunk(CHUNK_OBJECT_NODE, &cat(&parts))
    }

    fn file(objects: &[Vec<u8>], nodes: &[Vec<u8>]) -> Vec<u8> {
        let mut parts = vec![chunk(CHUNK_SCENE, &cat(objects))];
        if !nodes.is_empty() {
            parts.push(chunk(CHUNK_KEYFRAMER, &cat(nodes)));
        }
        chunk(CHUNK_MAIN, &cat(&parts))
    }

    fn xs(mesh: &Mesh) -> Vec<f32> {
        mesh.surfaces[0]
            .vertices
            .iter()
            .map(|v| v.position[0])
            .collect()
    }

    #[test]
    fn collapse_visits_children_before_their_parent() {
        let objects = [tri_object("a", 0.0), tri_object("b", 10.0)];
        // No keyframer: both hang off the root, in file order.
        let flat = parse(&file(&objects, &[])).unwrap();
        assert_eq!(xs(&flat), [0.0, 0.0, 1.0, 10.0, 10.0, 11.0]);
        // b is a's child, so b comes first.
        let nested = parse(&file(
            &objects,
            &[node(0, "a", NO_NODE, None), node(1, "b", 0, None)],
        ))
        .unwrap();
        assert_eq!(xs(&nested), [10.0, 10.0, 11.0, 0.0, 0.0, 1.0]);
        // A parent that was never declared leaves the object where it was.
        let orphan = parse(&file(&objects, &[node(1, "b", 7, None)])).unwrap();
        assert_eq!(xs(&orphan), xs(&flat));
    }

    #[test]
    fn pivot_offsets_the_vertices() {
        let objects = [tri_object("a", 0.0)];
        // The file's pivot (1, 2, 3) is Blitz's (1, 3, 2).
        let mesh = parse(&file(
            &objects,
            &[node(0, "a", NO_NODE, Some([1.0, 2.0, 3.0]))],
        ))
        .unwrap();
        let v = &mesh.surfaces[0].vertices;
        assert_eq!(v[0].position, [-1.0, -3.0, -2.0]);
        assert_eq!(v[1].position, [-1.0, -3.0, -1.0]);
        assert_eq!(v[2].position, [0.0, -3.0, -2.0]);
    }

    #[test]
    fn rejects_other_files_and_truncation() {
        assert!(parse(b"nope").is_err());
        let data = sample(false);
        assert!(parse(&data[..data.len() - 7]).is_err());
    }
}
