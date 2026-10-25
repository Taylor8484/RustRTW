//! Checks every `.md2` in the real game packs (`assets/<GAME>`, gitignored;
//! games whose folder is missing are skipped).

use std::path::Path;

use wonderland_formats::GameAssets;
use wonderland_formats::mesh::md2;

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

/// (name, frames, vertices, triangles) after Blitz's (vertex, uv) splitting.
const SHAPES: [(&str, usize, usize, usize); 6] = [
    ("bomb", 30, 826, 1124),
    ("chomper", 30, 314, 384),
    ("coily", 30, 710, 840),
    ("peegueb", 147, 604, 998),
    ("qookie", 147, 604, 998),
    ("stinky", 147, 606, 998),
];

fn md2_files(assets: &GameAssets) -> Vec<(String, Vec<u8>)> {
    assets
        .pack()
        .unwrap()
        .index()
        .entries()
        .iter()
        .filter(|e| e.name.to_lowercase().ends_with(".md2"))
        .map(|e| (e.name.clone(), assets.read(&e.name).unwrap().into_owned()))
        .collect()
}

#[test]
fn every_md2_parses_and_is_sane() {
    let expected = [("TOW", 3), ("RTWP", 6), ("WSW", 5)];
    for (game, count) in expected {
        let Some(assets) = open_game(game) else {
            continue;
        };
        let files = md2_files(&assets);
        assert_eq!(files.len(), count, "{game}: md2 count");
        for (name, data) in files {
            let m = md2::parse(&data).unwrap_or_else(|e| panic!("{game}: {name}: {e}"));
            let n = m.vertex_count();
            let (_, frames, verts, tris) = *SHAPES
                .iter()
                .find(|s| name.eq_ignore_ascii_case(&format!("graphics\\{}.md2", s.0)))
                .unwrap_or_else(|| panic!("{game}: unexpected file {name}"));
            assert_eq!(
                (m.frames.len(), n, m.triangles.len()),
                (frames, verts, tris),
                "{game}: {name}"
            );
            assert!(n > 0 && !m.triangles.is_empty() && !m.frames.is_empty());
            for t in &m.triangles {
                assert!(t.iter().all(|&i| (i as usize) < n), "{game}: {name}: index");
            }
            for uv in &m.uvs {
                assert!(uv.iter().all(|x| x.is_finite()));
            }
            for f in &m.frames {
                assert_eq!(f.positions.len(), n, "{game}: {name}: positions");
                assert_eq!(f.normals.len(), n, "{game}: {name}: normals");
                assert!(!f.name.is_empty(), "{game}: {name}: unnamed frame");
                for p in &f.positions {
                    assert!(p.iter().all(|x| x.is_finite()));
                }
                for v in &f.normals {
                    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                    assert!((len - 1.0).abs() < 1e-3, "{game}: {name}: normal {len}");
                }
            }
            let mesh = m.frame_mesh(0);
            assert_eq!(mesh.vertex_count(), n);
            assert_eq!(mesh.triangle_count(), m.triangles.len());
        }
    }
}

#[test]
fn md2_values_match_hand_checked_file_contents() {
    // stinky.md2's header: 256x256 skin, no skins, 0x93 = 147 frames, named
    // frame01..frame147.
    if let Some(assets) = open_game("RTWP") {
        let m = md2::parse(&assets.read("graphics\\stinky.md2").unwrap()).unwrap();
        assert_eq!(m.frames.len(), 147);
        assert_eq!(m.skin_size, [256, 256]);
        assert!(m.skins.is_empty());
        assert_eq!(m.frames[0].name, "frame01");
        assert_eq!(m.frames[146].name, "frame147");
        // Every frame is named by its number.
        for (i, f) in m.frames.iter().enumerate() {
            assert_eq!(f.name, format!("frame{:02}", i + 1));
        }
    }
    // The three games ship identical copies of the shared models.
    let mut copies = std::collections::HashMap::new();
    for game in ["TOW", "RTWP", "WSW"] {
        let Some(assets) = open_game(game) else {
            continue;
        };
        for (name, data) in md2_files(&assets) {
            let m = md2::parse(&data).unwrap();
            let first = copies.entry(name.clone()).or_insert_with(|| m.clone());
            assert_eq!(*first, m, "{game}: {name} differs between games");
        }
    }
}
