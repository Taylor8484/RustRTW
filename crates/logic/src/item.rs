//! Items: `CreateItem`, `ControlItems`, `PickUpItem`, `KillItem` and
//! `CleanUpItems` (`RTWP/gameobject.bb`), plus the coins and bonus coins
//! that rainbow spirits and shadow stinkers take in `CanMonsterMove`.
//!
//! Item types: 0 coin, 1 exit gate, 2 clock (30 more seconds), 3 bonus coin
//! (250 points). Taking the last coin opens the final gate. The exit gate
//! is never removed; each player who reaches it shrinks away, and the level
//! is won when no player is left.
//!
//! A taken item is only marked dead and stays in the list until
//! `CleanUpItems` at the end of the tick. Nothing checks the mark, so an
//! item can be taken twice in one tick: by a monster in `ControlMonsters`
//! and then by a player in `ControlItems`. The coin count drops twice.

use crate::World;
use crate::sign::LEVEL_COMPLETE;
use crate::world::{Completion, Event, Sound};

/// One item (the `Item*` arrays).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    /// `ItemType`: 0 coin, 1 exit gate, 2 clock, 3 bonus coin.
    pub kind: i32,
    /// Removed at the end of the tick.
    pub dead: bool,
    /// `ItemData`: a clock's starting yaw, `Rand(1, 360)`. The renderer
    /// turns clocks by it.
    pub data: i32,
}

impl World {
    /// `CreateItem`.
    pub(crate) fn create_item(&mut self, x: i32, y: i32, kind: i32) {
        match kind {
            0 => self.coins += 1,
            3 => self.bonus_coins += 1,
            _ => {}
        }
        let data = if kind == 2 {
            self.session.rng.rand(1, 360)
        } else {
            0
        };
        self.items.push(Item {
            x,
            y,
            kind,
            dead: false,
            data,
        });
    }

    /// `ControlItems`. Coins, clocks and the exit gate spin and pulse in the
    /// renderer; here the exit gate draws for its star particles, and a
    /// player past halfway onto an item's tile picks it up. One pick-up
    /// ends the pass, so the items after it wait for the next tick.
    pub(crate) fn control_items(&mut self) {
        for i in 0..self.items.len() {
            if self.items[i].kind == 1 && self.session.rng.rand(0, 100) < 5 {
                // A rising star: three speeds and a spin.
                self.session.rng.skip(4);
            }
            let (x, y) = (self.items[i].x, self.items[i].y);
            for k in 1..self.players.len() {
                let p = &self.players[k];
                if p.exists
                    && (p.x2, p.y2) == (x, y)
                    && p.moving
                    && p.moving_timer < p.speed / 2
                    && !self.completed(k)
                {
                    self.pick_up_item(i, k);
                    return;
                }
            }
        }
    }

    /// `PickUpItem`: player `who` takes item `i`.
    pub(crate) fn pick_up_item(&mut self, i: usize, who: usize) {
        match self.items[i].kind {
            0 => {
                self.events.push(Event::Sound(Sound::Coin(who)));
                // A sparkle with a random spin.
                self.session.rng.skip(1);
                self.coins -= 1;
                if self.coins == 0 {
                    self.events.push(Event::Sound(Sound::LastCoin));
                    self.toggle_laser_gate(0, true);
                }
            }
            1 => self.reach_exit(who),
            2 => {
                self.map_time += 30;
                self.events.push(Event::Sound(Sound::Clock(who)));
            }
            3 => {
                self.bonus_coins -= 1;
                self.events.push(Event::Sound(Sound::Bonus(who)));
                self.level_score += 250;
            }
            _ => {}
        }
        if self.items[i].kind != 1 {
            self.kill_item(i);
        }
    }

    /// The exit-gate half of `PickUpItem`: the player shrinks away, then
    /// either the focus moves on or, with nobody left, the level is won.
    fn reach_exit(&mut self, who: usize) {
        let p = &mut self.players[who];
        p.shrink = 60;
        p.shadow = false;
        self.events.push(Event::Sound(Sound::Exit(who)));
        self.both_completed += 1 << (who - 1);

        let all_out = (1..self.players.len()).all(|k| !self.players[k].exists || self.completed(k));
        if !all_out {
            self.focus_switch_timer = 60;
            self.switch_player(false);
            return;
        }
        self.completion = Completion::Won;
        self.completion_timer = 0;
        let text = self.level_complete_text();
        let perfect = self.total_bonus_coins == 0 || self.bonus_coins == 0;
        self.events.push(Event::LevelWon {
            score: self.level_score,
            perfect,
        });
        self.start_sign(&text, 7.0, LEVEL_COMPLETE, 1.0);
    }

    /// The level-complete sign. Building it adds the time bonus (at least
    /// one second, exactly one without a time limit) to the score, and in
    /// English draws which praise to show. Any other language has no text
    /// in the original, so it shows no sign and adds no time bonus.
    fn level_complete_text(&mut self) -> String {
        let (praise, bonus, time, total, new_record, no_record) = match self.options.language {
            0 => {
                let a = self.session.rng.rand(0, 100);
                let praise = match a {
                    ..20 => "WELL DONE!!! LEVEL COMPLETED!!!",
                    20..40 => "CONGRATULATIONS!!! LEVEL COMPLETED!!!",
                    40..60 => "EXCELLENT!!! LEVEL COMPLETED!!!",
                    60..80 => "GOOD JOB!!! LEVEL COMPLETED!!!",
                    _ => "NICELY DONE!!! LEVEL COMPLETED!!!",
                };
                (
                    praise,
                    "BONUS SCORE",
                    "TIME BONUS",
                    "TOTAL SCORE",
                    "!!! NEW LEVEL RECORD !!!",
                    "NO NEW RECORD.",
                )
            }
            1 => (
                "GUT GEMACHT!!! LEVEL BEENDET!!!",
                "BONUS PUNKTE",
                "ZEIT BONUS",
                "PUNKTE INSGESAMMT",
                "!!! NEUER LEVEL REKORD !!!",
                "LEIDER KEIN NEUER REKORD.",
            ),
            2 => (
                "BIEN REUSSI!!! NIVEAU COMPLETE!!!",
                "SCORE BONUS",
                "SCORE TEMPS",
                "SCORE TOTAL",
                "!!! NOUVEAU RECORD POUR CE NIVEAU !!!",
                "PAS DE NOUVEAU RECORD.",
            ),
            _ => return String::new(),
        };
        if self.map_time < 1 || self.options.no_time_limit {
            self.map_time = 1;
        }
        let mut text = format!(
            "{praise}# #{bonus}: {}    {time}: {}",
            self.level_score,
            self.map_time * 10
        );
        self.level_score += self.map_time * 10;
        text += &format!("#{total}: {}", self.level_score);
        if let Some(best) = self.record {
            text += "#";
            text += if self.level_score > best {
                new_record
            } else {
                no_record
            };
        }
        text
    }

    /// The end of a rainbow spirit or shadow stinker taking coin `j` in
    /// `CanMonsterMove`.
    pub(crate) fn take_coin(&mut self, j: usize) {
        self.coins -= 1;
        if self.coins == 0 {
            self.events.push(Event::Sound(Sound::LastCoin));
            self.toggle_laser_gate(0, true);
        }
        self.kill_item(j);
    }

    /// The end of a shadow stinker taking bonus coin `j`.
    pub(crate) fn take_bonus_coin(&mut self, j: usize) {
        self.bonus_coins -= 1;
        self.kill_item(j);
    }

    /// `KillItem`.
    pub(crate) fn kill_item(&mut self, i: usize) {
        self.items[i].dead = true;
    }

    /// `CleanUpItems`.
    pub(crate) fn clean_up_items(&mut self) {
        self.items.retain(|it| !it.dead);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;
    use crate::{BlitzRng, Input, Options, Ruleset, Session};

    const RIGHT: Input = Input {
        right: true,
        up: false,
        down: false,
        left: false,
        camera: false,
        switch: false,
    };

    /// A floor row with player 1 at x = 0 and these objects after it.
    fn row(objects: &[i32]) -> World {
        row_with(Options::default(), objects)
    }

    fn row_with(options: Options, objects: &[i32]) -> World {
        let mut o = vec![1];
        o.extend_from_slice(objects);
        let l = level(5, o.len(), &vec![100; o.len()], &o);
        World::start(&l, Ruleset::rtwp(), options, Session::default()).unwrap()
    }

    /// Walks right until the player's move ends.
    fn step_right(w: &mut World) -> Vec<Event> {
        let mut events = vec![];
        w.step(RIGHT);
        events.extend_from_slice(w.events());
        while w.players[1].moving {
            w.step(Input::default());
            events.extend_from_slice(w.events());
        }
        events
    }

    #[test]
    fn creation_counts_coins_and_draws_for_clocks() {
        // v5 codes: 5 coin, 6 exit, 7 clock, 8 bonus coin.
        let w = row(&[5, 7, 8, 5, 6]);
        assert_eq!((w.coins, w.total_coins), (2, 2));
        assert_eq!((w.bonus_coins, w.total_bonus_coins), (1, 1));
        let mut rng = BlitzRng::default();
        assert_eq!(w.items[1].data, rng.rand(1, 360));
        // The greeting comes after.
        assert_eq!(w.events(), [Event::Sound(Sound::Talk(rng.rand(0, 4)))]);
        assert_eq!(
            w.items.iter().map(|i| i.kind).collect::<Vec<_>>(),
            [0, 2, 3, 0, 1]
        );
    }

    #[test]
    fn coin_is_taken_past_halfway_and_the_last_opens_the_gate() {
        let l = level(5, 4, &[100, 100, 100, 1000], &[1, 5, 0, 0]);
        let mut w =
            World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        let mut ticks = 0;
        w.step(RIGHT);
        while w.items.len() == 1 {
            ticks += 1;
            w.step(Input::default());
        }
        // A 24-tick move: the item check comes before the move, so the
        // timer reads 11 on the move's 14th tick.
        assert_eq!(ticks, 13);
        assert_eq!(w.coins, 0);
        assert!(w.events().contains(&Event::Sound(Sound::Coin(1))));
        assert!(w.events().contains(&Event::Sound(Sound::LastCoin)));
        assert_eq!(w.level.grid.get(3, 0), 1050);
        assert_eq!(w.final_gate_anim, 59);
    }

    #[test]
    fn clock_adds_time_and_bonus_coin_scores() {
        let mut w = row(&[7, 8]);
        let time = w.map_time;
        let events = step_right(&mut w);
        assert!(events.contains(&Event::Sound(Sound::Clock(1))));
        assert_eq!(w.map_time, time + 30);
        let events = step_right(&mut w);
        assert!(events.contains(&Event::Sound(Sound::Bonus(1))));
        assert_eq!((w.bonus_coins, w.level_score), (0, 250));
        assert!(w.items.is_empty());
    }

    #[test]
    fn exit_wins_with_the_time_bonus() {
        let mut w = row(&[8, 6]);
        step_right(&mut w);
        w.map_time = 0;
        let rng_before = w.session.rng.clone();
        let events = step_right(&mut w);
        assert_eq!(w.completion, Completion::Won);
        // Taken on the move's 14th tick; the move ends 10 ticks later.
        assert_eq!(w.players[1].shrink, 60 - 11);
        assert_eq!(w.both_completed, 1);
        assert_eq!(w.level_score, 250 + 10);
        assert!(events.contains(&Event::LevelWon {
            score: 260,
            perfect: true
        }));
        assert_eq!(w.items.len(), 1, "the exit stays");
        let sign = w
            .signs_shown()
            .iter()
            .find(|s| s.tag == LEVEL_COMPLETE)
            .unwrap();
        assert!(
            sign.text
                .ends_with("# #BONUS SCORE: 250    TIME BONUS: 10#TOTAL SCORE: 260")
        );
        assert_ne!(w.session.rng, rng_before);
    }

    #[test]
    fn record_line_and_other_languages() {
        let mut w = row(&[6]);
        w.record = Some(500);
        w.map_time = 40;
        step_right(&mut w);
        let sign = w
            .signs_shown()
            .iter()
            .find(|s| s.tag == LEVEL_COMPLETE)
            .unwrap();
        assert!(sign.text.ends_with("#TOTAL SCORE: 400#NO NEW RECORD."));

        let german = Options {
            language: 1,
            ..Options::default()
        };
        let mut w = row_with(german, &[6]);
        w.map_time = 40;
        step_right(&mut w);
        let sign = w
            .signs_shown()
            .iter()
            .find(|s| s.tag == LEVEL_COMPLETE)
            .unwrap();
        assert_eq!(
            sign.text,
            "GUT GEMACHT!!! LEVEL BEENDET!!!# #BONUS PUNKTE: 0    ZEIT BONUS: 400#PUNKTE INSGESAMMT: 400"
        );

        let other = Options {
            language: 3,
            ..Options::default()
        };
        let mut w = row_with(other, &[6]);
        step_right(&mut w);
        assert_eq!(w.completion, Completion::Won);
        assert_eq!(w.level_score, 0);
        assert!(!w.signs_shown().iter().any(|s| s.tag == LEVEL_COMPLETE));
    }

    #[test]
    fn first_player_out_hands_the_focus_on() {
        // Players 1 and 2, the exit between them.
        let l = level(5, 3, &[100; 3], &[1, 6, 2]);
        let mut w =
            World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        let events = step_right(&mut w);
        assert_eq!(w.completion, Completion::Playing);
        assert!(events.contains(&Event::Sound(Sound::Exit(1))));
        assert!(w.players[2].focus);
        assert!(w.completed(1));
        assert!(!w.players[1].shadow);
    }

    #[test]
    fn a_coin_can_be_taken_twice_in_a_tick() {
        let mut w = row(&[5, 5]);
        w.take_coin(0);
        assert_eq!(w.coins, 1);
        w.items[0].dead = true;
        w.pick_up_item(0, 1);
        assert_eq!(w.coins, 0);
        w.clean_up_items();
        assert_eq!(w.items.len(), 1);
    }

    #[test]
    fn exit_gate_draws_for_stars() {
        let mut w = row(&[0, 6]);
        let mut rng = w.session.rng.clone();
        for _ in 0..300 {
            w.step(Input::default());
            // The exit's star roll and maybe a star, then lightning and
            // psyche.
            if rng.rand(0, 100) < 5 {
                rng.skip(4);
            }
            rng.skip(2);
        }
        assert_eq!(w.session.rng, rng);
    }
}
