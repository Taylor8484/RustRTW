//! `.x` meshes (DirectX text format), as Blitz3D's `LoadMesh` holds them.
//!
//! Ported from `blitz3d/loader_x.cpp` (`parseFile`, `parseFrame`,
//! `parseMesh`, `parseMaterialList`, `parseMaterial`), `blitz3d/meshloader.cpp`
//! (`addTriangle`, `endMesh`), `MeshModel::Rep::transform` / `add` in
//! `blitz3d/meshmodel.cpp`, `Surface::updateNormals` in `blitz3d/surface.cpp`,
//! and `bbLoadMesh` / `collapseMesh` in `bbruntime/bbblitz3d.cpp` of
//! <https://github.com/blitz-research/blitz3d>. That is the `master` branch;
//! it was not compared against the v1.83 release the games shipped with.
//! Blitz reads files through Microsoft's DirectXFile API; this module
//! replaces it with a small reader for the text format.
//!
//! Conventions (the `.x` conversion matrix in `bbblitz3d.cpp` is the
//! identity, so nothing is converted): DirectX and Blitz are both
//! left-handed and y-up, so positions, normals, UVs (`v` not flipped) and
//! winding are used as written.
//!
//! What is read, and how:
//!
//! - **Frame / FrameTransformMatrix / Mesh.** `LoadMesh` collapses the
//!   hierarchy: every frame's mesh is transformed by the frame's world
//!   transform (vertices by the matrix, normals by its cofactor matrix, not
//!   renormalised) and appended, children before their parent's own mesh.
//!   Matrix rows are the images of the x, y and z axes and row 4 is the
//!   translation. Meshes outside any frame sit at the root. Animation sets
//!   are skipped (`collapse`).
//! - **Polygons.** A face with `n >= 3` indices becomes the fan
//!   `(i0, i[j-1], i[j])` for `j` in `2..n`. Faces with fewer indices are
//!   dropped.
//! - **MeshMaterialList / Material / TextureFilename.** The face's material
//!   index selects a brush: diffuse colour, alpha (only if non-zero, else 1)
//!   and, if a `TextureFilename` child exists, the texture, which also resets
//!   the colour to white. Materials may be referenced by `{ Name }`. With no
//!   list every face uses a default brush. Blitz makes one surface per
//!   distinct brush (colour, alpha, texture) and copies into it only the
//!   vertices its triangles use, numbered by first use. Blitz orders
//!   surfaces by a byte comparison of internal render state; here they are
//!   in order of first use.
//! - **MeshNormals.** Used only when there is one normal per vertex
//!   (the face index part is ignored), normalised. Otherwise normals are
//!   recomputed per surface like `Surface::updateNormals`: each triangle's
//!   unit face normal `(v1 - v0) x (v2 - v0)` (skipped when its length is at
//!   most 1e-6) is summed on every vertex at the same position, and the sum
//!   normalised. Blitz compares positions with a 1e-6 tolerance; this uses
//!   exact equality. A vertex with no contributing triangle gets a zero
//!   normal where Blitz would produce NaN.
//! - **MeshTextureCoords.** Used only when there is one per vertex.
//! - **Ignored.** `MeshVertexColors`, specular and emissive material
//!   values, the shininess, and any template definitions in the file.
//!
//! Binary and compressed `.x` files (which DirectX accepts) fail with
//! [`Error::Invalid`].

use std::collections::HashMap;

use crate::mesh::{Material, Mesh, Surface, Vertex};
use crate::{Error, Result};

const MAX_DEPTH: usize = 64;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(format!("x: {}", msg.into()))
}

/// Parses a text `.x` file the way `LoadMesh` does.
pub fn parse(data: &[u8]) -> Result<Mesh> {
    if data.len() < 16 || &data[..4] != b"xof " {
        return Err(invalid("missing `xof` header"));
    }
    match &data[8..12] {
        b"txt " => {}
        other => {
            return Err(invalid(format!(
                "only the text format is supported, found {:?}",
                String::from_utf8_lossy(other)
            )));
        }
    }
    let mut p = Parser {
        data,
        pos: 16,
        materials: HashMap::new(),
    };
    let top = p.parse_objects(None, 0)?;

    let mut root = Node::default();
    for item in &top {
        match item {
            Child::Object(o) if o.ty == "Mesh" => root.add_mesh(o, &p.materials)?,
            Child::Object(o) if o.ty == "Frame" => {
                let node = p.frame(o)?;
                root.children.push(node);
            }
            _ => {}
        }
    }

    let mut out = Collapsed::default();
    root.collapse(&Xform::IDENTITY, &mut out);
    Ok(Mesh {
        surfaces: out.surfaces,
    })
}

// ---- tokens and object tree -------------------------------------------

#[derive(Debug, Clone)]
enum Value {
    Num { f: f32, i: i64 },
    Str(String),
}

#[derive(Debug, Clone)]
enum Child {
    Object(Object),
    Reference(String),
}

#[derive(Debug, Clone, Default)]
struct Object {
    ty: String,
    name: String,
    values: Vec<Value>,
    children: Vec<Child>,
}

impl Object {
    fn float(&self, i: usize) -> Result<f32> {
        match self.values.get(i) {
            Some(Value::Num { f, .. }) => Ok(*f),
            _ => Err(invalid(format!("{}: missing number {i}", self.ty))),
        }
    }

    fn int(&self, i: usize) -> Result<i64> {
        match self.values.get(i) {
            Some(Value::Num { i, .. }) => Ok(*i),
            _ => Err(invalid(format!("{}: missing integer {i}", self.ty))),
        }
    }

    /// A non-negative integer used as a count or index.
    fn index(&self, i: usize) -> Result<usize> {
        usize::try_from(self.int(i)?)
            .map_err(|_| invalid(format!("{}: negative count or index", self.ty)))
    }
}

#[derive(Debug, PartialEq)]
enum Tok {
    Open,
    Close,
    Semi,
    Comma,
    Guid,
    Word(String),
    Number(f32, i64),
    Str(String),
    End,
}

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
    /// Named `Material` objects, for `{ Name }` references.
    materials: HashMap<String, Object>,
}

fn is_delim(b: u8) -> bool {
    b.is_ascii_whitespace()
        || matches!(
            b,
            b'{' | b'}' | b'[' | b']' | b'(' | b')' | b'<' | b'>' | b';' | b',' | b'"'
        )
}

impl Parser<'_> {
    fn next(&mut self) -> Result<Tok> {
        let d = self.data;
        loop {
            while self.pos < d.len() && d[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            let Some(&b) = d.get(self.pos) else {
                return Ok(Tok::End);
            };
            if b == b'#' || (b == b'/' && d.get(self.pos + 1) == Some(&b'/')) {
                while self.pos < d.len() && d[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
        let b = d[self.pos];
        let single = match b {
            b'{' => Some(Tok::Open),
            b'}' => Some(Tok::Close),
            b';' => Some(Tok::Semi),
            b',' => Some(Tok::Comma),
            _ => None,
        };
        if let Some(t) = single {
            self.pos += 1;
            return Ok(t);
        }
        match b {
            b'<' => {
                while self.pos < d.len() && d[self.pos] != b'>' {
                    self.pos += 1;
                }
                if self.pos >= d.len() {
                    return Err(invalid("unterminated `<`"));
                }
                self.pos += 1;
                Ok(Tok::Guid)
            }
            b'"' => {
                let start = self.pos + 1;
                let len = d[start..]
                    .iter()
                    .position(|&c| c == b'"')
                    .ok_or_else(|| invalid("unterminated string"))?;
                self.pos = start + len + 1;
                Ok(Tok::Str(
                    d[start..start + len]
                        .iter()
                        .map(|&c| char::from(c))
                        .collect(),
                ))
            }
            b'[' | b']' | b'(' | b')' | b'>' => Err(invalid(format!(
                "unexpected `{}` at byte {}",
                char::from(b),
                self.pos
            ))),
            _ => {
                let start = self.pos;
                while self.pos < d.len() && !is_delim(d[self.pos]) {
                    self.pos += 1;
                }
                let word: String = d[start..self.pos].iter().map(|&c| char::from(c)).collect();
                let numeric = word
                    .bytes()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.'));
                if numeric && let Ok(f) = word.parse::<f32>() {
                    let i = word.parse::<i64>().unwrap_or(f as i64);
                    return Ok(Tok::Number(f, i));
                }
                Ok(Tok::Word(word))
            }
        }
    }

    /// Parses objects until `}` (inside an object) or the end of the data.
    /// Returns the data values separately for objects (`owner`).
    fn parse_objects(
        &mut self,
        mut owner: Option<&mut Object>,
        depth: usize,
    ) -> Result<Vec<Child>> {
        if depth > MAX_DEPTH {
            return Err(invalid("objects nested too deeply"));
        }
        let mut children = Vec::new();
        loop {
            match self.next()? {
                Tok::End => {
                    if owner.is_some() {
                        return Err(invalid("unexpected end of file inside an object"));
                    }
                    return Ok(children);
                }
                Tok::Close => {
                    if owner.is_none() {
                        return Err(invalid("unmatched `}`"));
                    }
                    return Ok(children);
                }
                Tok::Semi | Tok::Comma | Tok::Guid => {}
                Tok::Number(f, i) => match owner.as_deref_mut() {
                    Some(o) => o.values.push(Value::Num { f, i }),
                    None => return Err(invalid("stray number at top level")),
                },
                Tok::Str(s) => match owner.as_deref_mut() {
                    Some(o) => o.values.push(Value::Str(s)),
                    None => return Err(invalid("stray string at top level")),
                },
                Tok::Open => {
                    // `{ Name }` reference.
                    let name = match self.next()? {
                        Tok::Word(w) => w,
                        _ => return Err(invalid("malformed reference")),
                    };
                    loop {
                        match self.next()? {
                            Tok::Close => break,
                            Tok::Guid => {}
                            _ => return Err(invalid("malformed reference")),
                        }
                    }
                    children.push(Child::Reference(name));
                }
                Tok::Word(ty) => {
                    let mut name = String::new();
                    let mut t = self.next()?;
                    if let Tok::Word(w) = t {
                        name = w;
                        t = self.next()?;
                    }
                    if t == Tok::Guid {
                        t = self.next()?;
                    }
                    if t != Tok::Open {
                        return Err(invalid(format!("expected `{{` after `{ty}`")));
                    }
                    if ty == "template" {
                        self.skip_block()?;
                        continue;
                    }
                    let mut obj = Object {
                        ty,
                        name,
                        ..Object::default()
                    };
                    obj.children = self.parse_objects(Some(&mut obj), depth + 1)?;
                    if obj.ty == "Material" && !obj.name.is_empty() {
                        self.materials.insert(obj.name.clone(), obj.clone());
                    }
                    children.push(Child::Object(obj));
                }
            }
        }
    }

    fn skip_block(&mut self) -> Result<()> {
        let mut depth = 1;
        while depth > 0 {
            match self.next()? {
                Tok::Open => depth += 1,
                Tok::Close => depth -= 1,
                Tok::End => return Err(invalid("unterminated template")),
                _ => {}
            }
        }
        Ok(())
    }

    fn frame(&self, o: &Object) -> Result<Node> {
        self.frame_at(o, 0)
    }

    fn frame_at(&self, o: &Object, depth: usize) -> Result<Node> {
        if depth > MAX_DEPTH {
            return Err(invalid("frames nested too deeply"));
        }
        let mut node = Node::default();
        for c in &o.children {
            let Child::Object(c) = c else { continue };
            match c.ty.as_str() {
                "FrameTransformMatrix" => {
                    let m = |i| c.float(i);
                    node.local = Xform {
                        m: [
                            [m(0)?, m(1)?, m(2)?],
                            [m(4)?, m(5)?, m(6)?],
                            [m(8)?, m(9)?, m(10)?],
                        ],
                        t: [m(12)?, m(13)?, m(14)?],
                    };
                }
                "Mesh" => node.add_mesh(c, &self.materials)?,
                "Frame" => node.children.push(self.frame_at(c, depth + 1)?),
                _ => {}
            }
        }
        Ok(node)
    }
}

// ---- meshes -----------------------------------------------------------

/// `m[r]` is the image of axis `r`, as in a DirectX row matrix; a point
/// maps to `x * m[0] + y * m[1] + z * m[2] + t`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Xform {
    m: [[f32; 3]; 3],
    t: [f32; 3],
}

impl Xform {
    const IDENTITY: Xform = Xform {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        t: [0.0; 3],
    };

    fn dir(&self, v: [f32; 3]) -> [f32; 3] {
        let m = &self.m;
        std::array::from_fn(|c| v[0] * m[0][c] + v[1] * m[1][c] + v[2] * m[2][c])
    }

    fn point(&self, v: [f32; 3]) -> [f32; 3] {
        let d = self.dir(v);
        std::array::from_fn(|c| d[c] + self.t[c])
    }

    /// Blitz's `Matrix::cofactor` applied to a normal.
    fn normal(&self, n: [f32; 3]) -> [f32; 3] {
        // Blitz's matrix columns i, j, k are the rows of `m`.
        let (i, j, k) = (self.m[0], self.m[1], self.m[2]);
        let co_i = [
            j[1] * k[2] - j[2] * k[1],
            -(j[0] * k[2] - j[2] * k[0]),
            j[0] * k[1] - j[1] * k[0],
        ];
        let co_j = [
            -(i[1] * k[2] - i[2] * k[1]),
            i[0] * k[2] - i[2] * k[0],
            -(i[0] * k[1] - i[1] * k[0]),
        ];
        let co_k = [
            i[1] * j[2] - i[2] * j[1],
            -(i[0] * j[2] - i[2] * j[0]),
            i[0] * j[1] - i[1] * j[0],
        ];
        std::array::from_fn(|c| n[0] * co_i[c] + n[1] * co_j[c] + n[2] * co_k[c])
    }

    /// `self * inner`: apply `inner` first, then `self`.
    fn compose(&self, inner: &Xform) -> Xform {
        Xform {
            m: inner.m.map(|row| self.dir(row)),
            t: self.point(inner.t),
        }
    }
}

impl Default for Xform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Default)]
struct Node {
    local: Xform,
    surfaces: Vec<Surface>,
    children: Vec<Node>,
}

fn same_brush(a: &Material, b: &Material) -> bool {
    a.color == b.color
        && a.alpha == b.alpha
        && a.texture.as_ref().map(|t| t.to_lowercase())
            == b.texture.as_ref().map(|t| t.to_lowercase())
}

fn material(o: &Object) -> Result<Material> {
    let mut m = Material {
        name: o.name.clone(),
        color: [o.float(0)?, o.float(1)?, o.float(2)?],
        ..Material::default()
    };
    let a = o.float(3)?;
    if a != 0.0 {
        m.alpha = a;
    }
    for c in &o.children {
        if let Child::Object(c) = c
            && c.ty == "TextureFilename"
            && let Some(Value::Str(s)) = c.values.first()
        {
            m.texture = Some(s.clone());
            m.color = [1.0; 3];
        }
    }
    Ok(m)
}

impl Node {
    #[allow(clippy::collapsible_match)] // the `?` in the conditions cannot move into guards
    /// `parseMesh` followed by `MeshLoader::endMesh`.
    fn add_mesh(&mut self, o: &Object, named: &HashMap<String, Object>) -> Result<()> {
        let num_verts = o.index(0)?;
        let mut at = 1;
        let mut verts: Vec<Vertex> = Vec::new();
        for _ in 0..num_verts {
            verts.push(Vertex {
                position: [o.float(at)?, o.float(at + 1)?, o.float(at + 2)?],
                ..Vertex::default()
            });
            at += 3;
        }
        let num_faces = o.index(at)?;
        at += 1;
        // (start of the indices, count)
        let mut faces: Vec<(usize, usize)> = Vec::new();
        for _ in 0..num_faces {
            let n = o.index(at)?;
            faces.push((at + 1, n));
            at += n + 1;
        }
        // Check indices up front.
        for &(start, n) in &faces {
            for k in 0..n {
                if o.index(start + k)? >= num_verts {
                    return Err(invalid("face vertex index out of range"));
                }
            }
        }

        let mut mats: Vec<Material> = Vec::new();
        let mut face_mat = vec![0usize; faces.len()];
        let mut have_normals = false;
        for c in &o.children {
            let Child::Object(c) = c else { continue };
            match c.ty.as_str() {
                "MeshMaterialList" => {
                    let listed = c.index(1)?;
                    if listed > faces.len() {
                        return Err(invalid("material list longer than the face list"));
                    }
                    for (k, slot) in face_mat.iter_mut().enumerate().take(listed) {
                        *slot = c.index(2 + k)?;
                    }
                    for mc in &c.children {
                        let m = match mc {
                            Child::Object(m) if m.ty == "Material" => Some(m),
                            Child::Reference(name) => {
                                named.get(name).filter(|m| m.ty == "Material")
                            }
                            _ => None,
                        };
                        if let Some(m) = m {
                            mats.push(material(m)?);
                        }
                    }
                }
                "MeshTextureCoords" => {
                    if c.index(0)? == num_verts {
                        for (k, v) in verts.iter_mut().enumerate() {
                            v.uv = [c.float(1 + 2 * k)?, c.float(2 + 2 * k)?];
                        }
                    }
                }
                "MeshNormals" => {
                    if c.index(0)? == num_verts {
                        for (k, v) in verts.iter_mut().enumerate() {
                            let n = [
                                c.float(1 + 3 * k)?,
                                c.float(2 + 3 * k)?,
                                c.float(3 + 3 * k)?,
                            ];
                            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                            v.normal = if len > 0.0 { n.map(|x| x / len) } else { n };
                        }
                        have_normals = true;
                    }
                }
                _ => {}
            }
        }
        if mats.is_empty() {
            mats.push(Material::default());
        }

        // Group triangles by brush in order of first use.
        struct Group {
            material: Material,
            tris: Vec<[usize; 3]>,
        }
        let mut groups: Vec<Group> = Vec::new();
        for (f, &(start, n)) in faces.iter().enumerate() {
            if n < 3 {
                continue;
            }
            let material = mats
                .get(face_mat[f])
                .ok_or_else(|| invalid("material index out of range"))?;
            let g = match groups
                .iter()
                .position(|g| same_brush(&g.material, material))
            {
                Some(g) => g,
                None => {
                    groups.push(Group {
                        material: material.clone(),
                        tris: Vec::new(),
                    });
                    groups.len() - 1
                }
            };
            let first = o.index(start)?;
            for j in 2..n {
                groups[g]
                    .tris
                    .push([first, o.index(start + j - 1)?, o.index(start + j)?]);
            }
        }

        for g in groups {
            let at = self
                .surfaces
                .iter()
                .position(|s| same_brush(&s.material, &g.material));
            let at = at.unwrap_or_else(|| {
                self.surfaces.push(Surface {
                    material: g.material.clone(),
                    ..Surface::default()
                });
                self.surfaces.len() - 1
            });
            let surf = &mut self.surfaces[at];
            let mut remap: HashMap<usize, u32> = HashMap::new();
            for tri in g.tris {
                let mut out = [0u32; 3];
                for (slot, &v) in out.iter_mut().zip(&tri) {
                    *slot = *remap.entry(v).or_insert_with(|| {
                        surf.vertices.push(verts[v]);
                        (surf.vertices.len() - 1) as u32
                    });
                }
                surf.triangles.push(out);
            }
        }

        if !have_normals {
            for s in &mut self.surfaces {
                update_normals(s);
            }
        }
        Ok(())
    }

    /// `collapseMesh`: children first, then this node's own surfaces.
    fn collapse(&self, parent: &Xform, out: &mut Collapsed) {
        let world = parent.compose(&self.local);
        for c in &self.children {
            c.collapse(&world, out);
        }
        for s in &self.surfaces {
            let dest = match out
                .surfaces
                .iter()
                .position(|d| same_brush(&d.material, &s.material))
            {
                Some(d) => d,
                None => {
                    out.surfaces.push(Surface {
                        material: s.material.clone(),
                        ..Surface::default()
                    });
                    out.surfaces.len() - 1
                }
            };
            let dest = &mut out.surfaces[dest];
            let base = dest.vertices.len() as u32;
            dest.triangles
                .extend(s.triangles.iter().map(|t| t.map(|i| i + base)));
            dest.vertices.extend(s.vertices.iter().map(|v| Vertex {
                position: world.point(v.position),
                normal: world.normal(v.normal),
                uv: v.uv,
            }));
        }
    }
}

#[derive(Default)]
struct Collapsed {
    surfaces: Vec<Surface>,
}

/// `Surface::updateNormals`.
fn update_normals(s: &mut Surface) {
    // Positions as bit patterns, with -0.0 folded into 0.0.
    let key = |p: [f32; 3]| p.map(|x| (x + 0.0).to_bits());
    let mut sums: HashMap<[u32; 3], [f32; 3]> = HashMap::new();
    for t in &s.triangles {
        let p = t.map(|i| s.vertices[i as usize].position);
        let a: [f32; 3] = std::array::from_fn(|c| p[1][c] - p[0][c]);
        let b: [f32; 3] = std::array::from_fn(|c| p[2][c] - p[0][c]);
        let n = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len <= 0.000_001 {
            continue;
        }
        let n = n.map(|x| x / len);
        for q in p {
            let e = sums.entry(key(q)).or_insert([0.0; 3]);
            for c in 0..3 {
                e[c] += n[c];
            }
        }
    }
    for v in &mut s.vertices {
        let n = sums.get(&key(v.position)).copied().unwrap_or([0.0; 3]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        v.normal = if len > 0.0 {
            n.map(|x| x / len)
        } else {
            [0.0; 3]
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x(body: &str) -> Vec<u8> {
        format!("xof 0302txt 0064\nHeader {{ 1; 0; 1; }}\n{body}").into_bytes()
    }

    const QUAD: &str = "Mesh m {
        4; 0;0;0;, 1;0;0;, 1;1;0;, 0;1;0;;
        1; 4;0,1,2,3;;
        MeshMaterialList { 1; 1; 0;; Material { 0.5;0.25;0.125;0.0;; 10.0; 1;1;1;; 0;0;0;;
            TextureFilename { \"a.bmp\"; } } }
        MeshTextureCoords { 4; 0;0;, 1;0;, 1;1;, 0;1;; }
    }";

    #[test]
    fn polygon_becomes_a_fan_with_unflipped_winding() {
        let m = parse(&x(QUAD)).unwrap();
        assert_eq!(m.surfaces.len(), 1);
        let s = &m.surfaces[0];
        assert_eq!(s.triangles, [[0, 1, 2], [0, 2, 3]]);
        assert_eq!(s.vertices[2].position, [1.0, 1.0, 0.0]);
        // uv is used as written, v not flipped.
        assert_eq!(s.vertices[3].uv, [0.0, 1.0]);
    }

    #[test]
    fn material_texture_resets_colour_and_zero_alpha_is_opaque() {
        let m = parse(&x(QUAD)).unwrap();
        let mat = &m.surfaces[0].material;
        assert_eq!(mat.texture.as_deref(), Some("a.bmp"));
        assert_eq!(mat.color, [1.0; 3]);
        assert_eq!(mat.alpha, 1.0);
    }

    #[test]
    fn normals_are_computed_when_absent() {
        // (1,0,0)x(0,1,0) = +z for the triangle 0,1,2.
        let m = parse(&x(QUAD)).unwrap();
        for v in &m.surfaces[0].vertices {
            assert_eq!(v.normal, [0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn file_normals_are_used_and_normalised() {
        let body = "Mesh { 3; 0;0;0;, 1;0;0;, 0;1;0;; 1; 3;0,1,2;;
            MeshNormals { 3; 0;0;2;, 0;0;2;, 0;0;2;; 1; 3;0,1,2;; } }";
        let m = parse(&x(body)).unwrap();
        assert_eq!(m.surfaces[0].vertices[0].normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn frame_transforms_are_applied_and_children_come_first() {
        let body = "Frame outer {
            FrameTransformMatrix { 0,1,0,0, -1,0,0,0, 0,0,1,0, 10,0,0,1;; }
            Mesh { 3; 1;0;0;, 0;0;0;, 0;1;0;; 1; 3;0,1,2;; }
            Frame inner {
                FrameTransformMatrix { 1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,5,1;; }
                Mesh { 3; 0;0;0;, 1;0;0;, 0;1;0;; 1; 3;0,1,2;; }
            }
        }";
        let m = parse(&x(body)).unwrap();
        // Same default brush: one surface, the child's triangle first.
        assert_eq!(m.surfaces.len(), 1);
        let v = &m.surfaces[0].vertices;
        assert_eq!(v.len(), 6);
        // Inner (0,0,0) is at z=5 in outer space; outer maps (x,y,z) to
        // x*(0,1,0) + y*(-1,0,0) + z*(0,0,1) + (10,0,0).
        assert_eq!(v[0].position, [10.0, 0.0, 5.0]);
        assert_eq!(v[1].position, [10.0, 1.0, 5.0]);
        assert_eq!(v[2].position, [9.0, 0.0, 5.0]);
        // The outer mesh's own (1,0,0) is at (10,1,0).
        assert_eq!(v[3].position, [10.0, 1.0, 0.0]);
        assert_eq!(m.surfaces[0].triangles, [[0, 1, 2], [3, 4, 5]]);
    }

    #[test]
    fn distinct_materials_make_distinct_surfaces_with_shared_vertices_split() {
        let body = "Mesh { 4; 0;0;0;, 1;0;0;, 0;1;0;, 1;1;0;; 2; 3;0,1,2;, 3;2,1,3;;
            MeshMaterialList { 2; 2; 0, 1;;
                Material a { 1;0;0;1;; 0; 0;0;0;; 0;0;0;; }
                Material b { 0;1;0;1;; 0; 0;0;0;; 0;0;0;; } } }";
        let m = parse(&x(body)).unwrap();
        assert_eq!(m.surfaces.len(), 2);
        assert_eq!(m.surfaces[0].material.name, "a");
        assert_eq!(m.surfaces[1].material.color, [0.0, 1.0, 0.0]);
        // Vertices numbered by first use within each surface.
        assert_eq!(m.surfaces[1].triangles, [[0, 1, 2]]);
        assert_eq!(m.surfaces[1].vertices[0].position, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn rejects_binary_and_bad_indices() {
        let mut bin = b"xof 0302bin 0064".to_vec();
        bin.extend([0; 8]);
        assert!(matches!(parse(&bin), Err(Error::Invalid(m)) if m.contains("text")));
        assert!(parse(&x("Mesh { 1; 0;0;0;; 1; 3;0,1,2;; }")).is_err());
        assert!(parse(b"not an x file at all").is_err());
    }

    #[test]
    fn skips_comments_templates_and_cpp_style_comments() {
        let body =
            "// hi\n# also\ntemplate Foo { <00000000-0000-0000-0000-000000000000> DWORD a; }\n"
                .to_string()
                + QUAD;
        assert_eq!(parse(&x(&body)).unwrap().triangle_count(), 2);
    }
}
