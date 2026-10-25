//! Monsters and warpers: `CreateMonster` to `CleanUpMonsters`, the ten
//! `Control*` behaviours, `CreateWarper` and `ControlWarpers`, plus the
//! monster parts of `UpdateGame` (`RTWP/gameobject.bb`).
//!
//! Monster types: 0 beach ball (coily), 1 and 2 z-bots (left- and
//! right-handed), 3 and 4 bombers (kaboom), 5 and 6 saucers, 7 and 8
//! chompers (after players 1 and 2), 9 ghost, 10 rainbow spirit, 11 broken
//! z-bot, 12 mother ship, 13–16 shadow stinkers and 17–20 fire stinkers
//! (mirroring players 1–4).
//!
//! [`World::kill_monster`] parks a monster at tile (0, 0). Most types are
//! marked dead at once and removed by `CleanUpMonsters` at the end of the
//! tick; beach balls and saucers spend `BaseSpeed * 4` ticks dying at
//! (0, 0), where they still block other monsters. Shadow stinkers and the
//! mother ship have no dying check, so one killed earlier in the tick still
//! runs its control at (0, 0) that tick, hitting a player standing there.

use crate::World;
use crate::world::Sound;

/// `Const MaxMonsters`; warpers stop at 10 below it.
const MAX_MONSTERS: usize = 600;
/// Fireballs a saucer or fire stinker waits for (`NofFireBalls < 25`).
const MAX_FIREBALLS: usize = 25;

/// One monster (the `Monster*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Monster {
    /// `MonsterType`.
    pub kind: i32,
    /// Current tile; (0, 0) once killed.
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_x: i32,
    pub moving_y: i32,
    pub moving_timer: i32,
    pub speed: i32,
    pub dying: bool,
    pub dying_timer: i32,
    /// Removed at the end of the tick.
    pub dead: bool,
    /// `MonsterData`. By type:
    /// - beach ball: 0 bounce timer, 1 spin speed in eighths of a degree;
    /// - bombers: 0 left turns in a row, 1 heading;
    /// - z-bots, chompers, ghosts, spirits, stinkers: 1 heading;
    /// - saucers: 1 spin, 2 hum countdown, 3 firing cooldown, 9 free moves
    ///   left after leaving a mother ship;
    /// - broken z-bot: 1 wobble;
    /// - mother ship: 1 spin in half degrees, 2 hum countdown, 3 launch
    ///   timer (0 idle, then 1–801);
    /// - fire stinkers: 3 firing cooldown.
    pub data: [i32; 10],
    /// A beach ball's starting yaw, `Rand(1, 300)`.
    pub yaw: i32,
    /// The entity position (x, up, -row), as far as sounds need it: moves,
    /// beach ball bounces and the mother ship rising. The renderer adds
    /// turns and the dying animations.
    pub position: [f32; 3],
}

impl Monster {
    /// The tile it counts as on: the one it is leaving until halfway.
    pub(crate) fn nearest_tile(&self) -> (i32, i32) {
        if !self.moving || self.moving_timer >= self.speed / 2 {
            (self.x, self.y)
        } else {
            (self.x2, self.y2)
        }
    }

    fn is_stinker(&self) -> bool {
        (13..=20).contains(&self.kind)
    }

    /// Doesn't press buttons: saucers and ghosts float.
    fn presses_buttons(&self) -> bool {
        !matches!(self.kind, 5 | 6 | 9)
    }
}

/// A warp gate: sends out four monsters of its type once none are left.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Warper {
    pub x: i32,
    pub y: i32,
    pub kind: i32,
    /// 0 while waiting, then counts to 200; the renderer grows the gate
    /// over ticks 1–39 and shrinks it back by 200.
    pub timer: i32,
}

/// A change of direction, `(dx, dy)` to the new `(dx, dy)`.
type Turn = fn(i32, i32) -> (i32, i32);

/// `RobotRotationGoal` for z-bots, chompers, ghosts, spirits and stinkers.
fn heading(dx: i32, dy: i32) -> i32 {
    match (dx, dy) {
        (0, 1) => 90,
        (0, _) => 270,
        (1, _) => 0,
        _ => 180,
    }
}

/// The bombers' `RobotRotationGoal`, which is also every monster's
/// starting heading.
fn bomber_heading(dx: i32, dy: i32) -> i32 {
    match (dx, dy) {
        (0, 1) => 180,
        (0, _) => 0,
        (1, _) => 270,
        _ => 90,
    }
}

/// Turns `current` by `step` degrees towards `goal`.
fn turn(current: &mut i32, goal: i32, step: i32) {
    if *current == goal {
        return;
    }
    if (*current < goal && goal - *current < 180) || (*current > goal && *current - goal > 180) {
        *current += step;
    } else {
        *current -= step;
    }
    if *current == 360 {
        *current = 0;
    }
    if *current == -step {
        *current = 360 - step;
    }
}

impl World {
    /// `CreateMonster`. Facing `dir` 0 up, 1 right, 2 down, 3 left.
    pub(crate) fn create_monster(&mut self, x: i32, y: i32, kind: i32, dir: i32) {
        let base = self.ruleset.base_speed;
        let height = match kind {
            0 | 7 | 8 => 0.4,
            1 | 2 | 5 | 6 | 11 => 0.3,
            10 => 0.5,
            12 => -1.8,
            _ => 0.0,
        };
        let mut m = Monster {
            kind,
            x,
            y,
            x2: x,
            y2: y,
            position: [x as f32 + 0.5, height, -(y as f32 + 0.5)],
            ..Monster::default()
        };
        if kind == 0 {
            m.yaw = self.session.rng.rand(1, 300);
        }
        // Stinkers keep the stale direction of the array slot; it only
        // turns them.
        if !m.is_stinker() && kind != 0 {
            (m.moving_x, m.moving_y) = match dir {
                0 => (0, -1),
                1 => (1, 0),
                2 => (0, 1),
                _ => (-1, 0),
            };
        }
        // The mother ship keeps the slot's stale speed; it never moves.
        m.speed = match kind {
            0 | 5..=9 => base * 2,
            1 | 2 | 10 => base * 2 / 3,
            3 | 4 => base * 3 / 2,
            11 | 13..=20 => base,
            _ => 0,
        };
        match kind {
            0 => {
                m.data[0] = -m.speed / 2;
                m.data[1] = self.session.rng.rand(-16, 16);
            }
            1..=11 => m.data[1] = bomber_heading(m.moving_x, m.moving_y),
            _ => {}
        }
        self.monsters.push(m);
    }

    /// `CreateWarper`. Types 12 and 13 skip the mother ship, and the two
    /// bombers swap.
    pub(crate) fn create_warper(&mut self, x: i32, y: i32, kind: i32) {
        let kind = match kind {
            12 | 13 => kind + 1,
            3 | 4 => 7 - kind,
            _ => kind,
        };
        self.warpers.push(Warper {
            x,
            y,
            kind,
            timer: 0,
        });
    }

    /// `ControlWarpers`.
    pub(crate) fn control_warpers(&mut self) {
        for i in 0..self.warpers.len() {
            let w = self.warpers[i].clone();
            if w.timer != 0 {
                let t = &mut self.warpers[i].timer;
                *t += 1;
                if *t == 200 {
                    *t = 0;
                }
                continue;
            }
            let none_left = !self.monsters.iter().any(|m| m.kind == w.kind);
            if none_left && self.monsters.len() < MAX_MONSTERS - 10 {
                self.warpers[i].timer = 1;
                self.play_single_sound(w.x, w.y, 16);
                for dir in 0..4 {
                    self.create_monster(w.x, w.y, w.kind, dir);
                }
            }
        }
    }

    /// `ControlMonsters`. Monsters made this tick (by a mother ship) are
    /// controlled this tick too.
    pub(crate) fn control_monsters(&mut self) {
        let mut i = 0;
        while i < self.monsters.len() {
            match self.monsters[i].kind {
                0 => self.control_beach_ball(i),
                1 | 2 => self.control_robot(i),
                3 | 4 => self.control_bomber(i),
                5 | 6 => self.control_saucer(i),
                7 | 8 => self.control_chomper(i),
                9 => self.control_ghost(i),
                10 => self.control_spirit(i),
                11 => self.control_broken_zbot(i),
                12 => self.control_mother_ship(i),
                13..=20 => self.control_shadow_stinker(i),
                _ => {}
            }
            i += 1;
        }
    }

    /// The monster steps of `UpdateGame`: monsters move a step along.
    pub(crate) fn move_monsters(&mut self) {
        for m in &mut self.monsters {
            if m.moving {
                m.position[0] += m.moving_x as f32 / m.speed as f32;
                m.position[2] += -m.moving_y as f32 / m.speed as f32;
                m.moving_timer -= 1;
            }
        }
    }

    /// The monster moves that end this tick.
    pub(crate) fn end_monster_moves(&mut self) {
        for i in 0..self.monsters.len() {
            let m = &self.monsters[i];
            if m.moving && m.moving_timer == 0 {
                self.end_move_monster(i);
            }
        }
    }

    /// `CanMonsterMove`: whether monster `i` can step by `(dx, dy)`. A
    /// shadow stinker pushes boxes on the way, and stinkers and the rainbow
    /// spirit collect the coins they would step onto.
    pub(crate) fn can_monster_move(&mut self, i: usize, dx: i32, dy: i32) -> bool {
        let m = &self.monsters[i];
        let (kind, stinker) = (m.kind, m.is_stinker());
        let (x, y) = (m.x + dx, m.y + dy);
        if !self.level.grid.in_level(x, y) {
            return false;
        }
        let tile = self.level.grid.get(x, y);
        let class = tile / 100;
        let floats = matches!(kind, 5 | 6 | 10);
        let walkable = matches!(class, 1 | 7 | 11 | 13)
            || (class == 10 && tile % 100 >= 50)
            // Z-bots walk into water and blow up.
            || (matches!(class, 3 | 9) && matches!(kind, 1 | 2 | 11))
            || (matches!(class, 3..=6 | 9) && floats)
            || (matches!(class, 4..=6) && kind == 9)
            // Ghosts pass closed gates.
            || (class == 10 && kind == 9)
            || (class == 17 && floats);
        if !walkable {
            return false;
        }
        let hit = |ax: i32, bx: i32, ay: i32, by: i32| (x == ax || x == bx) && (y == ay || y == by);

        if kind == 9 {
            // Only ghost boxes stop a ghost.
            return !self
                .blocks
                .iter()
                .any(|b| hit(b.x, b.x2, b.y, b.y2) && b.kind == 5);
        }
        if !stinker {
            if self.blocks.iter().any(|b| hit(b.x, b.x2, b.y, b.y2)) {
                return false;
            }
        } else if !self.stinker_pushes(x, y, dx, dy) {
            return false;
        }

        for j in 0..self.monsters.len() {
            let m = &self.monsters[j];
            if hit(m.x, m.x2, m.y, m.y2) && i != j {
                return false;
            }
        }

        for j in 0..self.items.len() {
            let item = &self.items[j];
            if (item.x, item.y) != (x, y) {
                continue;
            }
            if item.kind == 0 && (kind == 10 || stinker) {
                // One particle with a random spin.
                self.session.rng.skip(1);
                self.play_at_monster(Sound::Single(15), i);
                self.take_coin(j);
            } else if item.kind == 3 && stinker {
                self.play_at_monster(Sound::Single(15), i);
                self.level_score += 250;
                self.take_bonus_coin(j);
            } else {
                return false;
            }
        }
        true
    }

    /// The box half of `CanMonsterMove` for a shadow stinker stepping onto
    /// `(x, y)`: pushes the boxes there like a player, false if one blocks.
    /// Unlike a player's push, a box moving away is pushed again, and the
    /// scan goes on past a pushed box.
    fn stinker_pushes(&mut self, x: i32, y: i32, dx: i32, dy: i32) -> bool {
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            if (b.x, b.y) != (x, y) && (b.x2, b.y2) != (x, y) {
                continue;
            }
            if b.moving {
                if (b.moving_x, b.moving_y) == (-dx, -dy) {
                    self.kill_box(j);
                } else if (b.moving_x, b.moving_y) != (dx, dy) {
                    return false;
                }
            }
            let (bx, by) = (self.blocks[j].x, self.blocks[j].y);
            if self.conveying(self.level.grid.get(bx, by)) || !self.can_box_move(j, dx, dy) {
                return false;
            }
            self.move_box(j, dx, dy, 0);
            let kind = self.blocks[j].kind;
            if (9..=12).contains(&kind) {
                // A link sphere: as in a player's push, the original tests
                // the pushed sphere's x, not each other's.
                for k in 0..self.blocks.len() {
                    let other = &self.blocks[k];
                    if j == k || other.kind != kind || self.blocks[j].x == 101 || other.moving {
                        continue;
                    }
                    let (ox, oy) = (other.x, other.y);
                    if !self.conveying(self.level.grid.get(ox, oy)) && self.can_box_move(k, dx, dy)
                    {
                        self.move_box(k, dx, dy, 0);
                    }
                }
            }
        }
        true
    }

    /// `MoveMonster`: starts a one-tile move.
    pub(crate) fn move_monster(&mut self, i: usize, dx: i32, dy: i32) {
        let m = &mut self.monsters[i];
        m.moving_timer = m.speed;
        m.x2 = m.x + dx;
        m.y2 = m.y + dy;
        m.moving_x = dx;
        m.moving_y = dy;
        m.moving = true;
    }

    /// `EndMoveMonster`: buttons, and z-bots drowning.
    fn end_move_monster(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        let (old_x, old_y) = (m.x, m.y);
        m.x = m.x2;
        m.y = m.y2;
        let (x, y, kind, presses) = (m.x, m.y, m.kind, m.presses_buttons());
        if presses && self.level.grid.get(old_x, old_y) / 100 == 11 {
            self.step_off_button(old_x, old_y);
        }
        self.monsters[i].moving = false;
        let class = self.level.grid.get(x, y) / 100;
        if presses && class == 11 {
            self.step_on_button(x, y);
        }
        if matches!(class, 3 | 9) && matches!(kind, 1 | 2 | 11) {
            self.kill_monster(i);
        }
    }

    /// `KillMonster`: scores 250 and parks the monster at (0, 0). Bombers
    /// blow up the four tiles around them.
    pub(crate) fn kill_monster(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        m.dying = true;
        let (x, y, kind, presses) = (m.x, m.y, m.kind, m.presses_buttons());
        if presses && self.level.grid.get(x, y) / 100 == 11 {
            self.step_off_button(x, y);
        }
        match kind {
            0 => self.play_at_monster(Sound::CoilyDeath, i),
            5 | 6 => {}
            _ => self.play_at_monster(Sound::BombExplode, i),
        }
        self.level_score += 250;
        self.monsters[i].dying_timer = self.ruleset.base_speed * 4;

        // Explosion particles, each drawing a velocity and a spin: one
        // kind for saucers and z-bots, two for the rest.
        let detail = usize::try_from(self.options.detail_level).unwrap_or(0);
        match kind {
            5 | 6 => {
                self.play_at_monster(Sound::SaucerDie(16000), i);
                self.session.rng.skip(4 * 3 * detail);
            }
            12 => {
                self.monsters[i].dead = true;
                self.play_at_monster(Sound::SaucerDie(10000), i);
                self.session.rng.skip(8 * 28 * detail);
            }
            _ if kind > 2 && kind != 11 => {
                self.monsters[i].dead = true;
                self.session.rng.skip(8 * 7 * detail);
            }
            _ if kind > 0 => {
                self.monsters[i].dead = true;
                self.session.rng.skip(4 * 14 * detail);
            }
            _ => {}
        }

        if kind == 3 || kind == 4 {
            let (cx, cy) = self.monsters[i].nearest_tile();
            for d in [-1, 1] {
                for (x, y) in [(cx + d, cy), (cx, cy + d)] {
                    self.blast(x, y);
                }
            }
        }

        let m = &mut self.monsters[i];
        (m.x, m.y, m.x2, m.y2) = (0, 0, 0, 0);
        m.moving = false;
    }

    /// `CleanUpMonsters`.
    pub(crate) fn clean_up_monsters(&mut self) {
        self.monsters.retain(|m| !m.dead);
    }

    fn play_at_monster(&mut self, sound: Sound, i: usize) {
        let at = self.monsters[i].position;
        self.play_at(sound, at);
    }

    /// Kills every player on the tile the monster counts as on.
    fn hit_players(&mut self, i: usize) {
        let at = self.monsters[i].nearest_tile();
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if p.exists && !p.dying && !p.dead && p.nearest_tile() == at {
                self.kill_player(k);
            }
        }
    }

    /// Moves on in the current direction if it can.
    fn go_straight(&mut self, i: usize) -> bool {
        let (dx, dy) = (self.monsters[i].moving_x, self.monsters[i].moving_y);
        if self.can_monster_move(i, dx, dy) {
            self.move_monster(i, dx, dy);
            true
        } else {
            false
        }
    }

    /// Sets the direction to `f(dx, dy)`, then moves that way if it can.
    fn turn_and_go(&mut self, i: usize, f: Turn) -> bool {
        let m = &mut self.monsters[i];
        (m.moving_x, m.moving_y) = f(m.moving_x, m.moving_y);
        self.go_straight(i)
    }

    /// Z-bots and saucers when blocked: left-handed ones try left, right,
    /// then back; right-handed ones right, left, then back.
    fn follow_wall(&mut self, i: usize, left_handed: bool) {
        let (first, last): (Turn, Turn) = if left_handed {
            (|x, y| (y, -x), |x, y| (-y, x))
        } else {
            (|x, y| (-y, x), |x, y| (y, -x))
        };
        let _ = self.turn_and_go(i, first)
            || self.turn_and_go(i, |x, y| (-x, -y))
            || self.turn_and_go(i, last);
    }

    /// `ControlBeachBall`: bounces about at random.
    fn control_beach_ball(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        if m.dying {
            // Deflating: the renderer shrinks and spins it.
            m.dying_timer -= 1;
            if m.dying_timer == 0 {
                m.dead = true;
            }
            return;
        }
        if m.data[0] > -m.speed / 2 {
            m.position[1] += m.data[0] as f32 / 500.0;
            m.data[0] -= 1;
        }
        self.hit_players(i);
        if self.monsters[i].moving {
            return;
        }
        let rng = &mut self.session.rng;
        let mut dir = rng.rand(-1, 1);
        let (dx, dy) = if dir == 0 {
            dir = if rng.rand(1, 100) < 50 { -1 } else { 1 };
            (0, dir)
        } else {
            (dir, 0)
        };
        if self.can_monster_move(i, dx, dy) {
            let pitch = 20000 + self.session.rng.rand(0, 4000);
            self.play_at_monster(Sound::BallBounce(pitch), i);
            self.move_monster(i, dx, dy);
            let m = &mut self.monsters[i];
            m.data[0] = m.speed / 2 - 1;
        }
    }

    /// `ControlRobot`: z-bots follow walls.
    fn control_robot(&mut self, i: usize) {
        if self.monsters[i].dying {
            self.monsters[i].dead = true;
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        turn(&mut m.data[1], heading(m.moving_x, m.moving_y), 6);
        if m.moving {
            return;
        }
        if (m.moving_x, m.moving_y) == (0, 0) {
            m.moving_x = if m.kind == 1 { -1 } else { 1 };
        }
        let left_handed = m.kind == 1;
        if !self.go_straight(i) {
            self.follow_wall(i, left_handed);
        }
    }

    /// `ControlSaucer`: follows walls over anything, firing at players in
    /// line.
    fn control_saucer(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        if m.dying {
            // Spinning down: the renderer moves it. It explodes where it
            // fell, which the volume here ignores.
            m.dying_timer -= 1;
            if m.dying_timer == 0 {
                let detail = usize::try_from(self.options.detail_level).unwrap_or(0);
                self.session.rng.skip(8 * 7 * detail);
                self.play_at_monster(Sound::BombExplode, i);
                self.monsters[i].dead = true;
            }
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        m.data[1] += 1;
        if m.data[1] >= 360 {
            m.data[1] = 0;
        }
        if m.data[2] <= 0 {
            let (x, y) = (m.x, m.y);
            m.data[2] = 100;
            self.play_single_sound(x, y, 4);
        } else {
            m.data[2] -= 1;
        }
        let m = &mut self.monsters[i];
        m.data[3] = (m.data[3] - 1).max(0);
        if m.moving {
            return;
        }
        if self.fireball_count() < MAX_FIREBALLS && self.monsters[i].data[9] == 0 {
            self.fire_at_players(i, false);
        }

        let m = &mut self.monsters[i];
        if (m.moving_x, m.moving_y) == (0, 0) {
            // As in ControlRobot, so always right.
            m.moving_x = if m.kind == 1 { -1 } else { 1 };
        }
        if m.data[9] > 0 {
            // Leaving the mother ship: three moves without looking.
            let (dx, dy) = (m.moving_x, m.moving_y);
            m.data[9] -= 1;
            self.move_monster(i, dx, dy);
            let m = &self.monsters[i];
            if !self.level.grid.in_level(m.x2, m.y2) {
                self.kill_monster(i);
            }
        } else if !self.go_straight(i) {
            let left_handed = self.monsters[i].kind == 5;
            self.follow_wall(i, left_handed);
        }
    }

    /// Saucers and fire stinkers fire at a player in the same row or
    /// column, unless a wall (2xx), a pillar (8xx) or a closed gate is next
    /// to them that way. A fire stinker past halfway through a move fires
    /// from the tile it is moving to.
    fn fire_at_players(&mut self, i: usize, from_nearest: bool) {
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if !p.exists || p.dying || p.dead {
                continue;
            }
            let m = &self.monsters[i];
            let (dx, dy) = if m.x == p.x || m.x == p.x2 {
                match m.y.cmp(&p.y) {
                    std::cmp::Ordering::Greater => (0, -1),
                    std::cmp::Ordering::Less => (0, 1),
                    std::cmp::Ordering::Equal => continue,
                }
            } else if m.y == p.y || m.y == p.y2 {
                match m.x.cmp(&p.x) {
                    std::cmp::Ordering::Greater => (-1, 0),
                    std::cmp::Ordering::Less => (1, 0),
                    std::cmp::Ordering::Equal => continue,
                }
            } else {
                continue;
            };
            let tile = self.level.grid.get(m.x + dx, m.y + dy);
            let class = tile / 100;
            if class == 2 || class == 8 || (class == 10 && tile % 100 < 50) || m.data[3] != 0 {
                continue;
            }
            let (x, y) = (m.x, m.y);
            let (fx, fy) = if from_nearest && m.moving && m.moving_timer < m.speed / 2 {
                (m.x2, m.y2)
            } else {
                (x, y)
            };
            self.play_single_sound(x, y, 3);
            self.create_fireball(fx + dx, fy + dy, dx, dy);
            self.monsters[i].data[3] = 30;
        }
    }

    /// `ControlBomber`: walks left- (3) or right-handed (4) and blows up
    /// next to a player, or when it can't move.
    fn control_bomber(&mut self, i: usize) {
        if self.monsters[i].dead {
            return;
        }
        let (cx, cy) = self.monsters[i].nearest_tile();
        // `xsbomb`, `ysbomb`: taken before anyone is hit.
        let near: Vec<(i32, i32)> = self.players.iter().map(|p| p.nearest_tile()).collect();
        let mut found = false;
        for d in [-1, 1] {
            for at in [(cx + d, cy), (cx, cy + d)] {
                for (k, &tile) in near.iter().enumerate().skip(1) {
                    let p = &self.players[k];
                    if p.exists && !p.dying && !p.dead && tile == at {
                        self.kill_player(k);
                        found = true;
                    }
                }
            }
        }
        if found {
            self.kill_monster(i);
            return;
        }

        let m = &mut self.monsters[i];
        turn(&mut m.data[1], bomber_heading(m.moving_x, m.moving_y), 3);
        if m.moving {
            return;
        }
        if (m.moving_x, m.moving_y) == (0, 0) {
            m.moving_x = 1;
        }
        let (toward, away): (Turn, Turn) = match m.kind {
            3 => (|x, y| (-y, x), |x, y| (y, -x)),
            4 => (|x, y| (y, -x), |x, y| (-y, x)),
            _ => return,
        };
        // Hug the wall, but no more than four turns in a row. The original
        // asks CanMonsterMove even after four turns (And doesn't short-
        // circuit); for a bomber that has no side effects.
        let (dx, dy) = toward(m.moving_x, m.moving_y);
        (m.moving_x, m.moving_y) = (dx, dy);
        let turns = m.data[0];
        if self.can_monster_move(i, dx, dy) && turns < 4 {
            self.move_monster(i, dx, dy);
            self.monsters[i].data[0] += 1;
            return;
        }
        self.monsters[i].data[0] = 0;
        // Straight on, the other way, then back.
        let moved =
            self.turn_and_go(i, away) || self.turn_and_go(i, away) || self.turn_and_go(i, away);
        if !moved {
            self.kill_monster(i);
        }
    }

    /// `ControlChomper`: chases player 1 (7) or 2 (8), or the next one
    /// still in the level.
    fn control_chomper(&mut self, i: usize) {
        if self.monsters[i].dying {
            self.monsters[i].dead = true;
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        turn(&mut m.data[1], heading(m.moving_x, m.moving_y), 6);
        if m.moving {
            return;
        }
        if self.players[1..].iter().any(|p| p.dead || p.dying) {
            return;
        }
        self.chomp_pitch = self.session.rng.rand(19000, 25000);
        let mut target = if self.monsters[i].kind == 7 { 1 } else { 2 };
        for _ in 0..4 {
            let left = self.both_completed & (1 << (target - 1)) != 0;
            if !left && self.players.get(target).is_some_and(|p| p.exists) {
                break;
            }
            target = target % 4 + 1;
        }
        let (px, py) = self.players.get(target).map_or((0, 0), |p| (p.x, p.y));
        let m = &self.monsters[i];
        let (mx, my) = (m.x, m.y);
        let towards_x = if px > mx { 1 } else { -1 };
        let towards_y = if py > my { 1 } else { -1 };
        // Along the longer way first, else across if the player is off
        // that line.
        let (first, second) = if (px - mx).abs() >= (py - my).abs() {
            ((towards_x, 0), (py != my).then_some((0, towards_y)))
        } else {
            ((0, towards_y), (px != mx).then_some((towards_x, 0)))
        };
        for (dx, dy) in std::iter::once(first).chain(second) {
            if self.can_monster_move(i, dx, dy) {
                self.move_monster(i, dx, dy);
                let kind = self.session.rng.rand(8, 11) as usize;
                self.play_single_sound(mx, my, kind);
                return;
            }
        }
    }

    /// `ControlGhost`: wanders, passing gates and everything but ghost
    /// boxes.
    fn control_ghost(&mut self, i: usize) {
        if self.monsters[i].dying {
            self.monsters[i].dead = true;
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        turn(&mut m.data[1], heading(m.moving_x, m.moving_y), 6);
        if m.moving {
            return;
        }
        let (mx, my) = (m.moving_x, m.moving_y);
        let rng = &mut self.session.rng;
        if rng.rand(0, 100) < 50 {
            // A random turn, swapping the axes.
            let side = if rng.rand(0, 100) < 50 { 1 } else { -1 };
            if self.can_monster_move(i, side * my, side * mx) {
                self.move_monster(i, side * my, side * mx);
                return;
            }
        }
        if self.go_straight(i) {
            return;
        }
        let rng = &mut self.session.rng;
        let horizontal = rng.rand(0, 100) < 50;
        let sign = if rng.rand(0, 100) < 50 { 1 } else { -1 };
        let (dx, dy) = if horizontal { (sign, 0) } else { (0, sign) };
        if self.can_monster_move(i, dx, dy) {
            self.move_monster(i, dx, dy);
        }
    }

    /// `ControlSpirit`: the rainbow spirit floats straight on, turning at
    /// random when blocked, and collects coins.
    fn control_spirit(&mut self, i: usize) {
        if self.monsters[i].dying {
            self.monsters[i].dead = true;
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        turn(&mut m.data[1], heading(m.moving_x, m.moving_y), 6);
        if m.moving {
            return;
        }
        let (mx, my) = (m.moving_x, m.moving_y);
        let side = if self.session.rng.rand(0, 100) < 50 {
            1
        } else {
            -1
        };
        for (dx, dy) in [
            (mx, my),
            (my * side, mx * side),
            (-my * side, -mx * side),
            (-mx, -my),
        ] {
            if self.can_monster_move(i, dx, dy) {
                self.move_monster(i, dx, dy);
                return;
            }
        }
    }

    /// `ControlBrokenZbot`: staggers about at random.
    fn control_broken_zbot(&mut self, i: usize) {
        if self.monsters[i].dying {
            self.monsters[i].dead = true;
            return;
        }
        self.hit_players(i);
        let m = &mut self.monsters[i];
        m.data[1] += 3;
        if m.data[1] >= 360 {
            m.data[1] = 0;
        }
        if m.moving {
            return;
        }
        let rng = &mut self.session.rng;
        let sign = if rng.rand(0, 100) < 50 { 1 } else { -1 };
        let (dx, dy) = if rng.rand(0, 100) < 50 {
            (sign, 0)
        } else {
            (0, sign)
        };
        if self.can_monster_move(i, dx, dy) {
            self.move_monster(i, dx, dy);
        }
    }

    /// `ControlMotherShip`: once no saucers are left, rises for 300 ticks,
    /// launches four (each taking three free moves) at 351, then sinks
    /// back by 700 and rests until 801.
    fn control_mother_ship(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        m.data[1] += 1;
        if m.data[1] >= 720 {
            m.data[1] = 0;
        }
        if m.data[2] <= 0 {
            let (x, y) = (m.x, m.y);
            m.data[2] = 400;
            self.play_single_sound(x, y, 4);
        } else {
            m.data[2] -= 1;
        }
        if self.monsters[i].data[3] == 0 && !self.monsters.iter().any(|m| matches!(m.kind, 5 | 6)) {
            self.play_single_sound(-999, -1, 14);
            self.monsters[i].data[3] = 1;
        }
        let t = self.monsters[i].data[3];
        if t <= 0 {
            return;
        }
        match t {
            ..=300 => self.monsters[i].position[1] += 0.01,
            351 => {
                let (x, y) = (self.monsters[i].x, self.monsters[i].y);
                for dir in 0..4 {
                    let kind = self.session.rng.rand(5, 6);
                    self.create_monster(x, y, kind, dir);
                    if let Some(saucer) = self.monsters.last_mut() {
                        saucer.data[9] = 3;
                    }
                }
            }
            401..=700 => self.monsters[i].position[1] -= 0.01,
            801.. => self.monsters[i].data[3] = -1,
            _ => {}
        }
        self.monsters[i].data[3] += 1;
    }

    /// `ControlShadowStinker`: stinkers only move when their player does
    /// (`MovePlayer`); fire stinkers also fire. No dying check: a dead one
    /// still runs this at (0, 0) in the tick it was killed.
    fn control_shadow_stinker(&mut self, i: usize) {
        // Unlike the others, a stinker counts as on the tile it leaves
        // until just past halfway.
        let m = &self.monsters[i];
        let at = if !m.moving || m.moving_timer > m.speed / 2 {
            (m.x, m.y)
        } else {
            (m.x2, m.y2)
        };
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if p.exists && !p.dying && !p.dead && p.nearest_tile() == at {
                self.kill_player(k);
            }
        }
        let m = &mut self.monsters[i];
        turn(&mut m.data[1], heading(m.moving_x, m.moving_y), 6);
        m.data[3] = (m.data[3] - 1).max(0);
        if self.fireball_count() < MAX_FIREBALLS && self.monsters[i].kind > 16 {
            self.fire_at_players(i, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;
    use crate::{BlitzRng, Options, Ruleset, Session};

    /// A 5x4 floor with player 1 at (0, 3).
    fn world() -> World {
        let tiles = [100; 20];
        let mut objects = [0; 20];
        objects[15] = 1;
        let l = level(5, 5, &tiles, &objects);
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    #[test]
    fn turn_wraps_both_ways() {
        let mut a = 0;
        turn(&mut a, 270, 6);
        assert_eq!(a, 354);
        let mut b = 354;
        turn(&mut b, 0, 6);
        assert_eq!(b, 0);
        let mut c = 90;
        turn(&mut c, 180, 3);
        assert_eq!(c, 93);
    }

    #[test]
    fn beach_ball_draws_yaw_then_spin() {
        let mut w = world();
        let mut rng = w.session.rng.clone();
        w.create_monster(1, 1, 0, 0);
        let m = &w.monsters[0];
        assert_eq!(m.yaw, rng.rand(1, 300));
        assert_eq!(m.data[1], rng.rand(-16, 16));
        assert_eq!(m.data[0], -24);
        assert_eq!(m.speed, 48);
        assert_eq!((m.moving_x, m.moving_y), (0, 0));
        assert_eq!(w.session.rng, rng);
    }

    #[test]
    fn speeds_and_headings() {
        let mut w = world();
        for (kind, speed) in [
            (1, 16),
            (3, 36),
            (5, 48),
            (9, 48),
            (10, 16),
            (11, 24),
            (13, 24),
        ] {
            w.create_monster(0, 0, kind, 1);
            assert_eq!(w.monsters.last().unwrap().speed, speed, "type {kind}");
        }
        assert_eq!(w.monsters[0].data[1], 270);
        assert_eq!((w.monsters[0].moving_x, w.monsters[0].moving_y), (1, 0));
        // Stinkers ignore the direction.
        assert_eq!(w.monsters[6].moving_x, 0);
    }

    #[test]
    fn warpers_skip_the_mother_ship_and_swap_bombers() {
        let mut w = world();
        for kind in [3, 4, 11, 12, 13] {
            w.create_warper(0, 0, kind);
        }
        let kinds: Vec<i32> = w.warpers.iter().map(|w| w.kind).collect();
        assert_eq!(kinds, [4, 3, 11, 13, 14]);
    }

    #[test]
    fn warper_makes_four_then_waits_200_ticks() {
        let mut w = world();
        w.create_warper(2, 2, 9);
        w.control_warpers();
        assert_eq!(w.monsters.len(), 4);
        let dirs: Vec<_> = w
            .monsters
            .iter()
            .map(|m| (m.moving_x, m.moving_y))
            .collect();
        assert_eq!(dirs, [(0, -1), (1, 0), (0, 1), (-1, 0)]);
        w.monsters.clear();
        for _ in 0..198 {
            w.control_warpers();
        }
        assert_eq!(w.warpers[0].timer, 199);
        w.control_warpers();
        assert_eq!(w.warpers[0].timer, 0);
        assert!(w.monsters.is_empty());
        w.control_warpers();
        assert_eq!(w.monsters.len(), 4);
    }

    #[test]
    fn robot_follows_the_wall_left_handed() {
        // 5x3 floor ringed by walls: a left-handed z-bot facing up at the
        // top wall turns left.
        let mut w = world();
        w.create_monster(2, 0, 1, 0);
        w.control_robot(0);
        let m = &w.monsters[0];
        assert!(m.moving);
        assert_eq!((m.moving_x, m.moving_y), (-1, 0));
        w.monsters[0].kind = 2;
        w.monsters[0].moving = false;
        w.monsters[0].moving_y = -1;
        w.monsters[0].moving_x = 0;
        w.control_robot(0);
        assert_eq!((w.monsters[0].moving_x, w.monsters[0].moving_y), (1, 0));
    }

    #[test]
    fn monster_moves_take_speed_ticks_and_end_on_the_tile() {
        let mut w = world();
        w.create_monster(1, 1, 1, 1);
        w.move_monster(0, 1, 0);
        for _ in 0..16 {
            w.move_monsters();
            w.end_monster_moves();
        }
        let m = &w.monsters[0];
        assert!(!m.moving);
        assert_eq!((m.x, m.y), (2, 1));
        assert!((m.position[0] - 2.5).abs() < 1e-4);
    }

    #[test]
    fn monsters_block_each_other_and_dead_ones_sit_at_the_origin() {
        let mut w = world();
        w.create_monster(1, 1, 1, 1);
        w.create_monster(2, 1, 1, 1);
        assert!(!w.can_monster_move(0, 1, 0));
        w.kill_monster(1);
        assert!(w.monsters[1].dead);
        assert_eq!((w.monsters[1].x, w.monsters[1].y), (0, 0));
        assert!(w.can_monster_move(0, 1, 0));
        assert_eq!(w.level_score, 250);
        w.clean_up_monsters();
        assert_eq!(w.monsters.len(), 1);
    }

    #[test]
    fn beach_ball_dies_slowly() {
        let mut w = world();
        w.create_monster(1, 1, 0, 0);
        w.kill_monster(0);
        assert!(w.monsters[0].dying && !w.monsters[0].dead);
        for _ in 0..95 {
            w.control_beach_ball(0);
        }
        assert!(!w.monsters[0].dead);
        w.control_beach_ball(0);
        assert!(w.monsters[0].dead);
    }

    #[test]
    fn kill_draws_particles_by_type_and_detail() {
        for (kind, draws) in [(0, 0), (1, 4 * 14), (3, 8 * 7), (5, 4 * 3), (12, 8 * 28)] {
            let mut w = world();
            w.options.detail_level = 2;
            w.create_monster(3, 2, kind, 0);
            let mut rng: BlitzRng = w.session.rng.clone();
            w.kill_monster(0);
            rng.skip(draws * 2);
            assert_eq!(w.session.rng, rng, "type {kind}");
        }
    }

    #[test]
    fn bomber_blows_up_next_to_a_player() {
        let mut w = world();
        let (px, py) = (w.players[1].x, w.players[1].y);
        w.create_monster(px + 1, py, 3, 0);
        w.control_monsters();
        assert!(w.players[1].dying);
        assert!(w.monsters[0].dead);
    }

    #[test]
    fn mother_ship_launches_four_saucers_at_351() {
        let mut w = world();
        w.create_monster(2, 1, 12, 0);
        for _ in 0..350 {
            w.control_mother_ship(0);
        }
        assert_eq!(w.monsters.len(), 1);
        w.control_mother_ship(0);
        assert_eq!(w.monsters.len(), 5);
        assert!(w.monsters[1..].iter().all(|m| m.data[9] == 3));
        assert!((w.monsters[0].position[1] - (-1.8 + 3.0)).abs() < 1e-3);
    }
}
