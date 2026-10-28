//! The menus as `RTWP/menu.bb` runs them: `StartMenu`, `EndMenu`, the
//! `MenuLoop` dispatch, the menu text and `MenuBkgUpdate`'s animation.
//!
//! This module has no Bevy state, so it is tested headless; `menu_scene`
//! draws it and carries out the [`Action`]s. Ported so far: the title (0),
//! the main menu (1), options (9), "Exit to Windows?" (10) and "Go
//! Online?" (15). START GAME (player select, 2) is a stop-gap that plays
//! the pack from its first level until profiles (step 4b) and the
//! campaign (4c) arrive.
//!
//! Each `MenuLoopN` is one frame: it lays out the text (the letters get
//! their alpha when first shown), runs the frame's ticks of
//! `MenuBkgUpdate`, renders, and then reads the keys. Keys are read once
//! per frame as held keys; `CursorKeyPressed` turns a held key into one
//! press.

use std::f32::consts::FRAC_1_SQRT_2;

use wonderland_formats::config::Options as ConfigOptions;

use crate::hud::{Edge, Letter, display_text, pinned};

pub const TITLE: i32 = 0;
pub const MAIN: i32 = 1;
pub const PLAYER_SELECT: i32 = 2;
pub const OPTIONS: i32 = 9;
pub const EXIT: i32 = 10;
pub const ONLINE: i32 = 15;

/// The keys the menus read, held this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuKeys {
    /// Cursor up or keypad 8.
    pub up: bool,
    /// Cursor down or keypad 2.
    pub down: bool,
    /// Return or keypad Enter.
    pub enter: bool,
    pub escape: bool,
    pub space: bool,
    /// The number keys 1 and 0 (the options menu's secret song).
    pub one: bool,
    pub zero: bool,
}

/// The sounds the menus play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSound {
    /// `SFXMenuSound(0)` (`sound\menumove.wav`) through `PlayMySound`, so
    /// at the sound level and not at all when sound is off.
    Move,
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
    /// Leave the menus and play (for now, the pack from its first level).
    StartGame,
    /// An option changed; the game's settings follow it.
    OptionsChanged,
    /// `SaveOptions`.
    SaveOptions,
    /// `GameExit`, then `ExecFile` of `url` if one was picked.
    Quit {
        url: Option<&'static str>,
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
/// at `(x, y, 1.1)` with half-size `(half_width, 0.02)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selector {
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
}

impl Selector {
    /// `PositionEntity MenuSelector, -.5+24.4*.02+.01,
    /// 0.372-Float(row)*.044+.027, 1.1` and `ScaleSprite width*.012, .02`.
    fn at_row(row: i32, width: f32) -> Self {
        Self {
            x: -0.5 + 24.4 * 0.02 + 0.01,
            y: 0.372 - row as f32 * 0.044 + 0.027,
            half_width: width * 0.012,
        }
    }

    pub const Z: f32 = 1.1;
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
    /// `StartMenu(0)`.
    pub fn new() -> Self {
        let mut m = Self::default();
        m.start(TITLE);
        m.actions.clear();
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
        }
        self.current = menu;
        self.update_selector();
        self.actions.push(Action::Entered(menu));
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
        match self.current {
            TITLE => self.title(keys, ticks),
            MAIN => self.main(keys, ticks),
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
                    self.actions.push(Action::StartGame);
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
        enter: false,
        escape: false,
        space: false,
        one: false,
        zero: false,
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
        assert_eq!(m.frame(KEYS, 0, &mut o), vec![Action::Entered(MAIN)]);
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
        assert!(!m.frame(ENTER, 1, &mut o).contains(&Action::StartGame));
        m.frame(KEYS, 1, &mut o);
        let a = press(&mut m, &mut o, ENTER);
        assert!(a.contains(&Action::StartGame));
        assert!(m.text.shown().is_empty() && !m.logos);
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
}
