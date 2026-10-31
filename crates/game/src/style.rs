//! The level textures `LoadStyleTextures` picks (`RTWP/setup.bb`) and the
//! fixed ones `Setup` loads once.
//!
//! The style sets the floor and wall textures; the background sets the
//! backdrop texture and its scale. Style 9 (custom textures) and background
//! 11 (custom backdrop) read per-level folders the port does not support
//! yet, so they fall back the way the original does when the files are
//! missing: style 9 to 2 (wood), background 11 to 3 (stars).

use wonderland_formats::{GameAssets, Level};

/// A texture file and the `MyLoadTexture` flags it is loaded with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureRef {
    /// The game path, Windows-style.
    pub path: String,
    /// Blitz texture flags: 1 colour, 2 alpha (from the brightness when the
    /// file has no alpha channel), 4 masked (black is transparent).
    pub flags: u32,
    /// `RotateTexture` in degrees (the conveyor textures).
    pub rotation: i32,
}

impl TextureRef {
    pub fn colour(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            flags: 1,
            rotation: 0,
        }
    }

    fn rotated(path: &str, rotation: i32) -> Self {
        Self {
            rotation,
            ..Self::colour(path)
        }
    }
}

/// The textures `ConstructLevel` uses.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelTextures {
    /// `FloorTexture` to `FloorTexture4`: tiles 100–103.
    pub floor: [TextureRef; 4],
    /// `FloorTextureSign`: tile 1300.
    pub floor_sign: TextureRef,
    /// `WallTextureTop1`.
    pub wall_top: TextureRef,
    /// `WallTextureSide1a`, `1b`, `2a`, `2b`, `3a`, `3b`.
    pub wall_side: [TextureRef; 6],
    /// `TeleportTexture(0–7)`: tiles 1200–1207.
    pub teleport: [TextureRef; 8],
    /// `IceTexture`, then `IceCurveTexture1–4`: tiles 400–404.
    pub ice: [TextureRef; 5],
    /// `ConveyorTexture1–4(1)`: tiles 500–503; the second set (508–511)
    /// is the same file, scrolled separately.
    pub conveyor: [TextureRef; 4],
    pub trampoline: TextureRef,
    pub water: TextureRef,
    pub lava: TextureRef,
    /// `BackGroundTexture` and its `ScaleTexture`, if the background has one.
    pub background: Option<(TextureRef, f32)>,
}

/// The background after the original's fallbacks.
pub fn effective_background(background: i32) -> i32 {
    if background == 11 { 3 } else { background }
}

/// The style after the original's fallbacks: background 10 forces the
/// rainbow style (-1), and style 9 falls back to 2.
pub fn effective_style(style: i32, background: i32) -> i32 {
    if background == 10 {
        -1
    } else if style == 9 {
        2
    } else {
        style
    }
}

/// `HouseTexture(1–4)`: the default houses, each replaced by the file of
/// that number in the level's custom houses folder if it is there.
pub fn house_textures(level: &Level, assets: &GameAssets) -> [String; 4] {
    let dir = level.houses.as_ref().filter(|d| d.enabled());
    std::array::from_fn(|i| {
        let k = i + 1;
        dir.map(|d| format!("customhouses\\{}\\house{k}.jpg", d.dir))
            .filter(|path| assets.resolve(path).is_some())
            .unwrap_or_else(|| format!("graphics\\house{k}.jpg"))
    })
}

/// `LoadStyleTextures` plus the fixed level textures from `Setup`.
/// `teleport_option` is `OptionsTeleport` from `options.cfg`: 1 uses the
/// generic teleporter textures.
pub fn level_textures(
    style: i32,
    background: i32,
    snow: bool,
    teleport_option: i32,
) -> LevelTextures {
    let background = effective_background(background);
    let style = effective_style(style, background);
    let g = |name: &str| TextureRef::colour(&format!("graphics\\{name}.jpg"));
    let rainbow = g("rainbow");

    let (prefix, sides): (&str, [&str; 6]) = match style {
        0 => ("cave", ["1a", "1b", "2a", "2b", "3a", "3b"]),
        1 => ("sand", ["1a", "1b", "2a", "2b", "3a", "3b"]),
        3 => ("metal", ["1a", "1b", "1a", "1b", "3a", "3b"]),
        4 => ("castle", ["1a", "1b", "", "", "3a", "3b"]),
        5 => ("green", ["1a", "1b", "1a", "1b", "1a", "1b"]),
        6 => ("spooky", ["1a", "1b", "2a", "2b", "3a", "3b"]),
        7 => ("garden", ["1a", "1b", "1a", "1b", "3a", "3b"]),
        8 => ("aztec", ["1a", "1b", "2a", "2b", "3a", "3b"]),
        _ => ("wood", ["1a", "1b", "2a", "2b", "3a", "3b"]),
    };

    let (floor, floor_sign, wall_top, wall_side, teleport) = if style == -1 {
        (
            std::array::from_fn(|_| rainbow.clone()),
            rainbow.clone(),
            rainbow.clone(),
            std::array::from_fn(|_| rainbow.clone()),
            std::array::from_fn(|_| rainbow.clone()),
        )
    } else {
        let mut floor: [TextureRef; 4] =
            std::array::from_fn(|i| g(&format!("{prefix}floor{}", i + 1)));
        // The cave loads its floors in the order 1, 4, 3, 2, and sand
        // borrows the garden's fourth floor.
        if style == 0 {
            floor = [
                g("cavefloor1"),
                g("cavefloor4"),
                g("cavefloor3"),
                g("cavefloor2"),
            ];
        }
        if style == 1 {
            floor[3] = g("gardenfloor4");
        }
        if snow {
            floor[3] = g("icefloor");
        }
        let wall_side = std::array::from_fn(|i| match (style, i) {
            // The castle's second wall pair is packed.
            (4, 2) => TextureRef::colour("res24.dat"),
            (4, 3) => TextureRef::colour("res25.dat"),
            _ => g(&format!("{prefix}wallside{}", sides[i])),
        });
        let teleport = std::array::from_fn(|i| {
            if teleport_option == 0 {
                g(&format!("{prefix}tele{i}"))
            } else {
                g(&format!("gentele{i}"))
            }
        });
        (
            floor,
            g(&format!("{prefix}floorsign")),
            g(&format!("{prefix}walltop1")),
            wall_side,
            teleport,
        )
    };

    let background = match background {
        0 => Some((TextureRef::colour("res27.dat"), 50.0)),
        1 => Some((g("grass4"), 50.0)),
        3 => Some((g("stars"), 50.0)),
        7 => Some((g("warp"), 0.25)),
        8 => Some((g("city"), 50.0)),
        9 | 10 => Some((rainbow.clone(), 50.0)),
        _ => None,
    };

    LevelTextures {
        floor,
        floor_sign,
        wall_top,
        wall_side,
        teleport,
        ice: [
            g("ice"),
            g("icecurve1"),
            g("icecurve2"),
            g("icecurve3"),
            g("icecurve4"),
        ],
        conveyor: [
            TextureRef::rotated("res14.dat", 0),
            TextureRef::rotated("res14.dat", 270),
            TextureRef::rotated("res14.dat", 180),
            TextureRef::rotated("res14.dat", 90),
        ],
        trampoline: TextureRef::colour("res59.dat"),
        water: TextureRef {
            flags: 2,
            ..g("water")
        },
        lava: g("lava"),
        background,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_follow_the_original_quirks() {
        let cave = level_textures(0, 0, false, 0);
        assert_eq!(cave.floor[1].path, r"graphics\cavefloor4.jpg");
        assert_eq!(cave.teleport[7].path, r"graphics\cavetele7.jpg");
        let sand = level_textures(1, 0, false, 1);
        assert_eq!(sand.floor[3].path, r"graphics\gardenfloor4.jpg");
        assert_eq!(sand.teleport[0].path, r"graphics\gentele0.jpg");
        let castle = level_textures(4, 1, true, 0);
        assert_eq!(castle.wall_side[2].path, "res24.dat");
        assert_eq!(castle.floor[3].path, r"graphics\icefloor.jpg");
        let green = level_textures(5, 2, false, 0);
        assert_eq!(green.wall_side[5].path, r"graphics\greenwallside1b.jpg");
        assert!(green.background.is_none());
        let rainbow = level_textures(3, 10, true, 0);
        assert_eq!(rainbow.floor[3].path, r"graphics\rainbow.jpg");
        assert_eq!(
            level_textures(9, 11, false, 0).wall_top.path,
            r"graphics\woodwalltop1.jpg"
        );
        assert_eq!(
            level_textures(9, 11, false, 0).background.unwrap().0.path,
            r"graphics\stars.jpg"
        );
    }
}
