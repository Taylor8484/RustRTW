//! TOW's ending (TOW's `MenuLoop12`), played on the `theend.lev` stage:
//! the level's monsters, items and fish run while the script moves the
//! camera over them, fades the light in and out and shows the text. At
//! the end the two players leave for their holiday and sparkles fill the
//! sky behind "THE END".
//!
//! TOW counts `LevelTimer` once per frame on top of once per tick; its
//! frames keep pace with its ticks, so here the timer goes up by two per
//! tick and the text and sparkles are laid out as if each tick was a
//! frame.
//!
//! This module has no Bevy state; `sim` runs it, `hud` shows its text and
//! `flow` leaves it on Esc.

use wonderland_logic::md2::LOOP;
use wonderland_logic::player::Skin;
use wonderland_logic::world::World;

use crate::menu::{MenuKeys, Text};

/// Rows of the text's first line and how many frames an MD2 can have.
const FIRST_ROW: f32 = 15.0;
const FRAMES: i32 = 1000;
/// The sparkles.
pub const STARS: usize = 51;
/// `CameraRange`'s far plane and `CameraFogRange`.
pub const FAR: f32 = 1000.0;
pub const FOG_RANGE: (f32, f32) = (100.0, 1000.0);

/// The text: from, to (exclusive) and the lines by language.
const PAGES: [(i32, i32, [[&str; 5]; 3]); 24] = [
    (
        501,
        901,
        [
            ["And so,", "", "", "", ""],
            ["Und so", "", "", "", ""],
            ["Alors,", "", "", "", ""],
        ],
    ),
    (
        901,
        1400,
        [
            ["And so,", "at the end of their journey,", "", "", ""],
            ["Und so", "sehen Stinky und Loof", "", "", ""],
            ["Alors,", "a la fin de leur periple,", "", "", ""],
        ],
    ),
    (
        1801,
        2600,
        [
            ["with all the Rainbow Coins", "safe and sound,", "", "", ""],
            ["am Ende ihrer Reise", "", "", "", ""],
            [
                "avec un tas de sous arc-en-ciel",
                "plein les poches,",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        3001,
        4000,
        [
            [
                "Stinky and Loof watched a new day",
                "dawn over Wonderland.",
                "",
                "",
                "",
            ],
            ["einem neuen Tag", "im Wonderland entgegen.", "", "", ""],
            [
                "Stinky et Loof contemplerent un autre",
                "coucher de soleil sur Wonderland.",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        5001,
        5750,
        [
            ["Peace spread across the land,", "", "", "", ""],
            ["Frieden breitet sich", "", "", "", ""],
            ["La paix se repandit dans le pays,", "", "", "", ""],
        ],
    ),
    (
        5750,
        7000,
        [
            [
                "Peace spread across the land,",
                "touching all creatures great and small.",
                "",
                "",
                "",
            ],
            ["Frieden breitet sich", "im Wonderland aus.", "", "", ""],
            [
                "La paix se repandit dans le pays,",
                "touchant toutes les creatures, grandes et petites.",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        8001,
        8500,
        [
            ["The Z-Bots", "", "", "", ""],
            ["Die Z-Bots", "", "", "", ""],
            ["Les Zi-Bots", "", "", "", ""],
        ],
    ),
    (
        8500,
        9500,
        [
            ["The Z-Bots", "promised to be less shocking.", "", "", ""],
            [
                "Die Z-Bots",
                "versprachen freundlicher zu sein.",
                "",
                "",
                "",
            ],
            [
                "Les Zi-Bots",
                "promirent d'etre moins tannants.",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        11001,
        11500,
        [
            ["The confounding Coilys", "", "", "", ""],
            ["Die Coilys", "", "", "", ""],
            ["Les deconcertants Coilys", "", "", "", ""],
        ],
    ),
    (
        11500,
        12250,
        [
            [
                "The confounding Coilys",
                "became more cooperative.",
                "",
                "",
                "",
            ],
            ["Die Coilys", "wurden ruhiger.", "", "", ""],
            [
                "Les deconcertants Coilys",
                "devinrent plus cooperatifs.",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        13501,
        14000,
        [
            ["And cooler heads prevailed", "", "", "", ""],
            ["Und die Kaboom's", "", "", "", ""],
            ["Et la plupart des Kaboom!s", "", "", "", ""],
        ],
    ),
    (
        14000,
        15000,
        [
            [
                "And cooler heads prevailed",
                "among the Kaboom!s.",
                "",
                "",
                "",
            ],
            ["Und die Kaboom's", "behielten kuehlere Koepfe.", "", "", ""],
            [
                "Et la plupart des Kaboom!s",
                "se calmerent le pompon.",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        16000,
        17000,
        [
            ["As for Stinky and Loof...", "", "", "", ""],
            ["Und Stinky und Loof?", "", "", "", ""],
            ["Stinky et Loof aussi...", "", "", "", ""],
        ],
    ),
    (
        17500,
        18000,
        [
            ["With their quest complete,", "", "", "", ""],
            ["Da ihre Reise erfolgreich war", "", "", "", ""],
            ["leur quete achevee,", "", "", "", ""],
        ],
    ),
    (
        18000,
        18500,
        [
            [
                "With their quest complete,",
                "and Wonderland saved,",
                "",
                "",
                "",
            ],
            [
                "Da ihre Reise erfolgreich war",
                "und Wonderland gerettet ist",
                "",
                "",
                "",
            ],
            ["leur quete achevee,", "et Wonderland libere,", "", "", ""],
        ],
    ),
    (
        18500,
        19500,
        [
            [
                "With their quest complete,",
                "and Wonderland saved,",
                "they decided to take a well-deserved vacation!",
                "",
                "",
            ],
            [
                "Da ihre Reise erfolgreich war",
                "und Wonderland gerettet ist",
                "haben beide sich einen Urlaub verdient!",
                "",
                "",
            ],
            [
                "leur quete achevee,",
                "et Wonderland libere,",
                "ils deciderent de prendre",
                "des vacances bien merites!",
                "",
            ],
        ],
    ),
    (
        20500,
        21000,
        [
            ["And so the time has come", "", "", "", ""],
            ["So ist es an der Zeit", "", "", "", ""],
            ["Ainsi donc, le temps est venu", "", "", "", ""],
        ],
    ),
    (
        21000,
        21500,
        [
            [
                "And so the time has come",
                "for Stinky and Loof",
                "",
                "",
                "",
            ],
            [
                "So ist es an der Zeit",
                "dir lebe wohl zu sagen.",
                "",
                "",
                "",
            ],
            [
                "Ainsi donc, le temps est venu",
                "pour Stinky et Loof",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        21500,
        22000,
        [
            [
                "And so the time has come",
                "for Stinky and Loof",
                "to take their leave of you.",
                "",
                "",
            ],
            [
                "So ist es an der Zeit",
                "dir lebe wohl zu sagen.",
                "",
                "",
                "",
            ],
            [
                "Ainsi donc, le temps est venu",
                "pour Stinky et Loof",
                "de vous quitter.",
                "",
                "",
            ],
        ],
    ),
    (
        23001,
        23500,
        [
            ["With heavy hearts", "", "", "", ""],
            ["Schweren Herzens", "", "", "", ""],
            ["Avec le coeur gros,", "", "", "", ""],
        ],
    ),
    (
        23501,
        24300,
        [
            ["With heavy hearts", "filled with fond memories", "", "", ""],
            ["Schweren Herzens", "sagen Stinky und Loof", "", "", ""],
            [
                "Avec le coeur gros,",
                "remplis de precieux souvenirs",
                "",
                "",
                "",
            ],
        ],
    ),
    (
        24801,
        27000,
        [
            ["", "", "they bid you Good Bye,", "", ""],
            ["", "", "auf Wiedersehn...", "", ""],
            ["", "", "ils vous disent Au Revoir,", "", ""],
        ],
    ),
    (
        28001,
        29000,
        [
            ["", "", "", "and Good Night!", ""],
            ["", "", "", "und Gute Nacht!", ""],
            ["", "", "", "et Bonne Nuit.", ""],
        ],
    ),
    (
        30501,
        i32::MAX,
        [
            ["THE END", "", "", "", ""],
            ["ENDE", "", "", "", ""],
            ["Fin", "", "", "", ""],
        ],
    ),
];

fn sin(deg: f32) -> f32 {
    deg.to_radians().sin()
}

fn cos(deg: f32) -> f32 {
    deg.to_radians().cos()
}

/// One sparkle: position, roll and size.
pub type Star = ([f32; 3], f32, f32);

/// The ending in progress.
#[derive(Debug, Clone)]
pub struct TowEnding {
    pub text: Text,
    language: usize,
    /// `ShiftKeyPressed`: the players have turned to go.
    turned: bool,
    /// `CursorKeyPressed`.
    cursor_key: bool,
    /// `CameraClsColor` and `CameraFogColor`, 0-255.
    pub sky: [i32; 3],
    pub stars: Vec<Star>,
    scale: f32,
    page: Option<usize>,
}

impl TowEnding {
    /// `StartMenu(12)`, after `StartLevel`: the players stand on their
    /// marks in their best faces.
    pub fn new(world: &mut World, language: i32) -> Self {
        for (k, offset, turn) in [(1, [0.2, 0.9, 0.05], 160), (2, [0.0, 0.9, 0.0], 170)] {
            if let Some(p) = world.players.get_mut(k) {
                (0..3).for_each(|i| p.position[i] += offset[i]);
                p.rotation += turn;
                p.skin = Skin::C;
            }
        }
        world.camera.position = [25.0, 0.0, -100.0];
        world.camera.pitch = 0.0;
        world.camera.yaw = 0.0;
        world.level_timer = 0;
        Self {
            text: Text::default(),
            language: language.clamp(0, 2) as usize,
            turned: false,
            cursor_key: true,
            sky: [170, 170, 255],
            stars: vec![([0.0; 3], 0.0, 0.2); STARS],
            scale: 0.2,
            page: None,
        }
    }

    /// One frame and one tick.
    pub fn tick(&mut self, world: &mut World) {
        world.level_timer += 1;
        self.lay_out(world.level_timer);
        self.script(world);
        world.step_stage();
        self.place_stars(world.level_timer);
    }

    fn lay_out(&mut self, timer: i32) {
        let page = PAGES.iter().position(|p| p.0 <= timer && timer < p.1);
        if page == self.page {
            return;
        }
        self.page = page;
        for row in 0..5 {
            let line = page.map_or("", |p| PAGES[p].2[self.language][row]);
            self.text
                .center(line, FIRST_ROW + row as f32, 1.0, 50 + row as i32);
        }
        self.text.activate(|| None);
    }

    /// The tick's camera, lights and players.
    fn script(&mut self, world: &mut World) {
        let t = world.level_timer;
        let f = t as f32;
        let c = &mut world.camera;
        if t < 10000 {
            world.lights.light = [-255 + 510 * t / 10000; 3];
            world.lights.ambient = [255 * t / 10000; 3];
        }
        if t < 6000 {
            let a = sin(f / 66.6666);
            c.position = [20.5, -23.0 + 30.0 * a, -102.0 + 6.0 * a];
        }
        if t > 3000 && t <= 8800 {
            c.pitch = 10.0 - 10.0 * cos((f - 3000.0) / 16.6666);
        }
        let z = -50.0 - 46.0 * cos((f - 6000.0) / 100.0);
        let x = 9.5 + 3.0 * cos((f - 6000.0) / 26.0);
        if t > 6000 && t < 10680 {
            c.position = [
                13.5 + 7.0 * cos((f - 6000.0) / 26.0),
                3.0 + 4.0 * cos((f - 6000.0) / 21.0),
                z,
            ];
        }
        if (10680..13560).contains(&t) {
            c.position = [x, 3.0 + 4.0 * cos((f - 6000.0) / 21.0), z];
        }
        if (13560..17340).contains(&t) {
            c.position = [x, 5.0 + 2.0 * cos((f - 6000.0) / 21.0), z];
        }
        if (17340..22000).contains(&t) {
            c.position = [x, 4.0 - cos((f - 17340.0) / 31.0), z];
        }
        if t > 22000 && t < 26000 {
            c.pitch = 10.0 - 10.0 * cos(5800.0 / 16.6666);
            c.yaw = -90.0 + 90.0 * cos((f - 22000.0) / 22.2222);
        }
        if t > 22000 && t < 30000 {
            if t < 22020 && !self.turned {
                self.turned = true;
                let stinky = world.players[1].position;
                for (k, speed, turn) in [(1, 0.07, 20), (2, 0.1, 10)] {
                    let p = &mut world.players[k];
                    p.anim.start(LOOP, speed, 44, 52, FRAMES);
                    p.rotation += turn;
                }
                world.players[2].position = [stinky[0] + 1.5, stinky[1], stinky[2]];
            }
            world.camera.position = [
                9.5 + 3.0 * cos(16000.0 / 26.0),
                4.0 - cos((22000.0 - 17340.0) / 31.0),
                -50.0 - 46.0 * cos((22000.0 - 6000.0) / 100.0) + ((t - 22000) / 200) as f32,
            ];
        }
        if t > 28000 && t < 30000 {
            let d = t - 28000;
            world.lights.light = [255 - 510 * d / 2000; 3];
            world.lights.ambient = [255 - 255 * d / 2000; 3];
            self.sky = [
                170 - 170 * d / 2000,
                170 - 170 * d / 2000,
                255 - 255 * d / 2000,
            ];
        }
        if t > 30000 {
            world.camera.position = [
                9.5 + 3.0 * cos(20000.0 / 26.0),
                4.0 - cos(((22000 - 17340) / 31) as f32),
                100.0,
            ];
        }
    }

    /// What follows the `Flip`: first a trail of sparkles over the land,
    /// then a wide swirl far off in the sky.
    fn place_stars(&mut self, timer: i32) {
        let t = timer as f32;
        if (12000..22000).contains(&timer) {
            self.scale = if timer > 18000 {
                (t - 18000.0) / 1000.0
            } else {
                0.0
            };
        }
        for (i, s) in self.stars.iter_mut().enumerate() {
            let fi = i as f32;
            let k = t - 60.0 * fi - 7000.0;
            s.0 = if timer < 12000 {
                [
                    20.0 - 22.0 * sin(k / 15.0) + sin(fi * 35.0),
                    7.0 * cos(k / 5.0) - k / 200.0 + sin(fi * 57.0),
                    -95.0 + k / 30.0 + cos(fi * 109.0),
                ]
            } else {
                [
                    34.0 + 90.0 * cos(k / 8.3333),
                    -10.0 + 30.0 * sin(k / 4.1666),
                    500.0,
                ]
            };
            s.1 = ((timer + 10 * i as i32) % 360) as f32;
            s.2 = self.scale;
        }
    }

    /// `KeyHit(1)`: whether to leave for the main menu.
    pub fn leave(&mut self, keys: MenuKeys) -> bool {
        let leave = keys.escape && !self.cursor_key;
        self.cursor_key = keys.escape;
        leave
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ending() -> TowEnding {
        TowEnding {
            text: Text::default(),
            language: 0,
            turned: false,
            cursor_key: true,
            sky: [170, 170, 255],
            stars: Vec::new(),
            scale: 0.2,
            page: None,
        }
    }

    fn text(e: &TowEnding) -> String {
        e.text.shown().iter().map(|l| l.ascii as char).collect()
    }

    #[test]
    fn pages_come_and_go() {
        let mut e = ending();
        e.lay_out(100);
        assert_eq!(text(&e), "");
        e.lay_out(600);
        assert!(text(&e).starts_with("And so,"));
        e.lay_out(1500);
        assert_eq!(text(&e), "");
        e.lay_out(30600);
        assert_eq!(text(&e), "THE END");
    }

    #[test]
    fn esc_needs_a_fresh_press() {
        let mut e = ending();
        let esc = MenuKeys {
            escape: true,
            ..MenuKeys::default()
        };
        assert!(!e.leave(esc));
        assert!(!e.leave(MenuKeys::default()));
        assert!(e.leave(esc));
    }

    #[test]
    fn every_page_is_in_all_three_languages_and_in_order() {
        for (i, p) in PAGES.iter().enumerate() {
            assert!(p.0 < p.1);
            assert!(i == 0 || PAGES[i - 1].1 <= p.0 + 1);
            for lines in p.2 {
                assert!(lines.iter().any(|l| !l.is_empty()));
            }
        }
        assert_eq!(PAGES.last().unwrap().2[0][0], "THE END");
    }
}
