//! The pieces a level is built from, and how they map to level codes.
//!
//! `RTWP/editor.bb` shows the raw codes; here each family of codes is one
//! [`Piece`] with named [`Axis`] choices, so a cannon is one piece with a
//! direction and a timing rather than 16 tiles. This covers the codes of v5
//! and v6 levels: the tiles `DoesTileExist` marks and objects 1–89.
//!
//! A piece's codes are `base` plus its choices as mixed-radix digits, the
//! first axis lowest. Every family in the format fits that shape.
//!
//! Names come from the original editor where it has them (`TileCategory$`)
//! and otherwise from its sprite sheets and the matching `logic` module.

/// Which of a level's two grids a piece is stored in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Tile,
    Object,
}

/// Where a piece belongs in a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Ground,
    Wall,
    Hazard,
    Switch,
    Box,
    Monster,
    Item,
    Character,
    Sign,
    Scenery,
    /// Applies to the whole level wherever it stands: weather, tint, 3D
    /// view, house style.
    Setting,
}

/// One choice a piece offers.
#[derive(Debug, PartialEq, Eq)]
pub struct Axis {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

const DIRECTION: Axis = Axis {
    name: "Direction",
    values: &["Up", "Right", "Down", "Left"],
};
const HAND: Axis = Axis {
    name: "Hand",
    values: &["Left", "Right"],
};
const CHARACTER: Axis = Axis {
    name: "Character",
    values: &["Stinky", "Loof", "Qookie", "Peegue"],
};
/// 1 is the fastest; each step adds one base period.
const TIMING: Axis = Axis {
    name: "Timing",
    values: &["1", "2", "3", "4"],
};
const LINE: Axis = Axis {
    name: "Line",
    values: &["Across", "Down"],
};
const GATE_COLOUR: Axis = Axis {
    name: "Colour",
    values: &["Magenta", "Yellow", "Green", "Cyan", "Red", "Blue", "White"],
};
const BUTTON_ACTION: Axis = Axis {
    name: "Action",
    values: &["Stays down", "Pops up", "Timer"],
};
const BUTTON_TARGET: Axis = Axis {
    name: "Works",
    values: &[
        "Conveyors 1",
        "Conveyors 2",
        "Magenta gates",
        "Yellow gates",
        "Green gates",
        "Cyan gates",
        "Red gates",
        "Blue gates",
        "White gates",
    ],
};
const TELEPORT_COLOUR: Axis = Axis {
    name: "Colour",
    values: &[
        "Red", "Yellow", "Green", "Blue", "Magenta", "Cyan", "White", "Black",
    ],
};
const CONVEYOR_KIND: Axis = Axis {
    name: "Kind",
    values: &["Belt", "Switcher"],
};
const CONVEYOR_SET: Axis = Axis {
    name: "Set",
    values: &["1", "2"],
};
const STEPS: Axis = Axis {
    name: "Steps",
    values: &["1", "2", "3"],
};
/// Sign tiles carry the number of their text; an editor assigns it.
const SIGN_SLOT: Axis = Axis {
    name: "Slot",
    values: &[
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
        "17", "18", "19", "20",
    ],
};
const BOX_KIND: Axis = Axis {
    name: "Box",
    values: &[
        "Wood",
        "Steel",
        "Reflector /",
        "Reflector \\",
        "Boulder",
        "Ghost box",
        "Boom box",
        "Prism",
        "Sticky cube",
        "Red sphere",
        "Yellow sphere",
        "Green sphere",
        "Blue sphere",
    ],
};
/// Codes 2200 up, in the order of the sheet. `CreateWarper` swaps the two
/// bomber kinds, so these are the sprites and not the monster kinds.
const WARP_MONSTER: Axis = Axis {
    name: "Monster",
    values: &[
        "Beach ball",
        "Left z-bot",
        "Right z-bot",
        "Left bomber",
        "Right bomber",
        "Left saucer",
        "Right saucer",
        "Red chomper",
        "Yellow chomper",
        "Ghost",
        "Rainbow spirit",
        "Broken z-bot",
    ],
};

/// A family of level codes.
#[derive(Debug, PartialEq, Eq)]
pub struct Piece {
    pub name: &'static str,
    pub group: Group,
    pub layer: Layer,
    /// The code with every choice at 0.
    pub base: i32,
    pub axes: &'static [Axis],
}

impl Piece {
    /// How many codes this piece covers.
    pub fn count(&self) -> i32 {
        self.axes.iter().map(|a| a.values.len() as i32).product()
    }

    pub fn contains(&self, code: i32) -> bool {
        (self.base..self.base + self.count()).contains(&code)
    }

    /// The code for `choices`, one value index per axis.
    pub fn code(&self, choices: &[usize]) -> i32 {
        debug_assert_eq!(choices.len(), self.axes.len());
        let mut code = self.base;
        let mut scale = 1;
        for (axis, &choice) in self.axes.iter().zip(choices) {
            debug_assert!(choice < axis.values.len());
            code += choice as i32 * scale;
            scale *= axis.values.len() as i32;
        }
        code
    }

    /// The reverse of [`Piece::code`], for a code this piece contains.
    pub fn choices(&self, code: i32) -> Vec<usize> {
        debug_assert!(self.contains(code));
        let mut rest = code - self.base;
        self.axes
            .iter()
            .map(|axis| {
                let n = axis.values.len() as i32;
                let choice = rest % n;
                rest /= n;
                choice as usize
            })
            .collect()
    }
}

/// A frame in one of the original editor's sprite sheets, a strip of square
/// frames in RTWP's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sprite {
    /// The game path, for [`crate::GameAssets`].
    pub sheet: &'static str,
    /// Frame width and height in pixels.
    pub size: u32,
    pub frame: u32,
}

const OBJECT_SHEET: &str = "graphicseditor\\objectsv5b.bmp";

/// `TileCategoryTexture`, by `code / 100`. The teleporters have a second
/// sheet in `res13.dat` for `OptionsTeleport`, which the port does not offer.
const TILE_SHEETS: [&str; 25] = [
    "graphicseditor\\ed-empty.bmp",
    "graphicseditor\\ed-floorv5.bmp",
    "graphicseditor\\ed-wallv5.bmp",
    "graphicseditor\\ed-water.bmp",
    "graphicseditor\\ed-ice.bmp",
    "graphicseditor\\ed-conveyorv5.bmp",
    "graphicseditor\\ed-breakawayv5.bmp",
    "graphicseditor\\ed-electro.bmp",
    "graphicseditor\\ed-cannon.bmp",
    "graphicseditor\\ed-lava.bmp",
    "graphicseditor\\ed-gatev5.bmp",
    "graphicseditor\\ed-buttonsv5.bmp",
    "graphicseditor\\ed-teleportv5.bmp",
    "graphicseditor\\ed-signsv5.bmp",
    "graphicseditor\\ed-spikes.bmp",
    "graphicseditor\\ed-fakev5.bmp",
    "res63.dat",
    "res68.dat",
    "res67.dat",
    "res64.dat",
    "res66.dat",
    "res65.dat",
    "res62.dat",
    "res61.dat",
    "res60.dat",
];

/// The sprite the original editor shows for a catalogued code.
pub fn sprite(layer: Layer, code: i32) -> Option<Sprite> {
    find(layer, code)?;
    Some(match layer {
        Layer::Tile => Sprite {
            sheet: TILE_SHEETS[code as usize / 100],
            size: 32,
            frame: code as u32 % 100,
        },
        Layer::Object => Sprite {
            sheet: OBJECT_SHEET,
            size: 16,
            frame: code as u32,
        },
    })
}

/// The piece that holds `code`. Object 0, an empty cell, is no piece.
pub fn find(layer: Layer, code: i32) -> Option<&'static Piece> {
    PIECES.iter().find(|p| p.layer == layer && p.contains(code))
}

const fn tile(name: &'static str, group: Group, base: i32, axes: &'static [Axis]) -> Piece {
    Piece {
        name,
        group,
        layer: Layer::Tile,
        base,
        axes,
    }
}

const fn object(name: &'static str, group: Group, base: i32, axes: &'static [Axis]) -> Piece {
    Piece {
        name,
        group,
        layer: Layer::Object,
        base,
        axes,
    }
}

/// Every piece, in code order per layer.
pub static PIECES: &[Piece] = &[
    tile("Empty", Group::Ground, 0, &[]),
    tile("Floor", Group::Ground, 100, &[]),
    tile("Floor B", Group::Ground, 101, &[]),
    tile("Floor C", Group::Ground, 102, &[]),
    tile("Floor D", Group::Ground, 103, &[]),
    // Looks like floor, but `ConstructLevel` makes it a wall.
    tile("Blocked floor", Group::Wall, 104, &[]),
    tile("Wall", Group::Wall, 200, &[]),
    tile("Deep hole", Group::Ground, 201, &[]),
    tile("Wall 1.5", Group::Wall, 202, &[]),
    tile("Wall 2.0", Group::Wall, 203, &[]),
    tile("Water", Group::Ground, 300, &[]),
    tile("Ice", Group::Ground, 400, &[]),
    tile("Ice corner, right and down", Group::Ground, 401, &[]),
    tile("Ice corner, left and down", Group::Ground, 402, &[]),
    tile("Ice corner, left and up", Group::Ground, 403, &[]),
    tile("Ice corner, right and up", Group::Ground, 404, &[]),
    tile(
        "Conveyor",
        Group::Ground,
        500,
        &[DIRECTION, CONVEYOR_KIND, CONVEYOR_SET],
    ),
    tile("Bridge over water", Group::Ground, 600, &[STEPS]),
    tile("Bridge over lava", Group::Ground, 603, &[STEPS]),
    tile("Bridge over nothing", Group::Ground, 606, &[STEPS]),
    tile("Electro floor", Group::Hazard, 700, &[TIMING]),
    tile("Cannon", Group::Hazard, 800, &[TIMING, DIRECTION]),
    tile("Lava", Group::Ground, 900, &[]),
    tile("Final gate", Group::Switch, 1000, &[LINE]),
    tile("Gate", Group::Switch, 1002, &[LINE, GATE_COLOUR]),
    tile(
        "Button",
        Group::Switch,
        1100,
        &[BUTTON_ACTION, BUTTON_TARGET],
    ),
    tile("Teleporter", Group::Switch, 1200, &[TELEPORT_COLOUR]),
    tile("Sign", Group::Sign, 1300, &[SIGN_SLOT]),
    tile("Spikes", Group::Hazard, 1400, &[TIMING]),
    tile("Fake wall", Group::Wall, 1500, &[]),
    tile("Fake wall 1.5", Group::Wall, 1501, &[]),
    tile("Fake wall 2.0", Group::Wall, 1502, &[]),
    tile("Box factory", Group::Box, 1600, &[BOX_KIND]),
    tile("Transporter on water", Group::Ground, 1700, &[]),
    tile("Transporter on nothing", Group::Ground, 1701, &[]),
    tile("Transporter on lava", Group::Ground, 1702, &[]),
    tile("Trampoline", Group::Ground, 1800, &[]),
    tile("Push cannon", Group::Box, 1900, &[TIMING, DIRECTION]),
    tile("Sticky cube", Group::Box, 2000, &[]),
    tile("Red link sphere", Group::Box, 2100, &[]),
    tile("Yellow link sphere", Group::Box, 2101, &[]),
    tile("Green link sphere", Group::Box, 2102, &[]),
    tile("Blue link sphere", Group::Box, 2103, &[]),
    tile("Warp gate", Group::Monster, 2200, &[WARP_MONSTER]),
    tile("3D view", Group::Setting, 2300, &[]),
    tile("Shadow stinker", Group::Monster, 2400, &[CHARACTER]),
    tile("Fire stinker", Group::Monster, 2404, &[CHARACTER]),
    object("Player", Group::Character, 1, &[CHARACTER]),
    object("Coin", Group::Item, 5, &[]),
    object("Exit", Group::Item, 6, &[]),
    object("Clock", Group::Item, 7, &[]),
    object("Bonus coin", Group::Item, 8, &[]),
    object("Wood box", Group::Box, 9, &[]),
    object("Steel box", Group::Box, 10, &[]),
    object("Reflector /", Group::Box, 11, &[]),
    object("Reflector \\", Group::Box, 12, &[]),
    object("Boulder", Group::Box, 13, &[]),
    object("Ghost box", Group::Box, 14, &[]),
    object("Boom box", Group::Box, 15, &[]),
    object("Prism", Group::Box, 16, &[]),
    object("Beach ball", Group::Monster, 17, &[]),
    object("Z-bot", Group::Monster, 18, &[HAND, DIRECTION]),
    object("Broken z-bot", Group::Monster, 26, &[]),
    object("Bomber", Group::Monster, 27, &[HAND, DIRECTION]),
    object("Saucer", Group::Monster, 35, &[HAND, DIRECTION]),
    object("Red chomper", Group::Monster, 43, &[]),
    object("Yellow chomper", Group::Monster, 44, &[]),
    object("Ghost", Group::Monster, 45, &[]),
    object("Rainbow spirit", Group::Monster, 46, &[]),
    object("Mother ship", Group::Monster, 47, &[]),
    object("Fish", Group::Scenery, 48, &[]),
    object("Fireball", Group::Hazard, 49, &[]),
    object("Pillar", Group::Scenery, 50, &[]),
    object("Small cone", Group::Scenery, 51, &[]),
    object("Large cone", Group::Scenery, 52, &[]),
    object("Fountain", Group::Scenery, 53, &[]),
    object("Large pyramid", Group::Scenery, 54, &[]),
    object("Small pyramid", Group::Scenery, 55, &[]),
    object("Big wall", Group::Scenery, 56, &[]),
    object("Lamp", Group::Scenery, 57, &[]),
    object(
        "House",
        Group::Scenery,
        58,
        &[Axis {
            name: "Facing",
            values: &["Down", "Left", "Up", "Right"],
        }],
    ),
    object(
        "House style",
        Group::Setting,
        62,
        &[Axis {
            name: "Style",
            values: &["1", "2", "3", "4"],
        }],
    ),
    object("Large tree", Group::Scenery, 66, &[]),
    object("Small tree", Group::Scenery, 67, &[]),
    object("Pine tree", Group::Scenery, 68, &[]),
    object("Snowy pine tree", Group::Scenery, 69, &[]),
    object("Mushroom", Group::Scenery, 70, &[]),
    object("Blossom tree", Group::Scenery, 71, &[]),
    object("Flowers", Group::Scenery, 72, &[]),
    object("Stinky statue", Group::Scenery, 73, &[]),
    object("Rainbow statue", Group::Scenery, 74, &[]),
    object("Snowman", Group::Scenery, 75, &[]),
    object(
        "Custom model",
        Group::Scenery,
        76,
        &[Axis {
            name: "Model",
            values: &["A", "B", "C", "D"],
        }],
    ),
    object(
        "Tint",
        Group::Setting,
        80,
        &[Axis {
            name: "Tint",
            values: &["Red", "Green", "Blue", "Yellow"],
        }],
    ),
    object("Snow", Group::Setting, 84, &[]),
    object("Rain", Group::Setting, 85, &[]),
    object("Lightning", Group::Setting, 86, &[]),
    object("Colour flashes", Group::Setting, 87, &[]),
    object("Hidden rainbow world", Group::Setting, 88, &[]),
    object("Green statue", Group::Scenery, 89, &[]),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The tile codes `DoesTileExist` marks in `RTWP/editor.bb`.
    fn editor_tiles() -> Vec<i32> {
        let ranges = [
            0..=0,
            100..=104,
            200..=203,
            300..=300,
            400..=404,
            500..=515,
            600..=608,
            700..=703,
            800..=815,
            900..=900,
            1000..=1015,
            1100..=1126,
            1200..=1207,
            1300..=1319,
            1400..=1403,
            1500..=1502,
            1600..=1612,
            1700..=1702,
            1800..=1800,
            1900..=1915,
            2000..=2000,
            2100..=2103,
            2200..=2211,
            2300..=2300,
            2400..=2407,
        ];
        ranges.into_iter().flatten().collect()
    }

    fn codes(layer: Layer) -> Vec<i32> {
        let mut codes: Vec<i32> = PIECES
            .iter()
            .filter(|p| p.layer == layer)
            .flat_map(|p| p.base..p.base + p.count())
            .collect();
        codes.sort_unstable();
        codes
    }

    #[test]
    fn covers_every_code_once() {
        assert_eq!(codes(Layer::Tile), editor_tiles());
        assert_eq!(codes(Layer::Object), (1..90).collect::<Vec<_>>());
    }

    #[test]
    fn code_and_choices_are_inverses() {
        for piece in PIECES {
            for code in piece.base..piece.base + piece.count() {
                assert_eq!(piece.code(&piece.choices(code)), code, "{}", piece.name);
            }
        }
    }

    #[test]
    fn axes_follow_the_format() {
        let cannon = find(Layer::Tile, 800).unwrap();
        assert_eq!(cannon.code(&[3, 2]), 811);
        let conveyor = find(Layer::Tile, 500).unwrap();
        assert_eq!(conveyor.code(&[1, 1, 1]), 513);
        let button = find(Layer::Tile, 1126).unwrap();
        assert_eq!(button.choices(1126), [2, 8]);
        let zbot = find(Layer::Object, 25).unwrap();
        assert_eq!(zbot.choices(25), [1, 3]);
    }

    #[test]
    fn unknown_codes_have_no_piece_or_sprite() {
        assert_eq!(find(Layer::Object, 0), None);
        assert_eq!(find(Layer::Object, 90), None);
        assert_eq!(find(Layer::Tile, 105), None);
        assert_eq!(find(Layer::Tile, -1), None);
        assert_eq!(sprite(Layer::Tile, 2500), None);
    }

    #[test]
    fn sprites_follow_the_sheets() {
        assert_eq!(
            sprite(Layer::Tile, 2207),
            Some(Sprite {
                sheet: "res62.dat",
                size: 32,
                frame: 7
            })
        );
        assert_eq!(
            sprite(Layer::Object, 42),
            Some(Sprite {
                sheet: OBJECT_SHEET,
                size: 16,
                frame: 42
            })
        );
    }
}
