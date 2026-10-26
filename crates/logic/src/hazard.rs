//! Electro tiles, spikes, cannons and fireballs: `ControlElectroTiles` to
//! `CleanUpFireballs` (`RTWP/gameobject.bb`).
//!
//! Electro tiles of type `t` (0–3) switch on for `ElectroBaseTimer` ticks
//! every `(t + 2) * ElectroBaseTimer`; spikes of speed `s` rise for
//! `SpikeBaseTimer` ticks every `(s + 3) * SpikeBaseTimer`. While on, they
//! destroy players, boxes and monsters on them (ghosts, and on electro
//! tiles saucers, are spared).
//!
//! Cannons fire a fireball every `(speed + 1) * CannonBaseTimer` ticks.
//! `UpdateGame` runs `ControlFireballs` twice per tick, so fireballs cross
//! a tile in half of `FireballSpeed`. Mirror boxes (types 2 and 3) turn
//! them, prisms (7) split them in two, and anything else they meet is hit
//! or stops them.
//!
//! The fireball arrays are kept as the original's slots, because one
//! fireball can be created dead past the end of the list (see
//! [`World::create_fireball`]) and `KillFireball` then plays a sound at that
//! slot's stale position. The original keeps the slots across levels; here
//! they start empty each level.

use crate::World;

/// `ElectroBaseTimer`: `BaseSpeed * 2.5`.
fn electro_base(w: &World) -> i32 {
    w.ruleset.base_speed * 5 / 2
}

/// `SpikeBaseTimer`: `BaseSpeed * 3`.
fn spike_base(w: &World) -> i32 {
    w.ruleset.base_speed * 3
}

/// `CannonBaseTimer`: `BaseSpeed * 5`.
fn cannon_base(w: &World) -> i32 {
    w.ruleset.base_speed * 5
}

/// `MaxFireballs`.
pub const MAX_FIREBALLS: usize = 200;

/// The four electro tile types (`ElectroExists`, `ElectroTimer`). The
/// original's arrays have a fifth entry that `ControlElectroTiles` never
/// reaches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Electro {
    pub exists: bool,
    /// Counts down; the tiles are on at `ElectroBaseTimer` and below.
    pub timer: i32,
}

/// One spike (the `Spike*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spike {
    pub x: i32,
    pub y: i32,
    /// `SpikeType`.
    pub speed: i32,
    pub timer: i32,
    /// The entity's height, from -0.5 (down) to 0.5 (up).
    pub height: f32,
}

/// One cannon (the `Cannon*` arrays).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cannon {
    pub x: i32,
    pub y: i32,
    pub speed: i32,
    pub dx: i32,
    pub dy: i32,
    pub timer: i32,
}

/// One fireball slot (the `Fireball*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fireball {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving_x: i32,
    pub moving_y: i32,
    pub moving_timer: i32,
    pub dead: bool,
    /// The entity position (x, up, -row). Particles are emitted from it.
    pub position: [f32; 3],
}

impl World {
    /// `CreateElectro`'s logic part, from the level's electro map.
    pub(crate) fn create_electros(&mut self) {
        let base = electro_base(self);
        for i in 0..self.level.electros.len() {
            let t = self.level.electros[i];
            if let Some(e) = usize::try_from(t)
                .ok()
                .and_then(|t| self.electros.get_mut(t))
            {
                e.exists = true;
                e.timer = (t + 2) * base;
            }
        }
    }

    /// `ElectroMap(x, y)`.
    fn electro_at(&self, x: i32, y: i32) -> i32 {
        self.level.electros[(y * self.level.grid.width() as i32 + x) as usize]
    }

    /// The player half of the electro and spike checks: a player is hit on
    /// the tile it is moving to once past halfway, otherwise on its own.
    fn hit_players_on(&mut self, x: i32, y: i32) {
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if !p.exists {
                continue;
            }
            let at = if p.moving && p.moving_timer < p.speed / 2 {
                (p.x2, p.y2)
            } else {
                (p.x, p.y)
            };
            if at == (x, y) {
                self.kill_player(k);
            }
        }
    }

    /// The box and monster halves: anything on, or moving to or from,
    /// `(x, y)`, except monsters of the `spared` types.
    fn hit_objects_on(&mut self, x: i32, y: i32, spared: &[i32]) {
        let hit = |ax: i32, bx: i32, ay: i32, by: i32| (x == ax || x == bx) && (y == ay || y == by);
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            if hit(b.x, b.x2, b.y, b.y2) {
                self.kill_box(j);
            }
        }
        for j in 0..self.monsters.len() {
            let m = &self.monsters[j];
            if hit(m.x, m.x2, m.y, m.y2) && !spared.contains(&m.kind) {
                self.kill_monster(j);
            }
        }
    }

    /// `ControlElectroTiles`.
    pub(crate) fn control_electro_tiles(&mut self) {
        let base = electro_base(self);
        let (width, height) = (
            self.level.grid.width() as i32,
            self.level.grid.height() as i32,
        );
        for i in 0..4 {
            if !self.electros[i].exists {
                continue;
            }
            let e = &mut self.electros[i];
            e.timer -= 1;
            if e.timer == -1 {
                e.timer = (i as i32 + 2) * base;
            }
            let timer = e.timer;
            if timer > base {
                continue;
            }
            for x in 0..width {
                for y in 0..height {
                    if self.electro_at(x, y) != i as i32 {
                        continue;
                    }
                    self.hit_players_on(x, y);
                    self.hit_objects_on(x, y, &[5, 6, 9]);
                    if timer == base {
                        self.play_single_sound(x, y, 0);
                    }
                }
            }
        }
    }

    /// `CreateSpike`.
    pub(crate) fn create_spike(&mut self, x: i32, y: i32, speed: i32) {
        let timer = spike_base(self) * (speed + 3);
        self.spikes.push(Spike {
            x,
            y,
            speed,
            timer,
            height: -0.5,
        });
    }

    /// `ControlSpikes`.
    pub(crate) fn control_spikes(&mut self) {
        let base = spike_base(self);
        for i in 0..self.spikes.len() {
            let s = &mut self.spikes[i];
            s.timer -= 1;
            if s.timer >= base && s.timer < base + 20 {
                s.height += 0.05;
            } else if s.timer < 20 {
                s.height -= 0.05;
            }
            if s.timer == 0 {
                s.timer = base * (s.speed + 3);
            }
            let (x, y, timer) = (s.x, s.y, s.timer);
            if timer > 5 && timer < base + 5 {
                self.hit_players_on(x, y);
                self.hit_objects_on(x, y, &[9]);
            }
            if timer == base + 20 {
                self.play_single_sound(x, y, 6);
            }
            if timer == 10 {
                self.play_single_sound(x, y, 5);
            }
        }
    }

    /// `CreateCannon`.
    pub(crate) fn create_cannon(&mut self, x: i32, y: i32, speed: i32, dx: i32, dy: i32) {
        let timer = (speed + 1) * cannon_base(self);
        self.cannons.push(Cannon {
            x,
            y,
            speed,
            dx,
            dy,
            timer,
        });
    }

    /// `ControlCannons`: a cannon waits while 25 fireballs are out.
    pub(crate) fn control_cannons(&mut self) {
        for i in 0..self.cannons.len() {
            self.cannons[i].timer -= 1;
            let c = self.cannons[i].clone();
            if c.timer <= 0 && self.fireball_count < 25 {
                self.play_single_sound(c.x, c.y, 1);
                self.create_fireball(c.x + c.dx, c.y + c.dy, c.dx, c.dy);
                self.cannons[i].timer = (c.speed + 1) * cannon_base(self);
            }
        }
    }

    /// `NofFireballs`.
    pub(crate) fn fireball_count(&self) -> usize {
        self.fireball_count
    }

    /// The fireballs in play.
    pub fn fireballs(&self) -> &[Fireball] {
        &self.fireball_slots[..self.fireball_count]
    }

    /// `CreateFireball`. A mirror box on the starting tile turns the shot
    /// at once. If that mirror is moving, the original means to kill the
    /// shot but kills the next slot instead: the shot lives on, waiting a
    /// tile's time before it moves.
    pub(crate) fn create_fireball(&mut self, x: i32, y: i32, dx: i32, dy: i32) {
        let n = self.fireball_count;
        if n >= MAX_FIREBALLS {
            return;
        }
        let speed = self.ruleset.base_speed;
        let f = &mut self.fireball_slots[n];
        f.moving_x = dx;
        f.moving_y = dy;
        f.position = [x as f32 + 0.5, 0.5, -(y as f32) - 0.5];
        f.x = x;
        f.y = y;
        f.x2 = x;
        f.y2 = y;
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            let on = (x == b.x || x == b.x2) && (y == b.y || y == b.y2);
            if !(on && (b.kind == 2 || b.kind == 3)) {
                continue;
            }
            let f = &mut self.fireball_slots[n];
            if b.moving {
                f.moving_timer = speed;
                f.dead = false;
                self.fireball_count += 1;
                self.kill_fireball(n + 1);
                return;
            }
            let (mx, my) = (f.moving_x, f.moving_y);
            (f.moving_x, f.moving_y) = if b.kind == 2 { (-my, -mx) } else { (my, mx) };
        }
        let f = &mut self.fireball_slots[n];
        f.x2 = x + f.moving_x;
        f.y2 = y + f.moving_y;
        f.moving_timer = speed;
        f.dead = false;
        self.fireball_count += 1;
    }

    /// `ControlFireballs`. Fireballs a prism makes during the pass are
    /// moved in the same pass.
    pub(crate) fn control_fireballs(&mut self) {
        let speed = self.ruleset.base_speed;
        let (width, height) = (
            self.level.grid.width() as i32,
            self.level.grid.height() as i32,
        );
        let mut i = 0;
        while i < self.fireball_count {
            if self.fireball_slots[i].dead {
                i += 1;
                continue;
            }
            let f = &mut self.fireball_slots[i];
            f.position[0] += f.moving_x as f32 / speed as f32;
            f.position[2] += -(f.moving_y as f32) / speed as f32;
            if f.moving_timer % 6 == 0 {
                // A trail particle: two speeds and a spin.
                self.session.rng.skip(3);
            }
            let f = &mut self.fireball_slots[i];
            f.moving_timer -= 1;
            if f.moving_timer == 0 {
                let (x2, y2) = (f.x2, f.y2);
                if x2 <= 0 || y2 <= 0 || x2 >= width - 1 || y2 >= height - 1 {
                    self.kill_fireball(i);
                }
                for j in 0..self.blocks.len() {
                    let b = &self.blocks[j];
                    let on = (x2 == b.x || x2 == b.x2) && (y2 == b.y || y2 == b.y2);
                    if !(on && matches!(b.kind, 2 | 3 | 7)) {
                        continue;
                    }
                    let (bx, by, kind) = (b.x, b.y, b.kind);
                    self.play_single_sound(bx, by, 7);
                    let f = &mut self.fireball_slots[i];
                    let (mx, my) = (f.moving_x, f.moving_y);
                    match kind {
                        2 => (f.moving_x, f.moving_y) = (-my, -mx),
                        3 => (f.moving_x, f.moving_y) = (my, mx),
                        _ => {
                            self.create_fireball(bx, by, my, mx);
                            self.create_fireball(bx, by, -my, -mx);
                            self.kill_box(j);
                        }
                    }
                }
                let f = &mut self.fireball_slots[i];
                f.moving_timer = speed;
                f.x = f.x2;
                f.y = f.y2;
                f.x2 = f.x + f.moving_x;
                f.y2 = f.y + f.moving_y;
            }
            if !self.can_fireball_move(i) {
                self.kill_fireball(i);
            }
            i += 1;
        }
    }

    /// `CanFireBallMove`: false if fireball `i` hits something on its
    /// tile (the one it is leaving until halfway). It burns wooden and boom
    /// boxes, monsters (not ghosts, nor a mother ship unless it is
    /// launching) and players; mirrors and prisms at rest let it on.
    fn can_fireball_move(&mut self, i: usize) -> bool {
        let speed = self.ruleset.base_speed;
        let f = &self.fireball_slots[i];
        let (x, y) = if f.moving_timer >= speed / 2 {
            (f.x, f.y)
        } else {
            (f.x2, f.y2)
        };
        let tile = self.level.grid.get(x, y);
        if tile / 100 == 2 || tile / 100 == 8 || (tile / 100 == 10 && tile % 100 < 50) {
            return false;
        }
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            let at = if !b.moving || b.moving_timer >= b.speed / 2 {
                (b.x, b.y)
            } else {
                (b.x2, b.y2)
            };
            if at == (x, y) {
                let (kind, moving) = (b.kind, b.moving);
                if kind == 0 || kind == 6 {
                    self.kill_box(j);
                }
                return matches!(kind, 2 | 3 | 7) && !moving;
            }
        }
        for j in 0..self.monsters.len() {
            let m = &self.monsters[j];
            let at = if !m.moving || m.moving_timer >= m.speed / 2 {
                (m.x, m.y)
            } else {
                (m.x2, m.y2)
            };
            let launching = m.data[3] > 300 && m.data[3] < 400;
            if at == (x, y) && m.kind != 9 && (m.kind != 12 || launching) {
                self.kill_monster(j);
                return false;
            }
        }
        if self.items.iter().any(|it| (it.x, it.y) == (x, y)) {
            return false;
        }
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if !p.exists {
                continue;
            }
            let at = if !p.moving || p.moving_timer >= p.speed / 2 {
                (p.x, p.y)
            } else {
                (p.x2, p.y2)
            };
            if at == (x, y) {
                self.kill_player(k);
                return false;
            }
        }
        true
    }

    /// `KillFireball`: slot `i` (which may be past the list) dies with a
    /// sound at its tile.
    pub(crate) fn kill_fireball(&mut self, i: usize) {
        let f = &mut self.fireball_slots[i];
        f.dead = true;
        let (x, y) = (f.x, f.y);
        self.play_single_sound(x, y, 2);
    }

    /// The fireball part of a gate closing on `(x, y)`.
    pub(crate) fn kill_fireballs_in_gate(&mut self, x: i32, y: i32) {
        for k in 0..self.fireball_count {
            let f = &self.fireball_slots[k];
            if (f.x == x || f.x2 == x) && (f.y == y || f.y2 == y) {
                self.kill_fireball(k);
            }
        }
    }

    /// `CleanUpFireballs`: each removal shifts the later slots down,
    /// including the first slot past the list.
    pub(crate) fn clean_up_fireballs(&mut self) {
        let mut i = 0;
        while i < self.fireball_count {
            if self.fireball_slots[i].dead {
                let n = self.fireball_count;
                if i < n - 1 {
                    self.fireball_slots[i..=n].rotate_left(1);
                    // The original copies rather than rotates: slot n keeps
                    // its contents.
                    self.fireball_slots[n] = self.fireball_slots[n - 1].clone();
                    self.fireball_count -= 1;
                    continue;
                }
                self.fireball_count -= 1;
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::setup::tests::level;
    use crate::{Input, Options, Ruleset, Session, World};

    fn world(width: usize, tiles: &[i32], objects: &[i32]) -> World {
        let l = level(5, width, tiles, objects);
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    #[test]
    fn electro_tiles_switch_on_and_kill() {
        // Player 2 tiles from an electro tile of type 0.
        let mut w = world(3, &[100, 100, 700], &[1, 0, 0]);
        assert!(w.electros[0].exists && !w.electros[1].exists);
        assert_eq!(w.electros[0].timer, 120);
        w.players[1].x = 2;
        w.players[1].x2 = 2;
        for _ in 0..59 {
            w.step(Input::default());
        }
        assert!(!w.players[1].dying);
        w.step(Input::default());
        assert!(w.players[1].dying);
    }

    #[test]
    fn electro_cycle_length() {
        let mut w = world(3, &[100, 100, 701], &[1, 0, 0]);
        let mut on = vec![];
        for t in 1..=400 {
            w.step(Input::default());
            if w.electros[1].timer == 60 {
                on.push(t);
            }
        }
        // 181 ticks per cycle (180 down to 0); on from 60 down.
        assert_eq!(on, [120, 301]);
    }

    #[test]
    fn spikes_rise_and_kill() {
        let mut w = world(2, &[100, 1400], &[1, 0]);
        assert_eq!(w.spikes[0].timer, 72 * 3);
        w.players[1].x = 1;
        w.players[1].x2 = 1;
        let mut hit_at = None;
        for t in 1..=144 {
            w.step(Input::default());
            if w.players[1].dying && hit_at.is_none() {
                hit_at = Some(t);
            }
        }
        // Deadly from timer 76 down: tick 216 - 76.
        assert_eq!(hit_at, Some(140));
        // Up after rising over timers 91 to 72.
        assert!((w.spikes[0].height - 0.5).abs() < 1e-4);
    }

    #[test]
    fn cannon_fires_and_fireball_hits_the_player() {
        // Wall row so the shot stays inside; a cannon (804: speed 1, pointing
        // right) on the left, the player on the right.
        let tiles = [
            200, 200, 200, 200, 200, 200, //
            200, 804, 100, 100, 100, 200, //
            200, 200, 200, 200, 200, 200,
        ];
        let mut objects = [0; 18];
        objects[10] = 1;
        let mut w = world(6, &tiles, &objects);
        assert_eq!(w.cannons.len(), 1);
        let c = w.cannons[0].clone();
        let mut fired = None;
        for t in 1..=400 {
            w.step(Input::default());
            if fired.is_none() && w.fireball_count() == 1 {
                fired = Some(t);
            }
            if w.players[1].dying {
                break;
            }
        }
        assert_eq!(fired, Some((c.speed + 1) * 120));
        assert!(w.players[1].dying);
        assert_eq!((c.dx, c.dy), (1, 0));
    }

    #[test]
    fn mirror_turns_a_fireball() {
        let mut tiles = vec![100; 25];
        for (i, t) in tiles.iter_mut().enumerate() {
            if !(5..20).contains(&i) || i % 5 == 0 || i % 5 == 4 {
                *t = 200;
            }
        }
        let mut objects = vec![0; 25];
        objects[18] = 1;
        // A mirror (box type 2, object code 11) at (3, 1).
        objects[8] = 11;
        let mut w = world(5, &tiles, &objects);
        assert_eq!(w.blocks[0].kind, 2);
        w.create_fireball(1, 1, 1, 0);
        for _ in 0..12 {
            w.step(Input::default());
        }
        // A tile per 12 ticks at double speed.
        let f = &w.fireballs()[0];
        assert_eq!((f.x, f.y, f.moving_x, f.moving_y), (2, 1, 1, 0));
        for _ in 0..12 {
            w.step(Input::default());
        }
        let f = &w.fireballs()[0];
        assert_eq!((f.x, f.y, f.moving_x, f.moving_y), (3, 1, 0, -1));
        // Reaching the mirror turned it: type 2 maps (1, 0) to (0, -1).
        // Then it burns out on the top wall.
        for _ in 0..12 {
            w.step(Input::default());
        }
        assert_eq!(w.fireball_count(), 0);
    }

    #[test]
    fn a_shot_onto_a_moving_mirror_lives_and_kills_the_next_slot() {
        let mut w = world(4, &[100, 100, 100, 100], &[1, 0, 11, 0]);
        w.blocks[0].moving = true;
        w.create_fireball(2, 0, 1, 0);
        assert_eq!(w.fireball_count(), 1);
        let f = &w.fireballs()[0];
        assert!(!f.dead);
        assert_eq!((f.x2, f.y2), (2, 0));
        assert!(w.fireball_slots[1].dead);
    }

    #[test]
    fn clean_up_shifts_slots() {
        let mut w = world(2, &[100, 100], &[1, 0]);
        for x in 0..3 {
            w.create_fireball(x, 0, 0, 1);
        }
        w.fireball_slots[0].dead = true;
        w.fireball_slots[1].dead = true;
        w.clean_up_fireballs();
        assert_eq!(w.fireball_count(), 1);
        assert_eq!(w.fireballs()[0].x, 2);
    }
}
