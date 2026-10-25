//! Boxes and the other pushables: `CreateBox` to `CleanUpBoxes`, the box
//! factories (`CreateBoxer`, `ControlBoxers`) and `BoxAlreadyInWater`,
//! plus the box parts of `UpdateGame`.
//!
//! Box types: 0 wood, 1 steel, 2 and 3 reflectors, 4 boulder, 5 ghost box,
//! 6 boom box (keg), 7 prism, 8 sticky cube, 9–12 link spheres (red,
//! yellow, green, blue) and 100–115 push cannons. Boulders and link
//! spheres roll on until stopped.
//!
//! [`World::kill_box`] only marks a box dead: it stays in the list, and in
//! the way, until `CleanUpBoxes` at the end of the tick. A sunk box is moved
//! to x = 101, off the level, and stays in the list.

use crate::World;
use crate::world::{Event, Sound};

/// `Const MaxBoxes`.
const MAX_BOXES: usize = 400;
/// `Const CannonBaseTimer = BaseSpeed * 5`.
const CANNON_BASE_TIMER: i32 = 24 * 5;
/// Fireballs a push cannon waits for (`NofFireBalls < 25`).
const MAX_CANNON_FIREBALLS: usize = 25;

/// One box (the `Box*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    /// Current tile; x is 101 once sunk, and both are 101 in the air.
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_x: i32,
    pub moving_y: i32,
    pub moving_timer: i32,
    pub speed: i32,
    /// `BoxType`.
    pub kind: i32,
    /// Removed at the end of the tick.
    pub dead: bool,
    /// `BoxSwayingTimer`: -1, or degrees through the bobbing of a wooden
    /// box floating at `sway_x, sway_y`.
    pub sway_timer: i32,
    pub sway_x: i32,
    pub sway_y: i32,
    pub jumping: bool,
    pub jumping_speed: i32,
    pub jumping_distance: i32,
    pub jumping_x: i32,
    pub jumping_y: i32,
    /// A push cannon's firing direction, speed and countdown.
    pub cannon_x: i32,
    pub cannon_y: i32,
    pub cannon_speed: i32,
    pub cannon_timer: i32,
    /// The entity position (x, up, -row). Boxes rest at 0.4, push cannons
    /// at 0; sunk boxes sit lower, which box factories check.
    pub position: [f32; 3],
}

/// A box factory: makes four boxes once none of its kind are left.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Boxer {
    pub x: i32,
    pub y: i32,
    pub kind: i32,
    /// 0 while waiting, then counts to 100; the renderer squashes the
    /// factory over ticks 1–99.
    pub timer: i32,
}

/// A transporter: a raft carrying players and boxes over water (tile
/// 1700), the void (1701) or lava (1702). Its tile reads 3 higher while
/// it moves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transporter {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_x: i32,
    pub moving_y: i32,
    pub moving_timer: i32,
    pub speed: i32,
}

fn rolls(kind: i32) -> bool {
    kind == 4 || (9..=12).contains(&kind)
}

/// Whether a moving box of type `a` beats a box of type `b` it runs into.
/// Pushing a resting box, a ghost box only beats boulders; head-on, it
/// beats link spheres too.
fn stronger(a: i32, b: i32, head_on: bool) -> bool {
    let link = (9..=12).contains(&b);
    ((a >= 1 && a != 6) && (b == 0 || b == 6))
        || ((a == 4 || a == 5 || (9..=12).contains(&a)) && matches!(b, 1 | 2 | 3 | 7))
        || (a == 5 && (b == 4 || (head_on && link)))
}

fn same_strength(a: i32, b: i32) -> bool {
    let mirror = |t: i32| matches!(t, 2 | 3 | 7);
    a == b
        || (mirror(a) && mirror(b))
        || (a == 1 && mirror(b))
        || (b == 1 && mirror(a))
        || ((1..=3).contains(&a) && rolls(b))
}

impl World {
    /// `CreateBox`.
    pub(crate) fn create_box(&mut self, x: i32, y: i32, kind: i32) {
        if self.blocks.len() >= MAX_BOXES {
            return;
        }
        let mut b = Block {
            x,
            y,
            x2: x,
            y2: y,
            kind,
            sway_timer: -1,
            jumping_distance: 1,
            position: [x as f32 + 0.5, 0.4, -(y as f32 + 0.5)],
            ..Block::default()
        };
        if kind >= 100 {
            (b.cannon_x, b.cannon_y) = match (kind - 100) / 4 {
                0 => (0, -1),
                1 => (1, 0),
                2 => (0, 1),
                3 => (-1, 0),
                _ => (0, 0),
            };
            b.cannon_speed = kind % 4 + 1;
            b.cannon_timer = (b.cannon_speed + 1) * CANNON_BASE_TIMER;
            b.position[1] = 0.0;
            b.position[2] = -(y as f32) - 0.5;
        }
        self.blocks.push(b);
    }

    /// `BoxAlreadyInWater`: boxes placed in water sink at once. The
    /// original moves them to x = 0, not 101, so they stay in the way at
    /// `(0, y)`.
    pub(crate) fn box_already_in_water(&mut self) {
        for i in 0..self.blocks.len() {
            let b = &self.blocks[i];
            let (x, y) = (b.x, b.y);
            if self.level.grid.get(x, y) / 100 != 3 {
                continue;
            }
            let kind = b.kind;
            let b = &mut self.blocks[i];
            if kind == 0 {
                self.level.grid.set(x, y, 100);
                b.position[1] += -0.9;
                (b.sway_x, b.sway_y, b.sway_timer) = (x, y, 0);
            } else if kind != 5 {
                self.level.grid.set(x, y, 399);
                b.position[1] += -1.9;
            } else {
                continue;
            }
            b.x = 0;
            b.x2 = 0;
            b.moving = false;
        }
    }

    fn play_at_box(&mut self, sound: Sound, i: usize) {
        let at = self.blocks[i].position;
        self.play_at(sound, at);
    }

    fn box_slippery(&self, i: usize) -> bool {
        let b = &self.blocks[i];
        let tile = self.level.grid.get(b.x, b.y);
        tile / 100 == 4 || self.conveying(tile)
    }

    /// Pushes box `j` on, at double speed if box `i` pushing it is on ice
    /// or a conveyor.
    fn push_on(&mut self, i: usize, j: usize, dx: i32, dy: i32) {
        let speed = if self.box_slippery(i) { 2 } else { 0 };
        self.move_box(j, dx, dy, speed);
    }

    /// `CanBoxMove`: whether box `i` can move by `(dx, dy)`. A moving box
    /// settles collisions on the way: it pushes or smashes what it hits, or
    /// breaks. A boom box explodes on anything that stops it.
    pub(crate) fn can_box_move(&mut self, i: usize, dx: i32, dy: i32) -> bool {
        let (x, y) = (self.blocks[i].x + dx, self.blocks[i].y + dy);
        if !self.level.grid.in_level(x, y) {
            return false;
        }
        let tile = self.level.grid.get(x, y);
        let class = tile / 100;
        let me = self.blocks[i].kind;
        if matches!(class, 0 | 2 | 6 | 8) || (class == 10 && tile % 100 < 50) {
            // Walls, breakaway tiles, cannons and closed gates.
            if me == 6 {
                self.kill_box(i);
            }
            return false;
        }

        for j in 0..self.blocks.len() {
            let other = &self.blocks[j];
            if j == i || ((other.x, other.y) != (x, y) && (other.x2, other.y2) != (x, y)) {
                continue;
            }
            let (them, they_move) = (other.kind, other.moving);
            let (their_dx, their_dy) = (other.moving_x, other.moving_y);
            if !self.blocks[i].moving {
                // A resting box cannot push another.
                if me == 6 {
                    self.kill_box(i);
                }
                return false;
            }
            if !they_move {
                if me == 6 {
                    self.kill_box(i);
                }
                if rolls(me) {
                    if them == 0 || them == 6 {
                        // A boulder smashes wood and kegs.
                        self.kill_box(j);
                        return true;
                    }
                    if rolls(them) {
                        // A boulder sets another rolling, and stops.
                        if self.can_box_move(j, dx, dy) {
                            self.push_on(i, j, dx, dy);
                        }
                    }
                    return false;
                }
                if stronger(me, them, false) {
                    if them == 6 {
                        self.kill_box(j);
                    }
                    // Pushes the weaker box on, or smashes it.
                    if self.can_box_move(j, dx, dy) {
                        self.push_on(i, j, dx, dy);
                    } else {
                        self.kill_box(j);
                    }
                    return true;
                }
                if same_strength(me, them) {
                    if them == 6 {
                        self.kill_box(j);
                    }
                    if self.can_box_move(j, dx, dy) {
                        self.push_on(i, j, dx, dy);
                        return true;
                    }
                    return false;
                }
                if them == 6 {
                    self.kill_box(j);
                }
                return false;
            }
            if (dx, dy) == (their_dx, their_dy) {
                // Both moving the same way.
                return true;
            }
            if stronger(me, them, true) {
                self.kill_box(j);
                return true;
            }
            if same_strength(me, them) {
                self.kill_box(i);
                self.kill_box(j);
            } else {
                self.kill_box(i);
            }
            return false;
        }

        for j in 0..self.monsters.len() {
            let m = &self.monsters[j];
            if (m.x, m.y) != (x, y) && (m.x2, m.y2) != (x, y) {
                continue;
            }
            // Only a ghost box touches a ghost (9).
            if m.kind == 9 && me != 5 {
                return true;
            }
            if !self.blocks[i].moving {
                return false;
            }
            self.kill_monster(j);
            if me == 6 {
                self.kill_box(i);
            }
            return true;
        }
        !self.items.iter().any(|it| (it.x, it.y) == (x, y))
    }

    /// `MoveBox`: starts a one-tile move. `speed` 0 moves at the terrain's
    /// speed; otherwise the move takes `BaseSpeed / speed` ticks. Anyone in
    /// the way is hit, unless walking away ahead of the box off a conveyor.
    pub(crate) fn move_box(&mut self, i: usize, dx: i32, dy: i32, speed: i32) {
        let base = self.ruleset.base_speed;
        let push = if self.blocks[i].kind == 4 {
            Sound::BoulderPush
        } else {
            Sound::BoxPush
        };
        let ticks = if speed != 0 {
            self.play_at_box(push, i);
            base / speed
        } else if self.box_slippery(i) {
            base / 2
        } else {
            self.play_at_box(push, i);
            base
        };
        let b = &mut self.blocks[i];
        b.speed = ticks;
        b.moving_timer = ticks;
        b.x2 = b.x + dx;
        b.y2 = b.y + dy;
        b.moving_x = dx;
        b.moving_y = dy;
        let to = (b.x2, b.y2);
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if !p.exists || ((p.x, p.y) != to && (p.x2, p.y2) != to) {
                continue;
            }
            let ahead = p.moving
                && (p.moving_x, p.moving_y) == (dx, dy)
                && self.level.grid.get(p.x, p.y) / 100 != 5;
            if !ahead {
                self.kill_player(k);
            }
        }
        self.blocks[i].moving = true;
    }

    /// `EndMoveBox`: box `i` has reached `(x2, y2)`.
    pub(crate) fn end_move_box(&mut self, i: usize) {
        let (old_x, old_y) = (self.blocks[i].x, self.blocks[i].y);
        let b = &mut self.blocks[i];
        b.x = b.x2;
        b.y = b.y2;

        if !self.end_move_trampoline {
            if self.level.grid.get(old_x, old_y) / 100 == 11 {
                self.step_off_button(old_x, old_y);
            }
            let old_tile = self.level.grid.get(old_x, old_y);
            if old_tile / 100 == 5 && (old_tile - 100) % 8 >= 4 {
                self.flip_switcher(old_x, old_y);
            }
        }

        if self.blocks[i].kind == 8 && self.stick(i) {
            return;
        }

        let (x, y) = (self.blocks[i].x, self.blocks[i].y);
        let kind = self.blocks[i].kind;
        let tile = self.level.grid.get(x, y);
        let (mx, my) = (self.blocks[i].moving_x, self.blocks[i].moving_y);
        match tile / 100 {
            3 => {
                self.play_at_box(Sound::BoxSplosh, i);
                if kind == 0 || kind == 6 {
                    // Wood floats and can be walked on.
                    self.level.grid.set(x, y, 100);
                    let b = &mut self.blocks[i];
                    b.position[1] += -0.9;
                    (b.sway_x, b.sway_y, b.sway_timer) = (x, y, 0);
                    self.sink(i);
                } else if kind == 5 {
                    self.kill_box(i);
                    return;
                } else if tile == 399 {
                    // The second heavy box fills the water up.
                    self.level.grid.set(x, y, 100);
                    self.blocks[i].position[1] += -0.9;
                    self.sink(i);
                } else {
                    self.level.grid.set(x, y, 399);
                    self.blocks[i].position[1] += -1.9;
                    self.sink(i);
                }
                self.events.push(Event::TileChanged { x, y });
                return;
            }
            4 => {
                let ((fx, fy), (bx, by)) = crate::player::ice_turn(tile, mx, my);
                if self.can_box_move(i, fx, fy) {
                    self.move_box(i, fx, fy, 0);
                } else if self.can_box_move(i, bx, by) {
                    self.move_box(i, bx, by, 0);
                } else {
                    self.kill_box(i);
                }
                return;
            }
            _ => {}
        }
        if self.conveying(tile) {
            let (dx, dy) = crate::player::conveyor_dir(tile);
            if self.can_box_move(i, dx, dy) {
                self.move_box(i, dx, dy, 0);
            } else {
                self.kill_box(i);
            }
            return;
        }
        if tile / 100 == 9 {
            if kind != 1 {
                self.kill_box(i);
                return;
            }
            // Steel fills lava in two.
            self.play_at_box(Sound::Lava, i);
            if tile == 999 {
                self.level.grid.set(x, y, 100);
                self.blocks[i].position[1] += -0.9;
            } else {
                self.level.grid.set(x, y, 999);
                self.blocks[i].position[1] += -1.9;
            }
            self.sink(i);
            self.events.push(Event::TileChanged { x, y });
            return;
        }
        if tile / 100 == 11 {
            self.step_on_button(x, y);
        }
        if tile / 100 == 12 && self.box_step_on_teleporter(i, x, y, mx, my) {
            return;
        }
        if tile / 100 == 17 && self.can_transporter_move(x, y, mx, my) {
            self.move_box(i, mx, my, 0);
            self.move_transporter(x, y, mx, my);
            self.play_at_box(Sound::Transporter, i);
            return;
        }
        if tile / 100 == 18 {
            let base = self.ruleset.base_speed;
            let b = &mut self.blocks[i];
            b.jumping_distance += 1;
            b.jumping = true;
            b.moving = true;
            b.moving_timer = base * b.jumping_distance;
            b.speed = base;
            b.jumping_speed = b.moving_timer;
            self.play_at_box(Sound::Trampoline, i);
            let b = &mut self.blocks[i];
            b.jumping_x = b.x + b.jumping_distance * mx;
            b.jumping_y = b.y + b.jumping_distance * my;
            (b.x, b.y, b.x2, b.y2) = (101, 101, 101, 101);
            return;
        }
        self.blocks[i].jumping_distance = 1;
        if rolls(kind) {
            if self.can_box_move(i, mx, my) {
                self.move_box(i, mx, my, 0);
                return;
            }
            self.play_at_box(Sound::BoulderHit, i);
        }
        self.blocks[i].moving = false;
    }

    /// Takes a sunk box off the level.
    fn sink(&mut self, i: usize) {
        let b = &mut self.blocks[i];
        b.x = 101;
        b.x2 = 101;
        b.moving = false;
    }

    /// A sticky cube that comes to rest next to another resting one: both
    /// set into wall (tile 200) and stay put.
    fn stick(&mut self, i: usize) -> bool {
        let (x, y) = (self.blocks[i].x, self.blocks[i].y);
        let mut stuck = false;
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            if i == j || b.kind != 8 || b.moving {
                continue;
            }
            let (jx, jy) = (b.x, b.y);
            if (jy == y && (jx == x - 1 || jx == x + 1))
                || (jx == x && (jy == y - 1 || jy == y + 1))
            {
                self.play_at_box(Sound::Sticky, i);
                self.level.grid.set(jx, jy, 200);
                self.level.grid.set(x, y, 200);
                self.events.push(Event::TileChanged { x: jx, y: jy });
                self.events.push(Event::TileChanged { x, y });
                self.blocks[i].moving = false;
                stuck = true;
            }
        }
        stuck
    }

    /// `KillBox`: box `i` breaks. A boom box takes everything on the four
    /// tiles around it with it.
    pub(crate) fn kill_box(&mut self, i: usize) {
        self.blocks[i].dead = true;
        let kind = self.blocks[i].kind;
        let sound = if kind == 7 {
            Sound::Single(12)
        } else {
            Sound::BoxExplode
        };
        self.play_at_box(sound, i);
        // 7*DetailLevel splinters, each drawing a velocity and a spin.
        let splinters = usize::try_from(7 * self.options.detail_level).unwrap_or(0);
        self.session.rng.skip(4 * splinters);

        if kind == 6 {
            let (cx, cy) = self.blocks[i].nearest_tile();
            for d in [-1, 1] {
                for (x, y) in [(cx + d, cy), (cx, cy + d)] {
                    self.blast(x, y);
                }
            }
        }
        let (x, y) = (self.blocks[i].x, self.blocks[i].y);
        if self.level.grid.get(x, y) / 100 == 11 {
            self.step_off_button(x, y);
        }
    }

    /// A boom box's or a bomber's blast on one tile.
    pub(crate) fn blast(&mut self, x: i32, y: i32) {
        let hit = |ax: i32, bx: i32, ay: i32, by: i32| (x == ax || x == bx) && (y == ay || y == by);
        for j in 0..self.blocks.len() {
            let b = &self.blocks[j];
            if !b.dead && hit(b.x, b.x2, b.y, b.y2) {
                self.kill_box(j);
            }
        }
        for j in 0..self.monsters.len() {
            let m = &self.monsters[j];
            if !m.dead && !m.dying && hit(m.x, m.x2, m.y, m.y2) && m.kind != 9 {
                self.kill_monster(j);
            }
        }
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if p.exists && !p.dying && !p.dead && hit(p.x, p.x2, p.y, p.y2) {
                self.kill_player(k);
            }
        }
    }

    /// `ControlBoxes`: trampoline flights off the level, floating boxes,
    /// boom boxes next to monsters and push cannons firing.
    pub(crate) fn control_boxes(&mut self) {
        let width = self.level.grid.width() as f32;
        let height = self.level.grid.height() as f32;
        let mut i = 0;
        while i < self.blocks.len() {
            let b = &self.blocks[i];
            if b.jumping {
                let [x, _, z] = b.position;
                if x - 0.5 < 0.0 || z + 0.5 > 0.0 || x + 0.5 > width || z - 0.5 < -height {
                    self.kill_box(i);
                    let b = &mut self.blocks[i];
                    b.moving = false;
                    b.jumping = false;
                }
            }
            let b = &mut self.blocks[i];
            if b.sway_timer >= 0 {
                // The renderer bobs the box, and a player standing on it.
                b.sway_timer += 1;
                if b.sway_timer == 360 {
                    b.sway_timer = 0;
                }
            }
            if b.kind == 6 && self.boom_box_near_monster(i) {
                // The original returns here, skipping the remaining boxes
                // this tick.
                self.kill_box(i);
                return;
            }
            let room = self.fireball_count() < MAX_CANNON_FIREBALLS;
            let b = &mut self.blocks[i];
            if b.kind >= 100 {
                b.cannon_timer -= 1;
                if !b.moving && b.cannon_timer <= 0 && room {
                    let (x, y, cx, cy) = (b.x, b.y, b.cannon_x, b.cannon_y);
                    self.play_single_sound(x, y, 1);
                    self.create_fireball(x + cx, y + cy, cx, cy);
                    let b = &mut self.blocks[i];
                    b.cannon_timer = (b.cannon_speed + 1) * CANNON_BASE_TIMER;
                }
            }
            i += 1;
        }
    }

    /// A boom box blows up when a monster stands next to it, killing every
    /// such monster.
    fn boom_box_near_monster(&mut self, i: usize) -> bool {
        let (x2, y2) = self.blocks[i].nearest_tile();
        let mut found = false;
        for k in 0..self.monsters.len() {
            let m = &self.monsters[k];
            if m.dead || m.dying {
                continue;
            }
            let near = m.nearest_tile();
            for d in [-1, 1] {
                for at in [(x2 + d, y2), (x2, y2 + d)] {
                    if at == near {
                        self.kill_monster(k);
                        found = true;
                    }
                }
            }
        }
        found
    }

    /// Boxes resting on a running conveyor move on once the way is clear.
    pub(crate) fn convey_boxes(&mut self) {
        let mut i = 0;
        while i < self.blocks.len() {
            if !self.blocks[i].moving {
                let tile = |w: &World| {
                    let b = &w.blocks[i];
                    w.level.grid.get(b.x, b.y)
                };
                if self.conveying(tile(self)) {
                    for (dir, dx, dy) in [(0, 0, -1), (2, 0, 1), (1, 1, 0), (3, -1, 0)] {
                        if tile(self) % 4 == dir && self.can_box_move(i, dx, dy) {
                            self.move_box(i, dx, dy, 0);
                        }
                    }
                }
            }
            i += 1;
        }
    }

    /// Object movement for the boxes.
    pub(crate) fn move_boxes(&mut self) {
        for b in &mut self.blocks {
            if !b.moving {
                continue;
            }
            b.position[0] += b.moving_x as f32 / b.speed as f32;
            b.position[2] += -b.moving_y as f32 / b.speed as f32;
            b.moving_timer -= 1;
            if b.jumping {
                let angle = 180.0 / b.jumping_speed as f32 * b.moving_timer as f32;
                b.position[1] += -0.2 * crate::player::cos_deg(angle);
            }
        }
    }

    /// The end of the boxes' moves, landing any jumps first.
    pub(crate) fn end_box_moves(&mut self) {
        let mut i = 0;
        while i < self.blocks.len() {
            let b = &self.blocks[i];
            if b.moving && b.moving_timer == 0 {
                self.end_move_trampoline = false;
                if b.jumping {
                    let b = &mut self.blocks[i];
                    b.jumping = false;
                    b.x2 = b.jumping_x;
                    b.y2 = b.jumping_y;
                    b.x = b.x2;
                    b.y = b.y2;
                    let rest = if b.kind < 100 { 0.4 } else { 0.0 };
                    b.position[1] += -b.position[1] + rest;
                    self.end_move_trampoline = true;
                    self.end_trampoline(true, i);
                }
                self.end_move_box(i);
            }
            i += 1;
        }
    }

    /// `CleanUpBoxes`: dead boxes leave the list, later ones moving up.
    pub(crate) fn clean_up_boxes(&mut self) {
        self.blocks.retain(|b| !b.dead);
    }

    /// `CreateBoxer`.
    pub(crate) fn create_boxer(&mut self, x: i32, y: i32, kind: i32) {
        self.boxers.push(Boxer {
            x,
            y,
            kind,
            timer: 0,
        });
    }

    /// `ControlBoxers`: a factory with none of its boxes left on the level
    /// makes four and pushes one out each way. Boxes that cannot leave
    /// break at once.
    pub(crate) fn control_boxers(&mut self) {
        for i in 0..self.boxers.len() {
            let Boxer { x, y, kind, timer } = self.boxers[i];
            if timer != 0 {
                let f = &mut self.boxers[i];
                f.timer += 1;
                if f.timer == 100 {
                    f.timer = 0;
                }
                continue;
            }
            // Sunk boxes and set sticky cubes do not count.
            let left = self.blocks.iter().any(|b| {
                b.kind == kind
                    && b.position[1] > -0.2
                    && (b.kind != 8 || self.level.grid.get(b.x, b.y) != 200)
            });
            if left || self.blocks.len() >= MAX_BOXES - 10 {
                continue;
            }
            self.boxers[i].timer = 1;
            self.play_single_sound(x, y, 13);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                self.create_box(x, y, kind);
                let n = self.blocks.len() - 1;
                if self.can_box_move(n, dx, dy) {
                    self.move_box(n, dx, dy, 0);
                } else {
                    self.blocks[n].dead = true;
                }
            }
        }
    }

    /// `CreateTransporter`.
    pub(crate) fn create_transporter(&mut self, x: i32, y: i32) {
        if self.transporters.len() >= 200 {
            return;
        }
        self.transporters.push(Transporter {
            x,
            y,
            x2: x,
            y2: y,
            ..Transporter::default()
        });
    }

    /// `CanTransporterMove`: whether the transporter at `(x, y)` can move
    /// by `(dx, dy)`. It stays on its own kind of tile, cannot carry
    /// anything into a box, and stops at an exit gate.
    pub(crate) fn can_transporter_move(&mut self, x: i32, y: i32, dx: i32, dy: i32) -> bool {
        let (x2, y2) = (x + dx, y + dy);
        if !self.level.grid.in_level(x2, y2) {
            return false;
        }
        let grid = &self.level.grid;
        let needs = match grid.get(x, y) {
            1700 => Some(300),
            1701 => Some(0),
            1702 => Some(900),
            _ => None,
        };
        if needs.is_some_and(|t| grid.get(x2, y2) != t) {
            return false;
        }
        if self
            .blocks
            .iter()
            .any(|b| (b.x, b.y) == (x2, y2) || (b.x2, b.y2) == (x2, y2))
        {
            return false;
        }
        let gate = |ix: i32, iy: i32| {
            self.items
                .iter()
                .any(|it| (it.x, it.y) == (ix, iy) && it.kind == 1)
        };
        if gate(x, y) {
            return false;
        }
        let carrying = self
            .blocks
            .iter()
            .any(|b| (b.x == x || b.x2 == x) && (b.y == y || b.y2 == y));
        !(gate(x2, y2) && carrying)
    }

    /// `MoveTransporter`: the transporter at `(x, y)` sets off, taking its
    /// moving tile with it.
    pub(crate) fn move_transporter(&mut self, x: i32, y: i32, dx: i32, dy: i32) {
        // The original takes the last match, or transporter 0 if none.
        let k = (0..self.transporters.len())
            .rev()
            .find(|&i| (self.transporters[i].x, self.transporters[i].y) == (x, y))
            .unwrap_or(0);
        let base = self.ruleset.base_speed;
        let Some(t) = self.transporters.get_mut(k) else {
            debug_assert!(false, "no transporter at ({x}, {y})");
            return;
        };
        t.moving_timer = base;
        t.speed = base;
        t.x2 = t.x + dx;
        t.y2 = t.y + dy;
        t.moving_x = dx;
        t.moving_y = dy;
        t.moving = true;
        let (tx, ty) = (t.x2, t.y2);
        let grid = &mut self.level.grid;
        let moving = grid.get(x, y) + 3;
        grid.set(x, y, moving);
        grid.set(tx, ty, moving);
        self.events.push(Event::TileChanged { x, y });
        self.events.push(Event::TileChanged { x: tx, y: ty });
    }

    /// Object movement for the transporters; the renderer slides them.
    pub(crate) fn move_transporters(&mut self) {
        for t in &mut self.transporters {
            if t.moving {
                t.moving_timer -= 1;
            }
        }
    }

    /// `EndMoveTransporter` for each transporter that has arrived. The tile
    /// left behind turns back into water, void or lava.
    pub(crate) fn end_transporter_moves(&mut self) {
        for k in 0..self.transporters.len() {
            let t = &self.transporters[k];
            if !(t.moving && t.moving_timer == 0) {
                continue;
            }
            let (x, y, x2, y2) = (t.x, t.y, t.x2, t.y2);
            let grid = &mut self.level.grid;
            let tile = grid.get(x2, y2) - 3;
            grid.set(x2, y2, tile);
            match tile {
                1700 => grid.set(x, y, 300),
                1701 => grid.set(x, y, 0),
                1702 => grid.set(x, y, 900),
                _ => {}
            }
            self.events.push(Event::TileChanged { x, y });
            self.events.push(Event::TileChanged { x: x2, y: y2 });
            let t = &mut self.transporters[k];
            t.x = x2;
            t.y = y2;
            t.moving = false;
            t.moving_x = 0;
            t.moving_y = 0;
        }
    }
}

impl Block {
    /// The tile a boom box counts as on: where it is until halfway through
    /// a move, then where it is going.
    fn nearest_tile(&self) -> (i32, i32) {
        if !self.moving || self.moving_timer >= self.speed / 2 {
            (self.x, self.y)
        } else {
            (self.x2, self.y2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;
    use crate::world::{Options, Session};
    use crate::{Input, Ruleset};

    /// Object codes: 1 player, 9 + type for boxes 0–7.
    const P: i32 = 1;
    const WOOD: i32 = 9;
    const STEEL: i32 = 10;
    const BOULDER: i32 = 13;
    const KEG: i32 = 15;

    fn world(tiles: &[i32], objects: &[i32]) -> World {
        let l = level(5, tiles.len(), tiles, objects);
        let mut w =
            World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        // Past the fade-in, so positional sounds can be heard.
        run(&mut w, 160, Input::default());
        w
    }

    const RIGHT: Input = Input {
        right: true,
        up: false,
        down: false,
        left: false,
        camera: false,
        switch: false,
    };

    fn run(w: &mut World, ticks: usize, input: Input) -> Vec<Event> {
        let mut events = vec![];
        for _ in 0..ticks {
            w.step(input);
            events.extend_from_slice(w.events());
        }
        events
    }

    fn heard(events: &[Event], sound: Sound) -> bool {
        events
            .iter()
            .any(|e| matches!(e, Event::SoundAt(s, _) if *s == sound))
    }

    #[test]
    fn pushing_a_box_moves_it_one_tile() {
        let mut w = world(&[100, 100, 100], &[P, WOOD, 0]);
        let events = run(&mut w, 1, RIGHT);
        assert!(heard(&events, Sound::BoxPush));
        assert_eq!(w.players[1].anim_current, 2);
        run(&mut w, 23, Input::default());
        assert_eq!((w.blocks[0].x, w.players[1].x), (2, 1));
        assert!(!w.blocks[0].moving);
        // Nowhere to go now.
        run(&mut w, 30, RIGHT);
        assert_eq!((w.blocks[0].x, w.players[1].x), (2, 1));
    }

    #[test]
    fn boxes_do_not_push_boxes() {
        let mut w = world(&[100, 100, 100, 100], &[P, WOOD, STEEL, 0]);
        run(&mut w, 30, RIGHT);
        assert_eq!((w.blocks[0].x, w.blocks[1].x), (1, 2));
    }

    #[test]
    fn wood_floats_and_steel_fills_water_in_two() {
        let mut w = world(&[100, 100, 300], &[P, WOOD, 0]);
        let events = run(&mut w, 24, RIGHT);
        assert!(heard(&events, Sound::BoxSplosh));
        assert_eq!(w.level.grid.get(2, 0), 100);
        let b = &w.blocks[0];
        assert_eq!((b.x, b.sway_x, b.sway_timer), (101, 2, 0));
        // The player can walk on it.
        run(&mut w, 24, RIGHT);
        assert_eq!(w.players[1].x, 2);

        let mut w = world(&[100, 100, 300], &[P, STEEL, 0]);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.level.grid.get(2, 0), 399);
        assert_eq!(w.blocks[0].x, 101);
    }

    #[test]
    fn boulders_roll_until_stopped() {
        let mut w = world(&[100, 100, 100, 100, 100, 0], &[P, BOULDER, 0, 0, 0, 0]);
        let mut events = run(&mut w, 1, RIGHT);
        events.extend(run(&mut w, 24 * 4, Input::default()));
        assert_eq!(w.blocks[0].x, 4);
        assert!(!w.blocks[0].moving);
        assert!(heard(&events, Sound::BoulderHit));
    }

    #[test]
    fn a_keg_pushed_into_a_wall_blows_up_the_pusher() {
        let mut w = world(&[100, 100, 0], &[P, KEG, 0]);
        let mut rng = w.session.rng.clone();
        let events = run(&mut w, 1, RIGHT);
        assert!(heard(&events, Sound::BoxExplode));
        assert!(w.players[1].dying);
        assert!(w.blocks.is_empty());
        // 7 splinters per detail level, 4 draws each, before the weather.
        rng.skip(4 * 7 * 3 + 2);
        assert_eq!(w.session.rng, rng);
    }

    #[test]
    fn sticky_cubes_set_into_wall() {
        let tiles = [100, 2000, 100, 2000];
        let mut w = world(&tiles, &[P, 0, 0, 0]);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.level.grid.get(2, 0), 200);
        assert_eq!(w.level.grid.get(3, 0), 200);
    }

    #[test]
    fn box_factories_refill_when_their_boxes_are_gone() {
        let l = level(5, 3, &[100, 1600, 100], &[P, 0, 0]);
        let mut w =
            World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        w.step(Input::default());
        // Four wood boxes; the two pushed up and down off the level break.
        // Boxes do not check for players, so the one pushed left runs the
        // player over.
        assert_eq!(w.blocks.len(), 2);
        assert_eq!(
            (w.blocks[0].x2, w.blocks[1].x2, w.blocks[0].kind),
            (2, 0, 0)
        );
        assert!(w.players[1].dying);
        assert_eq!(w.boxers[0].timer, 1);
        run(&mut w, 99, Input::default());
        assert_eq!(w.boxers[0].timer, 0);
        // Its boxes are still about, so it waits.
        run(&mut w, 10, Input::default());
        assert_eq!(w.blocks.len(), 2);
    }

    #[test]
    fn transporters_ferry_a_player_over_water() {
        let tiles = [100, 1700, 300, 300, 100];
        let mut w = world(&tiles, &[P, 0, 0, 0, 0]);
        run(&mut w, 1, RIGHT);
        run(&mut w, 23 + 24 * 2, Input::default());
        let t = &w.transporters[0];
        assert_eq!((t.x, t.moving), (3, false));
        assert_eq!(w.players[1].x, 3);
        let row: Vec<i32> = (0..5).map(|x| w.level.grid.get(x, 0)).collect();
        assert_eq!(row, [100, 300, 300, 1700, 100]);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.players[1].x, 4);
    }

    #[test]
    fn boxes_placed_in_water_sink_at_x_zero() {
        let mut w = world(&[100, 300, 300], &[P, WOOD, STEEL]);
        assert_eq!(w.level.grid.get(1, 0), 100);
        assert_eq!(w.level.grid.get(2, 0), 399);
        assert_eq!((w.blocks[0].x, w.blocks[1].x), (0, 0));
        // As in the original, the sunk boxes now block the start tile.
        run(&mut w, 30, RIGHT);
        run(
            &mut w,
            30,
            Input {
                left: true,
                ..Input::default()
            },
        );
        assert_eq!(w.players[1].x, 1);
    }

    #[test]
    fn push_cannons_count_down_to_fire() {
        let mut w = world(&[100, 1901, 100], &[P, 0, 0]);
        let b = &w.blocks[0];
        assert_eq!((b.kind, b.cannon_y, b.cannon_speed), (101, -1, 2));
        // Created with 3 * 120 ticks to go; 160 have passed.
        assert_eq!(b.cannon_timer, 360 - 160);
        let events = run(&mut w, 200, Input::default());
        assert!(heard(&events, Sound::Single(1)));
        assert_eq!(w.blocks[0].cannon_timer, 360);
    }
}
