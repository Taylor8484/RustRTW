//! The menus as `RTWP/menu.bb` runs them: `StartMenu`, `EndMenu`, the
//! `MenuLoop` dispatch, the menu text and `MenuBkgUpdate`'s animation.
//!
//! This module has no Bevy state, so it is tested headless; `menu_scene`
//! draws it and carries out the [`Action`]s. Ported so far: the title (0),
//! the main menu (1), player select (2), new player (3), the level
//! selector (4), "Confirm Player Deletion?" (6), options (9), "Exit to
//! Windows?" (10) and "Go Online?" (15).
//!
//! Stop-gaps until the menus they stand for are ported: choosing or
//! creating a player should open the level pack menu (13, step 4d); for
//! now it goes to the classic pack's level selector, as picking the
//! classic levels there does. Esc on the level selector goes to the main
//! menu, where 13 would come first. The endings (12 and 16, step 4f) go
//! to the main menu.
//!
//! Each `MenuLoopN` is one frame: it lays out the text (the letters get
//! their alpha when first shown), runs the frame's ticks of
//! `MenuBkgUpdate`, renders, and then reads the keys. Keys are read once
//! per frame as held keys; `CursorKeyPressed` turns a held key into one
//! press.

use std::collections::VecDeque;
use std::f32::consts::FRAC_1_SQRT_2;

use wonderland_campaign::profile::{Entry, MAX_PLAYERS, Players, Profile};
use wonderland_campaign::worlds::{self, LEVELS, Worlds, title_lines, total_score};
use wonderland_formats::config::Options as ConfigOptions;

use crate::hud::{Edge, Letter, display_text, pinned};

pub const TITLE: i32 = 0;
pub const MAIN: i32 = 1;
pub const PLAYER_SELECT: i32 = 2;
pub const NEW_PLAYER: i32 = 3;
pub const LEVEL_SELECT: i32 = 4;
pub const DELETE_PLAYER: i32 = 6;
pub const OPTIONS: i32 = 9;
pub const EXIT: i32 = 10;
pub const ONLINE: i32 = 15;
/// The level pack menu, not ported yet (step 4d).
const LEVEL_PACKS: i32 = 13;

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
    /// [`Menu::level`] of the world table.
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
    /// `StopChannel Music`.
    StopMusic,
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
    fn activate(&mut self, alpha: impl Fn() -> Option<f32>) {
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
    /// `CursorKeyPressed`.
    latched: bool,
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
    /// `MilliSecs()`, for a new player's id.
    pub millisecs: i32,
    /// The player chosen or created. Its `world` is `CurrentWorld`.
    pub profile: Option<Profile>,
    /// `CurrentLevel`.
    pub level: i32,
    /// `WhichWorldPack`.
    pub pack: i32,
    /// `LockWorlds` from the options: 1 opens every world's gate.
    lock_worlds: i32,
    /// The world table.
    pub worlds: Worlds,
    /// Each level's title as its file stores it (`#` between lines), by
    /// level number; empty when it could not be read.
    pub level_titles: Vec<String>,
    /// `CurrentWorldLevelN`.
    level_count: i32,
    /// `CurrentWorldLevelName$`: three lines for each square. Kept from
    /// one world to the next, as lines the selector does not set are.
    level_names: Vec<[String; 4]>,
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
            current: TITLE,
            option: 0,
            timer: 0,
            level_timer: 0,
            latched: false,
            secret_music: 0,
            version_no1: 10,
            language: 0,
            players: None,
            list: Vec::new(),
            to_delete: 0,
            player_name: String::new(),
            typed: VecDeque::new(),
            millisecs: 0,
            profile: None,
            level: 0,
            pack: 0,
            lock_worlds: 0,
            worlds: Worlds::rtwp(),
            level_titles: vec![String::new(); LEVELS],
            level_count: 0,
            level_names: vec![Default::default(); 13],
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
        if !matches!(menu, 4 | 5 | 12 | 16) {
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
            // `GetKey`'s queue holds every key typed since the game
            // started; only what is typed on this screen counts here.
            self.typed.clear();
        } else if menu == LEVEL_SELECT {
            // Coming from the level pack menu, which player select and
            // new player stand in for until it is ported.
            if matches!(self.current, LEVEL_PACKS | PLAYER_SELECT | NEW_PLAYER) {
                self.actions.push(Action::LevelMusic(self.world()));
            }
            self.make_squares();
        }
        self.current = menu;
        self.update_selector();
        self.actions.push(Action::Entered(menu));
    }

    /// A character typed (what `GetKey` returns), for the name entry.
    pub fn type_char(&mut self, c: u8) {
        if self.current == NEW_PLAYER {
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
        self.latched = true;
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
            DELETE_PLAYER => self.delete_player(keys, ticks),
            OPTIONS => self.options(keys, ticks, options),
            EXIT => self.exit(keys, ticks),
            ONLINE => self.online(keys, ticks),
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
            if !self.latched {
                self.sound();
                self.option -= 1;
                if self.option == -1 {
                    self.option = count - 1;
                }
                self.latched = true;
            }
        } else if keys.down {
            if !self.latched {
                self.sound();
                self.option += 1;
                if self.option == count {
                    self.option = 0;
                }
                self.latched = true;
            }
        } else if keys.enter {
            if !self.latched {
                self.sound();
                self.latched = true;
                return Some(Press::Enter);
            }
        } else if keys.escape {
            if !self.latched {
                self.latched = true;
                return Some(Press::Escape);
            }
        } else {
            self.latched = false;
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
            tx.center("(Press Space Key To Start)", 16.0, 1.0, 57);
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
        if (keys.escape || keys.space || keys.enter) && !self.latched {
            self.latched = true;
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
                    "LEVEL EDITOR",
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
                    "LEVEL EDITOR",
                    "SORTIE VERS WINDOWS",
                ],
                "Pour choisir, utilise les fleches et 'Entree'",
            ),
            _ => (
                [
                    "START GAME",
                    "OPTIONS",
                    "ONLINE SOLUTIONS",
                    "LEVEL EXCHANGE",
                    "LEVEL EDITOR",
                    "EXIT TO WINDOWS",
                ],
                "Use Cursor Keys and Enter to Select",
            ),
        };
        for (i, item) in items.iter().enumerate() {
            self.text.center(item, 9.0 + i as f32, 1.0, 51 + i as i32);
        }
        if self.level_timer % 200 < 100 {
            self.text.center(hint, 17.0, 1.0, 57);
        } else {
            self.text.center("", 17.0, 1.0, 57);
        }
        self.place_selector(9 + self.option, 25.0);
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

        match self.navigate(keys, 6) {
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
                // The original quits and runs the level editor, which is
                // not ported yet (step 8).
                4 => {}
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
        // Holding 1, 2 and 3 jumps to the ending (`StartMenu(12)`), which
        // comes with step 4f.
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
            if !self.latched {
                self.latched = true;
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
                        self.play();
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

    /// `StartMenu(13)`, the level pack menu, which comes with step 4d;
    /// until then it is as if the classic levels were picked there.
    fn play(&mut self) {
        self.pack = 0;
        if let Some(p) = &mut self.profile {
            p.world = p.world_packs[0];
        }
        self.level = self.world() * 10;
        self.start(LEVEL_SELECT);
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
            _ => ("KIDS LEVELS ", 30),
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
                    tx.center(b, 14.0, 1.0, 59);
                } else {
                    tx.display("", 0.0, 0.0, 1.0, 58);
                    tx.display("", 0.0, 0.0, 1.0, 59);
                }
                let (hint, x) = match lang {
                    1 => ("Benutze die Pfeiltasten zur Levelauswahl.", 5.0),
                    2 => ("Utilise les fleches pour choisir les niveaux.", 3.0),
                    _ => ("Use Cursor Keys to select different Levels.", 4.0),
                };
                tx.display(hint, x, 18.0, 1.0, 60);
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
            if !self.latched {
                self.latched = true;
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
                            // The endings (menus 12 and 16), and the end
                            // of the tutorials.
                            10 | 12 | 28 => self.start(MAIN),
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
            if !self.latched {
                self.sound();
                self.latched = true;
                self.level -= 1;
                if self.level < world * 10 {
                    self.level = world * 10 + n + back;
                }
            }
        } else if keys.right {
            if !self.latched {
                self.sound();
                self.latched = true;
                self.level += 1;
                if self.level >= world * 10 + n + 1 + back {
                    self.level = world * 10;
                }
            }
        } else if keys.up {
            if !self.latched && n > 5 {
                self.sound();
                self.latched = true;
                self.level -= 5;
                if self.level < world * 10 {
                    self.level = world * 10 + n - 1;
                }
            }
        } else if keys.down {
            if !self.latched && n > 5 {
                self.sound();
                self.latched = true;
                self.level += 5;
                if self.level > world * 10 + n - 1 {
                    self.level = world * 10 + n - 1;
                }
            }
        } else if keys.escape {
            if !self.latched {
                self.latched = true;
                self.free_squares();
                self.end();
                // The level pack menu (13) comes first in the original.
                self.start(MAIN);
            }
        } else {
            self.latched = false;
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
        // Moved above the top of the screen, out of sight.
        let half_width = self.selector_at.map_or(25.0 * 0.012, |s| s.half_width);
        self.selector_at = Some(Selector {
            x: -0.5 + 24.4 * 0.02 + 0.01,
            y: 0.375 - (-2.0) * 0.04 - 0.013,
            z: 1.001,
            half_width,
        });
        self.update_selector();
        self.text.activate(|| None);

        self.run_ticks(ticks, false);

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
            if !self.latched {
                self.actions.push(Action::Sound(MenuSound::Type));
                self.player_name.push(c);
                self.latched = true;
            }
        } else if keys.backspace && !self.player_name.is_empty() {
            if !self.latched {
                self.actions.push(Action::Sound(MenuSound::Type));
                self.player_name.pop();
                self.latched = true;
            }
        } else if keys.enter {
            if !self.latched {
                self.actions.push(Action::Sound(MenuSound::LastCoin));
                let profile = Profile::new(&self.player_name, self.millisecs);
                if let Some(Err(e)) = self.players.as_ref().map(|p| p.create(&profile)) {
                    self.actions
                        .push(Action::Warn(format!("{}: {e}", profile.name)));
                }
                self.profile = Some(profile);
                self.latched = true;
                self.end();
                self.play();
            }
        } else if keys.escape {
            if !self.latched {
                self.latched = true;
                self.end();
                // With no players this comes straight back here.
                self.start(PLAYER_SELECT);
            }
        } else {
            self.latched = false;
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

    /// `MenuLoop10`: "Exit to Windows?". Escape here quits too.
    fn exit(&mut self, keys: MenuKeys, ticks: u32) {
        let (title, rule, yes, no) = match self.language {
            1 => (
                "Wonderland Beenden?",
                "--------------------",
                "JA BITTE",
                "NEIN DANKE",
            ),
            2 => (
                "Sortie vers Windows?",
                "--------------------",
                "BIEN SUR",
                "NON MERCI",
            ),
            _ => (
                "Exit to Windows?",
                "----------------",
                "SURE THING",
                "NO THANKS",
            ),
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
                self.text.center("Sortie vers Windows?", 7.0, 1.0, 50);
                self.text.center("--------------------", 8.0, 1.0, 51);
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
        // Esc on "Exit to Windows?" quits too.
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
        assert_eq!(m.option, 5);
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
        // The classic levels' selector, with the first world's music.
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
        m.start(PLAYER_SELECT);
        m.end();
        m.play();
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
        assert_eq!((m.current, m.world()), (MAIN, 10));

        // Esc: the main menu, for now.
        let mut m = selector(0);
        press(&mut m, &mut o, ESCAPE);
        assert_eq!(m.current, MAIN);
    }
}
