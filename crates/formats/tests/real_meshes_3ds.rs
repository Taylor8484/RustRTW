//! Checks `.3ds` parsing against the real game data in `assets/<GAME>`
//! (gitignored; populate it with `tools/import-assets.sh`). Games whose folder
//! is missing are skipped.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use wonderland_formats::mesh::tds;
use wonderland_formats::{GameAssets, Mesh};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open_game(game: &str) -> Option<GameAssets> {
    let dir = repo_root().join("assets").join(game);
    if !dir.is_dir() {
        eprintln!("skipping {game}: {} not found", dir.display());
        return None;
    }
    Some(GameAssets::open(&dir).unwrap_or_else(|e| panic!("{game}: {e}")))
}

/// Every `.3ds` in a game's pack, as `(name, bytes)`.
fn pack_meshes(assets: &GameAssets) -> Vec<(String, Vec<u8>)> {
    let pack = assets.pack().expect("pack");
    pack.index()
        .entries()
        .iter()
        .filter(|e| e.name.to_lowercase().ends_with(".3ds"))
        .map(|e| (e.name.clone(), pack.entry_data(e).to_vec()))
        .collect()
}

/// The texture map file names (`MAPFILE`, A300) in a `.3ds`, found by a walk
/// that is independent of the parser.
fn texture_names(data: &[u8]) -> BTreeSet<String> {
    fn walk(d: &[u8], mut pos: usize, end: usize, out: &mut BTreeSet<String>) {
        while pos + 6 <= end {
            let id = u16::from_le_bytes([d[pos], d[pos + 1]]);
            let len = u32::from_le_bytes(d[pos + 2..pos + 6].try_into().unwrap()) as usize;
            let (body, chunk_end) = (pos + 6, (pos + len).min(end));
            match id {
                0x4D4D | 0x3D3D | 0xAFFF | 0xA200 => walk(d, body, chunk_end, out),
                0xA300 => {
                    let s = &d[body..chunk_end];
                    let s = &s[..s.iter().position(|&b| b == 0).unwrap_or(s.len())];
                    out.insert(s.iter().map(|&b| char::from(b)).collect());
                }
                _ => {}
            }
            pos += len.max(6);
        }
    }
    let mut out = BTreeSet::new();
    walk(data, 0, data.len(), &mut out);
    out
}

/// Models whose objects carry a scaled mesh matrix. Blitz transforms the
/// normals it computed in object space by the cofactor matrix without
/// renormalizing, so theirs are not unit length (the direction is right).
fn has_scaled_normals(name: &str) -> bool {
    [
        "creepykeep\\model1",
        "fireisle\\model3",
        "spookywalls\\model",
    ]
    .iter()
    .any(|m| name.contains(m))
}

fn check_sane(name: &str, data: &[u8], mesh: &Mesh) {
    assert!(!mesh.surfaces.is_empty(), "{name}: no surfaces");
    let file_textures = texture_names(data);
    let mut textured = 0;
    for (n, s) in mesh.surfaces.iter().enumerate() {
        let ctx = format!("{name} surface {n}");
        assert!(!s.vertices.is_empty(), "{ctx}: no vertices");
        assert!(!s.triangles.is_empty(), "{ctx}: no triangles");
        // Blitz stores triangle indices as `unsigned short`.
        assert!(s.vertices.len() <= 65535, "{ctx}: too many vertices");
        for t in &s.triangles {
            assert!(
                t.iter().all(|&i| (i as usize) < s.vertices.len()),
                "{ctx}: index out of range"
            );
        }
        for v in &s.vertices {
            let all = v.position.iter().chain(&v.normal).chain(&v.uv);
            assert!(all.clone().all(|f| f.is_finite()), "{ctx}: {v:?}");
            let len = v.normal.iter().map(|c| c * c).sum::<f32>().sqrt();
            if has_scaled_normals(name) {
                assert!(len < 1.0 + 1e-3, "{ctx}: normal length {len}");
            } else {
                // Zero only where every triangle at the vertex is degenerate.
                assert!(
                    len < 1e-6 || (len - 1.0).abs() < 1e-3,
                    "{ctx}: normal {len}"
                );
            }
        }
        assert_eq!(s.material.alpha, 1.0, "{ctx}");
        if let Some(t) = &s.material.texture {
            textured += 1;
            assert!(
                file_textures.contains(t),
                "{ctx}: texture {t:?} not in file"
            );
            assert_eq!(s.material.color, [1.0; 3], "{ctx}: textured but coloured");
        }
    }
    if !file_textures.is_empty() {
        assert!(
            textured > 0,
            "{name}: file has textures {file_textures:?}, mesh none"
        );
    }
}

#[test]
fn every_3ds_in_every_pack_parses() {
    let expected = [("TOW", 8), ("RTWP", 23), ("WSW", 53)];
    for (game, count) in expected {
        let Some(assets) = open_game(game) else {
            continue;
        };
        let files = pack_meshes(&assets);
        assert_eq!(files.len(), count, "{game}: 3ds count");
        for (name, data) in files {
            let mesh = tds::parse(&data).unwrap_or_else(|e| panic!("{game}: {name}: {e}"));
            check_sane(&format!("{game}: {name}"), &data, &mesh);
        }
    }
}

fn assert_bounds(mesh: &Mesh, lo: [f32; 3], hi: [f32; 3]) {
    let (min, max) = mesh.bounds().unwrap();
    for i in 0..3 {
        assert!((min[i] - lo[i]).abs() < 1e-4, "min {min:?} vs {lo:?}");
        assert!((max[i] - hi[i]).abs() < 1e-4, "max {max:?} vs {hi:?}");
    }
}

/// Values read off the chunk dumps by hand.
///
/// `graphics\box.3ds` is one object (`Box01`) with an identity mesh matrix,
/// a 24-vertex list spanning -0.4..0.4 on each axis and 12 faces.
/// `custommodels\spookywalls\model1.3ds` is two objects of 32 vertices and 12
/// faces each, with seven materials of which five end up used; the second
/// object's file-space x range is -1.5..-0.5, the first's z range is
/// 0.001..0.901, and file y becomes Blitz z.
#[test]
fn hand_checked_meshes() {
    if let Some(assets) = open_game("RTWP") {
        let mesh = tds::parse(&assets.read("graphics\\box.3ds").unwrap()).unwrap();
        assert_eq!(mesh.surfaces.len(), 1);
        assert_eq!((mesh.vertex_count(), mesh.triangle_count()), (24, 12));
        assert_bounds(&mesh, [-0.4; 3], [0.4; 3]);
        // Blitz smooths by position, so a corner's normal averages its faces
        // and points away from the centre.
        for v in &mesh.surfaces[0].vertices {
            let out: f32 = v.position.iter().zip(v.normal).map(|(p, n)| p * n).sum();
            assert!(out > 0.3, "{v:?}");
        }
    }
    if let Some(assets) = open_game("WSW") {
        let mesh = tds::parse(
            &assets
                .read("custommodels\\spookywalls\\model1.3ds")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(mesh.surfaces.len(), 5);
        assert_eq!((mesh.vertex_count(), mesh.triangle_count()), (64, 24));
        assert_bounds(&mesh, [-1.5, 0.0, -0.5], [0.501, 0.901, 0.501]);
    }
}
