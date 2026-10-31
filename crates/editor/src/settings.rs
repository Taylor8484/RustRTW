//! What applies to the whole level, as one set of choices.
//!
//! Some of it is in the header. The rest the format keeps as pieces that work
//! wherever they stand: weather, tint and house style are objects, and the 3D
//! view is a tile. The objects go on the outer wall, out of the way, and the
//! 3D view on a floor cell, since a tile would make a hole in the wall.

use wonderland_formats::catalogue::{self, Group, Layer};
use wonderland_formats::level::CustomDir;

use crate::{Editor, FLOOR};

/// `StyleName$`.
pub const STYLES: [&str; 10] = [
    "Cave", "Sand", "Wood", "Purple", "Castle", "Jade", "Spooky", "Garden", "Aztec", "Custom",
];
/// The style that needs a custom texture folder.
pub const CUSTOM_STYLE: i32 = 9;
/// `BackGroundName$`, by the number a v5 or v6 level stores.
pub const BACKGROUNDS: [(i32, &str); 11] = [
    (0, "Sky"),
    (1, "Forest"),
    (2, "Walls"),
    (3, "Stars"),
    (4, "Flat"),
    (5, "Water"),
    (6, "Lava"),
    (7, "Warp"),
    (8, "City"),
    (9, "Rainbow"),
    (10, "Custom"),
];
/// The background that needs a custom background folder.
pub const CUSTOM_BACKGROUND: i32 = 10;
/// `MusicName$`, for tracks 1 up.
pub const MUSIC: [&str; 5] = [
    "Welcome To Wonderland",
    "Puzzling Breeze",
    "Waterwaves",
    "Dark Caverns",
    "Bittersweet Endings",
];
/// The original editor's arrows, in steps of [`TIME_STEP`].
pub const MIN_TIME: i32 = 10;
pub const MAX_TIME: i32 = 900;
pub const TIME_STEP: i32 = 10;

pub const SNOW: i32 = 84;
pub const RAIN: i32 = 85;
pub const LIGHTNING: i32 = 86;
pub const FLASHES: i32 = 87;
pub const HIDDEN_RAINBOW: i32 = 88;
const TINT: i32 = 80;
pub const TINTS: [&str; 4] = ["Red", "Green", "Blue", "Yellow"];
const HOUSE_STYLE: i32 = 62;
pub const HOUSE_STYLES: usize = 4;
const VIEW_3D: i32 = 2300;

/// A folder of custom art a level can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Folder {
    Houses,
    Models,
    Textures,
    Backgrounds,
}

impl Folder {
    /// The folder under the game folder that holds these.
    pub fn root(self) -> &'static str {
        match self {
            Folder::Houses => "customhouses",
            Folder::Models => "custommodels",
            Folder::Textures => "customtextures",
            Folder::Backgrounds => "custombackgrounds",
        }
    }
}

/// One change to the level's settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    Time(i32),
    Style(i32),
    Background(i32),
    Music(i32),
    /// A piece that is on or off, such as [`SNOW`].
    Flag(i32, bool),
    Tint(Option<usize>),
    HouseStyle(Option<usize>),
    View3d(bool),
    Folder(Folder, Option<String>),
}

impl Editor {
    /// Whether the level holds object `code`.
    pub fn flag(&self, code: i32) -> bool {
        self.level.objects.contains(&code)
    }

    fn band(&self, base: i32, count: usize) -> Option<usize> {
        (0..count).find(|&i| self.flag(base + i as i32))
    }

    pub fn tint(&self) -> Option<usize> {
        self.band(TINT, TINTS.len())
    }

    pub fn house_style(&self) -> Option<usize> {
        self.band(HOUSE_STYLE, HOUSE_STYLES)
    }

    pub fn view3d(&self) -> bool {
        self.level.tiles.contains(&VIEW_3D)
    }

    pub fn folder(&self, folder: Folder) -> Option<&str> {
        let level = &self.level;
        let dir = match folder {
            Folder::Houses => &level.houses,
            Folder::Models => &level.models,
            Folder::Textures => &level.textures,
            Folder::Backgrounds => &level.backgrounds,
        };
        dir.as_ref().filter(|d| d.enabled()).map(|d| d.dir.as_str())
    }

    /// A cell of the outer wall with no object on it.
    fn free_edge(&self) -> Option<usize> {
        let level = &self.level;
        let (width, height) = (level.width, level.height);
        (0..level.objects.len()).find(|&i| {
            let (x, y) = (i % width, i / width);
            level.objects[i] == 0 && (x == 0 || y == 0 || x == width - 1 || y == height - 1)
        })
    }

    /// Takes every object in `codes` out of the level, and adds `code` if
    /// there is one.
    fn set_objects(&mut self, codes: std::ops::Range<i32>, code: Option<i32>) {
        let free = self.free_edge();
        let objects = &mut self.editing().objects;
        objects.iter_mut().for_each(|o| {
            if codes.contains(o) {
                *o = 0;
            }
        });
        if let (Some(code), Some(free)) = (code, free) {
            objects[free] = code;
        }
    }

    fn set_view3d(&mut self, on: bool) {
        let level = &self.level;
        let (width, height) = (level.width, level.height);
        let floor = (0..level.tiles.len()).find(|&i| {
            let (x, y) = (i % width, i / width);
            level.tiles[i] == FLOOR
                && level.objects[i] == 0
                && (1..width - 1).contains(&x)
                && (1..height - 1).contains(&y)
        });
        let tiles = &mut self.editing().tiles;
        tiles.iter_mut().for_each(|t| {
            if *t == VIEW_3D {
                *t = FLOOR;
            }
        });
        if let (true, Some(floor)) = (on, floor) {
            tiles[floor] = VIEW_3D;
        }
    }

    /// Makes `setting` one undo step.
    pub fn apply(&mut self, setting: Setting) {
        match setting {
            Setting::Time(time) => {
                self.editing().time = time.clamp(MIN_TIME, MAX_TIME);
            }
            Setting::Style(style) => self.editing().style = style,
            Setting::Background(background) => self.editing().background = background,
            Setting::Music(track) => self.editing().music_track = Some(track),
            Setting::Flag(code, on) => self.set_objects(code..code + 1, on.then_some(code)),
            Setting::Tint(tint) => self.set_objects(
                TINT..TINT + TINTS.len() as i32,
                tint.map(|t| TINT + t as i32),
            ),
            Setting::HouseStyle(style) => self.set_objects(
                HOUSE_STYLE..HOUSE_STYLE + HOUSE_STYLES as i32,
                style.map(|s| HOUSE_STYLE + s as i32),
            ),
            Setting::View3d(on) => self.set_view3d(on),
            Setting::Folder(folder, dir) => {
                let dir = Some(match dir {
                    Some(dir) => CustomDir { flag: 1, dir },
                    None => CustomDir {
                        flag: 0,
                        dir: String::new(),
                    },
                });
                let level = self.editing();
                match folder {
                    Folder::Houses => level.houses = dir,
                    Folder::Models => level.models = dir,
                    Folder::Textures => level.textures = dir,
                    Folder::Backgrounds => level.backgrounds = dir,
                }
            }
        }
        self.commit();
    }

    /// Puts any setting piece that growing left inside the level back on the
    /// outer wall, where it cannot be painted over.
    pub(crate) fn tuck_settings(&mut self) {
        let level = &self.level;
        let (width, height) = (level.width, level.height);
        let inside: Vec<usize> = (0..level.objects.len())
            .filter(|&i| {
                let (x, y) = (i % width, i / width);
                (1..width - 1).contains(&x)
                    && (1..height - 1).contains(&y)
                    && catalogue::find(Layer::Object, level.objects[i])
                        .is_some_and(|p| p.group == Group::Setting)
            })
            .collect();
        for i in inside {
            if let Some(free) = self.free_edge() {
                let level = self.editing();
                level.objects[free] = level.objects[i];
                level.objects[i] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MIN_SIZE, Side};

    fn outer(index: usize) -> bool {
        let (x, y) = (index % MIN_SIZE, index / MIN_SIZE);
        x == 0 || y == 0 || x == MIN_SIZE - 1 || y == MIN_SIZE - 1
    }

    #[test]
    fn a_setting_is_one_undo_step() {
        let mut editor = Editor::new();
        editor.apply(Setting::Time(120));
        editor.apply(Setting::Style(5));
        assert_eq!((editor.level().time, editor.level().style), (120, 5));
        assert!(editor.undo());
        assert_eq!(editor.level().style, 2);
        assert_eq!(editor.level().time, 120);
    }

    #[test]
    fn time_stays_in_the_arrows_range() {
        let mut editor = Editor::new();
        editor.apply(Setting::Time(5000));
        assert_eq!(editor.level().time, MAX_TIME);
    }

    #[test]
    fn weather_stands_on_the_outer_wall() {
        let mut editor = Editor::new();
        editor.apply(Setting::Flag(SNOW, true));
        editor.apply(Setting::Flag(RAIN, true));
        assert!(editor.flag(SNOW) && editor.flag(RAIN));
        let at = editor.level().objects.iter().position(|&o| o == SNOW);
        assert!(outer(at.unwrap()));
        editor.apply(Setting::Flag(SNOW, false));
        assert!(!editor.flag(SNOW));
        assert!(editor.flag(RAIN));
    }

    #[test]
    fn a_tint_replaces_a_tint() {
        let mut editor = Editor::new();
        editor.apply(Setting::Tint(Some(0)));
        editor.apply(Setting::Tint(Some(3)));
        assert_eq!(editor.tint(), Some(3));
        assert!(!editor.flag(TINT));
        editor.apply(Setting::Tint(None));
        assert_eq!(editor.tint(), None);
    }

    #[test]
    fn the_3d_view_takes_a_floor_cell_and_gives_it_back() {
        let mut editor = Editor::new();
        editor.apply(Setting::View3d(true));
        assert!(editor.view3d());
        assert!(editor.level().tile(0, 0) == 200, "not a hole in the wall");
        editor.apply(Setting::View3d(false));
        assert!(!editor.view3d());
        assert_eq!(*editor.level().tiles, *Editor::new().level().tiles);
    }

    #[test]
    fn a_folder_is_set_and_taken_off_by_its_flag() {
        let mut editor = Editor::new();
        editor.apply(Setting::Folder(Folder::Textures, Some("SNOW".into())));
        assert_eq!(editor.folder(Folder::Textures), Some("SNOW"));
        assert_eq!(editor.level().textures.as_ref().unwrap().flag, 1);
        editor.apply(Setting::Folder(Folder::Textures, None));
        assert_eq!(editor.folder(Folder::Textures), None);
    }

    #[test]
    fn growing_keeps_settings_on_the_outer_wall() {
        let mut editor = Editor::new();
        editor.apply(Setting::Flag(SNOW, true));
        editor.grow(Side::Top);
        editor.tuck_settings();
        editor.commit();
        let level = editor.level();
        let at = level.objects.iter().position(|&o| o == SNOW).unwrap();
        let (x, y) = (at % level.width, at / level.width);
        assert!(x == 0 || y == 0 || x == level.width - 1 || y == level.height - 1);
    }
}
