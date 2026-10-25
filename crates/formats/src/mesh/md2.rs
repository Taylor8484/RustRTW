//! `.md2` animated models (Quake 2), as Blitz3D's `LoadMD2` holds them.
//!
//! Ported from `MD2Rep::MD2Rep` and `MD2Rep::render` in `blitz3d/md2rep.cpp`,
//! `MD2Model::startMD2Anim` / `animate` in `blitz3d/md2model.cpp` and the
//! normal table in `blitz3d/md2norms.cpp` of
//! <https://github.com/blitz-research/blitz3d>. That is the `master` branch;
//! the MD2 files there have not changed in a way that matters here, but the
//! games shipped with v1.83 and the code was not compared against that
//! release.
//!
//! What Blitz keeps of the file:
//!
//! - **Axis conversion.** A Quake vertex `(x, y, z)` (x forward, y left,
//!   z up) becomes Blitz `(y, z, x)`, for the per-frame `scale` and `translate`
//!   as well as the stored byte coordinates. The three-way rotation does not
//!   mirror anything: Blitz does not negate any axis.
//! - **Frame decompression.** Each frame is `byte * scale + translate` per
//!   axis (all in `f32`, after the axis conversion above).
//! - **Normals.** The per-vertex normal byte indexes Blitz's 162-entry
//!   `md2norms` table, whose components are rotated the same way
//!   (`(x, y, z)` to `(y, z, x)`). Normals are never recomputed.
//! - **Vertex sharing.** Triangles pair a vertex index with a texture
//!   coordinate index. Blitz makes one vertex for every distinct
//!   `(vertex, uv)` pair, numbered in order of first use while walking the
//!   triangles. Every frame holds that same list of vertices.
//! - **UVs.** `u = s / skinWidth`, `v = t / skinHeight`, with `s` and `t` the
//!   signed 16-bit values from the file. `v` is not flipped.
//! - **Winding.** Each triangle `(a, b, c)` is stored as `(a, c, b)`.
//!
//! Blitz reads neither the skin names nor the frame names, and its
//! `AnimateMD2` takes frame numbers only. [`Md2Model`] keeps both anyway, as
//! they are in the file and cost nothing. Blitz also ignores `frameSize`,
//! reading frames back to back, and so does this parser.
//!
//! Where Blitz would read outside its buffers (indices past the end, a normal
//! byte of 162 or more, a zero skin size) or return a null model (bad magic,
//! version other than 8, no frames), parsing fails with [`Error::Invalid`].
//!
//! # How the games animate
//!
//! The games only call `AnimateMD2 entity, mode, speed, first, last` (no
//! transition). Mode 0 (stop) is used for `0,1,0,0`, which freezes frame 0;
//! mode 1 loops, mode 2 ping-pongs, and mode 3 is a one-shot (Blitz treats
//! any mode other than 1 and 2 as "stop at the end"):
//!
//! - Player (`stinky`, `peegueb`, 147 frames): `0,1,0,0` standing;
//!   `1,.25,2,18` walk; `1,.25,19,35` push-walk; `3,.2,36,42` jump; idle
//!   loops in `game.bb` `1,.1,105,147`, `1,.1,62,104` and `1,.2,53,61`.
//!   Note the last frame 147 is past the end (frames are 0-based, so
//!   0..=146); `startMD2Anim` clamps it to 146.
//! - Monsters (30 frames): `1,.4,1,24` (walk, coily) and `2,.3,0,6` (idle,
//!   chomper and bomb).
//! - Menu model: `1,.1,44,52`, `2,.1,38,44` and `2,.3,0,6`.
//!
//! Playback (`MD2Model::animate`) steps a fractional time through
//! `first..=last` and blends the two frames around it linearly, positions and
//! normals alike, without renormalising. That is later work; what it needs
//! from here is [`Md2Model::frames`] with per-frame positions and normals for
//! the shared vertex list, [`Md2Model::uvs`] and [`Md2Model::triangles`].

use std::collections::HashMap;

use crate::blitz::BlitzReader;
use crate::mesh::{Mesh, Surface, Vertex};
use crate::{Error, Result};

/// `'IDP2'` read as a little-endian int.
const MAGIC: i32 = 0x3250_4449;
const VERSION: i32 = 8;

/// A parsed `.md2` file.
#[derive(Debug, Clone, PartialEq)]
pub struct Md2Model {
    /// Skin size in pixels, which the UVs were divided by.
    pub skin_size: [u32; 2],
    /// Skin names (64-byte fields in the file). Blitz ignores them.
    pub skins: Vec<String>,
    /// One UV per vertex, in Blitz's convention (`v` grows downward).
    pub uvs: Vec<[f32; 2]>,
    /// Vertex indices in Blitz's winding order (the file's order with the
    /// last two swapped).
    pub triangles: Vec<[u32; 3]>,
    pub frames: Vec<Md2Frame>,
}

/// One animation frame, already decompressed into Blitz's axes.
#[derive(Debug, Clone, PartialEq)]
pub struct Md2Frame {
    /// The 16-byte name field up to its first NUL. Blitz discards it.
    pub name: String,
    /// One position per model vertex.
    pub positions: Vec<[f32; 3]>,
    /// One unit normal per model vertex, from the `md2norms` table.
    pub normals: Vec<[f32; 3]>,
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(format!("md2: {}", msg.into()))
}

fn count(v: i32, what: &str) -> Result<usize> {
    usize::try_from(v).map_err(|_| invalid(format!("negative {what} ({v})")))
}

fn c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| char::from(b)).collect()
}

/// Parses an `.md2` file the way `LoadMD2` does.
pub fn parse(data: &[u8]) -> Result<Md2Model> {
    let mut r = BlitzReader::new(data);
    let magic = r.read_int()?;
    let version = r.read_int()?;
    if magic != MAGIC || version != VERSION {
        return Err(invalid(format!(
            "bad magic or version ({magic:#x}, {version})"
        )));
    }
    let skin_width = r.read_int()?;
    let skin_height = r.read_int()?;
    let _frame_size = r.read_int()?;
    let num_skins = count(r.read_int()?, "skin count")?;
    let num_vertices = count(r.read_int()?, "vertex count")?;
    let num_tex_coords = count(r.read_int()?, "texture coordinate count")?;
    let num_triangles = count(r.read_int()?, "triangle count")?;
    let _num_gl_commands = r.read_int()?;
    let num_frames = count(r.read_int()?, "frame count")?;
    let offset_skins = count(r.read_int()?, "skin offset")?;
    let offset_tex_coords = count(r.read_int()?, "texture coordinate offset")?;
    let offset_triangles = count(r.read_int()?, "triangle offset")?;
    let offset_frames = count(r.read_int()?, "frame offset")?;
    if num_frames == 0 {
        return Err(invalid("no frames"));
    }
    if skin_width <= 0 || skin_height <= 0 {
        return Err(invalid(format!("skin size {skin_width}x{skin_height}")));
    }

    let at = |offset: usize| -> Result<BlitzReader<'_>> {
        data.get(offset..)
            .map(BlitzReader::new)
            .ok_or(Error::UnexpectedEof { offset, wanted: 0 })
    };

    let mut skins = Vec::with_capacity(num_skins.min(64));
    let mut r = at(offset_skins)?;
    for _ in 0..num_skins {
        skins.push(c_string(r.read_bytes(64)?));
    }

    let mut r = at(offset_tex_coords)?;
    let mut st = Vec::with_capacity(num_tex_coords.min(data.len() / 4));
    for _ in 0..num_tex_coords {
        let s = r.read_short()? as i16;
        let t = r.read_short()? as i16;
        st.push((s, t));
    }

    let mut r = at(offset_triangles)?;
    // (vertex index, uv index) -> output vertex, numbered by first use.
    let mut vertex_ids: HashMap<(u16, u16), u32> = HashMap::new();
    let mut sources: Vec<u16> = Vec::new();
    let mut uvs = Vec::new();
    let mut triangles = Vec::with_capacity(num_triangles.min(data.len() / 12));
    for _ in 0..num_triangles {
        let mut verts = [0u16; 3];
        let mut tex = [0u16; 3];
        for v in &mut verts {
            *v = r.read_short()?;
        }
        for t in &mut tex {
            *t = r.read_short()?;
        }
        let mut ids = [0u32; 3];
        for j in 0..3 {
            if usize::from(verts[j]) >= num_vertices {
                return Err(invalid(format!("vertex index {} out of range", verts[j])));
            }
            let &(s, t) = st
                .get(usize::from(tex[j]))
                .ok_or_else(|| invalid(format!("texture coordinate {} out of range", tex[j])))?;
            ids[j] = *vertex_ids.entry((verts[j], tex[j])).or_insert_with(|| {
                sources.push(verts[j]);
                uvs.push([
                    f32::from(s) / skin_width as f32,
                    f32::from(t) / skin_height as f32,
                ]);
                (sources.len() - 1) as u32
            });
        }
        triangles.push([ids[0], ids[2], ids[1]]);
    }

    let mut r = at(offset_frames)?;
    let mut frames = Vec::with_capacity(num_frames.min(data.len() / 40));
    for _ in 0..num_frames {
        let scale = [r.read_float()?, r.read_float()?, r.read_float()?];
        let trans = [r.read_float()?, r.read_float()?, r.read_float()?];
        let name = c_string(r.read_bytes(16)?);
        let raw = r.read_bytes(num_vertices.checked_mul(4).ok_or_else(|| invalid("size"))?)?;
        // Quake (x, y, z) -> Blitz (y, z, x).
        let scale = [scale[1], scale[2], scale[0]];
        let trans = [trans[1], trans[2], trans[0]];
        let mut positions = Vec::with_capacity(sources.len());
        let mut normals = Vec::with_capacity(sources.len());
        for &src in &sources {
            let v = &raw[usize::from(src) * 4..][..4];
            let (x, y, z) = (f32::from(v[1]), f32::from(v[2]), f32::from(v[0]));
            positions.push([
                x * scale[0] + trans[0],
                y * scale[1] + trans[1],
                z * scale[2] + trans[2],
            ]);
            let n = NORMALS
                .get(usize::from(v[3]))
                .ok_or_else(|| invalid(format!("normal index {} out of range", v[3])))?;
            normals.push([n[1], n[2], n[0]]);
        }
        frames.push(Md2Frame {
            name,
            positions,
            normals,
        });
    }

    Ok(Md2Model {
        skin_size: [skin_width as u32, skin_height as u32],
        skins,
        uvs,
        triangles,
        frames,
    })
}

impl Md2Model {
    /// Number of vertices in every frame.
    pub fn vertex_count(&self) -> usize {
        self.uvs.len()
    }

    /// One frame as a static mesh (a single surface with the default
    /// material; Blitz leaves texturing to `EntityTexture`).
    ///
    /// # Panics
    ///
    /// If `frame` is out of range.
    pub fn frame_mesh(&self, frame: usize) -> Mesh {
        let f = &self.frames[frame];
        let vertices = (0..self.uvs.len())
            .map(|i| Vertex {
                position: f.positions[i],
                normal: f.normals[i],
                uv: self.uvs[i],
            })
            .collect();
        Mesh {
            surfaces: vec![Surface {
                vertices,
                triangles: self.triangles.clone(),
                ..Surface::default()
            }],
        }
    }
}

/// Blitz's `md2norms` table, in Quake axes (rotated to Blitz's on use).
#[rustfmt::skip]
static NORMALS: [[f32; 3]; 162] = [
    [-0.525731, 0.000000, 0.850651],
    [-0.442863, 0.238856, 0.864188],
    [-0.295242, 0.000000, 0.955423],
    [-0.309017, 0.500000, 0.809017],
    [-0.162460, 0.262866, 0.951056],
    [0.000000, 0.000000, 1.000000],
    [0.000000, 0.850651, 0.525731],
    [-0.147621, 0.716567, 0.681718],
    [0.147621, 0.716567, 0.681718],
    [0.000000, 0.525731, 0.850651],
    [0.309017, 0.500000, 0.809017],
    [0.525731, 0.000000, 0.850651],
    [0.295242, 0.000000, 0.955423],
    [0.442863, 0.238856, 0.864188],
    [0.162460, 0.262866, 0.951056],
    [-0.681718, 0.147621, 0.716567],
    [-0.809017, 0.309017, 0.500000],
    [-0.587785, 0.425325, 0.688191],
    [-0.850651, 0.525731, 0.000000],
    [-0.864188, 0.442863, 0.238856],
    [-0.716567, 0.681718, 0.147621],
    [-0.688191, 0.587785, 0.425325],
    [-0.500000, 0.809017, 0.309017],
    [-0.238856, 0.864188, 0.442863],
    [-0.425325, 0.688191, 0.587785],
    [-0.716567, 0.681718, -0.147621],
    [-0.500000, 0.809017, -0.309017],
    [-0.525731, 0.850651, 0.000000],
    [0.000000, 0.850651, -0.525731],
    [-0.238856, 0.864188, -0.442863],
    [0.000000, 0.955423, -0.295242],
    [-0.262866, 0.951056, -0.162460],
    [0.000000, 1.000000, 0.000000],
    [0.000000, 0.955423, 0.295242],
    [-0.262866, 0.951056, 0.162460],
    [0.238856, 0.864188, 0.442863],
    [0.262866, 0.951056, 0.162460],
    [0.500000, 0.809017, 0.309017],
    [0.238856, 0.864188, -0.442863],
    [0.262866, 0.951056, -0.162460],
    [0.500000, 0.809017, -0.309017],
    [0.850651, 0.525731, 0.000000],
    [0.716567, 0.681718, 0.147621],
    [0.716567, 0.681718, -0.147621],
    [0.525731, 0.850651, 0.000000],
    [0.425325, 0.688191, 0.587785],
    [0.864188, 0.442863, 0.238856],
    [0.688191, 0.587785, 0.425325],
    [0.809017, 0.309017, 0.500000],
    [0.681718, 0.147621, 0.716567],
    [0.587785, 0.425325, 0.688191],
    [0.955423, 0.295242, 0.000000],
    [1.000000, 0.000000, 0.000000],
    [0.951056, 0.162460, 0.262866],
    [0.850651, -0.525731, 0.000000],
    [0.955423, -0.295242, 0.000000],
    [0.864188, -0.442863, 0.238856],
    [0.951056, -0.162460, 0.262866],
    [0.809017, -0.309017, 0.500000],
    [0.681718, -0.147621, 0.716567],
    [0.850651, 0.000000, 0.525731],
    [0.864188, 0.442863, -0.238856],
    [0.809017, 0.309017, -0.500000],
    [0.951056, 0.162460, -0.262866],
    [0.525731, 0.000000, -0.850651],
    [0.681718, 0.147621, -0.716567],
    [0.681718, -0.147621, -0.716567],
    [0.850651, 0.000000, -0.525731],
    [0.809017, -0.309017, -0.500000],
    [0.864188, -0.442863, -0.238856],
    [0.951056, -0.162460, -0.262866],
    [0.147621, 0.716567, -0.681718],
    [0.309017, 0.500000, -0.809017],
    [0.425325, 0.688191, -0.587785],
    [0.442863, 0.238856, -0.864188],
    [0.587785, 0.425325, -0.688191],
    [0.688191, 0.587785, -0.425325],
    [-0.147621, 0.716567, -0.681718],
    [-0.309017, 0.500000, -0.809017],
    [0.000000, 0.525731, -0.850651],
    [-0.525731, 0.000000, -0.850651],
    [-0.442863, 0.238856, -0.864188],
    [-0.295242, 0.000000, -0.955423],
    [-0.162460, 0.262866, -0.951056],
    [0.000000, 0.000000, -1.000000],
    [0.295242, 0.000000, -0.955423],
    [0.162460, 0.262866, -0.951056],
    [-0.442863, -0.238856, -0.864188],
    [-0.309017, -0.500000, -0.809017],
    [-0.162460, -0.262866, -0.951056],
    [0.000000, -0.850651, -0.525731],
    [-0.147621, -0.716567, -0.681718],
    [0.147621, -0.716567, -0.681718],
    [0.000000, -0.525731, -0.850651],
    [0.309017, -0.500000, -0.809017],
    [0.442863, -0.238856, -0.864188],
    [0.162460, -0.262866, -0.951056],
    [0.238856, -0.864188, -0.442863],
    [0.500000, -0.809017, -0.309017],
    [0.425325, -0.688191, -0.587785],
    [0.716567, -0.681718, -0.147621],
    [0.688191, -0.587785, -0.425325],
    [0.587785, -0.425325, -0.688191],
    [0.000000, -0.955423, -0.295242],
    [0.000000, -1.000000, 0.000000],
    [0.262866, -0.951056, -0.162460],
    [0.000000, -0.850651, 0.525731],
    [0.000000, -0.955423, 0.295242],
    [0.238856, -0.864188, 0.442863],
    [0.262866, -0.951056, 0.162460],
    [0.500000, -0.809017, 0.309017],
    [0.716567, -0.681718, 0.147621],
    [0.525731, -0.850651, 0.000000],
    [-0.238856, -0.864188, -0.442863],
    [-0.500000, -0.809017, -0.309017],
    [-0.262866, -0.951056, -0.162460],
    [-0.850651, -0.525731, 0.000000],
    [-0.716567, -0.681718, -0.147621],
    [-0.716567, -0.681718, 0.147621],
    [-0.525731, -0.850651, 0.000000],
    [-0.500000, -0.809017, 0.309017],
    [-0.238856, -0.864188, 0.442863],
    [-0.262866, -0.951056, 0.162460],
    [-0.864188, -0.442863, 0.238856],
    [-0.809017, -0.309017, 0.500000],
    [-0.688191, -0.587785, 0.425325],
    [-0.681718, -0.147621, 0.716567],
    [-0.442863, -0.238856, 0.864188],
    [-0.587785, -0.425325, 0.688191],
    [-0.309017, -0.500000, 0.809017],
    [-0.147621, -0.716567, 0.681718],
    [-0.425325, -0.688191, 0.587785],
    [-0.162460, -0.262866, 0.951056],
    [0.442863, -0.238856, 0.864188],
    [0.162460, -0.262866, 0.951056],
    [0.309017, -0.500000, 0.809017],
    [0.147621, -0.716567, 0.681718],
    [0.000000, -0.525731, 0.850651],
    [0.425325, -0.688191, 0.587785],
    [0.587785, -0.425325, 0.688191],
    [0.688191, -0.587785, 0.425325],
    [-0.955423, 0.295242, 0.000000],
    [-0.951056, 0.162460, 0.262866],
    [-1.000000, 0.000000, 0.000000],
    [-0.850651, 0.000000, 0.525731],
    [-0.955423, -0.295242, 0.000000],
    [-0.951056, -0.162460, 0.262866],
    [-0.864188, 0.442863, -0.238856],
    [-0.951056, 0.162460, -0.262866],
    [-0.809017, 0.309017, -0.500000],
    [-0.864188, -0.442863, -0.238856],
    [-0.951056, -0.162460, -0.262866],
    [-0.809017, -0.309017, -0.500000],
    [-0.681718, 0.147621, -0.716567],
    [-0.681718, -0.147621, -0.716567],
    [-0.850651, 0.000000, -0.525731],
    [-0.688191, 0.587785, -0.425325],
    [-0.587785, 0.425325, -0.688191],
    [-0.425325, 0.688191, -0.587785],
    [-0.425325, -0.688191, -0.587785],
    [-0.587785, -0.425325, -0.688191],
    [-0.688191, -0.587785, -0.425325],
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blitz::BlitzWriter;

    /// A model with a quad of 4 file vertices, 3 uvs and 2 triangles that
    /// share vertex 1 with different uvs, plus two frames.
    fn sample() -> Vec<u8> {
        let mut w = BlitzWriter::new();
        let header = 68;
        let (skins, st, tris, frames) =
            (header, header + 64, header + 64 + 12, header + 64 + 12 + 24);
        for v in [MAGIC, VERSION, 64, 32, 0, 1, 4, 3, 2, 0, 2] {
            w.write_int(v);
        }
        for v in [skins, st, tris, frames, 0, 0] {
            w.write_int(v);
        }
        assert_eq!(w.clone().into_bytes().len(), 68);
        let mut skin = b"skins\\a.pcx".to_vec();
        skin.resize(64, 0);
        w.write_bytes(&skin);
        for (s, t) in [(0i16, 0i16), (64, 0), (32, 16)] {
            w.write_short(s as u16);
            w.write_short(t as u16);
        }
        // Triangle (0,1,2) with uvs (0,1,2); triangle (1,2,3) with uvs (2,1,0).
        for v in [0u16, 1, 2, 0, 1, 2, 1, 2, 3, 2, 1, 0] {
            w.write_short(v);
        }
        for (name, scale, trans) in [
            ("stand01", [1.0f32, 2.0, 3.0], [10.0f32, 20.0, 30.0]),
            ("stand02", [0.5, 0.5, 0.5], [0.0, 0.0, 0.0]),
        ] {
            for v in scale.into_iter().chain(trans) {
                w.write_float(v);
            }
            let mut n = name.as_bytes().to_vec();
            n.resize(16, 0);
            w.write_bytes(&n);
            for k in 0..4u8 {
                // quake x, y, z, normal index
                w.write_bytes(&[k, k + 10, k + 20, k]);
            }
        }
        w.into_bytes()
    }

    #[test]
    fn parses_header_names_and_sharing() {
        let m = parse(&sample()).unwrap();
        assert_eq!(m.skins, ["skins\\a.pcx"]);
        assert_eq!(m.skin_size, [64, 32]);
        assert_eq!(m.frames[0].name, "stand01");
        assert_eq!(m.frames[1].name, "stand02");
        // (0,0) (1,1) (2,2) (1,2) (2,1)? pairs: t0 -> (0,0),(1,1),(2,2); t1 -> (1,2),(2,1),(3,0)
        assert_eq!(m.vertex_count(), 6);
        // Winding: second and third swapped.
        assert_eq!(m.triangles, [[0, 2, 1], [3, 5, 4]]);
        assert_eq!(m.uvs[0], [0.0, 0.0]);
        assert_eq!(m.uvs[1], [1.0, 0.0]);
        assert_eq!(m.uvs[2], [0.5, 0.5]);
        // Vertex 3 is file vertex 1 with uv 2.
        assert_eq!(m.uvs[3], [0.5, 0.5]);
    }

    #[test]
    fn decompresses_in_blitz_axes() {
        let m = parse(&sample()).unwrap();
        // File vertex 1 has quake bytes (1, 11, 21). Scale (1,2,3) and
        // translate (10,20,30) rotate to (2,3,1) and (20,30,10).
        // Blitz = (11*2+20, 21*3+30, 1*1+10).
        assert_eq!(m.frames[0].positions[1], [42.0, 93.0, 11.0]);
        assert_eq!(m.frames[1].positions[1], [5.5, 10.5, 0.5]);
    }

    #[test]
    fn normals_come_from_the_table_rotated() {
        let m = parse(&sample()).unwrap();
        // Normal index 1 is (-0.442863, 0.238856, 0.864188) in Quake axes.
        assert_eq!(m.frames[0].normals[1], [0.238_856, 0.864_188, -0.442_863]);
        for f in &m.frames {
            for n in &f.normals {
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                assert!((len - 1.0).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn frame_mesh_has_one_surface() {
        let m = parse(&sample()).unwrap();
        let mesh = m.frame_mesh(1);
        assert_eq!(mesh.surfaces.len(), 1);
        assert_eq!(mesh.vertex_count(), 6);
        assert_eq!(mesh.triangle_count(), 2);
        assert_eq!(mesh.surfaces[0].vertices[1].position, [5.5, 10.5, 0.5]);
        assert_eq!(mesh.surfaces[0].vertices[1].uv, [1.0, 0.0]);
    }

    #[test]
    fn rejects_bad_input() {
        let mut d = sample();
        assert!(matches!(parse(&d[..100]), Err(Error::UnexpectedEof { .. })));
        d[0] = b'X';
        assert!(matches!(parse(&d), Err(Error::Invalid(_))));
        let mut d = sample();
        d[4] = 7; // version
        assert!(matches!(parse(&d), Err(Error::Invalid(_))));
    }
}
