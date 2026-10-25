//! Checks every `.x` in the real game packs (`assets/<GAME>`, gitignored;
//! games whose folder is missing are skipped).

use std::path::Path;

use wonderland_formats::GameAssets;
use wonderland_formats::mesh::x;

fn open_game(game: &str) -> Option<GameAssets> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets")
        .join(game);
    if !dir.is_dir() {
        eprintln!("skipping {game}: {} not found", dir.display());
        return None;
    }
    Some(GameAssets::open(&dir).unwrap_or_else(|e| panic!("{game}: {e}")))
}

#[test]
fn every_x_parses_and_is_sane() {
    for game in ["TOW", "RTWP", "WSW"] {
        let Some(assets) = open_game(game) else {
            continue;
        };
        let files: Vec<_> = assets
            .pack()
            .unwrap()
            .index()
            .entries()
            .iter()
            .filter(|e| e.name.to_lowercase().ends_with(".x"))
            .collect();
        assert_eq!(files.len(), 1, "{game}: .x count");
        assert_eq!(files[0].name.to_lowercase(), "graphics\\fish.x");
        let mesh = x::parse(&assets.read(&files[0].name).unwrap()).unwrap();

        // Hand-checked against the file: one Mesh of 285 vertices and 564
        // triangles, one material textured "dolphin.bmp", no MeshNormals.
        assert_eq!(mesh.surfaces.len(), 1, "{game}");
        assert_eq!(mesh.vertex_count(), 285, "{game}");
        assert_eq!(mesh.triangle_count(), 564, "{game}");
        let mat = &mesh.surfaces[0].material;
        assert_eq!(mat.texture.as_deref(), Some("dolphin.bmp"));
        assert_eq!(mat.color, [1.0; 3]);
        // File vertex 0 and its UV (vertices are renumbered by first use).
        assert!(mesh.surfaces[0].vertices.iter().any(|v| v.position
            == [-23.495_01, 90.550_02, -8.13]
            && v.uv == [0.514_251, 0.392_014]));

        // The file has no normals, so they are recomputed. One vertex only
        // touches zero-area triangles; Blitz would give it NaN, this gives
        // a zero vector.
        let zero = mesh
            .surfaces
            .iter()
            .flat_map(|s| &s.vertices)
            .filter(|v| v.normal == [0.0; 3])
            .count();
        assert_eq!(zero, 1, "{game}");
        for s in &mesh.surfaces {
            let n = s.vertices.len() as u32;
            assert!(s.triangles.iter().flatten().all(|&i| i < n));
            for v in &s.vertices {
                assert!(v.position.iter().chain(&v.uv).all(|x| x.is_finite()));
                let len = v.normal.iter().map(|x| x * x).sum::<f32>().sqrt();
                if len != 0.0 {
                    assert!((len - 1.0).abs() < 1e-3, "{game}: normal length {len}");
                }
            }
        }
    }
}
