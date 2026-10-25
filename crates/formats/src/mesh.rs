//! Meshes: `.3ds` and `.x` static meshes and `.md2` animated models.
//!
//! The games load every static mesh with Blitz's `LoadMesh` (never
//! `LoadAnimMesh`), which collapses a file's hierarchy into one mesh made of
//! surfaces, one per material. `.md2` files go through `LoadMD2` and are
//! animated by frame with `AnimateMD2`.
//!
//! The parsers produce the mesh as Blitz holds it after loading: Blitz's
//! left-handed, y-up coordinates, its vertex order and winding, and its
//! normals. Converting to the renderer's coordinate system is the game
//! crate's job, so code ported from Blitz can keep using Blitz coordinates.
//! The format-specific parsers live in the submodules.

pub mod md2;
pub mod tds;
pub mod x;

/// A static mesh: what `LoadMesh` returns.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub surfaces: Vec<Surface>,
}

/// One Blitz surface: vertices and triangles sharing a brush.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Surface {
    pub vertices: Vec<Vertex>,
    /// Indices into `vertices`, in Blitz's winding order.
    pub triangles: Vec<[u32; 3]>,
    pub material: Material,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    /// Texture coordinates as Blitz stores them (`u`, `v`; v grows downward).
    pub uv: [f32; 2],
}

/// The brush a surface is drawn with.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    /// Diffuse colour, 0.0–1.0.
    pub color: [f32; 3],
    /// 1.0 is opaque.
    pub alpha: f32,
    /// The texture file name as written in the mesh file. Blitz looks for
    /// it next to the mesh; resolving it is up to the caller.
    pub texture: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::new(),
            color: [1.0; 3],
            alpha: 1.0,
            texture: None,
        }
    }
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        self.surfaces.iter().map(|s| s.vertices.len()).sum()
    }

    pub fn triangle_count(&self) -> usize {
        self.surfaces.iter().map(|s| s.triangles.len()).sum()
    }

    /// The axis-aligned bounds `(min, max)` of every vertex, or `None` for an
    /// empty mesh.
    pub fn bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        let mut it = self
            .surfaces
            .iter()
            .flat_map(|s| s.vertices.iter().map(|v| v.position));
        let first = it.next()?;
        Some(it.fold((first, first), |(mut lo, mut hi), p| {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
            (lo, hi)
        }))
    }
}
