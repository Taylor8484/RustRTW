//! The menus as `RTWP/menu.bb` runs them: `StartMenu`, `EndMenu`, the
//! `MenuLoop` dispatch, the menu text and `MenuBkgUpdate`'s animation.
//!
//! This module has no Bevy state, so it is tested headless; `menu_scene`
//! draws it and carries out the [`Action`]s. Ported so far: the title (0),
//! the main menu (1), player select (2), new player (3), the level
//! selector (4), "Confirm Player Deletion?" (6), options (9), "Exit?" (10),
//! the endings (12 and 16), the level packs (13), the
//! custom levels (14) and "Go Online?" (15). The in-game menu (5) is
//! [`crate::game_menu`].
//!
//! The level pack menu offers the tutorial, classic, deluxe and custom
//! levels; the kids levels' line is commented out in the original, so
//! their worlds are never reached.
//!
//! Each `MenuLoopN` is one frame: it lays out the text (the letters get
//! their alpha when first shown), runs the frame's ticks of
//! `MenuBkgUpdate`, renders, and then reads the keys. Keys are read once
//! per frame as held keys; `CursorKeyPressed` turns a held key into one
//! press.

use std::collections::VecDeque;
use std::f32::consts::FRAC_1_SQRT_2;

use wonderland_campaign::custom::{self, CustomLevels, FileEntry};
use wonderland_campaign::profile::{Entry, MAX_PLAYERS, Players, Profile};
use wonderland_campaign::worlds::{
    self, LEVELS, PLAYGROUND_LEVEL, TOW_PACK, Worlds, title_lines, total_score,
};
use wonderland_formats::config::Options as ConfigOptions;

use crate::ending::EndScene;
use crate::hud::{Edge, Letter, display_text, pinned};
use crate::pad::{Device, menu_prompt};

pub const TITLE: i32 = 0;
pub const MAIN: i32 = 1;
pub const PLAYER_SELECT: i32 = 2;
pub const NEW_PLAYER: i32 = 3;
pub const LEVEL_SELECT: i32 = 4;
pub const DELETE_PLAYER: i32 = 6;
/// TOW's playground level selector (TOW's menu 8).
pub const PLAYGROUND: i32 = 8;
pub const OPTIONS: i32 = 9;
pub const EXIT: i32 = 10;
/// The end of the classic levels.
pub const ENDING: i32 = 12;
pub const LEVEL_PACKS: i32 = 13;
pub const CUSTOM_LEVELS: i32 = 14;
pub const ONLINE: i32 = 15;
/// The end of the deluxe levels.
pub const ENDING_DELUXE: i32 = 16;

/// The keys the menus read, held this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuKeys {
    /// Cursor up or keypad 8.
    pub up: bool,
    /// Cursor down or keypad 2.
    pub down: bool,
    /// Cursor left or keypad 4, and right or keypad 6 (the level
    /// selector).
    pub left: bool,
    pub right: bool,
    /// The 2 and 3 of the top row, with `one` the way to the ending.
    pub two: bool,
    pub three: bool,
    /// Return or keypad Enter.
    pub enter: bool,
    pub escape: bool,
    pub space: bool,
    /// The number keys 1 and 0 (the options menu's secret song).
    pub one: bool,
    pub zero: bool,
    /// Delete (player select) and Backspace (new player).
    pub delete: bool,
    pub backspace: bool,
    /// Page Up and Page Down (the custom levels).
    pub page_up: bool,
    pub page_down: bool,
    /// F1 and F2: the custom levels sorted by name or not.
    pub f1: bool,
    pub f2: bool,
}

/// The sounds the menus play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSound {
    /// `SFXMenuSound(0)` (`sound\menumove.wav`) through `PlayMySound`, so
    /// at the sound level and not at all when sound is off.
    Move,
    /// `SFXMenuSound(1)` (`sound\type.wav`), typing a name.
    Type,
    /// `SFXLastCoin` (`sound\lastcoin.wav`), a new player created.
    LastCoin,
    /// `SFXTalk(5)` (`sound\stinkyno.wav`), the level selector's gate is
    /// closed.
    No,
    /// `newsounds\song1c.wav` through `PlaySound`, at full volume.
    SecretSong,
    /// `newsounds\exitN.wav`, played as the game closes.
    Exit(u8),
}

/// The exit sound for `MilliSecs() Mod 100`.
pub fn exit_sound(ms_mod_100: u32) -> MenuSound {
    MenuSound::Exit(match ms_mod_100 {
        0..15 => 3,
        15..30 => 4,
        30..60 => 5,
        60..75 => 6,
        _ => 8,
    })
}

/// What the front end has to do after a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Sound(MenuSound),
    /// `StartMenu` ran: restart the tick clock (`TweenTime`).
    Entered(i32),
    /// `StartLevel(CurrentLevel)`: leave the menus and play level
    /// [`Menu::level`] of the world table, or with `CurrentLevel=-999`
    /// the custom level [`Menu::custom_path`].
    StartLevel(i32),
    /// `StartLevelMusic` for world `CurrentWorld`.
    LevelMusic(i32),
    /// A player file could not be read or written.
    Warn(String),
    /// An option changed; the game's settings follow it.
    OptionsChanged,
    /// `SaveOptions`.
    SaveOptions,
    /// `GameExit`, then `ExecFile` of `url` if one was picked.
    Quit {
        url: Option<&'static str>,
    },
    /// `PlayMusic("res7.dat")`.
    TitleMusic,
    /// `PlayMusic("res4.dat")`, for the classic ending.
    EndingMusic,
    /// `StopChannel Music`.
    StopMusic,
    /// TOW's `StartMenu(12)`, `skip` ticks in.
    TowEnding {
        skip: u32,
    },
}

/// The pages the original sends `VersionNo1` codes to on exit.
fn exit_url(version_no1: i32) -> Option<&'static str> {
    match version_no1 {
        -999 => Some("http://www.midnightsynergy.com/returntowonderland/hints"),
        -998 => Some("http://www.midnightsynergy.com/returntowonderland/levelex"),
        -996 => Some("http://www.midnightsynergy.com"),
        _ => None,
    }
}

/// The forward gate's three lines on the level selector, open or closed.
fn gate_text(language: i32, open: bool) -> [&'static str; 3] {
    match (language, open) {
        (1, true) => ["Das Tor ist geoeffnet.", "'Enter' zum fortsetzen.", ""],
        (2, true) => [
            "La Grille est ouverte.",
            "Tape 'Entree' pour continuer.",
            "",
        ],
        (_, true) => ["The Gateway Is Open", "Press 'Enter' To Continue", ""],
        (1, false) => [
            "Das Tor ist geschlossen.",
            "Beende erst alle Levels dieser Welt",
            "um es zu oeffnen.",
        ],
        (2, false) => [
            "La Grille est fermee.",
            "Termine tous les niveaux de ce monde",
            "pour ouvrir cette Grille.",
        ],
        (_, false) => [
            "The Gateway is Closed",
            "Complete all Levels in this World",
            "in order to open this Gate",
        ],
    }
}

/// The back gate's first two lines on the level selector.
fn back_text(language: i32) -> [&'static str; 2] {
    match language {
        1 => ["Das Tor ist geoeffnet.", "'Enter' zum umkehren."],
        2 => [
            "La Grille est ouverte.",
            "Tape 'Entree' pour revenir en arriere.",
        ],
        _ => ["Gateway is Open", "Press 'Enter' to Travel Back"],
    }
}

/// The letter picker's cells, row by row; the one after them is OK.
const PICKER_LETTERS: &[u8; 27] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ ";
const PICKER_COLUMNS: usize = 9;
const PICKER_ROWS: usize = 3;
const PICKER_OK: usize = PICKER_LETTERS.len();
/// The first letter's column and row, on `DisplayText`'s grid.
const PICKER_LEFT: f32 = 12.0;
const PICKER_TOP: f32 = 10.0;

/// The cell a direction leads to from `cell`, wrapping round. OK is below
/// every column, and comes back to the row above in the middle.
fn picker_step(cell: usize, keys: MenuKeys) -> usize {
    let rows = PICKER_ROWS as i32 + 1;
    let columns = PICKER_COLUMNS as i32;
    let (mut row, mut column) = if cell == PICKER_OK {
        (PICKER_ROWS as i32, columns / 2)
    } else {
        (
            (cell / PICKER_COLUMNS) as i32,
            (cell % PICKER_COLUMNS) as i32,
        )
    };
    if keys.up {
        row = (row + rows - 1) % rows;
    } else if keys.down {
        row = (row + 1) % rows;
    } else if row < PICKER_ROWS as i32 {
        column = (column + if keys.left { columns - 1 } else { 1 }) % columns;
    }
    if row == PICKER_ROWS as i32 {
        PICKER_OK
    } else {
        (row * columns + column) as usize
    }
}

/// One letter sprite (`LetterType`).
#[derive(Debug, Clone, PartialEq)]
struct TagLetter {
    tag: i32,
    letter: Letter,
    /// `Active`: shown, with its alpha set when it was turned on.
    active: bool,
}

/// The letters on screen, by tag, as `text.bb` keeps them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Text {
    letters: Vec<TagLetter>,
}

impl Text {
    fn set(&mut self, tag: i32, letters: Vec<Letter>) {
        self.letters.retain(|l| l.tag != tag);
        self.letters
            .extend(letters.into_iter().map(|letter| TagLetter {
                tag,
                letter,
                active: false,
            }));
    }

    /// `DisplayText`.
    pub fn display(&mut self, text: &str, x: f32, y: f32, alpha: f32, tag: i32) {
        self.set(tag, display_text(text, x, y, alpha));
    }

    /// `CenterText`: the line centred on column 25.
    pub fn center(&mut self, text: &str, y: f32, alpha: f32, tag: i32) {
        let x = 25.0 - text.len() as f32 / 2.0;
        self.set(tag, display_text(text, x, y, alpha));
    }

    /// Turns on the letters not shown yet, giving each the alpha `alpha`
    /// returns for it (`None` keeps the one it was made with).
    pub(crate) fn activate(&mut self, alpha: impl Fn() -> Option<f32>) {
        for l in &mut self.letters {
            if !l.active {
                l.active = true;
                if let Some(a) = alpha() {
                    l.letter.alpha = a;
                }
            }
        }
    }

    fn clear(&mut self) {
        self.letters.clear();
    }

    /// The letters shown.
    pub fn shown(&self) -> Vec<Letter> {
        self.letters
            .iter()
            .filter(|l| l.active)
            .map(|l| l.letter)
            .collect()
    }
}

/// `MenuSelector`, the bar behind the chosen line: a sprite on the camera
/// at `(x, y, z)` with half-size `(half_width, 0.02)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub half_width: f32,
}

impl Selector {
    /// `PositionEntity MenuSelector, -.5+24.4*.02+.01,
    /// 0.372-Float(row)*.044+.027, 1.1` and `ScaleSprite width*.012, .02`.
    fn at_row(row: i32, width: f32) -> Self {
        Self {
            x: -0.5 + 24.4 * 0.02 + 0.01,
            y: 0.372 - row as f32 * 0.044 + 0.027,
            z: 1.1,
            half_width: width * 0.012,
        }
    }

    pub const HALF_HEIGHT: f32 = 0.02;
}

/// `MenuBkgStyle` 1, the "sphere of stars": what `MenuBkgUpdate` moves.
/// Positions are in Blitz space.
#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    /// Degrees the sky, both water planes and the mountains have turned.
    pub turn: f32,
    /// `LevelLight`'s colour and the ambient light, 0–255.
    pub light: [i32; 3],
    pub ambient: [i32; 3],
    /// `CameraFogColor`, 0–255.
    pub fog: [i32; 3],
    /// The sun (`MenuEntity(4)`).
    pub sun_at: [f32; 3],
    pub sun_colour: [i32; 3],
    /// The mountains' corner (`MenuEntity(3)`).
    pub terrain_at: [f32; 3],
    /// The upper water plane's height (`MenuEntity(2)`).
    pub water_y: f32,
    /// `PositionTexture` of the sky and the two water textures.
    pub tex: [[f32; 2]; 3],
    /// The 50 sparkles: position and `RotateSprite` angle.
    pub sparkles: Vec<([f32; 3], f32)>,
}

impl Default for Background {
    /// As `MenuSetup` leaves it.
    fn default() -> Self {
        Self {
            turn: 0.0,
            light: [0; 3],
            ambient: [0; 3],
            fog: [0; 3],
            sun_at: [0.0, -11.0, 128.0],
            sun_colour: [255, 255, 0],
            terrain_at: [-128.0, -5.3, -128.0],
            water_y: -5.0,
            tex: [[0.0; 2]; 3],
            sparkles: vec![([-50.0; 3], 0.0); 50],
        }
    }
}

fn sin(deg: f32) -> f32 {
    deg.to_radians().sin()
}

fn cos(deg: f32) -> f32 {
    deg.to_radians().cos()
}

impl Background {
    /// `MenuBkgUpdate` after `MenuTimer` has been counted up to `timer`.
    pub fn tick(&mut self, timer: i32, menu: i32) {
        if timer <= 0 {
            return;
        }
        let t = timer as f32;
        self.turn += 0.005;
        let a = 45.0 + t * 360.0 / 72000.0;
        // `Sin(45.0)`, as Blitz works it out in single precision.
        let s45 = FRAC_1_SQRT_2;
        self.terrain_at = [-cos(a) * s45 * 256.0, -5.3, -sin(a) * s45 * 256.0];
        self.tex = [
            [
                -((timer % 4000) as f32) / 4000.0,
                (timer % 100_000) as f32 / 100_000.0,
            ],
            [
                (timer % 24000) as f32 / 24000.0,
                -((timer % 6000) as f32) / 6000.0,
            ],
            [
                (timer % 12000) as f32 / 12000.0,
                -((timer % 3000) as f32) / 3000.0,
            ],
        ];
        self.water_y = -5.0 + 0.15 * sin((timer % 720) as f32 / 2.0);

        let b = t * 360.0 / 100_000.0;
        let sun = |y: f32| [-128.0 * sin(b), y, 128.0 * cos(b)];
        if menu == 12 {
            self.sun_at = sun(-10.5 + 40.0);
            self.light = [255; 3];
            self.ambient = [255; 3];
        } else if timer < 6400 {
            self.light = [timer / 25, timer / 50, timer / 50];
            self.ambient = [timer / 50; 3];
            self.fog = [
                128 * timer / 12800,
                175 * timer / 12800,
                255 * timer / 12800,
            ];
            self.sun_colour = [255, timer * 200 / 6400, 0];
            self.sun_at = sun(-10.5 + t * 40.0 / 12800.0);
        } else if timer < 12800 {
            self.light = [255, timer / 50, timer / 50];
            self.ambient = [timer / 50; 3];
            self.sun_at = sun(-10.5 + t * 40.0 / 12800.0);
        } else {
            self.sun_at = sun(-10.5 + 40.0);
            self.light = [255; 3];
            self.ambient = [255; 3];
        }

        for (i, s) in self.sparkles.iter_mut().enumerate() {
            let fi = i as f32;
            let u = ((t - 125.0 * fi) / 50.0) % 360.0;
            let v = ((t - 125.0 * fi) / 20.0) % 360.0;
            let k = i as i32;
            s.0 = [
                40.0 * sin(u) + 13.0 * sin((35 * k) as f32),
                15.0 - 5.0 * cos(v) - 2.0 * cos((47 * k) as f32),
                -80.0 * cos(u) + 17.0 * sin((51 * k) as f32),
            ];
            s.1 = (timer + k * 47) as f32;
        }
    }
}

/// What a square on the level selector shows: a part of
/// `graphics\levelsigns2.bmp` (`WorldLevelTexture`), or
/// `graphics\levelperfect.bmp` for a perfect level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    /// A level not won yet, or the closed gate: the top left quarter.
    NotWon,
    /// The top right quarter.
    Won,
    /// The whole perfect-level picture.
    Perfect,
    /// The open gate forward: the bottom right quarter.
    Forward,
    /// The gate back: the bottom left quarter.
    Back,
}

/// A square on the level selector (`CurrentWorldLevelEntity`): a flat
/// square at depth 4 facing the camera, half a side 0.15, unlit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelSquare {
    pub x: f32,
    pub y: f32,
    pub square: Square,
}

/// The level selector's cursor (`CurrentWorldLevelEntity(12)`,
/// `graphics\levelsigns2b.bmp`): at depth 3.99, half a side 0.16, turned
/// 3 degrees a tick in the screen's plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelCursor {
    pub x: f32,
    pub y: f32,
    pub turn: f32,
}

/// Fresh presses from held keys: what each `MenuLoop` does with
/// `CursorKeyPressed`.
enum Press {
    Enter,
    Escape,
}

/// The menus' state: the globals `menu.bb` keeps between frames.
#[derive(Debug, Clone)]
pub struct Menu {
    /// The ending in progress.
    pub ending: Option<EndScene>,
    /// `CurrentMenu`.
    pub current: i32,
    /// `CurrentMenuOption`.
    pub option: i32,
    /// `MenuTimer`: counts ticks of `MenuBkgUpdate`, from -500 on the
    /// title.
    pub timer: i32,
    /// `LevelTimer`, which some menus count up and the main menu's hint
    /// blinks by.
    pub level_timer: i32,
    /// `CursorKeyPressed`: a flag in most menus, a count of the frames
    /// a cursor key is held in the custom levels.
    cursor_key: i32,
    /// `SecretMusic`: frames since 1 was pressed on the options menu.
    secret_music: i32,
    /// `VersionNo1`: the major version, shown on the main menu, and set
    /// to a negative code to pick what opens on exit.
    pub version_no1: i32,
    /// `Language`, from the options.
    pub language: i32,
    /// The `players` folder; `None` lists no players and saves nothing.
    pub players: Option<Players>,
    /// `PlayerName$` and the rest of the select screen's list.
    list: Vec<Entry>,
    /// `Player2BDeleted`.
    to_delete: usize,
    /// `CurrentPlayerName$` as it is typed.
    pub player_name: String,
    /// What `GetKey` returns: the characters typed, one taken per frame.
    typed: VecDeque<u8>,
    /// What the player last used: the prompts and the name entry follow it.
    pub device: Device,
    /// The letter picker's cell on the new-player screen.
    picker: usize,
    /// `MilliSecs()`, for a new player's id.
    pub millisecs: i32,
    /// The player chosen or created. Its `world` is `CurrentWorld`.
    pub profile: Option<Profile>,
    /// `CurrentLevel`: [`custom::LEVEL`] while a custom level is played.
    pub level: i32,
    /// `CurrentLevelBackup`: `CurrentLevel` before a custom level.
    level_backup: i32,
    /// `WhichWorldPack`.
    pub pack: i32,
    /// `LockWorlds` from the options: 1 opens every world's gate.
    lock_worlds: i32,
    /// The world table.
    pub worlds: Worlds,
    /// TOW's data folder is there, so its levels can be played.
    pub tow: bool,
    /// Each level's title as its file stores it (`#` between lines), by
    /// level number; empty when it could not be read.
    pub level_titles: Vec<String>,
    /// `CurrentWorldLevelN`.
    level_count: i32,
    /// `CurrentWorldLevelName$`: three lines for each square. Kept from
    /// one world to the next, as lines the selector does not set are.
    level_names: Vec<[String; 4]>,
    /// The `CustomLevels` folder; `None` lists nothing.
    pub custom: Option<CustomLevels>,
    /// `FileDir$` and the rest of the custom level list.
    pub files: Vec<FileEntry>,
    /// `FileDirPos`: the list's first line on screen.
    pub file_pos: i32,
    /// `CurrentExtraLevelDirectory$`, one folder name (upper case) each.
    pub custom_dir: Vec<String>,
    /// `CurrentFileDir`: the custom level played, by its place in the
    /// list.
    pub current_file: i32,
    /// `CustomSortByName`.
    sort_by_name: bool,
    /// `ExtraLevelMusicTrack`: the music track of the last level loaded
    /// that names one.
    pub extra_music: i32,
    /// The level selector's squares and cursor, while it is shown.
    pub squares: Vec<LevelSquare>,
    pub cursor: Option<LevelCursor>,
    pub text: Text,
    /// `Logo1`, `Logo2` shown.
    pub logos: bool,
    /// The selector, when shown.
    pub selector: Option<Selector>,
    selector_shown: bool,
    selector_at: Option<Selector>,
    pub background: Background,
    actions: Vec<Action>,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            ending: None,
            current: TITLE,
            option: 0,
            timer: 0,
            level_timer: 0,
            cursor_key: 0,
            secret_music: 0,
            version_no1: 10,
            language: 0,
            players: None,
            list: Vec::new(),
            to_delete: 0,
            player_name: String::new(),
            typed: VecDeque::new(),
            device: Device::Keyboard,
            picker: 0,
            millisecs: 0,
            profile: None,
            level: 0,
            level_backup: 0,
            pack: 0,
            lock_worlds: 0,
            worlds: Worlds::game(),
            tow: true,
            level_titles: vec![String::new(); LEVELS],
            level_count: 0,
            level_names: vec![Default::default(); 13],
            custom: None,
            files: Vec::new(),
            file_pos: 0,
            custom_dir: Vec::new(),
            current_file: 0,
            sort_by_name: true,
            extra_music: 1,
            squares: Vec::new(),
            cursor: None,
            text: Text::default(),
            logos: false,
            selector: None,
            selector_shown: false,
            // `MenuSetup` puts it far above the screen.
            selector_at: None,
            background: Background::default(),
            actions: Vec::new(),
        }
    }
}

impl Menu {
    /// The menus as the game starts them: `MenuSetup`, then
    /// `StartMenu(0)`. The title music comes with the first frame.
    pub fn new() -> Self {
        let mut m = Self::default();
        m.start(TITLE);
        m
    }

    /// `StartMenu(menu)`, for the menus this step ports.
    pub fn start(&mut self, menu: i32) {
        self.option = 0;
        self.logos = menu == TITLE || menu == MAIN;
        if !matches!(menu, 4 | 5 | 8 | 12 | 16) {
            self.selector_shown = true;
        }
        if menu == TITLE {
            self.timer = -500;
            self.background.light = [0; 3];
            self.background.ambient = [0; 3];
            self.actions.push(Action::TitleMusic);
        } else if menu == MAIN {
            // Not when coming back from a menu that kept it playing.
            if !matches!(self.current, 7 | 9 | 10 | 0 | 15) {
                self.actions.push(Action::TitleMusic);
            }
        } else if menu == PLAYER_SELECT {
            self.actions.push(Action::StopMusic);
            self.list = self.list_players();
            // No players: straight to entering a name.
            if self.list.is_empty() {
                self.start(NEW_PLAYER);
                return;
            }
        } else if menu == NEW_PLAYER {
            self.actions.push(Action::StopMusic);
            self.player_name.clear();
            self.picker = 0;
            // `GetKey`'s queue holds every key typed since the game
            // started; only what is typed on this screen counts here.
            self.typed.clear();
        } else if menu == LEVEL_SELECT {
            if self.level == custom::LEVEL {
                self.level = self.level_backup;
            }
            if self.current == LEVEL_PACKS {
                self.actions.push(Action::LevelMusic(self.world()));
            }
            self.make_squares();
        } else if menu == PLAYGROUND {
            // TOW's `StartMenu(8)`: the music starts on the way in, and
            // the cursor is always on the first level.
            if self.current != PLAYGROUND {
                self.actions.push(Action::LevelMusic(PLAYGROUND_LEVEL / 10));
            }
            self.level_timer = 0;
            self.level = PLAYGROUND_LEVEL;
            self.make_playground_squares();
        } else if menu == ENDING || menu == ENDING_DELUXE {
            self.level_timer = 0;
            self.ending = Some(EndScene::new(menu == ENDING_DELUXE));
            self.actions.push(if menu == ENDING {
                Action::EndingMusic
            } else {
                Action::TitleMusic
            });
        } else if menu == LEVEL_PACKS {
            self.level_timer = 0;
            self.actions.push(Action::StopMusic);
            self.extra_music = 0;
            self.current_file = 0;
        } else if menu == CUSTOM_LEVELS {
            self.typed.clear();
            self.list_files();
        }
        self.current = menu;
        self.update_selector();
        self.actions.push(Action::Entered(menu));
    }

    /// Asks for TOW's ending, `skip` ticks in.
    pub fn start_tow_ending(&mut self, skip: u32) {
        self.actions.push(Action::TowEnding { skip });
    }

    /// A character typed (what `GetKey` returns), for the name entry.
    pub fn type_char(&mut self, c: u8) {
        if matches!(self.current, NEW_PLAYER | CUSTOM_LEVELS) {
            self.typed.push_back(c);
        }
    }

    /// `StartMenu(2)`'s list of the players on disk.
    fn list_players(&mut self) -> Vec<Entry> {
        let Some(players) = &self.players else {
            return Vec::new();
        };
        players.list().unwrap_or_else(|e| {
            self.actions
                .push(Action::Warn(format!("{}: {e}", players.dir.display())));
            Vec::new()
        })
    }

    /// Takes the keys held now as already pressed, as the original does
    /// by waiting for Esc to be let go before it opens a menu.
    pub fn hold_keys(&mut self) {
        self.cursor_key = 1;
    }

    /// `EndMenu`: hides the logos and selector and frees the letters.
    pub fn end(&mut self) {
        self.logos = false;
        self.selector_shown = false;
        self.update_selector();
        self.text.clear();
    }

    fn update_selector(&mut self) {
        self.selector = self.selector_shown.then_some(self.selector_at).flatten();
    }

    fn place_selector(&mut self, row: i32, width: f32) {
        self.selector_at = Some(Selector::at_row(row, width));
        self.update_selector();
    }

    /// One frame of `MenuLoop`: `ticks` ticks of the background, then the
    /// keys. The options menu changes `options` in place.
    pub fn frame(
        &mut self,
        keys: MenuKeys,
        ticks: u32,
        options: &mut ConfigOptions,
    ) -> Vec<Action> {
        self.language = options.language;
        self.lock_worlds = options.lock_worlds;
        match self.current {
            TITLE => self.title(keys, ticks),
            MAIN => self.main(keys, ticks),
            PLAYER_SELECT => self.player_select(keys, ticks),
            NEW_PLAYER => self.new_player(keys, ticks),
            LEVEL_SELECT => self.level_select(keys, ticks),
            PLAYGROUND => self.playground(keys, ticks),
            DELETE_PLAYER => self.delete_player(keys, ticks),
            OPTIONS => self.options(keys, ticks, options),
            EXIT => self.exit(keys, ticks),
            LEVEL_PACKS => self.level_packs(keys, ticks),
            CUSTOM_LEVELS => self.custom_levels(keys, ticks),
            ONLINE => self.online(keys, ticks),
            ENDING => self.ending(keys, ticks),
            ENDING_DELUXE => self.ending_deluxe(keys, ticks),
            // Ported in later steps.
            _ => self.run_ticks(ticks, false),
        }
        std::mem::take(&mut self.actions)
    }

    /// The tick loop: `MenuBkgUpdate` per tick, and `LevelTimer` in the
    /// menus that count it.
    fn run_ticks(&mut self, ticks: u32, level_timer: bool) {
        for _ in 0..ticks {
            if level_timer {
                self.level_timer += 1;
            }
            self.timer += 1;
            self.background.tick(self.timer, self.current);
        }
    }

    fn sound(&mut self) {
        self.actions.push(Action::Sound(MenuSound::Move));
    }

    /// The cursor keys, Enter and Escape as every list menu reads them,
    /// with `count` options.
    fn navigate(&mut self, keys: MenuKeys, count: i32) -> Option<Press> {
        if keys.up {
            if self.cursor_key == 0 {
                self.sound();
                self.option -= 1;
                if self.option == -1 {
                    self.option = count - 1;
                }
                self.cursor_key = 1;
            }
        } else if keys.down {
            if self.cursor_key == 0 {
                self.sound();
                self.option += 1;
                if self.option == count {
                    self.option = 0;
                }
                self.cursor_key = 1;
            }
        } else if keys.enter {
            if self.cursor_key == 0 {
                self.sound();
                self.cursor_key = 1;
                return Some(Press::Enter);
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                return Some(Press::Escape);
            }
        } else {
            self.cursor_key = 0;
        }
        None
    }

    fn quit(&mut self) {
        let url = exit_url(self.version_no1);
        self.actions.push(Action::Quit { url });
    }

    /// `MenuLoop0`: the credits and the story over the sunrise.
    fn title(&mut self, keys: MenuKeys, ticks: u32) {
        let t = self.timer;
        let credits: [(&str, &str); 6] = [
            ("A Midnight Synergy Game", ""),
            ("Program, Visuals, and SFX", "Patrick Maidorn"),
            ("Music", "Jonne 'Purple Motion' Valtonen"),
            ("Additional 3D Modeling", "Danny Hawrysio"),
            ("Software Development Consultant", "Claire Polster"),
            ("And Many Thanks", "To The Entire Wonderland Community!"),
        ];
        let c = t % 20000;
        if c < 8500 {
            self.text.center("", 9.0, 1.0, 50);
            self.text.center("", 9.0, 1.0, 51);
        } else if c < 14500 {
            let (a, b) = credits[((c - 8500) / 1000) as usize];
            self.text.center(a, 7.5, 1.0, 50);
            if b.is_empty() {
                self.text.center("", 9.0, 1.0, 51);
            } else {
                self.text.center(b, 8.5, 1.0, 51);
            }
        }

        let tx = &mut self.text;
        if t < 0 {
            tx.center("And now", 15.0, 1.0, 55);
            tx.center("our journey continues...", 16.0, 1.0, 56);
        } else if t < 1500 {
            tx.center("", 9.0, 1.0, 55);
            tx.center("", 9.0, 1.0, 56);
        } else if t < 2500 {
            tx.center("As a new day", 15.0, 1.0, 55);
            tx.center("dawns over Wonderland,", 16.0, 1.0, 56);
            tx.center("", 9.0, 1.0, 57);
        } else if t < 3500 {
            tx.center("its inhabitants wake up", 15.0, 1.0, 55);
            tx.center("to troubling news.", 16.0, 1.0, 56);
            tx.center("", 9.0, 1.0, 57);
        } else if t < 4500 {
            tx.center("The Rainbow Coins,", 15.0, 1.0, 55);
            tx.center("whose magic keeps Wonderland safe,", 16.0, 1.0, 56);
        } else if t < 5500 {
            // The tags swap here, so tag 55 keeps its faded line until
            // 8500.
            tx.center("have been scattered", 15.0, 1.0, 56);
            tx.center("to the far corners of the world!", 16.0, 1.0, 55);
        } else if t < 6500 {
            tx.center("Once again,", 15.0, 1.0, 56);
            tx.center("Wonderland's existence is at risk.", 16.0, 1.0, 57);
        } else if t < 6700 {
            tx.center("And, once again,", 15.0, 1.0, 56);
            tx.center("", 16.0, 1.0, 57);
        } else if t < 6900 {
            tx.center("And, once again,", 15.0, 1.0, 56);
            tx.center("Stinky and Loof", 16.0, 1.0, 57);
        } else if t < 7500 {
            tx.center("And, once again,", 15.0, 1.0, 56);
            tx.center("Stinky and Loof", 16.0, 1.0, 57);
            tx.center("must save the day!", 17.0, 1.0, 58);
        } else if t < 8500 {
            tx.center("", 9.0, 1.0, 55);
            tx.center("", 9.0, 1.0, 56);
            tx.center("", 9.0, 1.0, 57);
        } else {
            let start = menu_prompt("(Press Space Key To Start)", self.device);
            tx.center(&start, 16.0, 1.0, 57);
        }

        // Fade in and out: before 0 for the opening line, then a second
        // out of every ten in steps of 200 ticks. Nothing shows in 0–500.
        if t < 0 {
            self.text.activate(|| {
                let mut a = None;
                if t < -300 {
                    a = Some((500.0 + t as f32) / 200.0);
                }
                if t > -200 {
                    a = Some(-(t as f32) / 200.0);
                }
                a
            });
        } else if t > 500 {
            let p = (t - 500) % 1000;
            let f = ((t - 500) % 200) as f32 / 200.0;
            self.text.activate(|| {
                let mut a = None;
                if p < 200 {
                    a = Some(f);
                }
                if p > 800 {
                    a = Some(1.0 - f);
                }
                a
            });
        }

        self.run_ticks(ticks, false);

        // `CursorKeyPressed` is never cleared here.
        if (keys.escape || keys.space || keys.enter) && self.cursor_key == 0 {
            self.cursor_key = 1;
            self.end();
            if self.timer < 0 {
                self.timer = 0;
            }
            self.start(MAIN);
        }
    }

    /// `MenuLoop1`: the main menu.
    fn main(&mut self, keys: MenuKeys, ticks: u32) {
        let version = format!("v{}.00", self.version_no1);
        self.text.set(
            50,
            pinned(display_text(&version, 44.0, 0.0, 1.0), Edge::Right),
        );
        let (items, hint) = match self.language {
            1 => (
                [
                    "SPIEL STARTEN",
                    "OPTIONEN",
                    "ONLINE HILFE",
                    "ONLINE FORUM",
                    "SPIEL BEENDEN",
                ],
                "Benutze die Pfeiltasten und zur Auswahl 'Enter'",
            ),
            2 => (
                [
                    "JOUER",
                    "OPTIONS",
                    "SOLUCES EN LIGNE",
                    "FORUM EN LIGNE",
                    "SORTIE",
                ],
                "Pour choisir, utilise les fleches et 'Entree'",
            ),
            _ => (
                [
                    "START GAME",
                    "OPTIONS",
                    "ONLINE SOLUTIONS",
                    "LEVEL EXCHANGE",
                    "EXIT",
                ],
                "Use Cursor Keys and Enter to Select",
            ),
        };
        for (i, item) in items.iter().enumerate() {
            self.text.center(item, 9.0 + i as f32, 1.0, 51 + i as i32);
        }
        if self.level_timer % 200 < 100 {
            let hint = menu_prompt(hint, self.device);
            self.text.center(&hint, 17.0, 1.0, 57);
        } else {
            self.text.center("", 17.0, 1.0, 57);
        }
        self.place_selector(9 + self.option, 25.0);
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

        match self.navigate(keys, 5) {
            Some(Press::Enter) => match self.option {
                0 => {
                    self.end();
                    self.start(PLAYER_SELECT);
                }
                1 => {
                    self.end();
                    self.start(OPTIONS);
                }
                2 | 3 => {
                    self.end();
                    self.version_no1 = if self.option == 2 { -999 } else { -998 };
                    self.start(ONLINE);
                }
                _ => {
                    self.end();
                    self.start(EXIT);
                }
            },
            Some(Press::Escape) => {
                self.end();
                self.start(EXIT);
            }
            None => {}
        }
        if keys.one && keys.two && keys.three {
            self.end();
            self.cursor_key = 1;
            self.start(ENDING);
        }
    }

    /// Esc as the endings read it, back to the main menu.
    fn leave_ending(&mut self, keys: MenuKeys) {
        if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.ending = None;
                self.end();
                self.start(MAIN);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// `MenuLoop12`: the thanks and the cast's names, then the farewell.
    fn ending(&mut self, keys: MenuKeys, ticks: u32) {
        let t2 = self.level_timer % 7800;
        let tx = &mut self.text;
        if t2 < 6000 {
            tx.center("Congratulations!!!", 2.0, 1.0, 52);
            tx.center("You have completed", 4.0, 1.0, 53);
            tx.center("Return To Wonderland", 5.0, 1.0, 50);
            tx.center("Thank you for playing!", 7.0, 1.0, 51);
            tx.center("Special Thanks from", 15.0, 1.0, 54);
            let name = [
                "Stinky!",
                "Loof!",
                "Scouge!",
                "Coily!",
                "Kaboom!",
                "Z-Bot!",
                "The Flying Saucer!",
                "Red Chomper!",
                "Gold Chomper!",
                "and Ghost!",
            ][(t2 / 600) as usize];
            tx.center(name, 16.0, 1.0, 55);
        } else {
            tx.center("Well Done!", 6.0, 1.0, 50);
            tx.center("We hope you enjoyed the Game!", 8.0, 1.0, 51);
            tx.center("Best Wishes from your Friends at", 10.0, 1.0, 52);
            tx.center("Midnight Synergy", 11.0, 1.0, 53);
            tx.center("www.midnightsynergy.com", 13.0, 1.0, 54);
            tx.center("", 14.0, 1.0, 55);
        }
        tx.activate(|| None);
        for _ in 0..ticks {
            self.level_timer += 1;
            if let Some(e) = &mut self.ending {
                e.tick(self.level_timer);
            }
        }
        self.leave_ending(keys);
    }

    /// `MenuLoop16`: the story's end, a page every 1000 ticks. The last
    /// page stays.
    fn ending_deluxe(&mut self, keys: MenuKeys, ticks: u32) {
        let page = match self.level_timer / 1000 {
            0 | 10 => Some(["", "", ""]),
            1 => Some(["", "And so,", ""]),
            2 => Some(["", "after a long journey", ""]),
            3 => Some(["the Rainbow Spirits", "returned to the", "Rainbow World"]),
            4 => Some(["and balance", "was once again restored", "in Wonderland."]),
            5 => Some(["", "What happened next?", ""]),
            6 => Some([
                "Mayor Peegue returned to",
                "Wondertown and helped fellow",
                "Wonderlanders rebuild their city.",
            ]),
            7 => Some([
                "Qookie went back to the cottage,",
                "made friends with the wild wood chompers,",
                "and shared some giant mushrooms with them.",
            ]),
            8 => Some(["Stinky and Loof", "briefly considered retirement.", ""]),
            9 => Some([
                "But another adventure",
                "was sure to be",
                "just around the corner.",
            ]),
            11 => Some(["THE END", "", "Thank you for playing!"]),
            _ => None,
        };
        if let Some(page) = page {
            let tx = &mut self.text;
            tx.center(page[0], 1.0, 1.0, 52);
            tx.center(page[1], 2.0, 1.0, 53);
            tx.center(page[2], 3.0, 1.0, 50);
        }
        self.text.activate(|| None);
        for _ in 0..ticks {
            if let Some(e) = &mut self.ending {
                e.tick(self.level_timer);
            }
            self.level_timer += 1;
        }
        self.leave_ending(keys);
    }

    /// `MenuLoop2`: CREATE NEW PLAYER, then the players on disk with
    /// their levels, custom levels and score. Delete asks to delete the
    /// chosen player.
    fn player_select(&mut self, keys: MenuKeys, ticks: u32) {
        let tx = &mut self.text;
        match self.language {
            1 => {
                tx.center("SPIELERAUSWAHL", 4.0, 1.0, 50);
                tx.center("------------------------------", 5.0, 1.0, 51);
                tx.display("Name             Levels Punkte", 10.0, 6.0, 1.0, 52);
                tx.display("NEUER SPIELER", 10.0, 7.0, 1.0, 53);
            }
            2 => {
                tx.center("CHOISIS LE JOUEUR", 4.0, 1.0, 50);
                tx.center("------------------------------", 5.0, 1.0, 51);
                tx.display("Nom              Niveau  Score", 10.0, 6.0, 1.0, 52);
                tx.display("CREER NOUVEAU JOUEUR", 10.0, 7.0, 1.0, 53);
            }
            _ => {
                tx.center("SELECT PLAYER", 4.0, 1.0, 50);
                tx.center(RULE_46, 5.0, 1.0, 51);
                tx.display(
                    "Name             Levels  Custom    Total Score",
                    2.0,
                    6.0,
                    1.0,
                    52,
                );
                tx.display("CREATE NEW PLAYER", 2.0, 7.0, 1.0, 53);
            }
        }
        // Numbers end before the column given.
        let number = |tx: &mut Text, v: i32, end: i32, y: f32, tag: i32| {
            let s = v.to_string();
            tx.display(&s, (end - s.len() as i32) as f32, y, 1.0, tag);
        };
        let n = self.list.len() as i32;
        for (i, e) in self.list.iter().enumerate() {
            let (i, y) = (i as i32, 8.0 + i as f32);
            tx.display(&e.file_stem.to_ascii_uppercase(), 2.0, y, 1.0, 54 + 4 * i);
            number(tx, e.levels, 25, y, 55 + 4 * i);
            number(tx, e.extra_levels, 33, y, 57 + 4 * i);
            number(tx, e.score, 48, y, 56 + 4 * i);
        }
        tx.center(RULE_46, 8.0 + n as f32, 1.0, 55 + 4 * n);
        self.place_selector(7 + self.option, 48.5);
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

        // `KeyDown(211)` comes after Enter and before Esc.
        if keys.delete && !(keys.up || keys.down || keys.enter) {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if self.option > 0 {
                    self.to_delete = (self.option - 1) as usize;
                    self.end();
                    self.start(DELETE_PLAYER);
                }
            }
            return;
        }
        match self.navigate(keys, n + 1) {
            Some(Press::Enter) if self.option == 0 => {
                // With ten players there is no room for another.
                if self.list.len() < MAX_PLAYERS {
                    self.end();
                    self.start(NEW_PLAYER);
                }
            }
            Some(Press::Enter) => {
                self.end();
                let entry = &self.list[(self.option - 1) as usize];
                let loaded = match &self.players {
                    Some(p) => p.load(entry),
                    None => Err("no players folder".to_owned()),
                };
                match loaded {
                    Ok(profile) => {
                        self.profile = Some(profile);
                        self.start(LEVEL_PACKS);
                    }
                    Err(e) => {
                        self.actions.push(Action::Warn(e));
                        self.start(PLAYER_SELECT);
                    }
                }
            }
            Some(Press::Escape) => {
                self.end();
                self.start(MAIN);
            }
            None => {}
        }
    }

    /// `MenuLoop13`: the level packs. The description under the chosen
    /// pack scrolls up as `LevelTimer` counts frames to 50. Enter opens
    /// the pack's level selector in the world last played there, or the
    /// custom levels; Esc goes back to the main menu.
    ///
    /// TOW's levels and playground are two more lines before the custom
    /// levels; without TOW's data folder they only say no.
    fn level_packs(&mut self, keys: MenuKeys, ticks: u32) {
        const ABOUT: [[&str; 4]; 6] = [
            [
                "A set of twenty tutorial levels:",
                "Learn the ins and outs of basic gameplay",
                "and practice your Wonderland skills",
                "with a set of easy levels.",
            ],
            [
                "Take Stinky and Loof on and adventure",
                "through ten worlds and eighty-five",
                "diverse levels of increasing difficulty.",
                "",
            ],
            [
                "Fifty levels featuring new gameplay elements",
                "and a brand new story: Strange Rainbow",
                "Spirits have appeared in Wonderland. Can Stinky,",
                "Loof, and their new friends solve the mystery?",
            ],
            [
                "Where it all began: Stinky and Loof's first",
                "adventure, The Original Wonderland,",
                "with ten worlds and eighty levels.",
                "",
            ],
            [
                "Levels for very young or less experienced",
                "players: exploration, discovery and simple",
                "puzzles. Also contains the five bonus",
                "levels of The Original Wonderland.",
            ],
            [
                "Hundreds of custom levels created by other",
                "Return To Wonderland players.",
                "Visit the online community and add more",
                "free custom levels anytime!",
            ],
        ];
        let tx = &mut self.text;
        tx.center("SELECT LEVEL PACK", 2.0, 1.0, 50);
        tx.center("-----------------", 3.0, 1.0, 51);
        tx.center("Tutorial Levels", 7.0, 1.0, 52);
        tx.center("Classic Levels", 8.0, 1.0, 53);
        tx.center("Deluxe Levels", 9.0, 1.0, 54);
        tx.center("Original Levels", 10.0, 1.0, 55);
        tx.center("The Playground", 11.0, 1.0, 64);
        tx.center("Custom Levels", 12.0, 1.0, 56);
        if self.level_timer < 50 {
            self.level_timer += 1;
        }
        if let Some(lines) = ABOUT.get(self.option as usize) {
            let rise = self.level_timer as f32 / 10.0;
            for (i, line) in lines.iter().enumerate() {
                // The classic pack's empty last line rises at half speed.
                let rise = if self.option == 1 && i == 3 {
                    rise / 2.0
                } else {
                    rise
                };
                let i = i as i32;
                tx.center(line, (19 + i) as f32 - rise, 1.0, 57 + i);
            }
        }
        self.place_selector(7 + self.option, 21.5);
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

        if keys.up {
            if self.cursor_key == 0 {
                self.sound();
                self.option -= 1;
                if self.option == -1 {
                    self.option = 5;
                }
                self.cursor_key = 1;
                self.level_timer = 0;
            }
        } else if keys.down {
            if self.cursor_key == 0 {
                self.sound();
                self.option += 1;
                if self.option == 6 {
                    self.option = 0;
                }
                self.cursor_key = 1;
                self.level_timer = 0;
            }
        } else if keys.enter {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if matches!(self.option, 3 | 4) && !self.tow {
                    self.actions.push(Action::Sound(MenuSound::No));
                    return;
                }
                self.end();
                if self.option == 4 {
                    self.start(PLAYGROUND);
                } else if self.option < 4 {
                    // The tutorials are pack 1 and the classic levels 0;
                    // TOW's worlds are pack 3.
                    self.pack = match self.option {
                        0 => 1,
                        1 => 0,
                        3 => TOW_PACK,
                        p => p,
                    };
                    let pack = self.pack as usize;
                    if let Some(p) = &mut self.profile {
                        p.world = p.world_packs[pack];
                    }
                    self.level = self.world() * 10;
                    self.start(LEVEL_SELECT);
                } else {
                    self.start(CUSTOM_LEVELS);
                }
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.end();
                self.start(MAIN);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// `StartMenu(14)`: lists the folder `CurrentExtraLevelDirectory$`
    /// with the player's scores, and puts the cursor on `CurrentFileDir`.
    fn list_files(&mut self) {
        let extras = self.profile.as_ref().map_or(&[][..], |p| &p.extra);
        let listed = self.custom.as_ref().map(|c| {
            c.list(&self.custom_dir, self.sort_by_name, extras)
                .map_err(|e| format!("{}: {e}", c.dir.display()))
        });
        self.files = match listed {
            Some(Ok(files)) => files,
            Some(Err(e)) => {
                self.actions.push(Action::Warn(e));
                vec![custom::back()]
            }
            None => vec![custom::back()],
        };
        self.file_pos = 0;
        if self.current_file > 14 {
            self.file_pos = self.current_file - 14;
            self.option = 14;
        } else {
            self.option = self.current_file;
        }
    }

    /// The game path of the custom level `CurrentFileDir`.
    pub fn custom_path(&self) -> Option<String> {
        let e = self.files.get(usize::try_from(self.current_file).ok()?)?;
        Some(CustomLevels::game_path(&self.custom_dir, &e.name))
    }

    /// The custom level `CurrentFileDir`, while one is played.
    pub fn custom_file(&self) -> Option<&FileEntry> {
        if self.level != custom::LEVEL {
            return None;
        }
        self.files.get(usize::try_from(self.current_file).ok()?)
    }

    /// A custom level was loaded with `track` as its music track (v4 and
    /// later name one): the track to start when `ExtraLevelMusicTrack`
    /// changed.
    pub fn custom_music(&mut self, track: Option<i32>) -> Option<i32> {
        let before = self.extra_music;
        if let Some(t) = track {
            self.extra_music = t;
        }
        (self.extra_music != before).then_some(self.extra_music)
    }

    /// `MenuLoop14`: the custom levels, 15 lines at a time. The cursor
    /// keys repeat while held; a letter jumps to the first level from it
    /// on; Enter goes into a folder, back up, or plays the level; F1 and
    /// F2 list by name or not; Esc goes back to the level packs.
    fn custom_levels(&mut self, keys: MenuKeys, ticks: u32) {
        let n = self.files.len() as i32;
        let tx = &mut self.text;
        tx.display(
            "FILENAME:      LEVELNAME:                 SCORE:",
            1.0,
            1.0,
            1.0,
            51,
        );
        tx.display(RULE_48, 1.0, 2.0, 1.0, 52);
        for i in 0..15 {
            let Some(e) = self.files.get((self.file_pos + i) as usize) else {
                continue;
            };
            let y = (i + 3) as f32;
            let tag = 100 + i * 4;
            let name = if custom::is_level_name(&e.name) {
                &e.name[..e.name.len() - 4]
            } else {
                &e.name
            };
            tx.display(&left(name, 14), 1.0, y, 1.0, tag);
            if e.title.chars().count() < 27 {
                tx.display(&e.title, 16.0, y, 1.0, tag + 1);
            } else {
                tx.display(&(left(&e.title, 24) + "..."), 16.0, y, 1.0, tag + 1);
            }
            // A folder or the way back leaves the column as it was.
            if e.score == 0 {
                tx.display("-----", 44.0, y, 1.0, tag + 2);
            } else if e.score > 0 {
                tx.display(&left(&e.score.to_string(), 5), 44.0, y, 1.0, tag + 2);
            }
            let star = if e.perfect == 1 { "*" } else { "" };
            tx.display(star, 49.0, y, 1.0, tag + 3);
        }
        self.place_selector(3 + self.option, 50.0);
        self.text.activate(|| None);

        self.run_ticks(ticks, true);

        // `GetKey`: a letter goes to the first level from it on.
        if let Some(c) = self.typed.pop_front()
            && c.is_ascii_alphabetic()
        {
            let a = [c.to_ascii_uppercase()];
            let mut i = (0..=n)
                .find(|&i| {
                    self.files
                        .get(i as usize)
                        .is_some_and(|e| a[..] <= *e.name.as_bytes() && e.score >= 0)
                })
                .unwrap_or(n + 1);
            if i >= n {
                i = n - 1;
            }
            if n - i >= 15 {
                self.file_pos = i;
                self.option = 0;
            } else {
                self.file_pos = n - 15;
                if self.file_pos < 0 {
                    self.file_pos = 0;
                    self.option = i;
                } else {
                    self.option = 15 - (n - i);
                }
            }
        }

        // Held, the cursor moves on the first frame, then on every
        // second frame from the sixth.
        let repeat = self.cursor_key == 0 || (self.cursor_key > 5 && self.cursor_key % 2 == 0);
        if keys.up {
            if repeat {
                self.sound();
                self.option -= 1;
                if self.option == -1 {
                    self.option = 0;
                    if self.file_pos > 0 {
                        self.file_pos -= 1;
                    }
                }
            }
            self.cursor_key += 1;
        } else if keys.down {
            if repeat {
                self.sound();
                if self.option < n - 1 {
                    self.option += 1;
                }
                if self.option == 15 {
                    self.option = 14;
                    if self.file_pos + 15 < n {
                        self.file_pos += 1;
                    }
                }
            }
            self.cursor_key += 1;
        } else if keys.page_up {
            if self.cursor_key == 0 {
                self.sound();
                self.file_pos -= 15;
                if self.file_pos < 0 {
                    self.option += self.file_pos;
                    self.file_pos = 0;
                    if self.option < 0 {
                        self.option = 0;
                    }
                }
                self.cursor_key = 1;
            }
        } else if keys.page_down {
            if self.cursor_key == 0 {
                self.sound();
                if n > 15 {
                    self.file_pos += 15;
                    if self.file_pos + 15 >= n {
                        self.file_pos = n - 15;
                        self.option = 14;
                    }
                } else {
                    self.option = n - 1;
                }
                self.cursor_key = 1;
            }
        } else if keys.enter {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.open_file();
            }
        } else if keys.f1 || keys.f2 {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.end();
                self.sort_by_name = keys.f1;
                self.start(CUSTOM_LEVELS);
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.end();
                self.start(LEVEL_PACKS);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// Enter on the custom levels' line under the cursor.
    fn open_file(&mut self) {
        let at = self.option + self.file_pos;
        let Some(e) = self.files.get(at as usize).cloned() else {
            return;
        };
        if e.score == custom::BACK {
            self.end();
            if self.custom_dir.pop().is_none() {
                self.start(LEVEL_PACKS);
            } else {
                self.current_file = 0;
                self.start(CUSTOM_LEVELS);
            }
            self.sound();
        } else if e.score == custom::DIR {
            self.end();
            self.custom_dir.push(e.name);
            self.current_file = 0;
            self.start(CUSTOM_LEVELS);
            self.sound();
        } else if e.is_level() {
            self.end();
            if self.level != custom::LEVEL {
                self.level_backup = self.level;
            }
            self.level = custom::LEVEL;
            self.current_file = at;
            self.actions.push(Action::StartLevel(custom::LEVEL));
        }
    }

    /// `CurrentWorld`.
    pub fn world(&self) -> i32 {
        self.profile.as_ref().map_or(0, |p| p.world)
    }

    fn set_world(&mut self, world: i32) {
        if let Some(p) = &mut self.profile {
            p.world = world;
        }
    }

    fn scores(&self) -> &[i32] {
        self.profile.as_ref().map_or(&[], |p| &p.scores)
    }

    fn perfect(&self, level: i32) -> i32 {
        let p = self.profile.as_ref().map_or(&[][..], |p| &p.perfect);
        worlds::score(p, level)
    }

    /// The gate forward is open (`NextWorldFlag=0`, or all worlds
    /// unlocked).
    fn gate_open(&self) -> bool {
        self.worlds
            .gate_open(self.scores(), self.pack, self.world(), self.lock_worlds)
    }

    /// `StartMenu(4)`: a square for each level of the world, the gate
    /// forward, and the gate back in all but a pack's first world, with
    /// the text for each.
    fn make_squares(&mut self) {
        let world = self.world();
        let n = self.worlds.level_count(world);
        self.level_count = n;
        let open = self.gate_open();
        let lang = self.language;
        let back = i32::from(world % 10 != 0);
        self.squares.clear();
        for i in 0..=n + back {
            let level = world * 10 + i;
            let won = worlds::score(self.scores(), level) > 0;
            let square = if i == n {
                if open {
                    self.set_names(i, gate_text(lang, true));
                    Square::Forward
                } else {
                    self.set_names(i, gate_text(lang, false));
                    Square::NotWon
                }
            } else if i == n + 1 {
                // These set line 3, which is never shown, and leave line
                // 2 as it was.
                let [a, b] = back_text(lang);
                let names = &mut self.level_names[i as usize];
                names[0] = a.to_owned();
                names[1] = b.to_owned();
                names[3] = String::new();
                Square::Back
            } else if !won {
                Square::NotWon
            } else if self.perfect(level) == 0 {
                Square::Won
            } else {
                Square::Perfect
            };
            let (x, y) = if i < n {
                let row = if n > 5 {
                    0.6 - (i / 5) as f32 * 0.35
                } else {
                    0.425
                };
                (-0.8 + (i % 5) as f32 * 0.4, row)
            } else if i == n + 1 {
                (-1.3, 0.425)
            } else {
                (1.3, 0.425)
            };
            self.squares.push(LevelSquare { x, y, square });
            if i < n {
                // The original reads the German and French titles from
                // data that is not there and stops with "Out of data";
                // here every language shows the level's own title.
                let lines = title_lines(&self.level_titles[level as usize]);
                let k = lines.len() - 1;
                let names = &mut self.level_names[i as usize];
                for (j, line) in lines.into_iter().take(4).enumerate() {
                    names[j] = line;
                }
                if k < 2 {
                    names[2].clear();
                }
                if k < 1 {
                    names[1].clear();
                }
            }
        }
        self.cursor = Some(LevelCursor {
            x: 0.0,
            y: 0.0,
            turn: 0.0,
        });
    }

    fn set_names(&mut self, i: i32, lines: [&str; 3]) {
        let names = &mut self.level_names[i as usize];
        for (n, l) in names.iter_mut().zip(lines) {
            *n = l.to_owned();
        }
    }

    /// Frees the squares and cursor.
    fn free_squares(&mut self) {
        self.squares.clear();
        self.cursor = None;
    }

    /// `MenuLoop4`: the level selector. Enter plays the level under the
    /// cursor or goes through a gate; the cursor keys move the cursor.
    fn level_select(&mut self, keys: MenuKeys, ticks: u32) {
        let world = self.world();
        let n = self.level_count;
        let lang = self.language;
        let name = self.profile.as_ref().map_or("", |p| p.name.as_str());
        let tx = &mut self.text;
        tx.display(name, 0.0, 0.0, 1.0, 61);
        let (group, adjust) = match world {
            ..10 => ("CLASSIC LEVELS ", 0),
            10..20 => ("TUTORIAL LEVELS ", 10),
            20..30 => ("DELUXE LEVELS ", 20),
            _ => ("ORIGINAL LEVELS ", 30),
        };
        tx.center("", 15.0, 1.0, 63);
        let number = world + 1 - adjust;
        match lang {
            1 => tx.center(&format!("WELT  {number}"), 2.0, 1.0, 50),
            2 => tx.center(&format!("MONDE  {number}"), 2.0, 1.0, 50),
            _ => tx.center(&format!("{group}WORLD {number}"), 2.0, 1.0, 50),
        }
        tx.center(self.worlds.name(world, lang), 3.0, 1.0, 51);
        let scores = self.profile.as_ref().map_or(&[][..], |p| &p.scores);
        let total = total_score(scores, adjust).to_string();
        let label = match lang {
            1 => "     PUNKTE:",
            2 => "SCORE TOTAL:",
            _ => "TOTAL SCORE:",
        };
        tx.display(label, 38.0, 0.0, 1.0, 52);
        tx.display(&total, (50 - total.len() as i32) as f32, 1.0, 1.0, 53);

        let at = self.level - world * 10;
        if let Some(names) = usize::try_from(at)
            .ok()
            .and_then(|i| self.level_names.get(i))
        {
            for (i, line) in names.iter().take(3).enumerate() {
                tx.center(line, 10.5 + i as f32, 1.0, 55 + i as i32);
            }
        }
        let blink = self.level_timer % 160 < 120;
        let cursor;
        if at < n {
            let l = self.level;
            cursor = if n > 5 {
                (
                    -0.8 + (l % 5) as f32 * 0.4,
                    0.6 - ((l % 10) / 5) as f32 * 0.35,
                )
            } else {
                (-0.8 + (l % 5) as f32 * 0.4, 0.425)
            };
            tx.center(&format!("Level {}", l % 10 + 1), 9.5, 1.0, 54);
            let score = worlds::score(scores, l);
            if score == 0 {
                if blink {
                    let (a, b) = match lang {
                        1 => ("Noch nicht komplett.", "'Enter' um das Level zu starten."),
                        2 => (
                            "Pas encore complete.",
                            "Tape 'Entree' pour essayer ce niveau.",
                        ),
                        _ => ("Not Completed Yet", "Press 'Enter' to try this Level."),
                    };
                    tx.center(a, 13.0, 1.0, 58);
                    tx.center(&menu_prompt(b, self.device), 14.0, 1.0, 59);
                } else {
                    tx.display("", 0.0, 0.0, 1.0, 58);
                    tx.display("", 0.0, 0.0, 1.0, 59);
                }
                let (hint, x) = match lang {
                    1 => ("Benutze die Pfeiltasten zur Levelauswahl.", 5.0),
                    2 => ("Utilise les fleches pour choisir les niveaux.", 3.0),
                    _ => ("Use Cursor Keys to select different Levels.", 4.0),
                };
                if self.device.is_pad() {
                    tx.center(&menu_prompt(hint, self.device), 18.0, 1.0, 60);
                } else {
                    tx.display(hint, x, 18.0, 1.0, 60);
                }
            } else {
                let (done, word, hint, x) = match lang {
                    1 => (
                        "Beendet!",
                        "Punkte: ",
                        "Levels koennen jederzeit nochmal gespielt werden.",
                        0.5,
                    ),
                    2 => (
                        "Reussi!",
                        "Score: ",
                        "      Rejoue les niveaux n'importe quand.      ",
                        1.0,
                    ),
                    _ => (
                        "Completed!",
                        "Score: ",
                        "Replay Levels to try and beat your score anytime.",
                        0.5,
                    ),
                };
                tx.center(done, 13.0, 1.0, 58);
                tx.center(&format!("{word}{score}"), 14.0, 1.0, 59);
                if blink {
                    tx.display(hint, x, 18.0, 1.0, 60);
                } else {
                    tx.display("", 0.0, 0.0, 1.0, 60);
                }
                let perfect = self.profile.as_ref().map_or(&[][..], |p| &p.perfect);
                if lang == 0 && worlds::score(perfect, l) == 1 {
                    tx.center("All Bonus Coins Found!", 15.0, 1.0, 63);
                }
            }
        } else {
            let gate = if at == n {
                cursor = (1.3, 0.425);
                match lang {
                    1 => "Zur naechsten Welt!",
                    2 => "Monde suivant",
                    _ => "Next World",
                }
            } else {
                cursor = (-1.3, 0.425);
                match lang {
                    1 => "Zur vorherigen Welt!",
                    2 => "Monde precedant",
                    _ => "Previous World",
                }
            };
            tx.center(gate, 9.5, 1.0, 54);
            for tag in 58..=60 {
                tx.display("", 1.0, 1.0, 1.0, tag);
            }
        }
        if let Some(c) = &mut self.cursor {
            (c.x, c.y) = cursor;
        }
        self.text.activate(|| None);

        self.run_ticks(ticks, true);
        if let Some(c) = &mut self.cursor {
            c.turn = (c.turn + 3.0 * ticks as f32) % 360.0;
        }

        let back = i32::from(world % 10 != 0);
        if keys.enter {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if at < n {
                    self.free_squares();
                    self.end();
                    self.actions.push(Action::StartLevel(self.level));
                } else if at == n {
                    if self.gate_open() {
                        self.free_squares();
                        let world = world + 1;
                        self.set_world(world);
                        self.end();
                        self.level = world * 10;
                        match world {
                            10 => self.start(ENDING),
                            28 => self.start(ENDING_DELUXE),
                            40 => self.actions.push(Action::TowEnding { skip: 0 }),
                            // The end of the tutorials.
                            12 => self.start(MAIN),
                            _ => {
                                self.actions.push(Action::LevelMusic(world));
                                self.start(LEVEL_SELECT);
                            }
                        }
                    } else {
                        self.actions.push(Action::Sound(MenuSound::No));
                    }
                } else {
                    self.free_squares();
                    let world = world - 1;
                    self.set_world(world);
                    self.end();
                    self.level = world * 10;
                    self.actions.push(Action::LevelMusic(world));
                    self.start(LEVEL_SELECT);
                }
            }
        } else if keys.left {
            if self.cursor_key == 0 {
                self.sound();
                self.cursor_key = 1;
                self.level -= 1;
                if self.level < world * 10 {
                    self.level = world * 10 + n + back;
                }
            }
        } else if keys.right {
            if self.cursor_key == 0 {
                self.sound();
                self.cursor_key = 1;
                self.level += 1;
                if self.level >= world * 10 + n + 1 + back {
                    self.level = world * 10;
                }
            }
        } else if keys.up {
            if self.cursor_key == 0 && n > 5 {
                self.sound();
                self.cursor_key = 1;
                self.level -= 5;
                if self.level < world * 10 {
                    self.level = world * 10 + n - 1;
                }
            }
        } else if keys.down {
            if self.cursor_key == 0 && n > 5 {
                self.sound();
                self.cursor_key = 1;
                self.level += 5;
                if self.level > world * 10 + n - 1 {
                    self.level = world * 10 + n - 1;
                }
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.free_squares();
                self.end();
                self.start(LEVEL_PACKS);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// TOW's `StartMenu(8)`: a square for each playground level, all in
    /// the won quarter, with no gates and no scores.
    fn make_playground_squares(&mut self) {
        let world = PLAYGROUND_LEVEL / 10;
        let n = self.worlds.level_count(world);
        self.level_count = n;
        self.squares.clear();
        for i in 0..n {
            let row = if n > 5 {
                0.6 - (i / 5) as f32 * 0.35
            } else {
                0.425
            };
            self.squares.push(LevelSquare {
                x: -0.8 + (i % 5) as f32 * 0.4,
                y: row,
                square: Square::Won,
            });
            // TOW reads translated titles from its own data, as RTWP
            // does; here every language shows the file's title.
            let lines = title_lines(&self.level_titles[(world * 10 + i) as usize]);
            let k = lines.len() - 1;
            let names = &mut self.level_names[i as usize];
            for (j, line) in lines.into_iter().take(4).enumerate() {
                names[j] = line;
            }
            if k < 2 {
                names[2].clear();
            }
            if k < 1 {
                names[1].clear();
            }
        }
        self.cursor = Some(LevelCursor {
            x: 0.0,
            y: 0.0,
            turn: 0.0,
        });
    }

    /// TOW's `MenuLoop8`: the playground's level selector. Enter plays
    /// the level under the cursor, the cursor keys move it within the
    /// levels, and Esc goes back to the level packs (TOW's main menu).
    fn playground(&mut self, keys: MenuKeys, ticks: u32) {
        let world = PLAYGROUND_LEVEL / 10;
        let n = self.level_count;
        let lang = self.language;
        let tx = &mut self.text;
        let title = match lang {
            1 => "BEGINNER LEVELS",
            2 => "BIENVENUE AU TERRAIN DE JEU",
            _ => "WELCOME TO THE PLAYGROUND",
        };
        tx.center(title, 2.0, 1.0, 50);
        let at = self.level - world * 10;
        if let Some(names) = usize::try_from(at)
            .ok()
            .and_then(|i| self.level_names.get(i))
        {
            for (i, line) in names.iter().take(3).enumerate() {
                tx.center(line, 10.5 + i as f32, 1.0, 55 + i as i32);
            }
        }
        let l = self.level;
        let cursor = if n > 5 {
            (
                -0.8 + (l % 5) as f32 * 0.4,
                0.6 - ((l % 10) / 5) as f32 * 0.35,
            )
        } else {
            (-0.8 + (l % 5) as f32 * 0.4, 0.425)
        };
        tx.center(&format!("Level {}", l % 10 + 1), 9.5, 1.0, 54);
        let (enter, cursor_keys, replay) = match lang {
            1 => (
                "'Enter' um dieses Level zu spielen.",
                "Pfeiltasten um andere Levels auszuwaehlen.",
                "",
            ),
            2 => (
                "Tape 'Entree' pour essayer ce niveau.",
                "Utilise les fleches pour choisir les niveaux.",
                "",
            ),
            _ => (
                "Press 'Enter' to try this Level.",
                "Use Cursor Keys to select different Levels.",
                "Levels can be replayed anytime.",
            ),
        };
        tx.center(&menu_prompt(enter, self.device), 14.0, 1.0, 59);
        // Both lines share a tag, so only the second stays.
        tx.center(&menu_prompt(cursor_keys, self.device), 17.0, 1.0, 60);
        tx.center(replay, 18.0, 1.0, 60);
        if let Some(c) = &mut self.cursor {
            (c.x, c.y) = cursor;
        }
        self.text.activate(|| None);

        self.run_ticks(ticks, true);
        if let Some(c) = &mut self.cursor {
            c.turn = (c.turn + 3.0 * ticks as f32) % 360.0;
        }

        let first = world * 10;
        if keys.enter {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if at < n {
                    self.free_squares();
                    self.end();
                    self.actions.push(Action::StartLevel(self.level));
                }
            }
        } else if keys.left {
            if self.cursor_key == 0 {
                self.sound();
                self.cursor_key = 1;
                self.level -= 1;
                if self.level < first {
                    self.level = first + n - 1;
                }
            }
        } else if keys.right {
            if self.cursor_key == 0 {
                self.sound();
                self.cursor_key = 1;
                self.level += 1;
                if self.level >= first + n {
                    self.level = first;
                }
            }
        } else if keys.up {
            if self.cursor_key == 0 && n > 5 {
                self.sound();
                self.cursor_key = 1;
                self.level -= 5;
                if self.level < first {
                    self.level = first + n - 1;
                }
            }
        } else if keys.down {
            if self.cursor_key == 0 && n > 5 {
                self.sound();
                self.cursor_key = 1;
                self.level += 5;
                if self.level > first + n - 1 {
                    self.level = first + n - 1;
                }
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.free_squares();
                self.end();
                self.start(LEVEL_PACKS);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// The selector a level is chosen from: the playground's or the
    /// world's.
    pub fn selector_menu(&self) -> i32 {
        if self.level >= PLAYGROUND_LEVEL {
            PLAYGROUND
        } else {
            LEVEL_SELECT
        }
    }

    /// `MenuLoop3`: typing a new player's name, up to 15 letters and
    /// spaces. Enter creates the player.
    fn new_player(&mut self, keys: MenuKeys, ticks: u32) {
        let (title, rule) = match self.language {
            1 => (
                "SPIELER - BITTE NAMEN EINGEBEN:",
                "-------------------------------",
            ),
            2 => (
                "NOUVEAU JOUEUR - S.V.P. ECRIS TON NOM:",
                "--------------------------------------",
            ),
            _ => (
                "NEW PLAYER - PLEASE ENTER NAME:",
                "-------------------------------",
            ),
        };
        let tx = &mut self.text;
        tx.center(title, 4.0, 1.0, 50);
        tx.center(rule, 5.0, 1.0, 51);
        tx.center(&self.player_name, 7.0, 1.0, 52);
        // `LevelTimer` does not count here, so the cursor does not blink.
        let cursor = if self.level_timer % 50 < 25 { ":" } else { " " };
        let x = 25.0 + self.player_name.len() as f32 / 2.0;
        tx.display(cursor, x, 7.0, 1.0, 53);
        if self.device.is_pad() {
            self.show_picker();
        } else {
            for tag in 54..=58 {
                self.text.display("", 0.0, 0.0, 1.0, tag);
            }
        }
        // Moved above the top of the screen, out of sight.
        if !self.device.is_pad() {
            let half_width = self.selector_at.map_or(25.0 * 0.012, |s| s.half_width);
            self.selector_at = Some(Selector {
                x: -0.5 + 24.4 * 0.02 + 0.01,
                y: 0.375 - (-2.0) * 0.04 - 0.013,
                z: 1.001,
                half_width,
            });
        }
        self.update_selector();
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

        if self.device.is_pad() {
            self.pick(keys);
            return;
        }
        let mut key = self.typed.pop_front().map_or(0, i32::from);
        if key >= 97 {
            key -= 32;
        }
        let room = self.player_name.len() < 15;
        let typed = match key {
            65..=90 | 32 if room => Some(key as u8 as char),
            _ => None,
        };
        if let Some(c) = typed {
            if self.cursor_key == 0 {
                self.actions.push(Action::Sound(MenuSound::Type));
                self.player_name.push(c);
                self.cursor_key = 1;
            }
        } else if keys.backspace && !self.player_name.is_empty() {
            if self.cursor_key == 0 {
                self.actions.push(Action::Sound(MenuSound::Type));
                self.player_name.pop();
                self.cursor_key = 1;
            }
        } else if keys.enter {
            if self.cursor_key == 0 {
                self.create_player();
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                self.end();
                // With no players this comes straight back here.
                self.start(PLAYER_SELECT);
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// Makes the player named, and on to the level packs.
    fn create_player(&mut self) {
        self.actions.push(Action::Sound(MenuSound::LastCoin));
        let profile = Profile::new(&self.player_name, self.millisecs);
        if let Some(Err(e)) = self.players.as_ref().map(|p| p.create(&profile)) {
            self.actions
                .push(Action::Warn(format!("{}: {e}", profile.name)));
        }
        self.profile = Some(profile);
        self.cursor_key = 1;
        self.end();
        self.start(LEVEL_PACKS);
    }

    /// The letter picker: rows of letters and a blank, then OK.
    fn show_picker(&mut self) {
        let lang = self.language;
        for row in 0..PICKER_ROWS {
            let line: String = (0..PICKER_COLUMNS)
                .map(|c| PICKER_LETTERS[row * PICKER_COLUMNS + c] as char)
                .flat_map(|c| [c, ' ', ' '])
                .collect();
            let y = PICKER_TOP + row as f32;
            self.text
                .display(&line, PICKER_LEFT, y, 1.0, 54 + row as i32);
        }
        let ok_row = PICKER_TOP + PICKER_ROWS as f32;
        self.text.center("OK", ok_row, 1.0, 57);
        let hint = match lang {
            1 => "\x02  Buchstabe loeschen oder zurueck",
            2 => "\x02  Efface une lettre ou revient",
            _ => "\x02  Deletes a Letter or Goes Back",
        };
        self.text.center(hint, ok_row + 2.0, 1.0, 58);
        let (x, row, half_width) = if self.picker == PICKER_OK {
            (25.0, ok_row, 0.03)
        } else {
            let (row, column) = (self.picker / PICKER_COLUMNS, self.picker % PICKER_COLUMNS);
            let x = PICKER_LEFT + 0.5 + (column * 3) as f32;
            (x, PICKER_TOP + row as f32, 0.018)
        };
        self.selector_at = Some(Selector {
            x: (x - 25.0) * 0.022,
            half_width,
            ..Selector::at_row(row as i32, 1.0)
        });
    }

    /// The controller's keys on the letter picker.
    fn pick(&mut self, keys: MenuKeys) {
        if keys.up || keys.down || keys.left || keys.right {
            if self.cursor_key == 0 {
                self.picker = picker_step(self.picker, keys);
                self.actions.push(Action::Sound(MenuSound::Move));
                self.cursor_key = 1;
            }
        } else if keys.enter {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if self.picker == PICKER_OK {
                    // A name is needed, unlike at the keyboard's Enter.
                    if !self.player_name.is_empty() {
                        self.create_player();
                    }
                } else if self.player_name.len() < 15 {
                    self.actions.push(Action::Sound(MenuSound::Type));
                    self.player_name.push(PICKER_LETTERS[self.picker] as char);
                }
            }
        } else if keys.escape {
            if self.cursor_key == 0 {
                self.cursor_key = 1;
                if self.player_name.pop().is_some() {
                    self.actions.push(Action::Sound(MenuSound::Type));
                } else {
                    self.end();
                    self.start(PLAYER_SELECT);
                }
            }
        } else {
            self.cursor_key = 0;
        }
    }

    /// `MenuLoop6`: "Confirm Player Deletion?". YES keeps a `.bak` of the
    /// file and deletes it; either way it goes back to player select.
    fn delete_player(&mut self, keys: MenuKeys, ticks: u32) {
        let lines: [&str; 5] = match self.language {
            1 => [
                "SPIELER LOESCHEN?",
                "-----------------",
                "(Backup im Player Directory)",
                "Ja",
                "Nein",
            ],
            2 => [
                "CONFIRMEZ L'EFFACEMENT DU JOUER?",
                "-------------------------------",
                "La sauvegarde est stockee dans le dossier PLAYERS",
                "OUI",
                "NON",
            ],
            _ => [
                "CONFIRM PLAYER DELETION?",
                "------------------------",
                "(backup is stored in PLAYER directory)",
                "YES",
                "NO",
            ],
        };
        for (i, (line, y)) in lines.iter().zip([4.0, 5.0, 6.0, 8.0, 9.0]).enumerate() {
            self.text.center(line, y, 1.0, 50 + i as i32);
        }
        self.place_selector(8 + self.option, 4.5);
        self.text.activate(|| None);

        // Only `LevelTimer` counts: the background stands still. (The
        // original also scrolls `MenuBackGroundTex`, a name nothing else
        // uses.)
        self.level_timer += ticks as i32;

        // Esc does nothing here and does not hold the other keys.
        let keys = MenuKeys {
            escape: false,
            ..keys
        };
        if let Some(Press::Enter) = self.navigate(keys, 2) {
            if self.option == 0 {
                let entry = self.list.get(self.to_delete);
                if let (Some(players), Some(entry)) = (&self.players, entry)
                    && let Err(e) = players.delete(entry)
                {
                    self.actions
                        .push(Action::Warn(format!("{}: {e}", entry.path.display())));
                }
            }
            self.end();
            self.start(PLAYER_SELECT);
        }
    }

    /// `MenuLoop9`: options.
    fn options(&mut self, keys: MenuKeys, ticks: u32, o: &mut ConfigOptions) {
        let pick = |v: i32, table: &[(i32, &'static str)]| {
            table.iter().find(|(k, _)| *k == v).map_or("", |(_, s)| *s)
        };
        let tx = &mut self.text;
        match self.language {
            1 => {
                let sound = pick(
                    o.sound_level,
                    &[
                        (100, "Sound: Laut"),
                        (60, "Sound: Mittel"),
                        (20, "Sound: Leise"),
                        (0, "Kein Sound"),
                    ],
                );
                let music = pick(
                    o.music_level,
                    &[
                        (100, "Musik: Laut"),
                        (70, "Musik: Mittel"),
                        (40, "Musik: Leise"),
                        (0, "Keine Musik"),
                    ],
                );
                let detail = pick(
                    o.detail_level,
                    &[
                        (5, "Detaillevel: Hoch"),
                        (3, "Detaillevel: Mittel"),
                        (1, "Detaillevel: Niedrig"),
                    ],
                );
                let windowed = if o.windowed == 0 {
                    "Vollbild"
                } else {
                    "Fenster"
                };
                let time = if o.no_time_limit == 1 {
                    "Kein Zeitlimit"
                } else {
                    "Timer Modus"
                };
                tx.center("OPTIONEN", 5.0, 1.0, 50);
                tx.center("--------", 6.0, 1.0, 51);
                tx.center(sound, 8.0, 1.0, 52);
                tx.center(music, 9.0, 1.0, 53);
                tx.center(&resolution_text("Aufloesung", o.gfx_mode), 10.0, 1.0, 54);
                tx.center(detail, 11.0, 1.0, 55);
                tx.center(windowed, 12.0, 1.0, 56);
                tx.center("Deutsch", 13.0, 1.0, 58);
                tx.center(time, 14.0, 1.0, 60);
                tx.center("Unsere Webseite", 15.0, 1.0, 59);
                tx.center("HAUPTMENUE", 16.0, 1.0, 57);
            }
            2 => {
                let sound = pick(
                    o.sound_level,
                    &[
                        (100, "Niveau du Son: 3"),
                        (60, "Niveau du Son: 2"),
                        (20, "Niveau du Son: 1"),
                        (0, "Niveau du Son: 0"),
                    ],
                );
                let music = pick(
                    o.music_level,
                    &[
                        (100, "Niveau de la Musique: 3"),
                        (70, "Niveau de la Musique: 2"),
                        (40, "Niveau de la Musique: 1"),
                        (0, "Niveau de la Musique: 0"),
                    ],
                );
                let detail = pick(
                    o.detail_level,
                    &[
                        (5, "Detail: Haut"),
                        (3, "Detail: Medium"),
                        (1, "Detail: Bas"),
                    ],
                );
                let windowed = if o.windowed == 0 {
                    "Plein Ecran"
                } else {
                    "En Fenetre"
                };
                let time = if o.no_time_limit == 1 {
                    "Pas de Chrono"
                } else {
                    "Mode Chrono"
                };
                tx.center("OPTIONS", 5.0, 1.0, 50);
                tx.center("-------", 6.0, 1.0, 51);
                tx.center(sound, 8.0, 1.0, 52);
                tx.center(music, 9.0, 1.0, 53);
                tx.center(&resolution_text("Resolution", o.gfx_mode), 10.0, 1.0, 54);
                tx.center(detail, 11.0, 1.0, 55);
                tx.center(windowed, 12.0, 1.0, 56);
                tx.center("Francais", 13.0, 1.0, 58);
                tx.center(time, 14.0, 1.0, 60);
                tx.center("Notre Site Web", 15.0, 1.0, 59);
                tx.center("Menu Principal", 16.0, 1.0, 57);
            }
            _ => {
                let sound = pick(
                    o.sound_level,
                    &[
                        (100, "Sound Volume: High"),
                        (60, "Sound Volume: Medium"),
                        (20, "Sound Volume: Low"),
                        (0, "No Sound"),
                    ],
                );
                let music = pick(
                    o.music_level,
                    &[
                        (100, "Music Volume: High"),
                        (70, "Music Volume: Medium"),
                        (40, "Music Volume: Low"),
                        (0, "No Music"),
                    ],
                );
                let detail = pick(
                    o.detail_level,
                    &[
                        (5, "Detail Level: High"),
                        (3, "Detail Level: Medium"),
                        (1, "Detail Level: Low"),
                    ],
                );
                let time = if o.no_time_limit == 1 {
                    "No Time Limit"
                } else {
                    "Timed Mode"
                };
                let worlds = if o.lock_worlds == 0 {
                    "Normal World Progression"
                } else {
                    "All Worlds Unlocked"
                };
                let teleport = if o.teleport == 0 {
                    "Teleport: Pent"
                } else {
                    "Teleport: Star"
                };
                tx.center("OPTIONS", 5.0, 1.0, 50);
                tx.center("-------", 6.0, 1.0, 51);
                tx.center(sound, 8.0, 1.0, 52);
                tx.center(music, 9.0, 1.0, 53);
                tx.center(detail, 10.0, 1.0, 55);
                tx.center(time, 11.0, 1.0, 60);
                tx.center(worlds, 12.0, 1.0, 54);
                tx.center(teleport, 13.0, 1.0, 56);
                tx.center("Visit Us Online", 14.0, 1.0, 59);
                tx.center("MAIN MENU", 15.0, 1.0, 57);
            }
        }
        self.place_selector(8 + self.option, 22.0);
        self.text.activate(|| None);

        self.run_ticks(ticks, true);

        if keys.one {
            self.secret_music = 1;
        }
        if keys.zero && self.secret_music > 300 {
            self.secret_music = 0;
            self.actions.push(Action::Sound(MenuSound::SecretSong));
        }
        if self.secret_music > 0 {
            self.secret_music += 1;
        }

        match self.navigate(keys, 8) {
            Some(Press::Enter) => match self.option {
                0 => {
                    o.sound_level = match o.sound_level {
                        100 => 60,
                        60 => 20,
                        20 => 0,
                        _ => 100,
                    };
                    self.actions.push(Action::OptionsChanged);
                }
                1 => {
                    o.music_level = match o.music_level {
                        100 => 70,
                        70 => 40,
                        40 => 0,
                        _ => 100,
                    };
                    self.actions.push(Action::OptionsChanged);
                }
                2 => {
                    o.detail_level = match o.detail_level {
                        5 => 3,
                        3 => 1,
                        _ => 5,
                    };
                    self.actions.push(Action::OptionsChanged);
                }
                3 => {
                    o.no_time_limit = 1 - o.no_time_limit;
                    self.actions.push(Action::OptionsChanged);
                }
                4 => {
                    o.lock_worlds = 1 - o.lock_worlds;
                    self.actions.push(Action::OptionsChanged);
                }
                5 => {
                    o.teleport = 1 - o.teleport;
                    self.actions.push(Action::OptionsChanged);
                }
                6 => {
                    self.end();
                    self.version_no1 = -996;
                    self.start(ONLINE);
                }
                _ => self.leave_options(),
            },
            Some(Press::Escape) => self.leave_options(),
            None => {}
        }
    }

    fn leave_options(&mut self) {
        self.end();
        self.actions.push(Action::SaveOptions);
        self.start(MAIN);
    }

    /// `MenuLoop10`: "Exit?". Escape here quits too.
    fn exit(&mut self, keys: MenuKeys, ticks: u32) {
        let (title, rule, yes, no) = match self.language {
            1 => (
                "Wonderland Beenden?",
                "--------------------",
                "JA BITTE",
                "NEIN DANKE",
            ),
            2 => ("Sortie?", "-------", "BIEN SUR", "NON MERCI"),
            _ => ("Exit?", "-----", "SURE THING", "NO THANKS"),
        };
        self.text.center(title, 7.0, 1.0, 50);
        self.text.center(rule, 8.0, 1.0, 51);
        self.text.center(yes, 10.0, 1.0, 52);
        self.text.center(no, 11.0, 1.0, 53);
        self.place_selector(10 + self.option, 11.5);
        self.text.activate(|| None);

        self.run_ticks(ticks, true);

        match self.navigate(keys, 2) {
            Some(Press::Enter) if self.option != 0 => {
                self.end();
                self.start(MAIN);
            }
            Some(_) => {
                self.end();
                self.quit();
            }
            None => {}
        }
    }

    /// `MenuLoop15`: "Go Online?". Its German and French text is a copy
    /// of the exit menu's, as in the original.
    fn online(&mut self, keys: MenuKeys, ticks: u32) {
        match self.language {
            1 => {
                self.text.center("Wonderland Beenden?", 7.0, 1.0, 50);
                self.text.center("--------------------", 8.0, 1.0, 51);
                self.text.center("JA BITTE", 10.0, 1.0, 52);
                self.text.center("NEIN DANKE", 11.0, 1.0, 53);
            }
            2 => {
                self.text.center("Sortie?", 7.0, 1.0, 50);
                self.text.center("-------", 8.0, 1.0, 51);
                self.text.center("BIEN SUR", 10.0, 1.0, 52);
                self.text.center("NON MERCI", 11.0, 1.0, 53);
            }
            _ => {
                self.text.center("Go Online?", 4.0, 1.0, 50);
                self.text.center("----------", 5.0, 1.0, 51);
                self.text
                    .center("This option will exit Return to Wonderland", 7.0, 1.0, 52);
                self.text
                    .center("and connect you to an Internet Website.", 8.0, 1.0, 53);
                self.text.center("PROCEED", 10.0, 1.0, 54);
                self.text.center("NO THANKS", 11.0, 1.0, 55);
            }
        }
        self.place_selector(10 + self.option, 11.5);
        self.text.activate(|| None);

        self.run_ticks(ticks, true);

        let back = |m: &mut Self| {
            m.end();
            m.version_no1 = 1;
            m.start(MAIN);
        };
        match self.navigate(keys, 2) {
            Some(Press::Enter) if self.option == 0 => {
                self.end();
                self.quit();
            }
            Some(_) => back(self),
            None => {}
        }
    }
}

/// The 46-dash line on the player select screen.
const RULE_46: &str = "----------------------------------------------";

/// The 48-dash line on the custom levels.
const RULE_48: &str = "------------------------------------------------";

/// `Left$(s, n)`.
fn left(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `MyGfxModeWidth`/`MyGfxModeHeight`, as `Setup` fills them in.
pub const GFX_MODES: [(u32, u32); 5] = [
    (640, 480),
    (800, 600),
    (1024, 768),
    (1280, 960),
    (1280, 1024),
];

/// `OptionsResText$`, shown only in the German and French menus.
fn resolution_text(word: &str, gfx_mode: i32) -> String {
    let (w, h) = usize::try_from(gfx_mode)
        .ok()
        .and_then(|i| GFX_MODES.get(i))
        .copied()
        .unwrap_or((0, 0));
    format!("{word}: {w}x{h}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderland_formats::Game;

    fn press(m: &mut Menu, o: &mut ConfigOptions, keys: MenuKeys) -> Vec<Action> {
        let mut out = m.frame(keys, 1, o);
        out.extend(m.frame(MenuKeys::default(), 1, o));
        out
    }

    const DOWN: MenuKeys = MenuKeys { down: true, ..KEYS };
    const ENTER: MenuKeys = MenuKeys {
        enter: true,
        ..KEYS
    };
    const ESCAPE: MenuKeys = MenuKeys {
        escape: true,
        ..KEYS
    };
    const KEYS: MenuKeys = MenuKeys {
        two: false,
        three: false,
        up: false,
        down: false,
        left: false,
        right: false,
        enter: false,
        escape: false,
        space: false,
        one: false,
        zero: false,
        delete: false,
        backspace: false,
        page_up: false,
        page_down: false,
        f1: false,
        f2: false,
    };

    fn text(m: &Menu) -> String {
        let mut rows: Vec<(i32, f32, u8)> = m
            .text
            .shown()
            .iter()
            .map(|l| (l.y as i32, l.x, l.ascii))
            .collect();
        rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rows.iter().map(|r| r.2 as char).collect()
    }

    #[test]
    fn title_fades_the_opening_line_in() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        assert_eq!(m.timer, -500);
        m.frame(KEYS, 100, &mut o);
        // Laid out at -500, so fully transparent.
        assert!(text(&m).contains("And now"));
        assert!(m.text.shown().iter().all(|l| l.alpha == 0.0));
        m.frame(KEYS, 0, &mut o);
        assert!(m.text.shown().iter().all(|l| (l.alpha - 0.5).abs() < 1e-6));
        // Nothing shows between 0 and 500.
        m.frame(KEYS, 600, &mut o);
        m.frame(KEYS, 0, &mut o);
        assert_eq!(m.timer, 200);
        assert!(m.text.shown().is_empty());
    }

    #[test]
    fn space_goes_to_the_main_menu_and_esc_to_exit() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        let a = m.frame(
            MenuKeys {
                space: true,
                ..KEYS
            },
            1,
            &mut o,
        );
        assert_eq!(m.current, MAIN);
        assert_eq!(m.timer, 0);
        assert!(a.contains(&Action::Entered(MAIN)));
        // Still held: not a press yet.
        m.frame(
            MenuKeys {
                space: true,
                escape: true,
                ..KEYS
            },
            1,
            &mut o,
        );
        assert_eq!(m.current, MAIN);
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("START GAME"));
        assert!(text(&m).starts_with("v10.00"));
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, EXIT);
        // Esc on "Exit?" quits too.
        let a = press(&mut m, &mut o, ESCAPE);
        assert!(a.contains(&Action::Quit { url: None }));
    }

    #[test]
    fn the_cursor_wraps_and_plays_the_menu_sound() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        m.start(MAIN);
        assert!(m.frame(KEYS, 0, &mut o).contains(&Action::Entered(MAIN)));
        let a = press(&mut m, &mut o, MenuKeys { up: true, ..KEYS });
        assert_eq!(m.option, 4);
        assert_eq!(a, vec![Action::Sound(MenuSound::Move)]);
        // Held down for several frames still moves once.
        for _ in 0..5 {
            m.frame(DOWN, 1, &mut o);
        }
        assert_eq!(m.option, 0);
        let sel = m.selector.unwrap();
        assert!((sel.y - (0.372 - 9.0 * 0.044 + 0.027)).abs() < 1e-6);
        // Going from one key to the next needs a frame with neither.
        m.frame(ENTER, 1, &mut o);
        assert_eq!(m.current, MAIN);
        m.frame(KEYS, 1, &mut o);
        let a = press(&mut m, &mut o, ENTER);
        // No players folder: no players, so a new one.
        assert_eq!(m.current, NEW_PLAYER);
        assert!(a.contains(&Action::StopMusic));
        assert!(!m.logos);
    }

    #[test]
    fn options_cycle_and_save_on_the_way_out() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        m.start(OPTIONS);
        let mut seen = Vec::new();
        for _ in 0..4 {
            press(&mut m, &mut o, ENTER);
            seen.push(o.sound_level);
        }
        assert_eq!(seen, [60, 20, 0, 100]);
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("Sound Volume: High"));
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        assert_eq!(o.music_level, 70);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        assert_eq!(o.detail_level, 1);
        press(&mut m, &mut o, DOWN);
        let a = press(&mut m, &mut o, ENTER);
        assert_eq!(o.no_time_limit, 1);
        assert!(a.contains(&Action::OptionsChanged));
        let a = press(&mut m, &mut o, ESCAPE);
        assert!(a.contains(&Action::SaveOptions));
        assert_eq!(m.current, MAIN);
    }

    #[test]
    fn going_online_needs_a_yes() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        m.start(MAIN);
        for _ in 0..2 {
            press(&mut m, &mut o, DOWN);
        }
        press(&mut m, &mut o, ENTER);
        assert_eq!((m.current, m.version_no1), (ONLINE, -999));
        // "NO THANKS" leaves the version shown as 1.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!((m.current, m.version_no1), (MAIN, 1));
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).starts_with("v1.00"));
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        let a = press(&mut m, &mut o, ENTER);
        assert_eq!(
            a.last(),
            Some(&Action::Quit {
                url: Some("http://www.midnightsynergy.com/returntowonderland/levelex")
            })
        );
    }

    #[test]
    fn the_sunrise_ends_in_full_light() {
        let mut b = Background::default();
        b.tick(0, TITLE);
        assert_eq!(b, Background::default());
        b.tick(3200, TITLE);
        assert_eq!(b.light, [128, 64, 64]);
        assert_eq!(b.fog, [32, 43, 63]);
        assert_eq!(b.sun_colour, [255, 100, 0]);
        b.tick(13000, TITLE);
        assert_eq!((b.light, b.ambient), ([255; 3], [255; 3]));
        // The colours stay where the first stage left them.
        assert_eq!(b.sun_colour, [255, 100, 0]);
        assert!((b.sun_at[1] - 29.5).abs() < 1e-5);
        // The mountains go round the camera once in 72000 ticks.
        b.tick(72000, TITLE);
        assert!((b.terrain_at[0] + 128.0).abs() < 0.01);
    }

    #[test]
    fn holding_one_two_three_starts_the_ending_and_esc_leaves_it() {
        let mut m = Menu::new();
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        m.end();
        m.start(MAIN);
        let held = MenuKeys {
            one: true,
            two: true,
            three: true,
            ..KEYS
        };
        let a = m.frame(held, 0, &mut o);
        assert_eq!(m.current, ENDING);
        assert!(a.contains(&Action::EndingMusic));
        m.frame(KEYS, 700, &mut o);
        m.frame(KEYS, 0, &mut o);
        assert!(text(&m).contains("Loof!"));
        m.level_timer = 6100;
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("Midnight Synergy"));
        let esc = MenuKeys {
            escape: true,
            ..KEYS
        };
        m.frame(esc, 0, &mut o);
        assert_eq!(m.current, MAIN);
        assert_eq!(m.ending, None);
    }

    #[test]
    fn the_deluxe_ending_keeps_its_last_page() {
        let mut m = Menu::new();
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        m.end();
        m.start(ENDING_DELUXE);
        assert!(m.actions.contains(&Action::TitleMusic));
        m.level_timer = 3500;
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("Rainbow Spirits"));
        m.level_timer = 11500;
        m.frame(KEYS, 1, &mut o);
        m.level_timer = 40000;
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("THE END"));
    }

    #[test]
    fn exit_sounds_follow_the_clock() {
        assert_eq!(exit_sound(0), MenuSound::Exit(3));
        assert_eq!(exit_sound(59), MenuSound::Exit(5));
        assert_eq!(exit_sound(99), MenuSound::Exit(8));
    }

    fn players_menu(tag: &str) -> (Menu, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("wl-menu-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut m = Menu::new();
        m.players = Some(Players::new(&dir));
        m.end();
        m.start(MAIN);
        (m, dir)
    }

    fn type_name(m: &mut Menu, o: &mut ConfigOptions, name: &str) {
        for c in name.bytes() {
            m.type_char(c);
            m.frame(KEYS, 1, o);
            m.frame(KEYS, 1, o);
        }
    }

    #[test]
    fn a_controller_picks_a_name() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let (mut m, dir) = players_menu("picker");
        m.device = Device::Xbox;
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, NEW_PLAYER);
        press(&mut m, &mut o, KEYS);
        assert!(text(&m).contains("A  B  C"));
        // OK with no name does nothing.
        press(&mut m, &mut o, UP);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, NEW_PLAYER);
        // Down wraps to the middle of the top row, E; then F, picked twice.
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        let right = MenuKeys {
            right: true,
            ..KEYS
        };
        press(&mut m, &mut o, right);
        press(&mut m, &mut o, ENTER);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.player_name, "EFF");
        // Back takes a letter off.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.player_name, "EF");
        press(&mut m, &mut o, UP);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, LEVEL_PACKS);
        assert_eq!(m.profile.as_ref().unwrap().name, "EF");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn picker_steps_wrap_round() {
        let keys = |f: fn(&mut MenuKeys)| {
            let mut k = MenuKeys::default();
            f(&mut k);
            k
        };
        let left = keys(|k| k.left = true);
        let right = keys(|k| k.right = true);
        let (up, down) = (keys(|k| k.up = true), keys(|k| k.down = true));
        assert_eq!(picker_step(0, left), 8);
        assert_eq!(picker_step(8, right), 0);
        assert_eq!(picker_step(0, up), PICKER_OK);
        assert_eq!(picker_step(26, down), PICKER_OK);
        assert_eq!(picker_step(PICKER_OK, up), 22);
        assert_eq!(picker_step(PICKER_OK, down), 4);
        assert_eq!(picker_step(PICKER_OK, left), PICKER_OK);
    }

    #[test]
    fn the_prompts_follow_the_device() {
        let mut m = Menu::new();
        m.start(MAIN);
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        press(&mut m, &mut o, KEYS);
        assert!(text(&m).contains("Use Cursor Keys"));
        m.device = Device::Ps5;
        press(&mut m, &mut o, KEYS);
        assert!(text(&m).contains("Use \x03  and \x01  to Select"));
    }

    #[test]
    fn first_player_is_typed_and_created() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let (mut m, dir) = players_menu("create");
        press(&mut m, &mut o, ENTER);
        // No players yet: straight to the name.
        assert_eq!(m.current, NEW_PLAYER);
        type_name(&mut m, &mut o, "ab c9");
        assert_eq!(m.player_name, "AB C");
        let a = press(
            &mut m,
            &mut o,
            MenuKeys {
                backspace: true,
                ..KEYS
            },
        );
        assert!(a.contains(&Action::Sound(MenuSound::Type)));
        type_name(&mut m, &mut o, "d");
        assert!(text(&m).contains("AB D:"));
        // Esc with no players comes back here, with the name cleared.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!((m.current, m.player_name.as_str()), (NEW_PLAYER, ""));
        type_name(&mut m, &mut o, "Ab");
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::Sound(MenuSound::LastCoin)));
        assert_eq!(m.current, LEVEL_PACKS);
        // The classic levels' selector, with the first world's music.
        press(&mut m, &mut o, DOWN);
        let a = press(&mut m, &mut o, ENTER);
        assert_eq!((m.current, m.level), (LEVEL_SELECT, 0));
        assert!(a.contains(&Action::LevelMusic(0)));
        assert_eq!(m.profile.as_ref().unwrap().name, "AB");
        assert!(dir.join("players/ab.pla").is_file());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn players_are_listed_chosen_and_deleted() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let (mut m, dir) = players_menu("select");
        let players = m.players.clone().unwrap();
        let mut zed = Profile::new("ZED", 1);
        players.create(&Profile::new("AMY", 2)).unwrap();
        players.create(&zed).unwrap();
        zed.record_win(4, 1234, false);
        players.save(&mut zed).unwrap();

        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, PLAYER_SELECT);
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("CREATE NEW PLAYER"));
        assert!(t.contains("AMY00"), "{t}");
        assert!(t.contains("ZED101234"), "{t}");

        // Delete on CREATE NEW PLAYER does nothing.
        let del = MenuKeys {
            delete: true,
            ..KEYS
        };
        press(&mut m, &mut o, del);
        assert_eq!(m.current, PLAYER_SELECT);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, del);
        assert_eq!(m.current, DELETE_PLAYER);
        // NO keeps the player.
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, PLAYER_SELECT);
        assert!(dir.join("players/amy.pla").is_file());
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, del);
        press(&mut m, &mut o, ENTER);
        assert!(!dir.join("players/amy.pla").exists());
        assert!(dir.join("players/amy.bak").is_file());

        // ZED is now the only player.
        assert_eq!(m.current, PLAYER_SELECT);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, LEVEL_PACKS);
        press(&mut m, &mut o, DOWN);
        let a = press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, LEVEL_SELECT);
        assert!(a.contains(&Action::LevelMusic(0)));
        assert_eq!(m.profile.as_ref().unwrap().record(4), Some(1234));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    const LEFT: MenuKeys = MenuKeys { left: true, ..KEYS };
    const RIGHT: MenuKeys = MenuKeys {
        right: true,
        ..KEYS
    };
    const UP: MenuKeys = MenuKeys { up: true, ..KEYS };

    /// The level selector for a new player in `world`.
    fn selector(world: i32) -> Menu {
        let mut m = Menu::new();
        let mut p = Profile::new("ZED", 0);
        p.world_packs[0] = world;
        m.profile = Some(p);
        for (l, t) in m.level_titles.iter_mut().enumerate() {
            *t = format!("TITLE {l}#SECOND");
        }
        m.end();
        m.start(LEVEL_PACKS);
        // The classic levels.
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        m
    }

    #[test]
    fn the_selector_shows_the_world_and_its_levels() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = selector(0);
        m.profile.as_mut().unwrap().record_win(1, 500, true);
        m.end();
        m.start(LEVEL_SELECT);
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("CLASSIC LEVELS WORLD 1"), "{t}");
        assert!(t.contains("TOTAL SCORE:"), "{t}");
        assert!(t.contains("TITLE 0SECOND"), "{t}");
        assert!(t.contains("Not Completed Yet"), "{t}");
        // Eight levels and the gate forward, but no gate back in world 0.
        let kinds: Vec<Square> = m.squares.iter().map(|s| s.square).collect();
        assert_eq!(kinds.len(), 9);
        assert_eq!(kinds[0], Square::NotWon);
        assert_eq!(kinds[1], Square::Perfect);
        assert_eq!(kinds[8], Square::NotWon);
        // More than five levels: two rows.
        assert_eq!(m.squares[5].x, -0.8);
        assert!((m.squares[5].y - 0.25).abs() < 1e-6);
        assert_eq!(m.cursor.map(|c| (c.x, c.y)), Some((-0.8, 0.6)));

        press(&mut m, &mut o, RIGHT);
        let t = text(&m);
        assert!(t.contains("Score: 500"), "{t}");
        assert!(t.contains("All Bonus Coins Found!"), "{t}");
        // Left from the first level wraps to the gate.
        press(&mut m, &mut o, LEFT);
        let a = press(&mut m, &mut o, LEFT);
        assert!(a.contains(&Action::Sound(MenuSound::Move)));
        assert_eq!(m.level, 8);
        assert!(text(&m).contains("Next World"));
        // The gate is closed: "no", and nothing else.
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::Sound(MenuSound::No)));
        assert_eq!(m.current, LEVEL_SELECT);
        // Up from the gate lands five levels back.
        press(&mut m, &mut o, UP);
        assert_eq!(m.level, 3);
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::StartLevel(3)));
        assert!(m.squares.is_empty() && m.cursor.is_none());
    }

    #[test]
    fn gates_move_between_worlds() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = selector(4);
        m.frame(KEYS, 1, &mut o);
        // Five levels: one row, and gates both ways.
        assert_eq!(m.squares.len(), 7);
        assert_eq!(m.squares[6].square, Square::Back);
        assert_eq!(m.squares[0].y, 0.425);
        // Down does nothing with five levels or fewer.
        press(&mut m, &mut o, MenuKeys { down: true, ..KEYS });
        assert_eq!(m.level, 40);
        press(&mut m, &mut o, LEFT);
        assert_eq!(m.level, 46);
        assert!(text(&m).contains("Previous World"));
        // Its line 2 is left from before: here, nothing yet.
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::LevelMusic(3)));
        assert_eq!((m.current, m.world(), m.level), (LEVEL_SELECT, 3, 30));

        // All worlds unlocked: the gate forward opens without a win.
        o.lock_worlds = 1;
        m.frame(KEYS, 1, &mut o);
        m.end();
        m.start(LEVEL_SELECT);
        assert_eq!(m.squares[10].square, Square::Forward);
        m.level = 40 - 10 + 10;
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::LevelMusic(4)));
        assert_eq!(m.world(), 4);

        // Through world 9's gate: the ending (the main menu for now).
        let mut m = selector(9);
        o.lock_worlds = 0;
        m.profile.as_mut().unwrap().scores.fill(1);
        m.end();
        m.start(LEVEL_SELECT);
        m.level = 100;
        press(&mut m, &mut o, ENTER);
        assert_eq!((m.current, m.world()), (ENDING, 10));

        // Esc: the level packs.
        let mut m = selector(0);
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, LEVEL_PACKS);
    }

    #[test]
    fn level_packs_open_their_worlds() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        let mut p = Profile::new("ZED", 0);
        p.world_packs = [3, 12, 25, 30];
        m.profile = Some(p);
        m.end();
        m.start(LEVEL_PACKS);
        let a = m.frame(KEYS, 1, &mut o);
        assert!(a.contains(&Action::StopMusic));
        let t = text(&m);
        assert!(t.contains("SELECT LEVEL PACK"), "{t}");
        assert!(t.contains("Custom Levels"), "{t}");
        assert!(!t.contains("Kids"), "{t}");
        // The description rises a tenth of a line a frame, to 14.
        let first = |m: &Menu| m.text.shown().iter().map(|l| l.y).fold(0.0, f32::max);
        assert!((first(&m) - (22.0 - 0.1)).abs() < 1e-4);
        for _ in 0..60 {
            m.frame(KEYS, 1, &mut o);
        }
        assert!((first(&m) - 17.0).abs() < 1e-4);
        assert_eq!(m.level_timer, 50);

        // Up wraps to the custom levels and starts the text again.
        press(&mut m, &mut o, UP);
        assert_eq!((m.option, m.level_timer), (5, 1));
        assert!(text(&m).contains("Hundreds of custom levels"));
        press(&mut m, &mut o, DOWN);
        // Enter plays no sound; the tutorials are world pack 1.
        let a = press(&mut m, &mut o, ENTER);
        assert!(!a.contains(&Action::Sound(MenuSound::Move)));
        assert!(a.contains(&Action::LevelMusic(12)));
        assert_eq!(
            (m.current, m.pack, m.world(), m.level),
            (LEVEL_SELECT, 1, 12, 120)
        );

        // Esc on the selector comes back here; Esc again to the main
        // menu and its music.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, LEVEL_PACKS);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, ENTER);
        assert_eq!((m.current, m.pack, m.world()), (LEVEL_SELECT, 2, 25));
        press(&mut m, &mut o, ESCAPE);
        let a = press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, MAIN);
        assert!(a.contains(&Action::TitleMusic));
    }

    #[test]
    fn tow_worlds_are_pack_three() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        let mut p = Profile::new("ZED", 0);
        p.world_packs = [3, 12, 25, 32];
        p.record_win(320, 700, false);
        m.profile = Some(p);
        m.end();
        m.start(LEVEL_PACKS);
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("Original Levels"), "{t}");
        assert!(t.contains("The Playground"), "{t}");
        for _ in 0..3 {
            press(&mut m, &mut o, DOWN);
        }
        assert!(text(&m).contains("Stinky and Loof's first"));
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::LevelMusic(32)));
        assert_eq!(
            (m.current, m.pack, m.world(), m.level),
            (LEVEL_SELECT, TOW_PACK, 32, 320)
        );
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("ORIGINAL LEVELS WORLD 3"), "{t}");
        assert!(t.contains("WINTER IN WONDERLAND"), "{t}");
        assert!(t.contains("Score: 700"), "{t}");
        // TOW's world names come in German too.
        o.language = 1;
        m.frame(KEYS, 1, &mut o);
        assert!(text(&m).contains("WINTER IM WONDERLAND"));
        o.language = 0;
        // The first TOW world has no gate back.
        m.set_world(30);
        m.end();
        m.start(LEVEL_SELECT);
        assert_eq!(m.squares.len(), 6);
        // Through the last world's gate: TOW's ending (the main menu for
        // now).
        m.profile.as_mut().unwrap().scores[300..400].fill(1);
        m.set_world(39);
        m.end();
        m.start(LEVEL_SELECT);
        m.level = 390 + m.level_count;
        let actions = press(&mut m, &mut o, ENTER);
        assert_eq!(m.world(), 40);
        assert!(actions.contains(&Action::TowEnding { skip: 0 }));

        // Without TOW's folder the line only says no.
        let mut m = Menu::new();
        m.tow = false;
        m.profile = Some(Profile::new("ZED", 0));
        m.end();
        m.start(LEVEL_PACKS);
        for _ in 0..3 {
            press(&mut m, &mut o, DOWN);
        }
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::Sound(MenuSound::No)));
        assert_eq!(m.current, LEVEL_PACKS);
    }

    #[test]
    fn the_playground_has_ten_levels_and_no_gates() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let mut m = Menu::new();
        m.profile = Some(Profile::new("ZED", 0));
        for (l, t) in m.level_titles.iter_mut().enumerate() {
            *t = format!("TITLE {l}");
        }
        m.end();
        m.start(LEVEL_PACKS);
        for _ in 0..4 {
            press(&mut m, &mut o, DOWN);
        }
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::LevelMusic(40)));
        assert_eq!((m.current, m.level), (PLAYGROUND, PLAYGROUND_LEVEL));
        assert!(m.selector.is_none());
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("WELCOME TO THE PLAYGROUND"), "{t}");
        assert!(t.contains("TITLE 400"), "{t}");
        assert_eq!(m.squares.len(), 10);
        assert!(m.squares.iter().all(|s| s.square == Square::Won));
        // Left wraps to the last level, down stops at it.
        press(&mut m, &mut o, LEFT);
        assert_eq!(m.level, 409);
        press(&mut m, &mut o, UP);
        assert_eq!(m.level, 404);
        press(&mut m, &mut o, DOWN);
        press(&mut m, &mut o, DOWN);
        assert_eq!(m.level, 409);
        press(&mut m, &mut o, RIGHT);
        assert_eq!(m.level, 400);
        press(&mut m, &mut o, RIGHT);
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::StartLevel(401)));
        assert_eq!(m.selector_menu(), PLAYGROUND);
        // Back after a win: the cursor starts on the first level again,
        // and the music plays on.
        m.level += 1;
        let a = {
            m.start(PLAYGROUND);
            m.frame(KEYS, 1, &mut o)
        };
        assert!(!a.iter().any(|a| matches!(a, Action::LevelMusic(_))));
        assert_eq!(m.level, PLAYGROUND_LEVEL);
        // Esc: the level packs.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, LEVEL_PACKS);
        // The player's world is left alone.
        assert_eq!(m.world(), 0);
    }

    fn level_file(id: i32, title: &str) -> Vec<u8> {
        let mut b = Vec::new();
        for s in ["Stinky & Loof Level File v5", "x"] {
            b.extend((s.len() as u32).to_le_bytes());
            b.extend(s.as_bytes());
        }
        b.extend(id.to_le_bytes());
        b.extend((title.len() as u32).to_le_bytes());
        b.extend(title.as_bytes());
        b
    }

    /// The custom level menu over a `CustomLevels` folder holding the
    /// folder `PACK`, and `levels` level files named `L00.lev` on.
    fn custom_menu(tag: &str, levels: usize) -> (Menu, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("wl-custom-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.join(custom::FOLDER);
        std::fs::create_dir_all(root.join("Pack")).unwrap();
        std::fs::write(root.join("Pack/inner.lev"), level_file(99, "Inner")).unwrap();
        for i in 0..levels {
            let title = format!("Level number {i} with a long title");
            std::fs::write(
                root.join(format!("L{i:02}.lev")),
                level_file(i as i32, &title),
            )
            .unwrap();
        }
        let mut m = Menu::new();
        let mut p = Profile::new("ZED", 0);
        p.record_custom_win(1, 0, 1234567, true);
        m.profile = Some(p);
        m.custom = Some(CustomLevels::new(&dir));
        m.level = 37;
        m.end();
        m.start(CUSTOM_LEVELS);
        (m, dir)
    }

    #[test]
    fn custom_levels_list_open_folders_and_play() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let (mut m, dir) = custom_menu("play", 3);
        m.frame(KEYS, 1, &mut o);
        let t = text(&m);
        assert!(t.contains("FILENAME:"), "{t}");
        assert!(t.contains("..(Go Back)"), "{t}");
        assert!(t.contains("PACK(Folder)"), "{t}");
        // The extension is cut, long titles too, and a score has five
        // digits at most.
        assert!(t.contains("L00Level number 0 with a lo...-----"), "{t}");
        assert!(t.contains("L01Level number 1 with a lo...12345*"), "{t}");

        // Into the folder and back out.
        press(&mut m, &mut o, DOWN);
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::Sound(MenuSound::Move)));
        assert_eq!(m.custom_dir, ["PACK"]);
        assert_eq!(m.files.len(), 2);
        assert!(text(&m).contains("INNER"));
        press(&mut m, &mut o, ENTER);
        assert!(m.custom_dir.is_empty());
        assert_eq!((m.current, m.option), (CUSTOM_LEVELS, 0));

        // Playing L01 keeps `CurrentLevel` to come back to.
        for _ in 0..3 {
            press(&mut m, &mut o, DOWN);
        }
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::StartLevel(custom::LEVEL)));
        assert_eq!((m.level, m.current_file), (custom::LEVEL, 3));
        assert_eq!(m.custom_path().as_deref(), Some("CustomLevels\\L01.LEV"));
        assert_eq!(m.custom_file().map(|f| f.id), Some(1));
        // Its music starts when the track changes.
        m.extra_music = 0;
        assert_eq!(m.custom_music(None), None);
        assert_eq!(m.custom_music(Some(3)), Some(3));
        assert_eq!(m.custom_music(Some(3)), None);

        // Back in the list on the level played.
        m.start(CUSTOM_LEVELS);
        assert_eq!(m.option, 3);
        // Esc to the packs; their selector has the level back.
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, LEVEL_PACKS);
        press(&mut m, &mut o, ENTER);
        assert_eq!(m.current, LEVEL_SELECT);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn custom_levels_scroll_page_and_jump() {
        let mut o = ConfigOptions::defaults(Game::Rtwp);
        let (mut m, dir) = custom_menu("scroll", 30);
        // "..", PACK, then 30 levels.
        assert_eq!(m.files.len(), 32);
        // Held down: a move on the first frame, then on the 7th, 9th...
        for _ in 0..10 {
            m.frame(DOWN, 1, &mut o);
        }
        assert_eq!(m.option, 3);
        m.frame(KEYS, 1, &mut o);
        // Down past the last line scrolls.
        for _ in 0..13 {
            press(&mut m, &mut o, DOWN);
        }
        assert_eq!((m.file_pos, m.option), (2, 14));
        let pg_down = MenuKeys {
            page_down: true,
            ..KEYS
        };
        press(&mut m, &mut o, pg_down);
        assert_eq!((m.file_pos, m.option), (17, 14));
        let pg_up = MenuKeys {
            page_up: true,
            ..KEYS
        };
        press(&mut m, &mut o, pg_up);
        assert_eq!((m.file_pos, m.option), (2, 14));
        press(&mut m, &mut o, pg_up);
        assert_eq!((m.file_pos, m.option), (0, 1));
        // A letter: the first level from it on, folders skipped.
        m.type_char(b'p');
        m.frame(KEYS, 1, &mut o);
        assert_eq!((m.file_pos, m.option), (17, 14));
        m.type_char(b'l');
        m.frame(KEYS, 1, &mut o);
        assert_eq!((m.file_pos, m.option), (2, 0));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
